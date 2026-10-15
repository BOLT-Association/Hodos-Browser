//! Header sync (WS4): pulls headers from a source and feeds them to the verified
//! `HeaderChain`. The source says what exists; the chain alone decides validity and
//! which branch is active (most cumulative work).
//!
//! Sync walks back from the shorter of (our tip, server tip) until it finds a height
//! where both sides agree, then adds forward from there. A server that reorged shows
//! up as `AddOutcome::Reorg`; one that lies fails `add_header` validation. The walk never
//! goes below the chain's anchor (a lite chain's pinned checkpoint), and a source whose tip
//! is below the anchor or more than `MAX_WALK_BACK` behind ours is *unavailable*: an error
//! that leaves the chain untouched, never a reorg and never a silent "synced".

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use async_trait::async_trait;

use crate::header_chain::{AddOutcome, Entry, Header, HeaderChain};
use crate::header_trackers::HeaderTrackers;
use crate::services::providers::ChaintracksProvider;
use crate::services::{BlockKey, IndexerProvider};

/// Safety bound on how far back a single sync will look for a common ancestor.
const MAX_WALK_BACK: u32 = 10_000;
/// Headers asked for per request when adding forward (the walk-back stays per height).
const SYNC_BATCH: u32 = 2_000;

#[async_trait]
pub trait HeaderSource: Send + Sync {
    /// The source's claimed tip height (a hint only).
    async fn tip_height(&self) -> Result<u32, String>;
    /// 80-byte header hex at `height` on the source's chain.
    async fn header_hex_at(&self, height: u32) -> Result<String, String>;
    /// Up to `count` consecutive headers from `start`; fewer when the source has fewer.
    /// The default fetches per height and stops at the first failure, returning what it has
    /// so those headers are still applied (and any reorg among them reported); a failure on
    /// the very first height is the error.
    async fn headers_hex_from(&self, start: u32, count: u32) -> Result<Vec<String>, String> {
        let mut v = Vec::with_capacity(count as usize);
        for h in start..start.saturating_add(count) {
            match self.header_hex_at(h).await {
                Ok(hex) => v.push(hex),
                Err(e) if v.is_empty() => return Err(e),
                Err(_) => break,
            }
        }
        Ok(v)
    }
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

    async fn headers_hex_from(&self, start: u32, count: u32) -> Result<Vec<String>, String> {
        self.0.headers_bulk(start, count).await.map_err(|e| e.to_string())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SyncReport {
    pub added: u32,
    pub reorgs: Vec<AddOutcome>,
    pub tip_height: Option<u32>,
}

/// Like `sync_once`, but a failure part-way does not hide what had already happened: the
/// returned report still lists the reorgs applied before the error, so the caller can act on
/// them (re-check proofs above the fork) whatever the outcome.
pub async fn sync_once_reporting(
    chain: &Mutex<HeaderChain>,
    source: &dyn HeaderSource,
    now: u32,
    persist: &mut (dyn FnMut(&Entry) + Send),
) -> (SyncReport, Result<(), String>) {
    let mut report = SyncReport::default();
    let result = sync_into(chain, source, now, persist, &mut report).await;
    if result.is_ok() {
        report.tip_height = chain.lock().unwrap().tip_height();
    }
    (report, result)
}

#[allow(dead_code)] // the monitor uses `sync_once_reporting`; kept as the simple form for callers and tests
/// Sync `chain` from `source`. `persist` is called for every header newly stored
/// (outside any lock). Errors are returned, never turned into a "synced" verdict.
pub async fn sync_once(
    chain: &Mutex<HeaderChain>,
    source: &dyn HeaderSource,
    now: u32,
    persist: &mut (dyn FnMut(&Entry) + Send),
) -> Result<SyncReport, String> {
    let (report, result) = sync_once_reporting(chain, source, now, persist).await;
    result.map(|()| report)
}

async fn sync_into(
    chain: &Mutex<HeaderChain>,
    source: &dyn HeaderSource,
    now: u32,
    persist: &mut (dyn FnMut(&Entry) + Send),
    report: &mut SyncReport,
) -> Result<(), String> {
    let (anchor_height, local_tip) = {
        let c = chain.lock().unwrap();
        (c.params().anchor.0, c.tip_height())
    };
    let server_tip = source.tip_height().await?;

    // A source that cannot serve our window is unavailable, never a reorg. A public chaintracks
    // that reset and is re-bootstrapping from genesis still answers for the low heights it has,
    // which the walk-back would read as agreement and report as a successful sync.
    if server_tip < anchor_height {
        return Err(format!("source unavailable: its tip {} is below our anchor {}", server_tip, anchor_height));
    }
    if let Some(lt) = local_tip {
        if server_tip.saturating_add(MAX_WALK_BACK) < lt {
            return Err(format!(
                "source unavailable: its tip {} is more than {} behind ours ({})",
                server_tip, MAX_WALK_BACK, lt
            ));
        }
    }

    // Find the highest height where our active chain and the source agree. The anchor is
    // pinned: a source that differs there is on another chain, and nothing below it is asked for.
    let mut start: u32 = anchor_height;
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
            if h == anchor_height {
                return Err(format!("source anchor at height {} differs from ours", anchor_height));
            }
            walked += 1;
            if walked > MAX_WALK_BACK {
                return Err(format!("no common ancestor within {} blocks", MAX_WALK_BACK));
            }
            h -= 1;
        }
    }

