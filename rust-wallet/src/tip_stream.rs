//! spv mode: the chaintracks tip stream (`<HODOS_CHAINTRACKS_URL>/tip/stream`, under `/v2`).
//!
//! Arcade's chaintracks sends the present chain tip when a client connects and a frame for every
//! new block. The wallet listens so its header chain advances when a block is found, instead of at
//! the next 30 s monitor tick.
//!
//! ⛔ A frame is a wake-up and nothing else. What it carries is never put into the header chain:
//! the driver (`monitor/task_tip_stream.rs`) runs the ordinary header sync, which fetches the
//! headers itself and validates every one (`header_chain`). The driver reads the frame only to
//! tell a new tip from a repeat of the last one. A stream that lies, stalls or is down costs
//! latency only: `TaskSyncHeaders` still runs on the monitor's schedule.

use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::arcade_push::{backoff, SseItem, SseParser};

/// The tip stream of the chaintracks at `base` (`…/chaintracks/v2`, or `/v1`, or neither). The
/// stream exists only under `/v2`.
pub fn stream_url(base: &str) -> String {
    let base = base.trim().trim_end_matches('/');
    let base = base.strip_suffix("/v1").or_else(|| base.strip_suffix("/v2")).unwrap_or(base);
    format!("{}/v2/tip/stream", base)
}

/// True when `data` is a different frame from the last one seen (and remembers it). A reconnect
/// repeats the present tip; only a change is worth a header sync.
pub fn is_new_tip(last: &mut Option<String>, data: &str) -> bool {
    if last.as_deref() == Some(data) {
        return false;
    }
    *last = Some(data.to_string());
    true
}

pub struct ClientOptions {
    pub min_backoff: Duration,
    pub max_backoff: Duration,
    /// A stream that delivers nothing for this long is dropped and reopened: a half-open
    /// connection looks exactly like a chain with no new block. The reopened stream repeats the
    /// present tip, which the driver ignores when it has not changed.
    pub idle_timeout: Duration,
}

impl Default for ClientOptions {
    fn default() -> Self {
        ClientOptions {
            min_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            idle_timeout: Duration::from_secs(120),
        }
    }
}

