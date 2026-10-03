//! Header sync (WS4): pulls headers from a source and feeds them to the verified
//! `HeaderChain`. The source says what exists; the chain alone decides validity and
//! which branch is active (most cumulative work).
//!
//! Sync walks back from the shorter of (our tip, server tip) until it finds a height
//! where both sides agree, then adds forward from there. A server that reorged shows
//! up as `AddOutcome::Reorg`; one that lies fails `add_header` validation.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use async_trait::async_trait;

use crate::header_chain::{AddOutcome, Entry, Header, HeaderChain};
use crate::services::providers::ChaintracksProvider;
use crate::services::{BlockKey, IndexerProvider};

/// Safety bound on how far back a single sync will look for a common ancestor.
const MAX_WALK_BACK: u32 = 10_000;

#[async_trait]
pub trait HeaderSource: Send + Sync {
    /// The source's claimed tip height (a hint only).
    async fn tip_height(&self) -> Result<u32, String>;
    /// 80-byte header hex at `height` on the source's chain.
    async fn header_hex_at(&self, height: u32) -> Result<String, String>;
}

/// `HeaderSource` backed by an Arcade chaintracks server.
pub struct ChaintracksSource(pub ChaintracksProvider);

#[async_trait]
impl HeaderSource for ChaintracksSource {
    async fn tip_height(&self) -> Result<u32, String> {
        self.0.tip_height().await.map_err(|e| e.to_string())
    }

