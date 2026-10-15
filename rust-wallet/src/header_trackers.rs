//! Header trackers (spv-lite): headers below the verified window, from the trackers that know them.
//!
//! A lite chain (`header_chain::Params::anchor` above the genesis) verifies only a window at the
//! tip. A merkle proof at an older height still needs that height's header, and the wallet has no
//! chain of its own to check it against, so it asks every configured tracker and accepts a header
//! only when at least `min_agreeing` of them return the same bytes and none of them disagrees.
//! Every answer must hash to the hash it claims; one that contradicts a pinned checkpoint is
//! discarded as being on the wrong chain (or a reset server), never fatal by itself. An accepted
//! header is cached in memory for the life of the process. `Err` is "cannot say", never a
//! verdict: callers hold the proof and ask again later.
//!
//! Trust, stated plainly: with `min_agreeing` of 2 a forged old header needs the trackers'
//! operators to collude, and the verified window still catches a tracker that lies about recent
//! blocks. On a test network a single tracker is allowed (`min_agreeing` of 1) because no second
//! one exists; that is the one-trusted-tracker model of the SDK `ChainTracker` / wallet-toolbox,
//! and the wallet says so at startup. Mainnet must keep 2.
//!
//! A *confirmer* (`HashConfirmer`) is a source that serves a block by hash only, so it cannot be
//! a tracker (no parent hash, no header to check); once the trackers agree it is asked at which
//! height it has that hash. JungleBus is one on mainnet. The hash it is asked about is not a
//! cheap claim: it is the double-SHA256 of 80 bytes two trackers served and the wallet rebuilt,
//! so a third party agreeing on the height is a third independent reading of the same block.

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::header_chain::Header;
use crate::services::providers::{ChaintracksProvider, JungleBusProvider, WhatsOnChainProvider};
use crate::services::{BlockHeader, BlockKey, IndexerError, IndexerProvider};

/// Sources asked at once for one height. Chaintracks is cheap; a rate-limited public source
/// (WhatsOnChain, on mainnet) is not, and a BEEF with many BUMPs must not burst.
const IN_FLIGHT: usize = 2;

/// One tracker's answer for a height.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub source: String,
    /// 80-byte header, hex.
    pub header_hex: String,
    /// The block hash the source reported for it (display hex).
    pub block_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgreementError {
    /// Fewer usable answers than needed: sources failed, abstained or were discarded.
    InsufficientAnswers { height: u32, usable: usize, need: usize },
    /// Usable answers name different headers.
    Disagree { height: u32, hashes: Vec<String> },
    /// A confirmer places the agreed header's hash at another height, or does not know it.
    Unconfirmed { height: u32, hash: String, confirmer: String, found: Option<u32> },
}

impl std::fmt::Display for AgreementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgreementError::InsufficientAnswers { height, usable, need } => {
                write!(f, "header trackers at height {}: {} usable answer(s), {} needed", height, usable, need)
            }
            AgreementError::Disagree { height, hashes } => {
                write!(f, "header trackers at height {}: sources disagree ({})", height, hashes.join(", "))
            }
            AgreementError::Unconfirmed { height, hash, confirmer, found } => {
                write!(f, "header trackers at height {}: {} does not place {} at this height (", height, confirmer, hash)?;
                match found {
                    Some(h) => write!(f, "it has it at {})", h),
                    None => write!(f, "it does not know the hash)"),
                }
            }
        }
    }
}

/// A tracker that can be asked for a header.
#[async_trait]
pub trait TrackerSource: Send + Sync {
    fn name(&self) -> &str;
    /// `Ok(None)` = the source does not have this height (it abstains).
    async fn header_at(&self, height: u32) -> Result<Option<Answer>, String>;
}

/// A source asked to confirm that a block hash sits at a height, rather than for the header:
/// JungleBus serves a block by hash only. Consulted after the trackers agree; see `confirm`.
#[async_trait]
pub trait HashConfirmer: Send + Sync {
    fn name(&self) -> &str;
    /// The height the confirmer has this block hash at. `Ok(None)` = it does not know the hash.
    async fn height_of(&self, block_hash: &str) -> Result<Option<u32>, String>;
}

