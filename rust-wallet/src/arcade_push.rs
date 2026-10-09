//! Arcade push: a Server-Sent Events client that wakes the wallet's proof check the moment
//! Arcade reports a transaction mined (or otherwise changed), instead of waiting for the next
//! poll. spv mode only, and only when `HODOS_ARCADE_SSE_URL` is set (Arcade serves SSE from a
//! separate listener) and `HODOS_ARCADE_PUSH` is not `off`.
//!
//! ⛔ Push is a *wake-up*, never a source of truth. An event does not carry a proof the wallet
//! trusts: it only triggers an immediate header sync and `TaskCheckForProofs`, which fetch the
//! BUMP from Arcade, verify it against the wallet's own header chain, store it and promote the
//! output. `TaskCheckForProofs` keeps polling at its normal cadence regardless, because push
//! cannot be relied on: a tx not registered under this wallet's token produces no events, Arcade's
//! replay after a reconnect is best-effort, and the stream can be up but silent.
//!
//! The callback token scopes which events Arcade sends. It is derived from the wallet's master
//! private key (HMAC-SHA256), so it is stable across restarts (Arcade replays missed events for it)
//! and cannot be guessed from the public identity key.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Duration;

use serde::Deserialize;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

/// Arcade sends `: keepalive` every 15 s; three missed intervals means the stream is not healthy.
pub const HEALTHY_WINDOW_MS: u64 = 45_000;

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

static TOKEN: RwLock<Option<String>> = RwLock::new(None);
static NUDGE: Notify = Notify::const_new();
static CONNECTED: AtomicBool = AtomicBool::new(false);
static LAST_FRAME_MS: AtomicU64 = AtomicU64::new(0);
static PROOF_WAITING: AtomicBool = AtomicBool::new(false);

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn set_token(t: String) {
    *TOKEN.write().unwrap() = Some(t);
}

pub fn token() -> Option<String> {
    TOKEN.read().unwrap().clone()
}

/// Wake the proof driver. A wake-up sent while nothing is waiting is remembered (one permit).
pub fn nudge() {
    NUDGE.notify_one();
}

pub async fn nudged() {
    NUDGE.notified().await;
}

/// The stream is connected and has produced a frame (event or keepalive) recently.
pub fn healthy() -> bool {
    healthy_at(now_ms())
}

pub fn healthy_at(now: u64) -> bool {
    CONNECTED.load(Ordering::SeqCst)
        && now.saturating_sub(LAST_FRAME_MS.load(Ordering::SeqCst)) <= HEALTHY_WINDOW_MS
}

fn mark_connected() {
    CONNECTED.store(true, Ordering::SeqCst);
    LAST_FRAME_MS.store(now_ms(), Ordering::SeqCst);
}

fn mark_frame() {
    LAST_FRAME_MS.store(now_ms(), Ordering::SeqCst);
}

fn mark_disconnected() {
    CONNECTED.store(false, Ordering::SeqCst);
}

/// Set by the proof task when a mined tx could not get a verified proof yet (header chain behind,
/// or Arcade has no BUMP yet); the push driver keeps retrying quickly while it is set.
pub fn set_proof_waiting(v: bool) {
    PROOF_WAITING.store(v, Ordering::SeqCst);
}

pub fn proof_waiting() -> bool {
    PROOF_WAITING.load(Ordering::SeqCst)
}

// ---------------------------------------------------------------------------
// Token
// ---------------------------------------------------------------------------

/// Callback token for this wallet: `HMAC-SHA256(master_private_key, "hodos-arcade-callback-token-v1")`,
/// hex. Deterministic, per wallet, and not derivable from the (shared) identity public key.
pub fn derive_token(master_private_key: &[u8]) -> String {
    hex::encode(crate::crypto::signing::hmac_sha256(
        master_private_key,
        b"hodos-arcade-callback-token-v1",
    ))
}

// ---------------------------------------------------------------------------
// SSE parsing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseItem {
    Event { id: Option<String>, event: String, data: String },
    Comment,
}