    let mut next = start;
    while next <= server_tip {
        let want = (server_tip - next + 1).min(SYNC_BATCH);
        let batch = source.headers_hex_from(next, want).await?;
        if batch.is_empty() {
            return Err(format!("source returned no headers from height {}", next));
        }
        for hex in batch {
            let height = next;
            next += 1;
            add_one(chain, &hex, height, now, persist, report)?;
        }
    }
    Ok(())
}

fn add_one(
    chain: &Mutex<HeaderChain>,
    hex: &str,
    height: u32,
    now: u32,
    persist: &mut (dyn FnMut(&Entry) + Send),
    report: &mut SyncReport,
) -> Result<(), String> {
    let header = Header::from_hex(hex).map_err(|e| format!("header at {}: {}", height, e))?;
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
    Ok(())
}

// ---------------------------------------------------------------------------
// Process-wide service
// ---------------------------------------------------------------------------

/// The verified chain shared by proof checks and the sync task. Present only when a
/// header source is configured (`HODOS_CHAINTRACKS_URL`); absent means "unchanged
/// legacy behaviour", never "verified".
pub struct HeaderService {
    pub chain: Mutex<HeaderChain>,
    /// Lite chains only: headers below the window's anchor, from the configured trackers.
    pub trackers: Option<HeaderTrackers>,
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

pub fn init(chain: HeaderChain, trackers: Option<HeaderTrackers>) -> Arc<HeaderService> {
    let svc = Arc::new(HeaderService { chain: Mutex::new(chain), trackers });
    let _ = SERVICE.set(svc.clone());
    svc
}

pub fn global() -> Option<Arc<HeaderService>> {
    SERVICE.get().cloned()
}

impl HeaderService {
    /// Tip height of the wallet's own verified chain (`None` until the first header).
    pub fn tip_height(&self) -> Option<u32> {
        self.chain.lock().unwrap().tip_height()
    }

    /// 80-byte header hex at `height` on the active verified chain.
    pub fn header_hex_at_height(&self, height: u32) -> Option<String> {
        self.chain.lock().unwrap().header_at_height(height).map(|e| e.header.to_hex())
    }

    /// Block hash (display hex) at `height` on the active verified chain, computed from the
    /// header itself. Use this instead of a hash an indexer or push event claims.
    pub fn block_hash_at_height(&self, height: u32) -> Option<String> {
        self.chain.lock().unwrap().header_at_height(height).map(|e| e.header.hash_hex())
    }

    /// Make `height` answerable. Stub: never.
    pub async fn ensure_header(&self, height: u32) -> Result<(), String> {
        Err(format!("stub: height {} not ensured", height))
    }