    async fn header_hex_at(&self, height: u32) -> Result<String, String> {
        self.0
            .get_block_header(BlockKey::Height(height))
            .await
            .map(|h| h.header_hex)
            .map_err(|e| e.to_string())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SyncReport {
    pub added: u32,
    pub reorgs: Vec<AddOutcome>,
    pub tip_height: Option<u32>,
}

/// Sync `chain` from `source`. `persist` is called for every header newly stored
/// (outside any lock). Errors are returned, never turned into a "synced" verdict.
pub async fn sync_once(
    chain: &Mutex<HeaderChain>,
    source: &dyn HeaderSource,
    now: u32,
    persist: &mut (dyn FnMut(&Entry) + Send),
) -> Result<SyncReport, String> {
    let server_tip = source.tip_height().await?;
    let mut report = SyncReport::default();

    // Find the highest height where our active chain and the source agree.
    let local_tip = chain.lock().unwrap().tip_height();
    let mut start: u32 = 0;
    if let Some(lt) = local_tip {
        let mut h = lt.min(server_tip);
        let mut walked = 0;
        loop {
            let theirs = Header::from_hex(&source.header_hex_at(h).await?)
                .map_err(|e| format!("source header at {}: {}", h, e))?
                .hash_hex();
            let ours = chain.lock().unwrap().header_at_height(h).map(|e| e.hash.clone());
            if ours.as_deref() == Some(theirs.as_str()) {
                start = h + 1;
                break;
            }
            if h == 0 {
                return Err("source genesis differs from ours".to_string());
            }
            walked += 1;
            if walked > MAX_WALK_BACK {
                return Err(format!("no common ancestor within {} blocks", MAX_WALK_BACK));
            }
            h -= 1;
        }
    }

    for height in start..=server_tip {
        let hex = source.header_hex_at(height).await?;
        let header = Header::from_hex(&hex).map_err(|e| format!("header at {}: {}", height, e))?;
        let (outcome, entry) = {
            let mut c = chain.lock().unwrap();
            let outcome = c
                .add_header(header.clone(), now)
                .map_err(|e| format!("header at {} rejected: {}", height, e))?;
            let entry = c.get(&header.hash_hex()).cloned();
            (outcome, entry)
        };
        match &outcome {
            AddOutcome::Known => {}
            AddOutcome::Reorg { .. } => {
                report.added += 1;
                report.reorgs.push(outcome.clone());
            }
            _ => report.added += 1,
        }
        if outcome != AddOutcome::Known {
            if let Some(e) = entry {
                persist(&e);
            }
        }
    }
    report.tip_height = chain.lock().unwrap().tip_height();
    Ok(report)
}

// ---------------------------------------------------------------------------
// Process-wide service
// ---------------------------------------------------------------------------

/// The verified chain shared by proof checks and the sync task. Present only when a
/// header source is configured (`HODOS_CHAINTRACKS_URL`); absent means "unchanged
/// legacy behaviour", never "verified".
pub struct HeaderService {
    pub chain: Mutex<HeaderChain>,
}

static SERVICE: OnceLock<Arc<HeaderService>> = OnceLock::new();

/// Lowest height whose stored proofs need re-checking against the active chain
/// (`u32::MAX` = nothing pending). Set after a reorg and once after the first
/// successful sync; consumed by `monitor/task_recheck_proofs.rs`.
static RECHECK_FROM: AtomicU32 = AtomicU32::new(u32::MAX);
static STARTUP_RECHECK_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn request_proof_recheck(from_height: u32) {
    RECHECK_FROM.fetch_min(from_height, Ordering::SeqCst);
}

pub fn pending_proof_recheck() -> Option<u32> {
    let v = RECHECK_FROM.load(Ordering::SeqCst);
    (v != u32::MAX).then_some(v)
}

/// Clear the request only if no newer (lower) one arrived while we worked.
pub fn clear_proof_recheck_if(seen: u32) {
    let _ = RECHECK_FROM.compare_exchange(seen, u32::MAX, Ordering::SeqCst, Ordering::SeqCst);
}

/// Ask for one full re-check after the first successful sync of this process.
pub fn request_startup_recheck_once() {
    if !STARTUP_RECHECK_REQUESTED.swap(true, Ordering::SeqCst) {
        request_proof_recheck(0);
    }
}

pub fn init(chain: HeaderChain) -> Arc<HeaderService> {
    let svc = Arc::new(HeaderService { chain: Mutex::new(chain) });
    let _ = SERVICE.set(svc.clone());
    svc
}

pub fn global() -> Option<Arc<HeaderService>> {
    SERVICE.get().cloned()
}

impl HeaderService {
    /// `Ok(true/false)` = verdict against the verified chain; `Err` = we cannot say
    /// (no header at that height yet), which callers must not treat as a verdict.
    pub fn check_merkle_root(&self, height: u32, root_hex: &str) -> Result<bool, String> {
        let c = self.chain.lock().unwrap();
        if c.header_at_height(height).is_none() {
            return Err(format!(
                "header chain has no header at height {} (tip {:?}); sync pending",
                height,
                c.tip_height()
            ));
        }
        Ok(c.verify_merkle_root(height, root_hex))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header_chain::tests::{fixture, mine, NOW};

    #[test]
    fn recheck_request_keeps_the_lowest_height_and_clears_only_if_unchanged() {
        // Process-wide state: use heights no other test uses, and restore at the end.
        clear_proof_recheck_if(pending_proof_recheck().unwrap_or(u32::MAX));
        request_proof_recheck(500);
        request_proof_recheck(300);
        request_proof_recheck(400);
        assert_eq!(pending_proof_recheck(), Some(300));
        request_proof_recheck(100); // newer, lower request arrives mid-run
        clear_proof_recheck_if(300); // the run that saw 300 finishes
        assert_eq!(pending_proof_recheck(), Some(100), "must not drop the newer request");
        clear_proof_recheck_if(100);
        assert_eq!(pending_proof_recheck(), None);
    }
    use crate::header_chain::{compact_to_target, Params, U256};
    use std::collections::HashMap;

    /// A fake server holding one chain, replaceable to simulate a reorg.
    struct Fake {
        by_height: Mutex<HashMap<u32, String>>,
        tip: Mutex<u32>,
    }

    impl Fake {
        fn from_chain(headers: &[Header]) -> Fake {
            let f = Fake { by_height: Mutex::new(HashMap::new()), tip: Mutex::new(0) };
            f.set(headers);
            f
        }
        fn set(&self, headers: &[Header]) {
            let mut m = self.by_height.lock().unwrap();
            m.clear();
            for (i, h) in headers.iter().enumerate() {
                m.insert(i as u32, h.to_hex());
            }
            *self.tip.lock().unwrap() = headers.len() as u32 - 1;
        }
    }

    #[async_trait]
    impl HeaderSource for Fake {
        async fn tip_height(&self) -> Result<u32, String> {
            Ok(*self.tip.lock().unwrap())
        }
        async fn header_hex_at(&self, h: u32) -> Result<String, String> {
            self.by_height.lock().unwrap().get(&h).cloned().ok_or(format!("no header {}", h))
        }
    }

    fn chain_of(g: &Header, n: u32, salt0: u32) -> Vec<Header> {
        let mut v = vec![g.clone()];
        for i in 0..n {
            v.push(mine(v.last().unwrap().hash(), salt0 + i, 0x207f_ffff));
        }
        v
    }

    /// Empty chain pinned to `g` as its genesis.
    fn empty_with_genesis_of(g: &Header) -> Mutex<HeaderChain> {
        let params = Params {
            name: "test",
            fixed_bits: 0x207f_ffff,
            genesis_hash: Box::leak(g.hash_hex().into_boxed_str()),
        };
        Mutex::new(HeaderChain::new(params))
    }

    #[tokio::test]
    async fn initial_sync_adds_everything_and_persists_each() {
        let (_, g) = fixture();
        let server = Fake::from_chain(&chain_of(&g, 5, 100));
        let chain = empty_with_genesis_of(&g);
        let mut persisted = Vec::new();
        let r = sync_once(&chain, &server, NOW, &mut |e| persisted.push(e.height)).await.unwrap();
        assert_eq!(r.added, 6);
        assert_eq!(r.tip_height, Some(5));
        assert_eq!(persisted, vec![0, 1, 2, 3, 4, 5]);
        // Second run is a no-op.
        let r2 = sync_once(&chain, &server, NOW, &mut |_| panic!("nothing new")).await.unwrap();
        assert_eq!(r2.added, 0);
    }

    #[tokio::test]
    async fn server_reorg_to_heavier_chain_is_followed_and_reported() {
        let (_, g) = fixture();
        let a = chain_of(&g, 3, 100);
        let server = Fake::from_chain(&a);
        let chain = empty_with_genesis_of(&g);
        sync_once(&chain, &server, NOW, &mut |_| {}).await.unwrap();
        assert_eq!(chain.lock().unwrap().tip().unwrap().hash, a[3].hash_hex());

        // Server switches to a branch from height 1 that is longer.
        let mut b = vec![a[0].clone(), a[1].clone()];
        for i in 0..4 {
            b.push(mine(b.last().unwrap().hash(), 500 + i, 0x207f_ffff));
        }
        server.set(&b);
        let r = sync_once(&chain, &server, NOW, &mut |_| {}).await.unwrap();
        assert_eq!(r.reorgs.len(), 1);
        match &r.reorgs[0] {
            AddOutcome::Reorg { fork_height, depth, .. } => {
                assert_eq!(*fork_height, 1);
                assert_eq!(*depth, 2);
            }
            o => panic!("{:?}", o),
        }
        assert_eq!(chain.lock().unwrap().tip().unwrap().hash, b[5].hash_hex());
    }

    #[tokio::test]
    async fn shorter_lighter_server_chain_does_not_displace_ours() {
        let (_, g) = fixture();
        let a = chain_of(&g, 4, 100);
        let server = Fake::from_chain(&a);
        let chain = empty_with_genesis_of(&g);
        sync_once(&chain, &server, NOW, &mut |_| {}).await.unwrap();

        let mut b = vec![a[0].clone(), a[1].clone()];
        b.push(mine(b.last().unwrap().hash(), 900, 0x207f_ffff));
        server.set(&b); // height-2 tip, less work than our height-4 tip
        let r = sync_once(&chain, &server, NOW, &mut |_| {}).await.unwrap();
        assert!(r.reorgs.is_empty());
        assert_eq!(chain.lock().unwrap().tip().unwrap().hash, a[4].hash_hex());
    }

    #[tokio::test]
    async fn server_with_bad_pow_is_rejected_not_adopted() {
        let (_, g) = fixture();
        let mut a = chain_of(&g, 2, 100);
        // Corrupt the last header's nonce until PoW fails.
        loop {
            a[2].nonce = a[2].nonce.wrapping_add(1);
            let mut x = a[2].hash();
            x.reverse();
            if U256::from_be_bytes(x) > compact_to_target(a[2].bits).unwrap() {
                break;
            }
        }
        let server = Fake::from_chain(&a);
        let chain = empty_with_genesis_of(&g);
        let err = sync_once(&chain, &server, NOW, &mut |_| {}).await.unwrap_err();
        assert!(err.contains("InsufficientWork"), "{}", err);
        assert_eq!(chain.lock().unwrap().tip_height(), Some(1)); // good prefix kept
    }

    #[tokio::test]
    async fn different_genesis_is_an_error() {
        let (_, g) = fixture();
        let other = mine([0; 32], 77, 0x207f_ffff);
        let server = Fake::from_chain(&chain_of(&other, 2, 100));
        let chain = empty_with_genesis_of(&g);
        assert!(sync_once(&chain, &server, NOW, &mut |_| {}).await.is_err());
    }

    /// Live: sync the real spv-testnet chain from genesis with the real regtest rules.
    /// `cargo test --lib header_sync_live -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn header_sync_live_real_chain() {
        let base = std::env::var(crate::chain_mode::ENV_CHAINTRACKS)
            .unwrap_or_else(|_| "http://localhost:8083/chaintracks/v2".into());
        let src = ChaintracksSource(ChaintracksProvider::new(reqwest::Client::new(), base));
        let chain = Mutex::new(HeaderChain::new(Params::regtest()));
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        let r = sync_once(&chain, &src, now, &mut |_| {}).await.expect("sync");
        println!("report: {:?}", r);
        assert!(r.added > 100);
        let again = sync_once(&chain, &src, now, &mut |_| {}).await.expect("resync");
        assert!(again.added <= 3, "only newly mined blocks: {:?}", again);
    }

    async fn rpc(c: &reqwest::Client, method: &str, params: serde_json::Value) -> serde_json::Value {
        let r: serde_json::Value = c
            .post("http://localhost:29292")
            .basic_auth("bitcoin", Some("bitcoin"))
            .json(&serde_json::json!({"method": method, "params": params}))
            .timeout(std::time::Duration::from_secs(240))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(r["error"].is_null(), "rpc {} failed: {}", method, r["error"]);
        r["result"].clone()
    }

    /// Live reorg against the real stack. Needs the background miner stopped
    /// (`docker stop cb-block-generator`). Invalidates the last 2 blocks, mines 4 more,
    /// and checks the wallet's own chain follows by most-work.
    /// `cargo test --lib header_sync_live_reorg -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn header_sync_live_reorg() {
        let base = "http://localhost:8083/chaintracks/v2".to_string();
        let src = ChaintracksSource(ChaintracksProvider::new(reqwest::Client::new(), base.clone()));
        let chain = Mutex::new(HeaderChain::new(Params::regtest()));
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u32;
        sync_once(&chain, &src, now, &mut |_| {}).await.expect("initial sync");
        let (old_tip_h, old_tip) = {
            let c = chain.lock().unwrap();
            let t = c.tip().unwrap();
            (t.height, t.hash.clone())
        };

        let http = reqwest::Client::new();
        let victim = rpc(&http, "getblockhash", serde_json::json!([old_tip_h - 1])).await;
        rpc(&http, "invalidateblock", serde_json::json!([victim])).await;
        rpc(&http, "generate", serde_json::json!([4])).await;

        // Wait for chaintracks to follow the node (it learns from P2P announcements).
        let want = old_tip_h - 2 + 4;
        for _ in 0..60 {
            if src.tip_height().await.unwrap() >= want { break; }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        let r = sync_once(&chain, &src, now + 10, &mut |_| {}).await.expect("resync");
        println!("report: {:?}", r);
        assert_eq!(r.reorgs.len(), 1, "{:?}", r);
        match &r.reorgs[0] {
            AddOutcome::Reorg { fork_height, depth, old_tip: ot, .. } => {
                assert_eq!(*fork_height, old_tip_h - 2);
                assert_eq!(*depth, 2);
                assert_eq!(ot, &old_tip);
            }
            o => panic!("{:?}", o),
        }
        assert!(!chain.lock().unwrap().is_active(&old_tip));
        assert_eq!(chain.lock().unwrap().tip_height(), Some(want));
    }

    #[test]
    fn check_merkle_root_distinguishes_unknown_from_mismatch() {
        let (c, g) = fixture();
        let svc = HeaderService { chain: Mutex::new(c) };
        assert_eq!(svc.check_merkle_root(0, &g.merkle_root_hex()), Ok(true));
        assert_eq!(svc.check_merkle_root(0, &"ff".repeat(32)), Ok(false));
        assert!(svc.check_merkle_root(5, &g.merkle_root_hex()).is_err());
    }
}