/// Judge the confirmers' verdicts on the header the trackers agreed at `height`. Pure. A
/// confirmer that places the hash at this height confirms; one that places it elsewhere or does
/// not know it contradicts (the trackers serve only heights far below the tip, so "not indexed
/// yet" is not a reading); one that failed abstains, with a warning. No confirmers = confirmed.
pub fn confirm(height: u32, hash: &str, verdicts: &[(String, Result<Option<u32>, String>)]) -> Result<(), AgreementError> {
    for (confirmer, verdict) in verdicts {
        match verdict {
            Ok(Some(h)) if *h == height => {}
            Ok(found) => {
                return Err(AgreementError::Unconfirmed {
                    height,
                    hash: hash.to_string(),
                    confirmer: confirmer.clone(),
                    found: *found,
                })
            }
            Err(e) => log::warn!(
                "   header trackers: confirmer {} could not be asked about {} at {}: {} (accepted on the trackers alone)",
                confirmer, hash, height, e
            ),
        }
    }
    Ok(())
}

/// Decide one height from the answers that came back. Pure. An answer is usable when its bytes
/// parse, hash to the hash it claims, and (where one is pinned) equal the checkpoint; usable
/// answers must all name the same header and there must be at least `min_agreeing` of them.
pub fn decide(height: u32, answers: &[Answer], checkpoint: Option<&str>, min_agreeing: usize) -> Result<Header, AgreementError> {
    let need = min_agreeing.max(1);
    let mut usable: Vec<(String, Header)> = Vec::with_capacity(answers.len());
    for a in answers {
        let header = match Header::from_hex(&a.header_hex) {
            Ok(h) => h,
            Err(e) => {
                log::warn!("   header trackers: {} at {} discarded: {}", a.source, height, e);
                continue;
            }
        };
        let hash = header.hash_hex();
        if !hash.eq_ignore_ascii_case(&a.block_hash) {
            log::warn!(
                "   header trackers: {} at {} discarded: bytes hash to {} but it claimed {}",
                a.source, height, hash, a.block_hash
            );
            continue;
        }
        if let Some(cp) = checkpoint {
            if !hash.eq_ignore_ascii_case(cp) {
                log::warn!(
                    "   header trackers: {} at {} discarded: {} contradicts the pinned checkpoint {} (wrong chain, or a reset server)",
                    a.source, height, hash, cp
                );
                continue;
            }
        }
        usable.push((hash, header));
    }
    let distinct: BTreeSet<&str> = usable.iter().map(|(h, _)| h.as_str()).collect();
    if distinct.len() > 1 {
        return Err(AgreementError::Disagree { height, hashes: distinct.into_iter().map(String::from).collect() });
    }
    if usable.len() < need {
        return Err(AgreementError::InsufficientAnswers { height, usable: usable.len(), need });
    }
    Ok(usable.swap_remove(0).1)
}

pub struct HeaderTrackers {
    sources: Vec<Arc<dyn TrackerSource>>,
    confirmers: Vec<Arc<dyn HashConfirmer>>,
    min_agreeing: usize,
    checkpoints: &'static [(u32, &'static str)],
    cache: Mutex<HashMap<u32, Header>>,
}

