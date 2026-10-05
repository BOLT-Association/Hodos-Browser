//! `HODOS_CHAIN_MODE` — one switch for where the wallet gets chain data.
//!
//! * `public` (default, unset): today's behaviour — public indexers (WhatsOnChain,
//!   JungleBus, Bitails, GorillaPool, ARC GorillaPool/TAAL).
//! * `spv`: chain data comes ONLY from Arcade (`HODOS_ARCADE_URL`: broadcast, tx status,
//!   BUMPs, fee policy) and its chaintracks server (`HODOS_CHAINTRACKS_URL`: headers,
//!   verified by the wallet's own header chain). Public indexers are never contacted
//!   for chain state. Outputs and parent transactions arrive inside BEEFs
//!   (`internalizeAction`) instead of being discovered by address.
//!
//! Fail closed: an unrecognised value is treated as `spv` (never as `public`), and
//! `validate_startup` refuses to start on it. In `spv` mode both URLs are required.
//!
//! The egress helpers (`get`, `post`) are the enforcement point for code that makes
//! direct HTTP calls: a request to a blocked public-indexer host fails like a network
//! error, which every call site already treats as "unknown" and never as a verdict.
//! The exchange-rate lookup (`price_cache`) is deliberately not routed through them:
//! it is not chain state.

use reqwest::{Client, RequestBuilder};

pub const ENV_MODE: &str = "HODOS_CHAIN_MODE";
pub const ENV_ARCADE: &str = "HODOS_ARCADE_URL";
pub const ENV_CHAINTRACKS: &str = "HODOS_CHAINTRACKS_URL";
/// Arcade serves Server-Sent Events from a separate listener, so push needs its own URL.
pub const ENV_ARCADE_SSE: &str = "HODOS_ARCADE_SSE_URL";
/// `off` / `0` / `false` / `no` disables push even when the SSE URL is set. Polling is never off.
pub const ENV_ARCADE_PUSH: &str = "HODOS_ARCADE_PUSH";
/// `off` / `0` / `false` / `no` disables the chaintracks tip stream. The scheduled header sync is never off.
pub const ENV_TIP_STREAM: &str = "HODOS_TIP_STREAM";
/// `off` / `0` / `false` / `no` disables zero-conf acceptance of received outputs.
pub const ENV_ZERO_CONF: &str = "HODOS_ZERO_CONF";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainMode {
    Public,
    Spv,
}

/// Public indexer hosts (and their subdomains) that spv mode never contacts.
const BLOCKED_HOSTS: &[&str] = &[
    "whatsonchain.com",
    "gorillapool.io", // arc., mapi., junglebus., ordinals.
    "taal.com",       // arc.taal.com
    "bitails.io",
];

/// Where blocked requests are pointed. `.invalid` never resolves (RFC 2606), so
/// nothing leaves the machine and the caller sees an ordinary transport error.
const BLOCKED_SINK: &str = "http://spv-mode-public-lookup-blocked.invalid/";

pub fn parse(value: Option<&str>) -> Result<ChainMode, String> {
    match value.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
        None | Some("") | Some("public") => Ok(ChainMode::Public),
        Some("spv") => Ok(ChainMode::Spv),
        Some(other) => Err(format!(
            "{} = '{}' is not recognised (expected 'public' or 'spv')",
            ENV_MODE, other
        )),
    }
}

/// Current mode, read from the environment each call. Fails closed to `Spv`.
pub fn current() -> ChainMode {
    parse(std::env::var(ENV_MODE).ok().as_deref()).unwrap_or(ChainMode::Spv)
}

pub fn is_spv() -> bool {
    current() == ChainMode::Spv
}

fn non_empty_env(name: &str) -> bool {
    std::env::var(name).map(|v| !v.trim().is_empty()).unwrap_or(false)
}