/// Run until `shutdown`: read the stream at `url`, call `on_tip` with the data of every frame,
/// reconnect with backoff when the stream ends, fails or goes silent. Never returns an error:
/// every failure means "try again", and the scheduled header sync covers the gap.
pub async fn run_client<W>(url: String, on_tip: W, shutdown: CancellationToken, opts: ClientOptions)
where
    W: Fn(&str),
{
    let http = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let mut attempt: u32 = 0;

    loop {
        if shutdown.is_cancelled() {
            return;
        }
        let mut got_frame = false;
        match crate::chain_mode::get(&http, &url).send().await {
            Ok(mut resp) if resp.status().is_success() => {
                let mut parser = SseParser::new();
                loop {
                    let chunk = tokio::select! {
                        _ = shutdown.cancelled() => return,
                        c = tokio::time::timeout(opts.idle_timeout, resp.chunk()) => c,
                    };
                    let Ok(chunk) = chunk else {
                        break; // silent for idle_timeout: reopen
                    };
                    match chunk {
                        Ok(Some(bytes)) => {
                            for item in parser.feed(&bytes) {
                                if let SseItem::Event { data, .. } = item {
                                    got_frame = true;
                                    on_tip(&data);
                                }
                            }
                        }
                        Ok(None) => break, // server closed the stream
                        Err(e) => {
                            log::warn!("tip stream: stream error: {}", e);
                            break;
                        }
                    }
                }
            }
            Ok(resp) => log::warn!("tip stream: {} answered {}", url, resp.status()),
            Err(e) => log::warn!("tip stream: cannot connect to {}: {}", url, e),
        }

        // A connection that delivered a frame resets the backoff; one that never did backs off.
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
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    const TIP_398: &str = r#"{"height":398,"hash":"639dce7e8fcfe494d64efe9a17cecf19c0d41054ea8abcf7dbd14daaaa6fc988"}"#;
    const TIP_399: &str = r#"{"height":399,"hash":"29a5fd4659abd242853d298cbbf0706af8da7b6f0a6cfdc25916bc564cdd4950"}"#;

    #[test]
    fn the_stream_is_under_v2_whichever_version_the_base_names() {
        let want = "http://localhost:8083/chaintracks/v2/tip/stream";
        assert_eq!(stream_url("http://localhost:8083/chaintracks/v2"), want);
        assert_eq!(stream_url("http://localhost:8083/chaintracks/v2/"), want);
        assert_eq!(stream_url("http://localhost:8083/chaintracks/v1"), want);
        assert_eq!(stream_url(" http://localhost:8083/chaintracks "), want);
    }

    #[test]
    fn only_a_changed_frame_is_a_new_tip() {
        let mut last = None;
        assert!(is_new_tip(&mut last, TIP_398), "the first frame");
        assert!(!is_new_tip(&mut last, TIP_398), "a reconnect repeats the present tip");
        assert!(is_new_tip(&mut last, TIP_399), "a new block");
        assert!(is_new_tip(&mut last, TIP_398), "a reorg back to an earlier tip is a change too");
        assert!(is_new_tip(&mut last, "not json"), "the frame is compared, never parsed");
    }

    /// Minimal fake stream server. For each accepted connection it records the request, answers
    /// `status`, writes the scripted body, then either closes (`hold == false`) or keeps the
    /// socket open.
    async fn fake_stream(status: &'static str, bodies: Vec<String>, hold: bool) -> (String, Arc<Mutex<Vec<String>>>) {
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
                let body = bodies.get(i).cloned().unwrap_or_default();
                i += 1;
                let head = format!("HTTP/1.1 {}\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n", status);
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
        (format!("http://{}/chaintracks/v2/tip/stream", addr), requests)
    }

    fn fast() -> ClientOptions {
        ClientOptions { min_backoff: Duration::from_millis(20), max_backoff: Duration::from_millis(80), idle_timeout: Duration::from_secs(30) }
    }

    /// Runs the client against `url` for `ms`, returns the frames `on_tip` was given.
    async fn collect(url: String, opts: ClientOptions, ms: u64) -> Vec<String> {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let s = seen.clone();
        let shutdown = CancellationToken::new();
        let h = tokio::spawn(run_client(url, move |d| s.lock().unwrap().push(d.to_string()), shutdown.clone(), opts));
        tokio::time::sleep(Duration::from_millis(ms)).await;
        shutdown.cancel();
        let _ = h.await;
        let out = seen.lock().unwrap().clone();
        out
    }

    #[tokio::test]
    async fn every_frame_reaches_on_tip_and_a_keepalive_does_not() {
        // The frames chaintracks sends: bare `data:` lines, the present tip first.
        let body = format!("data: {}\n\n: keepalive\n\ndata: {}\n\n", TIP_398, TIP_399);
        let (url, requests) = fake_stream("200 OK", vec![body], true).await;
        let seen = collect(url, fast(), 400).await;
        assert_eq!(seen, vec![TIP_398.to_string(), TIP_399.to_string()]);
        let reqs = requests.lock().unwrap();
        assert!(reqs[0].starts_with("GET /chaintracks/v2/tip/stream "), "{}", reqs[0]);
    }

    #[tokio::test]
    async fn it_reconnects_after_the_server_closes_the_stream() {
        let (url, requests) =
            fake_stream("200 OK", vec![format!("data: {}\n\n", TIP_398), format!("data: {}\n\n", TIP_399)], false).await;
        let seen = collect(url, fast(), 600).await;
        assert!(requests.lock().unwrap().len() >= 2, "it reconnected");
        assert_eq!(&seen[..2], &[TIP_398.to_string(), TIP_399.to_string()], "frames from both connections arrived");
    }

    #[tokio::test]
    async fn a_silent_stream_is_dropped_and_reopened() {
        // Each connection delivers one frame and then goes silent while staying open (a half-open
        // socket looks exactly like this).
        let (url, requests) =
            fake_stream("200 OK", vec![format!("data: {}\n\n", TIP_398), format!("data: {}\n\n", TIP_399)], true).await;
        let opts = ClientOptions { idle_timeout: Duration::from_millis(150), ..fast() };
        let seen = collect(url, opts, 900).await;
        assert!(requests.lock().unwrap().len() >= 2, "it gave up on the silent stream and reopened it");
        assert_eq!(&seen[..2], &[TIP_398.to_string(), TIP_399.to_string()]);
    }

    #[tokio::test]
    async fn a_refusal_gives_no_tip_and_is_retried() {
        // A 404 (a chaintracks without the stream) whose body happens to look like a frame.
        let (url, requests) = fake_stream("404 Not Found", vec![format!("data: {}\n\n", TIP_398); 8], false).await;
        let seen = collect(url, fast(), 400).await;
        assert!(seen.is_empty(), "an error response is never read as a tip: {:?}", seen);
        assert!(requests.lock().unwrap().len() >= 2, "it kept trying");
    }

    #[tokio::test]
    async fn it_keeps_retrying_when_nothing_is_listening_and_stops_on_shutdown() {
        let shutdown = CancellationToken::new();
        let h = tokio::spawn(run_client("http://127.0.0.1:9/chaintracks/v2/tip/stream".to_string(), |_| {}, shutdown.clone(), fast()));
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(!h.is_finished(), "still retrying, not exited");
        shutdown.cancel();
        tokio::time::timeout(Duration::from_secs(5), h).await.expect("stops on shutdown").unwrap();
    }
}