/// Incremental `text/event-stream` parser. Feed it arbitrary byte chunks (frames may be split
/// anywhere, including inside a UTF-8 character).
#[derive(Default)]
pub struct SseParser {
    buf: Vec<u8>,
    id: Option<String>,
    event: Option<String>,
    data: Vec<String>,
}

impl SseParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, chunk: &[u8]) -> Vec<SseItem> {
        self.buf.extend_from_slice(chunk);
        let mut out = Vec::new();
        while let Some(pos) = self.buf.iter().position(|b| *b == b'\n') {
            let mut line: Vec<u8> = self.buf.drain(..=pos).collect();
            line.pop(); // \n
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            let line = String::from_utf8_lossy(&line).into_owned();
            if line.is_empty() {
                // dispatch the accumulated event
                if !self.data.is_empty() || self.event.is_some() {
                    out.push(SseItem::Event {
                        id: self.id.clone(),
                        event: self.event.take().unwrap_or_else(|| "message".to_string()),
                        data: self.data.join("\n"),
                    });
                }
                self.data.clear();
                self.event = None;
                continue;
            }
            if line.starts_with(':') {
                out.push(SseItem::Comment);
                continue;
            }
            let (field, value) = match line.split_once(':') {
                Some((f, v)) => (f, v.strip_prefix(' ').unwrap_or(v)),
                None => (line.as_str(), ""),
            };
            match field {
                "id" => self.id = Some(value.to_string()),
                "event" => self.event = Some(value.to_string()),
                "data" => self.data.push(value.to_string()),
                _ => {} // retry: and unknown fields are ignored
            }
        }
        out
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct StatusEvent {
    pub txid: String,
    #[serde(rename = "txStatus")]
    pub tx_status: String,
    #[serde(rename = "blockHeight", default)]
    pub block_height: Option<u64>,
    #[serde(rename = "blockHash", default)]
    pub block_hash: Option<String>,
    /// BUMP (hex) carried by the MINED frame. Held until the wallet can verify it; never trusted.
    #[serde(rename = "merklePath", default)]
    pub merkle_path: Option<String>,
}

/// Most events kept between driver wake-ups; the oldest are dropped first.
pub const EVENT_BUFFER_CAP: usize = 256;

static EVENTS: std::sync::Mutex<Vec<StatusEvent>> = std::sync::Mutex::new(Vec::new());

/// Remember a wake-up event so the driver can read its payload (the MINED merkle path).
pub fn buffer_event(e: &StatusEvent) {
    let mut buf = EVENTS.lock().unwrap_or_else(|p| p.into_inner());
    buf.push(e.clone());
    let excess = buf.len().saturating_sub(EVENT_BUFFER_CAP);
    if excess > 0 {
        buf.drain(..excess); // drop the oldest
    }
}

/// Take (and clear) the buffered events, oldest first.
pub fn take_events() -> Vec<StatusEvent> {
    std::mem::take(&mut *EVENTS.lock().unwrap_or_else(|p| p.into_inner()))
}

pub fn parse_status(data: &str) -> Option<StatusEvent> {
    serde_json::from_str(data).ok()
}

/// Statuses that can change what the proof task would do. Intermediate ones (RECEIVED,
/// SEEN_ON_NETWORK, ...) cannot, so they do not wake it.
pub fn is_wake_status(tx_status: &str) -> bool {
    matches!(
        tx_status,
        "MINED"
            | "IMMUTABLE"
            | "REJECTED"
            | "DOUBLE_SPEND_ATTEMPTED"
            | "SEEN_IN_ORPHAN_MEMPOOL"
            | "MINED_IN_STALE_BLOCK"
    )
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct ClientOptions {
    pub min_backoff: Duration,
    pub max_backoff: Duration,
    /// How often to look for a token while the wallet is still locked / not created.
    pub token_poll: Duration,
    /// A stream that delivers nothing (not even a keepalive) for this long is treated as dead and
    /// reconnected; without it a half-open connection would leave push silent until restart.
    pub idle_timeout: Duration,
}

impl Default for ClientOptions {
    fn default() -> Self {
        ClientOptions {
            min_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            token_poll: Duration::from_secs(5),
            idle_timeout: Duration::from_millis(HEALTHY_WINDOW_MS),
        }
    }
}

/// Exponential backoff: `min`, `2·min`, `4·min`, … capped at `max`.
pub fn backoff(attempt: u32, min: Duration, max: Duration) -> Duration {
    let factor = 1u64.checked_shl(attempt.min(20)).unwrap_or(u64::MAX);
    let ms = (min.as_millis() as u64).saturating_mul(factor);
    Duration::from_millis(ms).min(max)
}

/// Run until `shutdown`: connect to `{base}/events?callbackToken=…`, wake `on_wake` for each
/// status event that matters, reconnect with backoff (sending `Last-Event-ID`) when the stream
/// ends or fails. Never returns an error: every failure just means "try again", and polling
/// covers the gap.
pub async fn run_client<T, W>(
    base: String,
    token_source: T,
    on_wake: W,
    shutdown: CancellationToken,
    opts: ClientOptions,
) where
    T: Fn() -> Option<String>,
    W: Fn(&StatusEvent),
{
    let http = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let mut last_event_id: Option<String> = None;
    let mut attempt: u32 = 0;

    loop {
        if shutdown.is_cancelled() {
            return;
        }
        let Some(token) = token_source() else {
            tokio::select! {
                _ = shutdown.cancelled() => return,
                _ = tokio::time::sleep(opts.token_poll) => continue,
            }
        };

        let mut req = http.get(format!("{}/events", base.trim_end_matches('/'))).query(&[("callbackToken", token.as_str())]);
        if let Some(id) = &last_event_id {
            req = req.header("Last-Event-ID", id.as_str());
        }

        let mut got_frame = false;
        match req.send().await {
            Ok(mut resp) if resp.status().is_success() => {
                mark_connected();
                let mut parser = SseParser::new();
                loop {
                    let chunk = tokio::select! {
                        _ = shutdown.cancelled() => { mark_disconnected(); return; }
                        c = tokio::time::timeout(opts.idle_timeout, resp.chunk()) => c,
                    };
                    let Ok(chunk) = chunk else {
                        log::warn!("arcade push: nothing received for {:?}; reconnecting", opts.idle_timeout);
                        break;
                    };
                    match chunk {
                        Ok(Some(bytes)) => {
                            for item in parser.feed(&bytes) {
                                got_frame = true;
                                mark_frame();
                                if let SseItem::Event { id, event, data } = item {
                                    if let Some(id) = id {
                                        last_event_id = Some(id);
                                    }
                                    if event == "status" {
                                        if let Some(st) = parse_status(&data) {
                                            if is_wake_status(&st.tx_status) {
                                                on_wake(&st);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(None) => break,  // server closed the stream
                        Err(e) => {
                            log::warn!("arcade push: stream error: {}", e);
                            break;
                        }
                    }
                }
            }
            Ok(resp) => log::warn!("arcade push: {} answered {}", base, resp.status()),
            Err(e) => log::warn!("arcade push: cannot connect to {}: {}", base, e),
        }

        mark_disconnected();
        // A connection that delivered something resets the backoff; one that never did backs off.
        attempt = if got_frame { 0 } else { attempt.saturating_add(1) };
        let wait = backoff(attempt, opts.min_backoff, opts.max_backoff);
        tokio::select! {
            _ = shutdown.cancelled() => return,
            _ = tokio::time::sleep(wait) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    // The health flags are process-wide; serialise the tests that touch them.
    static STATE_LOCK: Mutex<()> = Mutex::new(());

    const STATUS_SEEN: &str = "id: 1\nevent: status\ndata: {\"txid\":\"aa\",\"txStatus\":\"SEEN_ON_NETWORK\"}\n\n";
    const STATUS_MINED: &str = "id: 2\nevent: status\ndata: {\"txid\":\"aa\",\"txStatus\":\"MINED\",\"blockHeight\":7,\"merklePath\":\"00\"}\n\n";

    #[test]
    fn parser_reads_the_frames_arcade_actually_sends() {
        let mut p = SseParser::new();
        let items = p.feed(format!("{}{}: keepalive\n\n", STATUS_SEEN, STATUS_MINED).as_bytes());
        assert_eq!(items.len(), 3); // 2 events + the comment line (a blank line after a comment dispatches nothing)
        assert!(matches!(&items[0], SseItem::Event { id: Some(i), event, .. } if i == "1" && event == "status"));
        match &items[1] {
            SseItem::Event { data, .. } => {
                let st = parse_status(data).unwrap();
                assert_eq!((st.txid.as_str(), st.tx_status.as_str(), st.block_height), ("aa", "MINED", Some(7)));
            }
            other => panic!("{:?}", other),
        }
        assert_eq!(items[2], SseItem::Comment);
    }

    #[test]
    fn parser_survives_chunks_split_anywhere_including_mid_utf8() {
        let full = format!("{}: keepalive\n\n{}", STATUS_SEEN, "id: 3\nevent: status\ndata: {\"txid\":\"é\",\"txStatus\":\"MINED\"}\n\n");
        let bytes = full.as_bytes();
        for split in 1..bytes.len() {
            let mut p = SseParser::new();
            let mut got = p.feed(&bytes[..split]);
            got.extend(p.feed(&bytes[split..]));
            let events = got.iter().filter(|i| matches!(i, SseItem::Event { .. })).count();
            assert_eq!(events, 2, "split at {}", split);
        }
    }

    #[test]
    fn parser_handles_crlf_multiline_data_and_ignores_unknown_fields() {
        let mut p = SseParser::new();
        let items = p.feed(b"retry: 3000\r\nid: 9\r\nevent: x\r\ndata: a\r\ndata: b\r\nfoo: bar\r\n\r\n");
        assert_eq!(items, vec![SseItem::Event { id: Some("9".into()), event: "x".into(), data: "a\nb".into() }]);
    }

    #[test]
    fn a_mined_frame_exposes_its_block_hash_and_merkle_path() {
        let st = parse_status(r#"{"txid":"aa","txStatus":"MINED","blockHash":"bb","blockHeight":7,"merklePath":"fd7f01"}"#).unwrap();
        assert_eq!(st.block_hash.as_deref(), Some("bb"));
        assert_eq!(st.merkle_path.as_deref(), Some("fd7f01"));
        assert_eq!(st.block_height, Some(7));
        let seen = parse_status(r#"{"txid":"aa","txStatus":"SEEN_ON_NETWORK"}"#).unwrap();
        assert_eq!((seen.block_hash, seen.merkle_path), (None, None), "intermediate frames carry no proof");
    }

    fn ev(txid: &str, st: &str, mp: Option<&str>) -> StatusEvent {
        StatusEvent { txid: txid.into(), tx_status: st.into(), block_height: None, block_hash: None, merkle_path: mp.map(|s| s.to_string()) }
    }

    #[test]
    fn buffered_events_come_back_oldest_first_and_the_buffer_empties() {
        let _g = STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        take_events(); // start clean
        buffer_event(&ev("a", "MINED", Some("01")));
        buffer_event(&ev("b", "MINED", Some("02")));
        let got = take_events();
        assert_eq!(got.iter().map(|e| e.txid.as_str()).collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(got[1].merkle_path.as_deref(), Some("02"), "payload preserved");
        assert!(take_events().is_empty(), "taking clears it");
    }

    #[test]
    fn the_event_buffer_is_bounded_and_drops_the_oldest() {
        let _g = STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        take_events();
        for i in 0..(EVENT_BUFFER_CAP + 10) {
            buffer_event(&ev(&format!("t{}", i), "MINED", None));
        }
        let got = take_events();
        assert_eq!(got.len(), EVENT_BUFFER_CAP);
        assert_eq!(got.first().unwrap().txid, "t10", "the 10 oldest were dropped");
        assert_eq!(got.last().unwrap().txid, format!("t{}", EVENT_BUFFER_CAP + 9));
    }

    #[test]
    fn only_state_changing_statuses_wake_the_proof_task() {
        for s in ["MINED", "IMMUTABLE", "REJECTED", "DOUBLE_SPEND_ATTEMPTED", "MINED_IN_STALE_BLOCK", "SEEN_IN_ORPHAN_MEMPOOL"] {
            assert!(is_wake_status(s), "{}", s);
        }
        for s in ["RECEIVED", "ACCEPTED_BY_NETWORK", "SEEN_ON_NETWORK", "SEEN_ON_MULTIPLE_NODES", "SENT_TO_NETWORK"] {
            assert!(!is_wake_status(s), "{}", s);
        }
        assert!(parse_status("not json").is_none());
    }

    #[test]
    fn token_is_deterministic_per_key_and_not_the_key_or_its_pubkey() {
        let a = derive_token(&[1u8; 32]);
        assert_eq!(a, derive_token(&[1u8; 32]));
        assert_ne!(a, derive_token(&[2u8; 32]));
        assert_eq!(a.len(), 64);
        assert!(!a.contains(&hex::encode([1u8; 32])));
    }

    #[test]
    fn backoff_doubles_and_caps() {
        let (min, max) = (Duration::from_secs(1), Duration::from_secs(60));
        let secs: Vec<u64> = (0..9).map(|a| backoff(a, min, max).as_secs()).collect();
        assert_eq!(secs, vec![1, 2, 4, 8, 16, 32, 60, 60, 60]);
        assert_eq!(backoff(u32::MAX, min, max), max);
    }

    #[test]
    fn health_needs_a_connection_and_a_recent_frame() {
        let _g = STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        mark_disconnected();
        assert!(!healthy_at(now_ms()));
        mark_connected();
        assert!(healthy_at(now_ms()));
        assert!(!healthy_at(now_ms() + HEALTHY_WINDOW_MS + 1000), "silent for too long");
        mark_disconnected();
        assert!(!healthy_at(now_ms()));
    }

    /// Minimal fake SSE server. For each accepted connection it records the request, writes the
    /// scripted body, then either closes (`hold == false`) or keeps the socket open.
    async fn fake_sse(bodies: Vec<&'static str>, hold: bool) -> (String, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let reqs = requests.clone();
        tokio::spawn(async move {
            let mut i = 0usize;
            loop {
                let (mut sock, _) = listener.accept().await.unwrap();
                let mut buf = vec![0u8; 4096];
                let n = sock.read(&mut buf).await.unwrap_or(0);
                reqs.lock().unwrap().push(String::from_utf8_lossy(&buf[..n]).into_owned());
                let body = bodies.get(i).copied().unwrap_or("");
                i += 1;
                let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n";
                let _ = sock.write_all(head.as_bytes()).await;
                let _ = sock.write_all(body.as_bytes()).await;
                let _ = sock.flush().await;
                if hold {
                    tokio::spawn(async move {
                        let _s = sock;
                        tokio::time::sleep(Duration::from_secs(30)).await;
                    });
                }
            }
        });
        (format!("http://{}", addr), requests)
    }

    fn fast() -> ClientOptions {
        ClientOptions { min_backoff: Duration::from_millis(20), max_backoff: Duration::from_millis(80), token_poll: Duration::from_millis(20), idle_timeout: Duration::from_secs(30) }
    }

    #[tokio::test]
    async fn client_wakes_on_mined_but_not_on_intermediate_statuses() {
        let _g = STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (url, requests) = fake_sse(vec![concat!("id: 1\nevent: status\ndata: {\"txid\":\"aa\",\"txStatus\":\"SEEN_ON_NETWORK\"}\n\n", "id: 2\nevent: status\ndata: {\"txid\":\"aa\",\"txStatus\":\"MINED\"}\n\n: keepalive\n\n")], true).await;
        let wakes = Arc::new(AtomicUsize::new(0));
        let w = wakes.clone();
        let shutdown = CancellationToken::new();
        let sd = shutdown.clone();
        let h = tokio::spawn(run_client(url, || Some("tok".to_string()), move |_| { w.fetch_add(1, Ordering::SeqCst); }, sd, fast()));
        tokio::time::sleep(Duration::from_millis(400)).await;
        assert_eq!(wakes.load(Ordering::SeqCst), 1, "only MINED wakes");
        assert!(healthy(), "connected and frames flowing");
        assert!(requests.lock().unwrap()[0].contains("callbackToken=tok"), "token sent as the query parameter");
        shutdown.cancel();
        let _ = h.await;
        assert!(!healthy(), "shut down => not healthy");
    }

    #[tokio::test]
    async fn client_reconnects_after_a_drop_and_sends_last_event_id() {
        let _g = STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // 1st connection: one event then the server closes. 2nd connection: another MINED, held open.
        let (url, requests) = fake_sse(
            vec![
                "id: 41\nevent: status\ndata: {\"txid\":\"aa\",\"txStatus\":\"MINED\"}\n\n",
                "id: 42\nevent: status\ndata: {\"txid\":\"bb\",\"txStatus\":\"MINED\"}\n\n",
            ],
            false,
        )
        .await;
        let wakes = Arc::new(AtomicUsize::new(0));
        let w = wakes.clone();
        let shutdown = CancellationToken::new();
        let sd = shutdown.clone();
        let h = tokio::spawn(run_client(url, || Some("tok".to_string()), move |_| { w.fetch_add(1, Ordering::SeqCst); }, sd, fast()));
        tokio::time::sleep(Duration::from_millis(600)).await;
        shutdown.cancel();
        let _ = h.await;
        assert!(wakes.load(Ordering::SeqCst) >= 2, "events from both connections arrived");
        let reqs = requests.lock().unwrap();
        assert!(reqs.len() >= 2, "it reconnected");
        assert!(!reqs[0].to_lowercase().contains("last-event-id"));
        assert!(reqs[1].to_lowercase().contains("last-event-id: 41"), "replay point sent: {}", reqs[1]);
    }

    #[tokio::test]
    async fn a_silent_stream_is_dropped_and_reconnected_with_last_event_id() {
        let _g = STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Both connections deliver one event and then go silent while staying open (a half-open
        // socket looks exactly like this).
        let (url, requests) = fake_sse(
            vec![
                "id: 51
event: status
data: {\"txid\":\"aa\",\"txStatus\":\"MINED\"}

",
                "id: 52
event: status
data: {\"txid\":\"bb\",\"txStatus\":\"MINED\"}

",
            ],
            true,
        )
        .await;
        let wakes = Arc::new(AtomicUsize::new(0));
        let w = wakes.clone();
        let shutdown = CancellationToken::new();
        let sd = shutdown.clone();
        let opts = ClientOptions { idle_timeout: Duration::from_millis(150), ..fast() };
        let h = tokio::spawn(run_client(url, || Some("tok".to_string()), move |_| { w.fetch_add(1, Ordering::SeqCst); }, sd, opts));
        tokio::time::sleep(Duration::from_millis(900)).await;
        shutdown.cancel();
        let _ = h.await;
        let reqs = requests.lock().unwrap();
        assert!(reqs.len() >= 2, "it gave up on the silent stream and reconnected ({} requests)", reqs.len());
        assert!(reqs[1].to_lowercase().contains("last-event-id: 51"), "{}", reqs[1]);
        assert!(wakes.load(Ordering::SeqCst) >= 2);
    }

    #[tokio::test]
    async fn client_keeps_retrying_when_nothing_is_listening_and_waits_for_a_token() {
        let _g = STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Nothing listens on this port: every connect fails; it must not panic or return.
        let shutdown = CancellationToken::new();
        let sd = shutdown.clone();
        let h = tokio::spawn(run_client("http://127.0.0.1:9".to_string(), || None, |_| {}, sd, fast()));
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(!h.is_finished(), "no token yet: still waiting, not exited");
        assert!(!healthy());
        shutdown.cancel();
        let _ = h.await;
    }
}