/// Check the configuration at startup. `Err` means the wallet must not start.
pub fn validate_startup() -> Result<ChainMode, String> {
    let mode = parse(std::env::var(ENV_MODE).ok().as_deref())?;
    if mode == ChainMode::Spv {
        let missing: Vec<&str> = [ENV_ARCADE, ENV_CHAINTRACKS]
            .into_iter()
            .filter(|n| !non_empty_env(n))
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "{}=spv requires {} to be set",
                ENV_MODE,
                missing.join(" and ")
            ));
        }
        // The chain URLs must be the operator's own Arcade, never a public indexer: that would
        // defeat the "no public indexer" guarantee without a single log line.
        for var in [ENV_ARCADE, ENV_CHAINTRACKS, ENV_ARCADE_SSE] {
            if let Ok(url) = std::env::var(var) {
                if is_blocked_url(url.trim()) {
                    return Err(format!(
                        "{} points at a public indexer ({}), which spv mode does not use",
                        var,
                        url.trim()
                    ));
                }
            }
        }
    }
    Ok(mode)
}

/// The base URL of an env var, but only in spv mode (public mode ignores it).
pub fn spv_url(var: &str) -> Option<String> {
    if !is_spv() {
        return None;
    }
    std::env::var(var)
        .ok()
        .map(|u| u.trim().trim_end_matches('/').to_string())
        .filter(|u| !u.is_empty())
}

fn host_blocked(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    BLOCKED_HOSTS
        .iter()
        .any(|b| host == *b || host.ends_with(&format!(".{}", b)))
}

/// True when spv mode forbids contacting this URL.
pub fn is_blocked_url(url: &str) -> bool {
    if !is_spv() {
        return false;
    }
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(host_blocked))
        .unwrap_or(false)
}

/// At most `max` characters of `s`, cut on a character boundary (a plain byte slice panics when
/// a multi-byte character straddles the cut, and these are slices of remote error bodies).
pub fn truncate_chars(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}

/// Why a public lookup is unavailable, for error messages.
pub fn denied(what: &str) -> String {
    format!(
        "{} needs a public indexer, which is disabled in spv mode ({}=spv)",
        what, ENV_MODE
    )
}

/// Zero-conf acceptance of received outputs: on by default in spv mode; `HODOS_ZERO_CONF=off`
/// (or `0` / `false` / `no`) restores "wait for a verified proof". No effect in public mode.
pub fn zero_conf_enabled() -> bool {
    is_spv()
        && !matches!(
            std::env::var(ENV_ZERO_CONF).ok().map(|v| v.trim().to_ascii_lowercase()).as_deref(),
            Some("off") | Some("0") | Some("false") | Some("no")
        )
}

/// Gate for a user-initiated action that would talk to MessageBox as the wallet's identity
/// (PeerPay send). Refused in spv mode before anything is broadcast, so no tx is sent whose
/// payment token could not be delivered.
pub fn require_messagebox(what: &str) -> Result<(), String> {
    if message_polling_allowed() {
        Ok(())
    } else {
        Err(format!(
            "{} would contact MessageBox as the wallet's identity, which is disabled in spv mode ({}=spv)",
            what, ENV_MODE
        ))
    }
}

/// Base URL of Arcade's SSE service (spv mode only).
pub fn arcade_sse_url() -> Option<String> {
    spv_url(ENV_ARCADE_SSE)
}

/// Push (SSE wake-up) is on only in spv mode, with an SSE URL, and not switched off.
pub fn push_enabled() -> bool {
    if arcade_sse_url().is_none() {
        return false;
    }
    !matches!(
        std::env::var(ENV_ARCADE_PUSH).ok().map(|v| v.trim().to_ascii_lowercase()).as_deref(),
        Some("off") | Some("0") | Some("false") | Some("no")
    )
}

/// The chaintracks tip stream (a wake-up for the header sync) is on in spv mode unless switched off.
pub fn tip_stream_enabled() -> bool {
    spv_url(ENV_CHAINTRACKS).is_some()
        && !matches!(
            std::env::var(ENV_TIP_STREAM).ok().map(|v| v.trim().to_ascii_lowercase()).as_deref(),
            Some("off") | Some("0") | Some("false") | Some("no")
        )
}

/// MessageBox (PeerPay) polling and outbox retries are off in spv mode. They contact an
/// external relay as the wallet's identity and acknowledge messages there, which a local
/// chain run must never do (least of all from a copy of a real wallet database).
pub fn message_polling_allowed() -> bool {
    !is_spv()
}

/// `Err` in spv mode: a public-indexer lookup cannot be made.
pub fn ensure_public(what: &str) -> Result<(), String> {
    if is_spv() {
        Err(denied(what))
    } else {
        Ok(())
    }
}