    /// Make every height answerable, each fetched at most once. Stub: never.
    pub async fn ensure_headers(&self, heights: &[u32]) -> Result<(), String> {
        Err(format!("stub: {} height(s) not ensured", heights.len()))
    }

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
        if c.verify_merkle_root(height, root_hex) {
            return Ok(true);
        }
        // A root that belongs to a competing branch we know of is not a verdict either way until
        // one branch wins; calling it `false` would make callers drop a proof that may be right.
        if c.side_branch_has_root(height, root_hex) {
            return Err(format!(
                "proof matches a competing branch at height {} that is not the active chain (yet)",
                height
            ));
        }
        Ok(false)
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
        /// Every height asked for, in order.
        asked: Mutex<Vec<u32>>,
    }

    impl Fake {
        fn from_chain(headers: &[Header]) -> Fake {
            Fake::from_chain_at(0, headers)
        }
        /// Serves `headers` at heights `start..`.
        fn from_chain_at(start: u32, headers: &[Header]) -> Fake {
            let f = Fake { by_height: Mutex::new(HashMap::new()), tip: Mutex::new(0), asked: Mutex::new(Vec::new()) };
            f.set_at(start, headers);
            f
        }
        fn set(&self, headers: &[Header]) {
            self.set_at(0, headers)
        }
        fn set_at(&self, start: u32, headers: &[Header]) {
            let mut m = self.by_height.lock().unwrap();
            m.clear();
            for (i, h) in headers.iter().enumerate() {
                m.insert(start + i as u32, h.to_hex());
            }
            *self.tip.lock().unwrap() = start + headers.len() as u32 - 1;
        }
        fn lowest_asked(&self) -> Option<u32> {
            self.asked.lock().unwrap().iter().copied().min()
        }
    }