impl HeaderTrackers {
    pub fn new(sources: Vec<Arc<dyn TrackerSource>>, min_agreeing: usize, checkpoints: &'static [(u32, &'static str)]) -> Self {
        HeaderTrackers {
            sources,
            confirmers: Vec::new(),
            min_agreeing: min_agreeing.max(1),
            checkpoints,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Confirmers asked, after the trackers agree, whether the hash sits at the height.
    pub fn with_confirmers(mut self, confirmers: Vec<Arc<dyn HashConfirmer>>) -> Self {
        self.confirmers = confirmers;
        self
    }

    pub fn source_names(&self) -> Vec<String> {
        self.sources.iter().map(|s| s.name().to_string()).collect()
    }

    pub fn confirmer_names(&self) -> Vec<String> {
        self.confirmers.iter().map(|c| c.name().to_string()).collect()
    }

    pub fn min_agreeing(&self) -> usize {
        self.min_agreeing
    }

    /// The accepted header at `height`, if it has been fetched.
    pub fn cached(&self, height: u32) -> Option<Header> {
        self.cache.lock().unwrap().get(&height).cloned()
    }

    /// The accepted header at `height`: from the cache, else from the sources (at most
    /// `IN_FLIGHT` asked at once). `Err` = cannot say; nothing is cached then.
    pub async fn header_at(&self, height: u32) -> Result<Header, String> {
        if let Some(h) = self.cached(height) {
            return Ok(h);
        }
        let mut answers: Vec<Answer> = Vec::new();
        let mut failures: Vec<String> = Vec::new();
        for pair in self.sources.chunks(IN_FLIGHT) {
            let results: Vec<(&str, Result<Option<Answer>, String>)> = match pair {
                [a] => vec![(a.name(), a.header_at(height).await)],
                [a, b] => {
                    let (ra, rb) = tokio::join!(a.header_at(height), b.header_at(height));
                    vec![(a.name(), ra), (b.name(), rb)]
                }
                _ => unreachable!("chunks of {}", IN_FLIGHT),
            };
            for (name, r) in results {
                match r {
                    Ok(Some(a)) => answers.push(a),
                    Ok(None) => log::debug!("   header trackers: {} has no header at {}", name, height),
                    Err(e) => failures.push(format!("{}: {}", name, e)),
                }
            }
        }
        let checkpoint = self.checkpoints.iter().find(|(h, _)| *h == height).map(|(_, hash)| *hash);
        let agreed = decide(height, &answers, checkpoint, self.min_agreeing);
        let confirmed = match &agreed {
            Ok(header) => {
                let hash = header.hash_hex();
                let mut verdicts = Vec::with_capacity(self.confirmers.len());
                for c in &self.confirmers {
                    verdicts.push((c.name().to_string(), c.height_of(&hash).await));
                }
                confirm(height, &hash, &verdicts)
            }
            Err(_) => Ok(()),
        };
        match agreed.and_then(|header| confirmed.map(|()| header)) {
            Ok(header) => {
                self.cache.lock().unwrap().insert(height, header.clone());
                Ok(header)
            }
            Err(e) if failures.is_empty() => Err(e.to_string()),
            Err(e) => Err(format!("{}; source failure(s): {}", e, failures.join("; "))),
        }
    }

    /// Make every height answerable, each fetched at most once (duplicates and cached heights are
    /// skipped). The first failure is returned after the rest were still attempted.
    pub async fn ensure(&self, heights: &[u32]) -> Result<(), String> {
        let wanted: BTreeSet<u32> = heights.iter().copied().collect();
        let mut first_err = None;
        for h in wanted {
            if let Err(e) = self.header_at(h).await {
                first_err.get_or_insert(e);
            }
        }
        match first_err {
            None => Ok(()),
            Some(e) => Err(e),
        }
    }
}

/// What one tracker's `get_block_header` result means to the agreement: a header is an answer,
/// a 404 is an abstention, anything else is a failure to report.
fn answer_from(name: &str, result: Result<BlockHeader, IndexerError>) -> Result<Option<Answer>, String> {
    match result {
        Ok(h) => Ok(Some(Answer { source: name.to_string(), header_hex: h.header_hex, block_hash: h.block_hash })),
        Err(IndexerError::NotFound) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// WhatsOnChain as a tracker (mainnet only: the provider's base URL is the mainnet API). A header
/// below the verified window is the one thing spv mode asks a public indexer for.
pub struct WhatsOnChainTracker {
    provider: WhatsOnChainProvider,
}

impl WhatsOnChainTracker {
    pub fn new(client: reqwest::Client) -> Self {
        WhatsOnChainTracker { provider: WhatsOnChainProvider::new(client) }
    }
}

#[async_trait]
impl TrackerSource for WhatsOnChainTracker {
    fn name(&self) -> &str {
        "whatsonchain"
    }

    async fn header_at(&self, height: u32) -> Result<Option<Answer>, String> {
        answer_from(self.name(), self.provider.get_block_header(BlockKey::Height(height)).await)
    }
}

/// An Arcade chaintracks server as a tracker.
pub struct ChaintracksTracker {
    name: String,
    provider: ChaintracksProvider,
}

impl ChaintracksTracker {
    pub fn new(client: reqwest::Client, base_url: &str) -> Self {
        let base = base_url.trim().trim_end_matches('/').to_string();
        ChaintracksTracker { name: base.clone(), provider: ChaintracksProvider::new(client, base) }
    }
}

#[async_trait]
impl TrackerSource for ChaintracksTracker {
    fn name(&self) -> &str {
        &self.name
    }

    async fn header_at(&self, height: u32) -> Result<Option<Answer>, String> {
        answer_from(&self.name, self.provider.get_block_header(BlockKey::Height(height)).await)
    }
}

/// What a by-hash `get_block_header` result means to a confirmer: the height it reports is its
/// verdict, a 404 is "does not know the hash", anything else is a failure to report.
fn height_from(result: Result<BlockHeader, IndexerError>) -> Result<Option<u32>, String> {
    match result {
        Ok(h) => Ok(Some(h.height)),
        Err(IndexerError::NotFound) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// JungleBus as a confirmer (mainnet only: the provider's base URL is the mainnet API). It serves
/// a block by hash, without the parent hash, so it cannot be a tracker; it can say at which
/// height it has the hash the trackers agreed on.
pub struct JungleBusConfirmer {
    provider: JungleBusProvider,
}

impl JungleBusConfirmer {
    pub fn new(client: reqwest::Client) -> Self {
        JungleBusConfirmer { provider: JungleBusProvider::new(client) }
    }
}

#[async_trait]
impl HashConfirmer for JungleBusConfirmer {
    fn name(&self) -> &str {
        "junglebus"
    }

    async fn height_of(&self, block_hash: &str) -> Result<Option<u32>, String> {
        height_from(self.provider.get_block_header(BlockKey::Hash(block_hash.to_string())).await)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::header_chain::tests::mine;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Shared by fake sources to measure how many are in flight at once.
    #[derive(Default)]
    pub(crate) struct InFlight {
        now: AtomicUsize,
        pub max: AtomicUsize,
    }

    /// A fake tracker: serves the headers it was given, counts calls, can fail or dawdle.
    pub(crate) struct FakeSource {
        pub name: String,
        pub headers: HashMap<u32, Header>,
        pub calls: AtomicUsize,
        pub fail: Option<String>,
        pub delay_ms: u64,
        pub in_flight: Option<Arc<InFlight>>,
    }

    impl FakeSource {
        pub(crate) fn new(name: &str, headers: &[(u32, Header)]) -> Self {
            FakeSource {
                name: name.to_string(),
                headers: headers.iter().cloned().collect(),
                calls: AtomicUsize::new(0),
                fail: None,
                delay_ms: 0,
                in_flight: None,
            }
        }
        pub(crate) fn failing(name: &str, err: &str) -> Self {
            let mut f = FakeSource::new(name, &[]);
            f.fail = Some(err.to_string());
            f
        }
    }

    #[async_trait]
    impl TrackerSource for FakeSource {
        fn name(&self) -> &str {
            &self.name
        }
        async fn header_at(&self, height: u32) -> Result<Option<Answer>, String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if let Some(f) = &self.in_flight {
                let n = f.now.fetch_add(1, Ordering::SeqCst) + 1;
                f.max.fetch_max(n, Ordering::SeqCst);
            }
            if self.delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
            }
            if let Some(f) = &self.in_flight {
                f.now.fetch_sub(1, Ordering::SeqCst);
            }
            if let Some(e) = &self.fail {
                return Err(e.clone());
            }
            Ok(self.headers.get(&height).map(|h| answer(&self.name, h)))
        }
    }

    /// A confirmer that knows some hashes' heights, or fails.
    pub(crate) struct FakeConfirmer {
        name: String,
        heights: HashMap<String, u32>,
        fail: Option<String>,
        pub calls: AtomicUsize,
    }

    impl FakeConfirmer {
        pub(crate) fn knowing(name: &str, at: &[(&str, u32)]) -> Self {
            FakeConfirmer {
                name: name.to_string(),
                heights: at.iter().map(|(h, n)| (h.to_string(), *n)).collect(),
                fail: None,
                calls: AtomicUsize::new(0),
            }
        }
        pub(crate) fn failing(name: &str, err: &str) -> Self {
            let mut c = FakeConfirmer::knowing(name, &[]);
            c.fail = Some(err.to_string());
            c
        }
    }

    #[async_trait]
    impl HashConfirmer for FakeConfirmer {
        fn name(&self) -> &str {
            &self.name
        }
        async fn height_of(&self, block_hash: &str) -> Result<Option<u32>, String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if let Some(e) = &self.fail {
                return Err(e.clone());
            }
            Ok(self.heights.get(block_hash).copied())
        }
    }

    fn hdr(salt: u32) -> Header {
        mine([0; 32], salt, 0x207f_ffff)
    }

    pub(crate) fn answer(source: &str, h: &Header) -> Answer {
        Answer { source: source.to_string(), header_hex: h.to_hex(), block_hash: h.hash_hex() }
    }

    fn leak(s: String) -> &'static str {
        Box::leak(s.into_boxed_str())
    }

    #[test]
    fn a_trackers_result_maps_to_answer_abstention_or_failure() {
        let h = hdr(1);
        let bh = BlockHeader { block_hash: h.hash_hex(), height: 5, header_hex: h.to_hex() };
        assert_eq!(answer_from("x", Ok(bh)), Ok(Some(answer("x", &h))));
        assert_eq!(answer_from("x", Err(IndexerError::NotFound)), Ok(None), "404 abstains");
        let e = answer_from("x", Err(IndexerError::Transport("refused".into()))).unwrap_err();
        assert!(e.contains("refused"), "{}", e);
        assert_eq!(WhatsOnChainTracker::new(reqwest::Client::new()).name(), "whatsonchain");
    }

    #[test]
    fn two_identical_answers_agree() {
        let h = hdr(1);
        assert_eq!(decide(5, &[answer("a", &h), answer("b", &h)], None, 2), Ok(h));
    }

    #[test]
    fn any_disagreement_among_usable_answers_is_an_error() {
        let (h, other) = (hdr(1), hdr(2));
        let r = decide(5, &[answer("a", &h), answer("b", &h), answer("c", &other)], None, 2);
        match r {
            Err(AgreementError::Disagree { height: 5, hashes }) => {
                assert_eq!(hashes.len(), 2);
                assert!(hashes.contains(&h.hash_hex()) && hashes.contains(&other.hash_hex()));
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn one_answer_is_insufficient_at_two_and_enough_at_one() {
        let h = hdr(1);
        assert_eq!(
            decide(5, &[answer("a", &h)], None, 2),
            Err(AgreementError::InsufficientAnswers { height: 5, usable: 1, need: 2 })
        );
        assert_eq!(decide(5, &[answer("a", &h)], None, 1), Ok(h.clone()));
        assert_eq!(decide(5, &[], None, 1), Err(AgreementError::InsufficientAnswers { height: 5, usable: 0, need: 1 }));
        assert_eq!(decide(5, &[answer("a", &h)], None, 0), Ok(h), "a minimum of zero still needs one answer");
    }

    #[test]
    fn an_answer_whose_bytes_do_not_hash_to_its_claim_is_discarded() {
        let (h, other) = (hdr(1), hdr(2));
        let mut bad = answer("c", &other);
        bad.block_hash = h.hash_hex(); // claims h, carries other
        assert_eq!(decide(5, &[answer("a", &h), answer("b", &h), bad.clone()], None, 2), Ok(h));
        assert_eq!(decide(5, &[bad], None, 1), Err(AgreementError::InsufficientAnswers { height: 5, usable: 0, need: 1 }));
    }

    #[test]
    fn an_answer_contradicting_the_pinned_checkpoint_is_discarded() {
        let (h, other) = (hdr(1), hdr(2));
        let cp = leak(h.hash_hex());
        assert_eq!(decide(5, &[answer("a", &h), answer("b", &h), answer("c", &other)], Some(cp), 2), Ok(h.clone()));
        assert_eq!(
            decide(5, &[answer("a", &h), answer("c", &other)], Some(cp), 2),
            Err(AgreementError::InsufficientAnswers { height: 5, usable: 1, need: 2 })
        );
        // Every source on the wrong chain: never an answer, whatever they agree on.
        assert_eq!(
            decide(5, &[answer("a", &other), answer("b", &other)], Some(cp), 1),
            Err(AgreementError::InsufficientAnswers { height: 5, usable: 0, need: 1 })
        );
    }

    #[test]
    fn unparseable_hex_is_discarded() {
        let h = hdr(1);
        let junk = Answer { source: "c".into(), header_hex: "zz".into(), block_hash: h.hash_hex() };
        assert_eq!(decide(5, &[answer("a", &h), junk.clone()], None, 1), Ok(h));
        assert!(matches!(decide(5, &[junk], None, 1), Err(AgreementError::InsufficientAnswers { usable: 0, .. })));
    }

    #[tokio::test]
    async fn each_source_is_asked_once_per_height_and_the_answer_is_cached() {
        let h = hdr(1);
        let a = Arc::new(FakeSource::new("a", &[(5, h.clone())]));
        let b = Arc::new(FakeSource::new("b", &[(5, h.clone())]));
        let q = HeaderTrackers::new(vec![a.clone(), b.clone()], 2, &[]);
        assert_eq!(q.cached(5), None);
        assert_eq!(q.ensure(&[5, 5, 5]).await, Ok(()));
        assert_eq!(q.header_at(5).await, Ok(h.clone()));
        assert_eq!(q.cached(5), Some(h));
        assert_eq!((a.calls.load(Ordering::SeqCst), b.calls.load(Ordering::SeqCst)), (1, 1));
    }

    #[tokio::test]
    async fn at_most_two_sources_are_in_flight() {
        let h = hdr(1);
        let flight = Arc::new(InFlight::default());
        let sources: Vec<Arc<dyn TrackerSource>> = (0..3)
            .map(|i| {
                let mut s = FakeSource::new(&format!("s{}", i), &[(5, h.clone())]);
                s.delay_ms = 30;
                s.in_flight = Some(flight.clone());
                Arc::new(s) as Arc<dyn TrackerSource>
            })
            .collect();
        let q = HeaderTrackers::new(sources, 2, &[]);
        assert_eq!(q.header_at(5).await, Ok(h));
        let max = flight.max.load(Ordering::SeqCst);
        assert!(max == 2, "max in flight {} (sources are asked two at a time)", max);
    }

    #[tokio::test]
    async fn abstaining_and_failing_sources_are_reported_not_counted() {
        let h = hdr(1);
        let a = Arc::new(FakeSource::new("a", &[])); // abstains
        let b = Arc::new(FakeSource::new("b", &[(5, h.clone())]));
        let c = Arc::new(FakeSource::failing("c", "boom"));
        let strict = HeaderTrackers::new(vec![a.clone(), b.clone(), c.clone()], 2, &[]);
        let e = strict.header_at(5).await.unwrap_err();
        assert!(e.contains("1 usable") && e.contains("c: boom"), "{}", e);
        assert_eq!(strict.cached(5), None, "nothing cached on failure");
        let lenient = HeaderTrackers::new(vec![a, b, c], 1, &[]);
        assert_eq!(lenient.header_at(5).await, Ok(h));
    }

    #[tokio::test]
    async fn a_checkpoint_at_the_height_is_enforced_by_the_trackers() {
        let (h, other) = (hdr(1), hdr(2));
        let cps: &'static [(u32, &'static str)] = Box::leak(vec![(5u32, leak(h.hash_hex()))].into_boxed_slice());
        let wrong = Arc::new(FakeSource::new("w", &[(5, other.clone())]));
        let q = HeaderTrackers::new(vec![wrong], 1, cps);
        assert!(q.header_at(5).await.is_err(), "the only source is on the wrong chain");
        let right = Arc::new(FakeSource::new("r", &[(5, h.clone())]));
        let q = HeaderTrackers::new(vec![right], 1, cps);
        assert_eq!(q.header_at(5).await, Ok(h));
    }

    /// A chaintracks server that knows height 5 only.
    async fn tiny_chaintracks() -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = listener.accept().await else { break };
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 2048];
                    let n = sock.read(&mut buf).await.unwrap_or(0);
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let path = req.lines().next().and_then(|l| l.split(' ').nth(1)).unwrap_or("").to_string();
                    let (status, body) = if path.ends_with("/header/height/5") {
                        ("200 OK", r#"{"version":536870912,"previousHash":"231deda25164376fdb28cc5490770f35f02ad27919e48903a74bc0ae88f00c0c","merkleRoot":"9d34d4193f6fa3a62e872d4832cd4b16a579541b05daeadee9cbd12a406f1146","time":1791033023,"bits":545259519,"nonce":3,"height":5,"hash":"5ee1c44ffb4c19f3526a5c4fdd9049234123609d1252cb7cf7d5efac215eb6a0"}"#)
                    } else {
                        ("404 Not Found", r#"{"error":"not found"}"#)
                    };
                    let resp = format!("HTTP/1.1 {}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}", status, body.len(), body);
                    let _ = sock.write_all(resp.as_bytes()).await;
                });
            }
        });
        format!("http://{}/chaintracks/v2", addr)
    }

    #[tokio::test]
    async fn chaintracks_source_answers_on_200_and_abstains_on_404() {
        let base = tiny_chaintracks().await;
        let src = ChaintracksTracker::new(reqwest::Client::new(), &format!("{}/", base));
        assert_eq!(src.name(), base, "trailing slash trimmed");
        let a = src.header_at(5).await.unwrap().expect("height 5 is served");
        assert_eq!(a.block_hash, "5ee1c44ffb4c19f3526a5c4fdd9049234123609d1252cb7cf7d5efac215eb6a0");
        assert_eq!(Header::from_hex(&a.header_hex).unwrap().hash_hex(), a.block_hash);
        assert_eq!(src.header_at(6).await, Ok(None), "404 is an abstention");
        let down = ChaintracksTracker::new(reqwest::Client::new(), "http://127.0.0.1:1");
        assert!(down.header_at(5).await.is_err(), "a refused connection is a failure, not an abstention");
    }

    #[test]
    fn confirm_is_pure_over_the_confirmers_verdicts() {
        let hash = hdr(1).hash_hex();
        let v = |name: &str, r: Result<Option<u32>, &str>| (name.to_string(), r.map_err(String::from));
        assert_eq!(confirm(5, &hash, &[]), Ok(()), "no confirmers configured");
        assert_eq!(confirm(5, &hash, &[v("j", Ok(Some(5)))]), Ok(()));
        assert_eq!(
            confirm(5, &hash, &[v("j", Ok(Some(6)))]),
            Err(AgreementError::Unconfirmed { height: 5, hash: hash.clone(), confirmer: "j".into(), found: Some(6) })
        );
        assert_eq!(
            confirm(5, &hash, &[v("j", Ok(None))]),
            Err(AgreementError::Unconfirmed { height: 5, hash: hash.clone(), confirmer: "j".into(), found: None }),
            "a hash the confirmer does not know is a contradiction, not an abstention"
        );
        assert_eq!(confirm(5, &hash, &[v("j", Err("down"))]), Ok(()), "a failed confirmer abstains");
        assert!(confirm(5, &hash, &[v("j", Ok(Some(5))), v("k", Ok(Some(4)))]).is_err(), "one contradiction is enough");
        let shown = confirm(5, &hash, &[v("j", Ok(Some(6)))]).unwrap_err().to_string();
        assert!(shown.contains("j does not place") && shown.contains("at 6"), "{}", shown);
    }

    #[tokio::test]
    async fn the_confirmer_is_asked_about_the_agreed_hash_and_a_contradiction_caches_nothing() {
        let h = hdr(1);
        let a = Arc::new(FakeSource::new("a", &[(5, h.clone())]));
        let b = Arc::new(FakeSource::new("b", &[(5, h.clone())]));
        let trackers = || HeaderTrackers::new(vec![a.clone(), b.clone()], 2, &[]);

        let yes = Arc::new(FakeConfirmer::knowing("yes", &[(&h.hash_hex(), 5)]));
        let q = trackers().with_confirmers(vec![yes.clone()]);
        assert_eq!(q.confirmer_names(), vec!["yes".to_string()]);
        assert_eq!(q.header_at(5).await, Ok(h.clone()));
        assert_eq!(q.cached(5), Some(h.clone()));
        assert_eq!(yes.calls.load(Ordering::SeqCst), 1);
        assert_eq!(q.header_at(5).await, Ok(h.clone()));
        assert_eq!(yes.calls.load(Ordering::SeqCst), 1, "a cached header is not re-confirmed");

        let elsewhere = Arc::new(FakeConfirmer::knowing("elsewhere", &[(&h.hash_hex(), 6)]));
        let q = trackers().with_confirmers(vec![elsewhere]);
        let e = q.header_at(5).await.unwrap_err();
        assert!(e.contains("elsewhere") && e.contains("at 6"), "{}", e);
        assert_eq!(q.cached(5), None, "nothing cached when the confirmer contradicts");

        let unknown = Arc::new(FakeConfirmer::knowing("unknown", &[]));
        let q = trackers().with_confirmers(vec![unknown]);
        assert!(q.header_at(5).await.unwrap_err().contains("does not know"));
        assert_eq!(q.cached(5), None);

        let down = Arc::new(FakeConfirmer::failing("down", "refused"));
        let q = trackers().with_confirmers(vec![down]);
        assert_eq!(q.header_at(5).await, Ok(h.clone()), "a failed confirmer abstains");
        assert_eq!(q.cached(5), Some(h));
    }

    #[tokio::test]
    async fn the_confirmer_is_not_asked_when_the_trackers_do_not_agree() {
        let (h, other) = (hdr(1), hdr(2));
        let a = Arc::new(FakeSource::new("a", &[(5, h.clone())]));
        let b = Arc::new(FakeSource::new("b", &[(5, other)]));
        let c = Arc::new(FakeConfirmer::knowing("c", &[(&h.hash_hex(), 5)]));
        let q = HeaderTrackers::new(vec![a, b], 2, &[]).with_confirmers(vec![c.clone()]);
        assert!(q.header_at(5).await.is_err());
        assert_eq!(c.calls.load(Ordering::SeqCst), 0, "nothing to confirm");
    }

    #[test]
    fn a_junglebus_result_maps_to_a_height_abstention_or_failure() {
        let bh = BlockHeader { block_hash: "ab".into(), height: 945_000, header_hex: String::new() };
        assert_eq!(height_from(Ok(bh)), Ok(Some(945_000)));
        assert_eq!(height_from(Err(IndexerError::NotFound)), Ok(None), "404: it does not know the hash");
        let e = height_from(Err(IndexerError::Transport("refused".into()))).unwrap_err();
        assert!(e.contains("refused"), "{}", e);
        assert_eq!(JungleBusConfirmer::new(reqwest::Client::new()).name(), "junglebus");
    }
}