/// `client.get(url)`, except that a blocked public-indexer URL in spv mode becomes a
/// request to an unresolvable host (and a warning), so the caller sees a transport error.
pub fn get(client: &Client, url: &str) -> RequestBuilder {
    if is_blocked_url(url) {
        log::warn!("🚫 spv mode: blocked public lookup {}", url);
        return client.get(BLOCKED_SINK);
    }
    client.get(url)
}

/// `client.post(url)` with the same blocking rule as [`get`].
pub fn post(client: &Client, url: &str) -> RequestBuilder {
    if is_blocked_url(url) {
        log::warn!("🚫 spv mode: blocked public lookup {}", url);
        return client.post(BLOCKED_SINK);
    }
    client.post(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Env vars are process-wide; serialise the tests that touch them.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env<F: FnOnce()>(mode: Option<&str>, arcade: Option<&str>, ct: Option<&str>, f: F) {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let set = |k: &str, v: Option<&str>| match v {
            Some(v) => std::env::set_var(k, v),
            None => std::env::remove_var(k),
        };
        let old: Vec<_> = [ENV_MODE, ENV_ARCADE, ENV_CHAINTRACKS]
            .iter()
            .map(|k| (*k, std::env::var(k).ok()))
            .collect();
        set(ENV_MODE, mode);
        set(ENV_ARCADE, arcade);
        set(ENV_CHAINTRACKS, ct);
        f();
        for (k, v) in old {
            set(k, v.as_deref());
        }
    }

    #[test]
    fn parse_values() {
        assert_eq!(parse(None), Ok(ChainMode::Public));
        assert_eq!(parse(Some("")), Ok(ChainMode::Public));
        assert_eq!(parse(Some("PUBLIC")), Ok(ChainMode::Public));
        assert_eq!(parse(Some(" spv ")), Ok(ChainMode::Spv));
        assert!(parse(Some("local")).is_err());
    }

    #[test]
    fn unknown_value_fails_closed_to_spv_and_refuses_startup() {
        with_env(Some("typo"), Some("http://a"), Some("http://c"), || {
            assert_eq!(current(), ChainMode::Spv);
            assert!(validate_startup().is_err());
        });
    }

    #[test]
    fn spv_requires_both_urls() {
        with_env(Some("spv"), Some("http://a"), None, || {
            let e = validate_startup().unwrap_err();
            assert!(e.contains(ENV_CHAINTRACKS) && !e.contains(ENV_ARCADE), "{}", e);
        });
        with_env(Some("spv"), None, None, || {
            let e = validate_startup().unwrap_err();
            assert!(e.contains(ENV_ARCADE) && e.contains(ENV_CHAINTRACKS));
        });
        with_env(Some("spv"), Some("http://a"), Some("http://c"), || {
            assert_eq!(validate_startup(), Ok(ChainMode::Spv));
        });
    }

    #[test]
    fn spv_refuses_to_start_when_a_chain_url_points_at_a_public_indexer() {
        for bad in ["https://arc.gorillapool.io", "https://api.whatsonchain.com/v1", "https://arc.taal.com"] {
            with_env(Some("spv"), Some(bad), Some("http://c"), || {
                let e = validate_startup().unwrap_err();
                assert!(e.contains(ENV_ARCADE) && e.contains("public"), "{}", e);
            });
            with_env(Some("spv"), Some("http://a"), Some(bad), || {
                assert!(validate_startup().unwrap_err().contains(ENV_CHAINTRACKS));
            });
        }
        with_env(Some("spv"), Some("http://a"), Some("http://c"), || {
            std::env::set_var(ENV_ARCADE_SSE, "https://junglebus.gorillapool.io/sse");
            let e = validate_startup();
            std::env::remove_var(ENV_ARCADE_SSE);
            assert!(e.unwrap_err().contains(ENV_ARCADE_SSE));
        });
    }

    #[test]
    fn spv_accepts_local_and_self_hosted_chain_urls() {
        with_env(Some("spv"), Some("http://127.0.0.1:8080"), Some("https://arcade.example.org/chaintracks/v2"), || {
            assert_eq!(validate_startup(), Ok(ChainMode::Spv));
        });
    }

    #[test]
    fn truncate_chars_never_splits_a_character() {
        assert_eq!(truncate_chars("abc", 10), "abc");
        assert_eq!(truncate_chars("abcdef", 3), "abc");
        let s = format!("{}é{}", "a".repeat(199), "b"); // 'é' spans bytes 199..201
        let t = truncate_chars(&s, 200);
        assert_eq!(t.chars().count(), 200);
        assert!(t.ends_with('é'));
    }

    #[test]
    fn messagebox_traffic_is_refused_up_front_in_spv_mode_only() {
        with_env(None, None, None, || assert_eq!(require_messagebox("PeerPay send"), Ok(())));
        with_env(Some("spv"), Some("http://a"), Some("http://c"), || {
            let e = require_messagebox("PeerPay send").unwrap_err();
            assert!(e.contains("PeerPay send") && e.contains("spv"), "{}", e);
        });
    }

    #[test]
    fn zero_conf_is_on_in_spv_mode_unless_switched_off_and_never_in_public_mode() {
        let with_zc = |mode: Option<&str>, zc: Option<&str>, f: &dyn Fn()| {
            let old = std::env::var(ENV_ZERO_CONF).ok();
            with_env(mode, Some("http://a"), Some("http://c"), || {
                match zc { Some(v) => std::env::set_var(ENV_ZERO_CONF, v), None => std::env::remove_var(ENV_ZERO_CONF) }
                f();
            });
            match old { Some(v) => std::env::set_var(ENV_ZERO_CONF, v), None => std::env::remove_var(ENV_ZERO_CONF) }
        };
        with_zc(Some("spv"), None, &|| assert!(zero_conf_enabled(), "default on"));
        with_zc(Some("spv"), Some("on"), &|| assert!(zero_conf_enabled()));
        for off in ["off", "OFF", "0", "false", "no"] {
            with_zc(Some("spv"), Some(off), &|| assert!(!zero_conf_enabled(), "{}", off));
        }
        with_zc(None, None, &|| assert!(!zero_conf_enabled(), "public mode: feature does not exist"));
    }

    #[test]
    fn push_needs_spv_mode_an_sse_url_and_not_to_be_switched_off() {
        let url = "http://localhost:8082";
        let with_push = |mode: Option<&str>, sse: Option<&str>, push: Option<&str>, f: &dyn Fn()| {
            // ENV_LOCK is taken inside with_env; set the extra vars around it.
            let old_sse = std::env::var(ENV_ARCADE_SSE).ok();
            let old_push = std::env::var(ENV_ARCADE_PUSH).ok();
            let set = |k: &str, v: Option<&str>| match v { Some(v) => std::env::set_var(k, v), None => std::env::remove_var(k) };
            with_env(mode, Some("http://a"), Some("http://c"), || {
                set(ENV_ARCADE_SSE, sse);
                set(ENV_ARCADE_PUSH, push);
                f();
            });
            set(ENV_ARCADE_SSE, old_sse.as_deref());
            set(ENV_ARCADE_PUSH, old_push.as_deref());
        };
        with_push(Some("spv"), Some(url), None, &|| assert!(push_enabled()));
        with_push(Some("spv"), Some(url), Some("on"), &|| assert!(push_enabled()));
        for off in ["off", "OFF", "0", "false", "no"] {
            with_push(Some("spv"), Some(url), Some(off), &|| assert!(!push_enabled(), "{}", off));
        }
        with_push(Some("spv"), None, None, &|| assert!(!push_enabled(), "no SSE url => no push"));
        with_push(None, Some(url), None, &|| assert!(!push_enabled(), "public mode ignores the SSE url"));
    }

    #[test]
    fn the_tip_stream_needs_spv_mode_and_not_to_be_switched_off() {
        let with_switch = |mode: Option<&str>, switch: Option<&str>, f: &dyn Fn()| {
            let old = std::env::var(ENV_TIP_STREAM).ok();
            let set = |v: Option<&str>| match v { Some(v) => std::env::set_var(ENV_TIP_STREAM, v), None => std::env::remove_var(ENV_TIP_STREAM) };
            with_env(mode, Some("http://a"), Some("http://c"), || {
                set(switch);
                f();
            });
            set(old.as_deref());
        };
        with_switch(Some("spv"), None, &|| assert!(tip_stream_enabled(), "on by default in spv mode"));
        with_switch(Some("spv"), Some("on"), &|| assert!(tip_stream_enabled()));
        for off in ["off", "OFF", "0", "false", "no"] {
            with_switch(Some("spv"), Some(off), &|| assert!(!tip_stream_enabled(), "{}", off));
        }
        with_switch(None, None, &|| assert!(!tip_stream_enabled(), "public mode has no header chain to sync"));
    }

    #[test]
    fn messagebox_polling_is_only_allowed_outside_spv_mode() {
        with_env(None, None, None, || assert!(message_polling_allowed()));
        with_env(Some("spv"), Some("http://a"), Some("http://c"), || assert!(!message_polling_allowed()));
        with_env(Some("typo"), None, None, || assert!(!message_polling_allowed(), "unknown value fails closed"));
    }

    #[test]
    fn public_mode_ignores_the_urls_and_blocks_nothing() {
        with_env(None, Some("http://a"), Some("http://c"), || {
            assert_eq!(validate_startup(), Ok(ChainMode::Public));
            assert_eq!(spv_url(ENV_ARCADE), None, "urls only take effect in spv mode");
            assert!(!is_blocked_url("https://api.whatsonchain.com/v1/bsv/main/chain/info"));
            assert!(ensure_public("x").is_ok());
        });
    }

    #[test]
    fn spv_mode_blocks_public_indexers_and_only_them() {
        with_env(Some("spv"), Some("http://localhost:8080/"), Some("http://localhost:8083/x"), || {
            for u in [
                "https://api.whatsonchain.com/v1/bsv/main/tx/ab/hex",
                "https://junglebus.gorillapool.io/v1/transaction/get/ab",
                "https://ordinals.gorillapool.io/api/txos/address/x/unspent",
                "https://arc.gorillapool.io/v1/tx",
                "https://arc.taal.com/v1/tx",
                "https://api.bitails.io/tx/ab",
                "https://WhatsOnChain.com/x",
            ] {
                assert!(is_blocked_url(u), "{} should be blocked", u);
            }
            for u in [
                "http://localhost:8080/tx/ab",
                "http://localhost:8083/chaintracks/v2/tip",
                "https://messagebox.babbage.systems/x",
                "https://notwhatsonchain.com/x", // suffix must be a whole label
                "not a url",
            ] {
                assert!(!is_blocked_url(u), "{} should pass", u);
            }
            assert_eq!(spv_url(ENV_ARCADE).as_deref(), Some("http://localhost:8080"));
            assert!(ensure_public("address UTXO lookup").unwrap_err().contains("spv"));
        });
    }

    #[tokio::test]
    async fn address_lookups_are_errors_in_spv_mode_never_empty_results() {
        let r = with_env_async(Some("spv"), || async {
            let a = crate::utxo_fetcher::fetch_all_utxos(&[]).await;
            let b = crate::utxo_fetcher::fetch_utxos_for_address("1abc", 0).await;
            let c = crate::utxo_fetcher::fetch_utxos_single_address_with_unconfirmed("1abc", 0).await;
            let d = crate::utxo_fetcher::address_has_history("1abc").await;
            (a.is_err(), b.is_err(), c.is_err(), d.is_err())
        })
        .await;
        assert_eq!(r, (true, true, true, true), "an empty Ok would be read as 'no coins'");
    }

    #[tokio::test]
    async fn blocked_request_fails_as_a_transport_error_without_leaving_the_machine() {
        let c = Client::new();
        let blocked = with_env_async(Some("spv"), || async {
            get(&c, "https://api.whatsonchain.com/v1/bsv/main/chain/info").send().await
        })
        .await;
        let err = blocked.expect_err("must not succeed");
        assert!(err.is_connect() || err.is_request(), "{:?}", err);
    }

    async fn with_env_async<F, Fut, T>(mode: Option<&str>, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let old = std::env::var(ENV_MODE).ok();
        match mode {
            Some(m) => std::env::set_var(ENV_MODE, m),
            None => std::env::remove_var(ENV_MODE),
        }
        let r = f().await;
        match old {
            Some(v) => std::env::set_var(ENV_MODE, v),
            None => std::env::remove_var(ENV_MODE),
        }
        r
    }
}