    #[async_trait]
    impl HeaderSource for Fake {
        async fn tip_height(&self) -> Result<u32, String> {
            Ok(*self.tip.lock().unwrap())
        }
        async fn header_hex_at(&self, h: u32) -> Result<String, String> {
            self.asked.lock().unwrap().push(h);
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
            difficulty: crate::header_chain::Difficulty::Fixed(0x207f_ffff),
            anchor: (0, Box::leak(g.hash_hex().into_boxed_str())),
            assume_valid_height: 0,
            checkpoints: &[],
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

    /// Serves headers only in bulk, at most `cap` per call, and counts the calls.
    struct BulkOnly {
        headers: Vec<String>,
        cap: u32,
        calls: Mutex<u32>,
    }

    #[async_trait]
    impl HeaderSource for BulkOnly {
        async fn tip_height(&self) -> Result<u32, String> {
            Ok(self.headers.len() as u32 - 1)
        }
        async fn header_hex_at(&self, h: u32) -> Result<String, String> {
            Err(format!("per-height fetch of {} on a bulk-only source", h))
        }
        async fn headers_hex_from(&self, start: u32, count: u32) -> Result<Vec<String>, String> {
            *self.calls.lock().unwrap() += 1;
            let end = (start + count.min(self.cap)) as usize;
            Ok(self.headers[start as usize..end.min(self.headers.len())].to_vec())
        }
    }

    #[tokio::test]
    async fn initial_sync_fetches_in_batches_and_follows_short_ones() {
        let (_, g) = fixture();
        let headers: Vec<String> = chain_of(&g, 4_500, 1_000).iter().map(|h| h.to_hex()).collect();
        let server = BulkOnly { headers, cap: 1_500, calls: Mutex::new(0) };
        let chain = empty_with_genesis_of(&g);
        let mut persisted = 0u32;
        let r = sync_once(&chain, &server, NOW, &mut |_| persisted += 1).await.unwrap();
        assert_eq!(r.tip_height, Some(4_500));
        assert_eq!((r.added, persisted), (4_501, 4_501));
        assert_eq!(*server.calls.lock().unwrap(), 4, "4501 headers at <= 1500 per call");
    }

    #[tokio::test]
    async fn a_source_that_returns_nothing_is_an_error_not_a_loop() {
        let (_, g) = fixture();
        let headers: Vec<String> = chain_of(&g, 3, 1_000).iter().map(|h| h.to_hex()).collect();
        let server = BulkOnly { headers, cap: 0, calls: Mutex::new(0) };
        let chain = empty_with_genesis_of(&g);
        let e = sync_once(&chain, &server, NOW, &mut |_| {}).await.unwrap_err();
        assert!(e.contains("no headers from height 0"), "{}", e);
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

    /// A server whose chain stops answering after `last_ok` (tip still claimed higher).
    #[tokio::test]
    async fn a_reorg_applied_before_a_later_failure_is_still_reported() {
        let (_, g) = fixture();
        let a = chain_of(&g, 3, 100);
        let server = Fake::from_chain(&a);
        let chain = empty_with_genesis_of(&g);
        sync_once(&chain, &server, NOW, &mut |_| {}).await.unwrap();

        // Heavier branch from height 1: heights 2..=4 answer, 5 is claimed but never served.
        let mut b = vec![a[0].clone(), a[1].clone()];
        for i in 0..4 {
            b.push(mine(b.last().unwrap().hash(), 500 + i, 0x207f_ffff));
        }
        server.set(&b);
        server.by_height.lock().unwrap().remove(&5);

        let (report, result) = sync_once_reporting(&chain, &server, NOW, &mut |_| {}).await;
        assert!(result.is_err(), "the missing header is still an error");
        assert_eq!(report.reorgs.len(), 1, "but the reorg that already happened is not lost");
        assert!(matches!(report.reorgs[0], AddOutcome::Reorg { fork_height: 1, .. }));
        assert_eq!(chain.lock().unwrap().tip().unwrap().hash, b[4].hash_hex(), "chain is on the new branch");
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

    // -----------------------------------------------------------------------
    // Lite chains: the sync starts at the anchor, never asks the source below it, and treats a
    // source that cannot serve the window (tip below the anchor, or far behind ours after a
    // reset) as unavailable rather than as a reorg or a silent no-op.
    // -----------------------------------------------------------------------

    /// Empty chain anchored at height `h` with `g` as its anchor header (fixed difficulty).
    fn empty_anchored_at(h: u32, g: &Header) -> Mutex<HeaderChain> {
        let params = Params {
            name: "lite-test",
            difficulty: crate::header_chain::Difficulty::Fixed(0x207f_ffff),
            anchor: (h, Box::leak(g.hash_hex().into_boxed_str())),
            assume_valid_height: 0,
            checkpoints: &[],
        };
        Mutex::new(HeaderChain::new(params))
    }

    #[tokio::test]
    async fn a_lite_chain_syncs_from_its_anchor_and_never_asks_below_it() {
        let (_, g) = fixture();
        let server = Fake::from_chain_at(100, &chain_of(&g, 10, 300)); // serves 100..=110
        let chain = empty_anchored_at(100, &g);
        let mut persisted = Vec::new();
        let r = sync_once(&chain, &server, NOW, &mut |e| persisted.push(e.height)).await.unwrap();
        assert_eq!((r.added, r.tip_height), (11, Some(110)));
        assert_eq!(persisted, (100..=110).collect::<Vec<_>>());
        assert!(server.lowest_asked().unwrap_or(u32::MAX) >= 100, "asked {:?}", server.lowest_asked());
        let r2 = sync_once(&chain, &server, NOW, &mut |_| panic!("nothing new")).await.unwrap();
        assert_eq!(r2.added, 0);
        assert!(server.lowest_asked().unwrap() >= 100);
    }

    #[tokio::test]
    async fn a_source_whose_tip_is_below_our_anchor_is_unavailable() {
        let (_, g) = fixture();
        let server = Fake::from_chain(&chain_of(&g, 50, 300)); // tip 50
        let chain = empty_anchored_at(100, &g);
        let (report, result) = sync_once_reporting(&chain, &server, NOW, &mut |_| panic!("nothing stored")).await;
        let e = result.unwrap_err();
        assert!(e.contains("unavailable"), "{}", e);
        assert!(report.reorgs.is_empty());
        assert_eq!(chain.lock().unwrap().tip_height(), None, "chain untouched");
    }

    #[tokio::test]
    async fn a_source_far_behind_ours_is_unavailable_not_a_silent_no_op() {
        // The public chaintracks reset case: a server that restarted from genesis still answers
        // for the low heights it has, so the old walk-back found agreement and reported success.
        let (_, g) = fixture();
        let headers = chain_of(&g, 10_200, 2_000);
        let chain = empty_with_genesis_of(&g);
        sync_once(&chain, &Fake::from_chain(&headers), NOW, &mut |_| {}).await.unwrap();
        assert_eq!(chain.lock().unwrap().tip_height(), Some(10_200));

        let reset = Fake::from_chain(&headers[..=100]); // tip 100, more than MAX_WALK_BACK behind
        let (report, result) = sync_once_reporting(&chain, &reset, NOW, &mut |_| panic!("nothing stored")).await;
        let e = result.unwrap_err();
        assert!(e.contains("unavailable"), "{}", e);
        assert!(report.reorgs.is_empty());
        assert_eq!(chain.lock().unwrap().tip_height(), Some(10_200), "chain untouched");

        // Merely behind (within the bound) is still fine: nothing to add, no error.
        let behind = Fake::from_chain(&headers[..=9_000]);
        let r = sync_once(&chain, &behind, NOW, &mut |_| panic!("nothing stored")).await.unwrap();
        assert_eq!((r.added, r.tip_height), (0, Some(10_200)));
    }

    #[tokio::test]
    async fn a_walk_back_that_reaches_the_anchor_without_agreement_is_an_error_not_a_reorg() {
        let (_, g) = fixture();
        let chain = empty_anchored_at(100, &g);
        sync_once(&chain, &Fake::from_chain_at(100, &chain_of(&g, 3, 300)), NOW, &mut |_| {}).await.unwrap();
        assert_eq!(chain.lock().unwrap().tip_height(), Some(103));

        // Another chain that also starts at height 100, from a different anchor.
        let other = mine([0; 32], 78, 0x207f_ffff);
        let server = Fake::from_chain_at(100, &chain_of(&other, 4, 500)); // tip 104
        let (report, result) = sync_once_reporting(&chain, &server, NOW, &mut |_| panic!("nothing stored")).await;
        let e = result.unwrap_err();
        assert!(e.contains("anchor"), "{}", e);
        assert!(report.reorgs.is_empty());
        assert!(server.lowest_asked().unwrap() >= 100, "never asked below the anchor: {:?}", server.lowest_asked());
        assert_eq!(chain.lock().unwrap().tip_height(), Some(103), "chain untouched");
    }

    // -----------------------------------------------------------------------
    // HeaderService on a lite chain: the window answers what it holds, the trackers answer
    // heights below the anchor (fetched once, then cached), and anything neither can say is an
    // error, never `Ok(false)`.
    // -----------------------------------------------------------------------

    use crate::header_trackers::{tests::FakeSource, HeaderTrackers, TrackerSource};
    use std::sync::atomic::Ordering;

    /// A lite chain anchored at 100 (the fixture genesis as the anchor) holding 100..=103.
    fn lite_chain_100_103(g: &Header) -> (HeaderChain, Vec<Header>) {
        let headers = chain_of(g, 3, 700);
        let mut c = empty_anchored_at(100, g).into_inner().unwrap();
        for h in &headers {
            c.add_header(h.clone(), NOW).unwrap();
        }
        assert_eq!(c.tip_height(), Some(103));
        (c, headers)
    }

    fn lite_service(c: HeaderChain, sources: Vec<Arc<dyn TrackerSource>>, min: usize) -> HeaderService {
        HeaderService { chain: Mutex::new(c), trackers: Some(HeaderTrackers::new(sources, min, &[])) }
    }

    #[tokio::test]
    async fn inside_the_window_the_trackers_are_never_asked() {
        let (_, g) = fixture();
        let (c, headers) = lite_chain_100_103(&g);
        let src = Arc::new(FakeSource::new("a", &[]));
        let svc = lite_service(c, vec![src.clone()], 1);
        assert_eq!(svc.ensure_header(101).await, Ok(()));
        assert_eq!(svc.ensure_headers(&[100, 103, 103]).await, Ok(()));
        assert_eq!(svc.check_merkle_root(101, &headers[1].merkle_root_hex()), Ok(true));
        assert_eq!(src.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn below_the_anchor_the_trackers_answer_once_and_are_cached() {
        let (_, g) = fixture();
        let (c, _) = lite_chain_100_103(&g);
        let h50 = mine([7; 32], 50, 0x207f_ffff);
        let src = Arc::new(FakeSource::new("a", &[(50, h50.clone())]));
        let svc = lite_service(c, vec![src.clone()], 1);
        assert!(svc.check_merkle_root(50, &h50.merkle_root_hex()).is_err(), "not fetched yet: cannot say, never false");
        assert_eq!(svc.block_hash_at_height(50), None);
        assert_eq!(svc.ensure_header(50).await, Ok(()));
        assert_eq!(src.calls.load(Ordering::SeqCst), 1);
        assert_eq!(svc.check_merkle_root(50, &h50.merkle_root_hex()), Ok(true));
        assert_eq!(svc.check_merkle_root(50, &"ff".repeat(32)), Ok(false));
        assert_eq!(svc.block_hash_at_height(50), Some(h50.hash_hex()));
        assert_eq!(svc.header_hex_at_height(50), Some(h50.to_hex()));
        assert_eq!(svc.ensure_headers(&[50, 50, 101]).await, Ok(()));
        assert_eq!(src.calls.load(Ordering::SeqCst), 1, "cached and in-window heights are not fetched again");
    }

    #[tokio::test]
    async fn a_tracker_failure_is_cannot_say_never_a_verdict() {
        let (_, g) = fixture();
        let (c, _) = lite_chain_100_103(&g);
        let src = Arc::new(FakeSource::failing("a", "connection refused"));
        let svc = lite_service(c, vec![src], 1);
        let e = svc.ensure_header(50).await.unwrap_err();
        assert!(e.contains("connection refused"), "{}", e);
        assert!(svc.check_merkle_root(50, &"ff".repeat(32)).is_err());
        assert_eq!(svc.block_hash_at_height(50), None);
        assert_eq!(svc.header_hex_at_height(50), None);
    }

    #[tokio::test]
    async fn above_the_tip_is_sync_pending_not_a_tracker_question() {
        let (_, g) = fixture();
        let (c, _) = lite_chain_100_103(&g);
        let src = Arc::new(FakeSource::new("a", &[(104, mine([8; 32], 104, 0x207f_ffff))]));
        let svc = lite_service(c, vec![src.clone()], 1);
        let e = svc.ensure_header(104).await.unwrap_err();
        assert!(e.contains("sync pending"), "{}", e);
        assert_eq!(src.calls.load(Ordering::SeqCst), 0, "the window, not the trackers, own heights at and above the anchor");
        assert!(svc.check_merkle_root(104, &"ff".repeat(32)).is_err());
    }

    #[tokio::test]
    async fn without_trackers_a_height_below_the_anchor_cannot_be_answered() {
        let (_, g) = fixture();
        let (c, _) = lite_chain_100_103(&g);
        let svc = HeaderService { chain: Mutex::new(c), trackers: None };
        let e = svc.ensure_header(50).await.unwrap_err();
        assert!(e.contains("trackers"), "{}", e);
        assert!(svc.check_merkle_root(50, &"ff".repeat(32)).is_err());
        assert_eq!(svc.ensure_header(101).await, Ok(()), "the window still answers");
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

    /// Live: the whole public teratestnet chain, verified under the real rules (DAA, 20-minute
    /// minimum difficulty, checkpoints). `HODOS_CHAINTRACKS_URL` overrides the default server.
    /// `cargo test --lib header_sync_live_ttn -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn header_sync_live_ttn() {
        let base = std::env::var(crate::chain_mode::ENV_CHAINTRACKS)
            .unwrap_or_else(|_| "https://arcade-ttn-us-1.bsvblockchain.tech/chaintracks/v2".into());
        let src = ChaintracksSource(ChaintracksProvider::new(reqwest::Client::new(), base));
        let chain = Mutex::new(HeaderChain::new(Params::teratestnet()));
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        let t0 = std::time::Instant::now();
        let r = sync_once(&chain, &src, now, &mut |_| {}).await.expect("sync");
        println!("synced {} headers to {:?} in {:?}", r.added, r.tip_height, t0.elapsed());
        let c = chain.lock().unwrap();
        for (h, hash) in Params::teratestnet().checkpoints {
            assert_eq!(&c.header_at_height(*h).expect("checkpoint height").hash, hash);
        }
        assert!(r.tip_height.unwrap() > 36_000);
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
    fn cwi_accessors_come_from_the_verified_chain_only() {
        let (mut c, g) = fixture();
        let svc0 = HeaderService { chain: Mutex::new(HeaderChain::new(crate::header_chain::Params::regtest())), trackers: None };
        assert_eq!(svc0.tip_height(), None, "unsynced chain has no height to report");
        assert_eq!(svc0.header_hex_at_height(0), None);

        let a1 = mine(g.hash(), 1, 0x207f_ffff);
        c.add_header(a1.clone(), NOW).unwrap();
        let svc = HeaderService { chain: Mutex::new(c), trackers: None };
        assert_eq!(svc.tip_height(), Some(1));
        assert_eq!(svc.header_hex_at_height(1), Some(a1.to_hex()));
        assert_eq!(svc.header_hex_at_height(2), None);
    }

    #[test]
    fn block_hash_at_height_is_the_verified_headers_own_hash() {
        let (mut c, g) = fixture();
        let a1 = mine(g.hash(), 1, 0x207f_ffff);
        c.add_header(a1.clone(), NOW).unwrap();
        let svc = HeaderService { chain: Mutex::new(c), trackers: None };
        assert_eq!(svc.block_hash_at_height(1), Some(a1.hash_hex()));
        assert_eq!(svc.block_hash_at_height(0), Some(g.hash_hex()));
        assert_eq!(svc.block_hash_at_height(2), None);
    }

    #[test]
    fn a_root_from_an_equal_work_competing_branch_is_not_a_verdict() {
        // Two blocks at height 1 with equal work: we stay on the first. A proof for the other
        // one is neither valid nor bogus yet, so it must be "cannot say" (held), not "false"
        // (dropped); it becomes a verdict once one branch wins.
        let (mut c, g) = fixture();
        let a1 = mine(g.hash(), 1, 0x207f_ffff);
        let b1 = mine(g.hash(), 2, 0x207f_ffff);
        assert_ne!(a1.merkle_root_hex(), b1.merkle_root_hex(), "fixture must give distinct roots");
        c.add_header(a1.clone(), NOW).unwrap();
        assert_eq!(c.add_header(b1.clone(), NOW).unwrap(), AddOutcome::Fork);
        let svc = HeaderService { chain: Mutex::new(c), trackers: None };
        assert_eq!(svc.check_merkle_root(1, &a1.merkle_root_hex()), Ok(true));
        assert!(svc.check_merkle_root(1, &b1.merkle_root_hex()).is_err(), "competing branch: cannot say yet");
        assert_eq!(svc.check_merkle_root(1, &"ff".repeat(32)), Ok(false), "a root no branch has is still false");

        // Once the other branch pulls ahead, the loser's proofs are plainly orphaned: a verdict.
        let b2 = mine(b1.hash(), 3, 0x207f_ffff);
        assert!(matches!(svc.chain.lock().unwrap().add_header(b2, NOW).unwrap(), AddOutcome::Reorg { .. }));
        assert_eq!(svc.check_merkle_root(1, &a1.merkle_root_hex()), Ok(false), "orphaned branch: not on the chain");
        assert_eq!(svc.check_merkle_root(1, &b1.merkle_root_hex()), Ok(true));
    }

    #[test]
    fn check_merkle_root_distinguishes_unknown_from_mismatch() {
        let (c, g) = fixture();
        let svc = HeaderService { chain: Mutex::new(c), trackers: None };
        assert_eq!(svc.check_merkle_root(0, &g.merkle_root_hex()), Ok(true));
        assert_eq!(svc.check_merkle_root(0, &"ff".repeat(32)), Ok(false));
        assert!(svc.check_merkle_root(5, &g.merkle_root_hex()).is_err());
    }
}
