//! Wallet-side rails for the page's `window.BOLT` interface.
//!
//! The BOLT token logic (b017) runs in the page; the wallet gives it the two things a page cannot do
//! for itself:
//!
//! - `POST /boltBroadcast`: submit a token transaction to the network and report the network's
//!   verdict. A page cannot reach the chain service (a site's CSP and CORS forbid it), and the
//!   wallet is the component that knows which service this installation uses.
//! - `POST /boltTokens`: keep the tokens this wallet holds (V28 `bolt_tokens`), so they belong to
//!   the wallet and not to one site's storage.
//!
//! Neither spends the wallet's coins: funding and signing still go through `createAction` /
//! `createSignature` and their own permission gates. Both are reachable by any domain the user has
//! approved (`domain_trust_mw`), like the rest of the BRC-100 surface.
//!
//! **AuthBOLT identities are the wallet's own** (ChainBrowsers `docs/authbolt-registration.md`).
//! Hodos's own UI (no `X-Requesting-Domain`) mints and presents them and keeps its notes on each
//! (`attributes.wallet`: its keys and the sites it is linked to, written with `op: "annotate"`). A
//! site sees only the identities linked to it, and cannot put, retire or annotate an identity row.
//! Every other token type is unchanged.

use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::database::{BoltTokenRepository, BoltTokenRow};
use crate::services::provider::{TxState, TxStatus};
use crate::AppState;

/// How long `/boltBroadcast` waits for a network status. The browser's wallet bridge gives a call
/// 30 s, so this must stay below it.
const STATUS_WAIT: Duration = Duration::from_secs(20);
const STATUS_POLL: Duration = Duration::from_millis(400);
/// Largest transaction accepted, in hex characters (4 MB of transaction).
const MAX_TX_HEX: usize = 8 * 1024 * 1024;
/// Largest Atomic BEEF accepted for a token row, in hex characters.
const MAX_BEEF_HEX: usize = 8 * 1024 * 1024;

fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

fn bad_request(msg: impl Into<String>) -> HttpResponse {
    HttpResponse::BadRequest().json(serde_json::json!({ "error": msg.into() }))
}

fn is_hex(s: &str) -> bool {
    !s.is_empty() && s.len() % 2 == 0 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn is_hex_len(s: &str, chars: usize) -> bool {
    s.len() == chars && is_hex(s)
}

// ---------------------------------------------------------------- broadcast

/// What the network says about a transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NetVerdict {
    /// On the network (seen by nodes, accepted pending a parent, or mined). Carries the status.
    Seen(String),
    /// Refused by the network. Carries the status.
    Refused(String),
    /// No verdict yet (received/queued/announced, or unknown).
    Pending,
}

/// Map a provider status to a verdict. `ACCEPTED_BY_NETWORK` counts as seen: it is where a child of
/// an unmined parent stays until a block, and a token transfer is exactly such a chain. A provider
/// with no status vocabulary of its own (public-mode fallbacks) reports only the coarse state.
pub(crate) fn net_verdict(status: &TxStatus) -> NetVerdict {
    let raw = status.raw_provider_status.clone().unwrap_or_default();
    match status.state {
        TxState::Mined => NetVerdict::Seen(if raw.is_empty() { "MINED".to_string() } else { raw }),
        TxState::Rejected | TxState::DoubleSpendAttempted => {
            NetVerdict::Refused(if raw.is_empty() { format!("{:?}", status.state) } else { raw })
        }
        TxState::InMempool => match raw.as_str() {
            "" => NetVerdict::Seen("IN_MEMPOOL".to_string()),
            "SEEN_ON_NETWORK" | "SEEN_ON_MULTIPLE_NODES" | "ACCEPTED_BY_NETWORK" => NetVerdict::Seen(raw),
            _ => NetVerdict::Pending,
        },
        TxState::Unknown => NetVerdict::Pending,
    }
}

#[derive(Deserialize)]
struct BroadcastRequest {
    /// The transaction, hex: Extended Format when it spends unmined parents, else raw.
    tx: String,
    /// Its txid (hex), used to ask whether the network already has it.
    #[serde(default)]
    txid: String,
}

fn broadcast_reply(status: &str, detail: impl Into<String>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": status, "detail": detail.into() }))
}

/// `POST /boltBroadcast` `{ tx, txid }` → `{ status: "accepted" | "already-seen" | "rejected", detail }`.
///
/// Always answers 200 with a verdict once the request is well formed: a refusal by the network is a
/// result, not an HTTP error.
pub async fn bolt_broadcast(state: web::Data<AppState>, body: web::Bytes) -> HttpResponse {
    let req: BroadcastRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return bad_request(format!("invalid request: {e}")),
    };
    if req.tx.len() > MAX_TX_HEX || !is_hex(&req.tx) {
        return bad_request("tx must be a hex transaction");
    }
    if !req.txid.is_empty() && !is_hex_len(&req.txid, 64) {
        return bad_request("txid must be 64 hex characters");
    }
    let bytes = match hex::decode(&req.tx) {
        Ok(b) => b,
        Err(e) => return bad_request(format!("tx is not hex: {e}")),
    };

    // Already on the network? Then there is nothing to send. (A transaction the network refused
    // earlier is sent again: what it lacked then, e.g. a parent, may be there now.)
    if !req.txid.is_empty() {
        if let Ok(status) = state.services.tx_status(&req.txid).await {
            if let NetVerdict::Seen(detail) = net_verdict(&status) {
                return broadcast_reply("already-seen", detail);
            }
        }
    }

    let txid = match state.services.broadcast_beef(&bytes).await {
        Ok(sent) if !sent.txid.is_empty() => sent.txid,
        Ok(_) => req.txid.clone(),
        Err(e) => return broadcast_reply("rejected", e.to_string()),
    };
    if txid.is_empty() {
        return broadcast_reply("rejected", "the broadcaster returned no txid and none was given");
    }

    // The broadcaster answers "received" for anything well formed, so the verdict is the status
    // that follows.
    let deadline = Instant::now() + STATUS_WAIT;
    loop {
        if let Ok(status) = state.services.tx_status(&txid).await {
            match net_verdict(&status) {
                NetVerdict::Seen(detail) => return broadcast_reply("accepted", detail),
                NetVerdict::Refused(detail) => return broadcast_reply("rejected", detail),
                NetVerdict::Pending => {}
            }
        }
        if Instant::now() >= deadline {
            return broadcast_reply("rejected", "no network status in time");
        }
        tokio::time::sleep(STATUS_POLL).await;
    }
}

// ---------------------------------------------------------------- token store

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
enum TokensRequest {
    Put { row: BoltTokenRow },
    Get { outpoint: String },
    List {
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        issuer: Option<String>,
        #[serde(default, rename = "type")]
        token_type: Option<String>,
    },
    Spend { outpoint: String },
    /// The wallet's own notes on a token (Hodos's UI only).
    Annotate { outpoint: String, wallet: serde_json::Value },
}

/// The token type of an AuthBOLT identity.
const IDENTITY_TYPE: &str = "AuthBOLT";

/// The site asking (`X-Requesting-Domain`), or `None` for Hodos's own UI.
fn requesting_site(http_req: &HttpRequest) -> Option<String> {
    http_req
        .headers()
        .get(crate::permission_service::request_gate::X_REQUESTING_DOMAIN)
        .and_then(|v| v.to_str().ok())
        .filter(|d| !d.is_empty())
        .map(str::to_string)
}

/// Whether `site` may see a row: every token type but an identity; an identity only when the
/// wallet linked it to that site. Hodos's own UI (`None`) sees everything.
pub(crate) fn visible_to(row: &BoltTokenRow, site: Option<&str>) -> bool {
    let Some(site) = site else { return true };
    if row.token_type != IDENTITY_TYPE {
        return true;
    }
    let attrs: serde_json::Value = serde_json::from_str(&row.attributes).unwrap_or_default();
    attrs["wallet"]["apps"]
        .as_array()
        .is_some_and(|apps| apps.iter().any(|a| a["domain"].as_str() == Some(site)))
}

fn identity_refused() -> HttpResponse {
    HttpResponse::Forbidden().json(serde_json::json!({
        "error": "AuthBOLT identities are the wallet's own: a site cannot store, retire or annotate them",
        "code": "IDENTITY_TOKEN"
    }))
}

fn is_outpoint(s: &str) -> bool {
    match s.split_once('.') {
        Some((txid, vout)) => is_hex_len(txid, 64) && !vout.is_empty() && vout.len() <= 10 && vout.bytes().all(|b| b.is_ascii_digit()),
        None => false,
    }
}

fn is_type_name(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn is_json_object(s: &str, max: usize) -> bool {
    s.len() <= max && matches!(serde_json::from_str::<serde_json::Value>(s), Ok(serde_json::Value::Object(_)))
}

/// Shape checks for a row a page asks the wallet to keep. They bound what is stored; they cannot
/// show that the row describes its BEEF (the wallet does not run the token scripts).
pub(crate) fn validate_row(t: &BoltTokenRow) -> Result<(), String> {
    if !is_outpoint(&t.outpoint) {
        return Err("outpoint must be <64 hex>.<vout>".into());
    }
    if !is_type_name(&t.token_type) {
        return Err("type must be a short token type name".into());
    }
    if !is_hex_len(&t.issuer, 66) {
        return Err("issuer must be a 33-byte public key (hex)".into());
    }
    if let Some(pkh) = &t.owner_pkh {
        if !is_hex_len(pkh, 40) {
            return Err("owner_pkh must be 20 bytes (hex)".into());
        }
    }
    if let Some(amount) = &t.amount {
        // a 128-bit balance is at most 39 decimal digits
        if amount.is_empty() || amount.len() > 39 || !amount.bytes().all(|b| b.is_ascii_digit()) {
            return Err("amount must be a decimal string of at most 39 digits".into());
        }
    }
    if !is_json_object(&t.attributes, 64 * 1024) {
        return Err("attributes must be a JSON object (64 KB at most)".into());
    }
    if t.beef.len() > MAX_BEEF_HEX || !is_hex(&t.beef) {
        return Err("beef must be hex".into());
    }
    if let Some(txid) = &t.anchor_txid {
        if !is_hex_len(txid, 64) {
            return Err("anchor_txid must be 64 hex characters".into());
        }
    }
    if let Some(kind) = &t.anchor_kind {
        if kind != "mint" && kind != "settle" {
            return Err("anchor_kind must be mint or settle".into());
        }
    }
    if let Some(network) = &t.anchor_network {
        if network.len() > 32 {
            return Err("anchor_network is too long".into());
        }
    }
    if let Some(root) = &t.anchor_merkle_root {
        if !is_hex_len(root, 64) {
            return Err("anchor_merkle_root must be 64 hex characters".into());
        }
    }
    if let Some(height) = t.anchor_height {
        if !(0..=i64::from(u32::MAX)).contains(&height) {
            return Err("anchor_height is out of range".into());
        }
    }
    if let Some(p) = &t.provenance {
        if !is_json_object(p, 4 * 1024) {
            return Err("provenance must be a JSON object (4 KB at most)".into());
        }
    }
    Ok(())
}

fn db_error(e: impl std::fmt::Display) -> HttpResponse {
    HttpResponse::InternalServerError().json(serde_json::json!({ "error": format!("token store: {e}") }))
}

/// `POST /boltTokens` with an `op`:
/// - `{op:"put", row}` → `{ok:true}`: keep a held token (see `BoltTokenRepository::put`)
/// - `{op:"get", outpoint}` → `{row}` (`null` when unknown)
/// - `{op:"list", status?, issuer?, type?}` → `{rows}` (`status` defaults to `held`)
/// - `{op:"spend", outpoint}` → `{ok, spent}`: retire a held token; nothing is deleted
pub async fn bolt_tokens(state: web::Data<AppState>, http_req: HttpRequest, body: web::Bytes) -> HttpResponse {
    let req: TokensRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return bad_request(format!("invalid request: {e}")),
    };
    let site = requesting_site(&http_req);
    let db = state.database.lock().unwrap();
    let repo = BoltTokenRepository::new(db.connection());
    match req {
        TokensRequest::Put { row } => {
            if site.is_some() && row.token_type == IDENTITY_TYPE {
                return identity_refused();
            }
            if let Err(e) = validate_row(&row) {
                return bad_request(e);
            }
            match repo.put(&row, now_secs()) {
                Ok(()) => HttpResponse::Ok().json(serde_json::json!({ "ok": true })),
                Err(e) => db_error(e),
            }
        }
        TokensRequest::Get { outpoint } => {
            if !is_outpoint(&outpoint) {
                return bad_request("outpoint must be <64 hex>.<vout>");
            }
            match repo.get(&outpoint) {
                Ok(row) => {
                    let row = row.filter(|r| visible_to(r, site.as_deref()));
                    HttpResponse::Ok().json(serde_json::json!({ "row": row }))
                }
                Err(e) => db_error(e),
            }
        }
        TokensRequest::List { status, issuer, token_type } => {
            let status = status.unwrap_or_else(|| "held".to_string());
            if status != "held" && status != "spent" {
                return bad_request("status must be held or spent");
            }
            match repo.list(&status, issuer.as_deref(), token_type.as_deref()) {
                Ok(rows) => {
                    let rows: Vec<_> = rows.into_iter().filter(|r| visible_to(r, site.as_deref())).collect();
                    HttpResponse::Ok().json(serde_json::json!({ "rows": rows }))
                }
                Err(e) => db_error(e),
            }
        }
        TokensRequest::Spend { outpoint } => {
            if !is_outpoint(&outpoint) {
                return bad_request("outpoint must be <64 hex>.<vout>");
            }
            if site.is_some() && matches!(repo.get(&outpoint), Ok(Some(r)) if r.token_type == IDENTITY_TYPE) {
                return identity_refused();
            }
            match repo.mark_spent(&outpoint, now_secs()) {
                Ok(spent) => HttpResponse::Ok().json(serde_json::json!({ "ok": true, "spent": spent })),
                Err(e) => db_error(e),
            }
        }
        TokensRequest::Annotate { outpoint, wallet } => {
            if site.is_some() {
                return identity_refused();
            }
            if !is_outpoint(&outpoint) {
                return bad_request("outpoint must be <64 hex>.<vout>");
            }
            if !wallet.is_object() || wallet.to_string().len() > 16 * 1024 {
                return bad_request("wallet must be a JSON object (16 KB at most)");
            }
            match repo.annotate(&outpoint, &wallet, now_secs()) {
                Ok(found) => HttpResponse::Ok().json(serde_json::json!({ "ok": true, "found": found })),
                Err(e) => db_error(e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(state: TxState, raw: Option<&str>) -> TxStatus {
        TxStatus {
            txid: "00".repeat(32),
            state,
            block_height: None,
            block_hash: None,
            merkle_path_bump: None,
            raw_provider_status: raw.map(str::to_string),
        }
    }

    #[test]
    fn verdict_follows_the_network_status() {
        use NetVerdict::*;
        assert_eq!(net_verdict(&status(TxState::Mined, Some("MINED"))), Seen("MINED".into()));
        assert_eq!(net_verdict(&status(TxState::Mined, None)), Seen("MINED".into()));
        for seen in ["SEEN_ON_NETWORK", "SEEN_ON_MULTIPLE_NODES", "ACCEPTED_BY_NETWORK"] {
            assert_eq!(net_verdict(&status(TxState::InMempool, Some(seen))), Seen(seen.into()));
        }
        // received by the broadcaster is not yet a verdict
        for early in ["RECEIVED", "QUEUED", "STORED", "ANNOUNCED_TO_NETWORK", "SENT_TO_NETWORK"] {
            assert_eq!(net_verdict(&status(TxState::InMempool, Some(early))), Pending, "{early}");
        }
        assert_eq!(net_verdict(&status(TxState::InMempool, None)), Seen("IN_MEMPOOL".into()));
        assert_eq!(net_verdict(&status(TxState::Unknown, None)), Pending);
        assert_eq!(net_verdict(&status(TxState::Rejected, Some("REJECTED"))), Refused("REJECTED".into()));
        assert_eq!(
            net_verdict(&status(TxState::DoubleSpendAttempted, Some("DOUBLE_SPEND_ATTEMPTED"))),
            Refused("DOUBLE_SPEND_ATTEMPTED".into())
        );
    }

    fn row() -> BoltTokenRow {
        serde_json::from_value(serde_json::json!({
            "outpoint": format!("{}.0", "11".repeat(32)),
            "type": "SimpleMultiBOLT",
            "issuer": format!("02{}", "ab".repeat(32)),
            "owner_pkh": "cd".repeat(20),
            "amount": "1000",
            "attributes": "{}",
            "beef": "0101010100beef",
            "anchor_txid": "11".repeat(32),
            "anchor_kind": "settle",
            "anchor_network": "accepted",
            "anchor_proven": 0,
            "provenance": "{\"kind\":\"mint\"}"
        }))
        .unwrap()
    }

    #[test]
    fn a_well_formed_row_passes_and_malformed_ones_do_not() {
        assert_eq!(validate_row(&row()), Ok(()));

        let cases: Vec<(&str, Box<dyn Fn(&mut BoltTokenRow)>)> = vec![
            ("outpoint", Box::new(|t| t.outpoint = "nope".into())),
            ("outpoint vout", Box::new(|t| t.outpoint = format!("{}.x", "11".repeat(32)))),
            ("type", Box::new(|t| t.token_type = "a b; DROP".into())),
            ("issuer", Box::new(|t| t.issuer = "02ab".into())),
            ("owner", Box::new(|t| t.owner_pkh = Some("zz".repeat(20)))),
            ("amount", Box::new(|t| t.amount = Some("-5".into()))),
            ("amount size", Box::new(|t| t.amount = Some("9".repeat(40)))),
            ("attributes", Box::new(|t| t.attributes = "[1,2]".into())),
            ("beef", Box::new(|t| t.beef = "not hex".into())),
            ("anchor kind", Box::new(|t| t.anchor_kind = Some("other".into()))),
            ("merkle root", Box::new(|t| t.anchor_merkle_root = Some("ab".into()))),
            ("height", Box::new(|t| t.anchor_height = Some(-1))),
            ("provenance", Box::new(|t| t.provenance = Some("\"text\"".into()))),
        ];
        for (name, mutate) in cases {
            let mut t = row();
            mutate(&mut t);
            assert!(validate_row(&t).is_err(), "{name} should be refused");
        }
    }

    #[test]
    fn a_site_sees_every_token_but_identities_not_linked_to_it() {
        let fungible = row();
        let mut identity = row();
        identity.token_type = "AuthBOLT".into();
        identity.attributes = r#"{"wallet":{"apps":[{"domain":"peerloop.example","appPubKey":"02aa"}]}}"#.into();
        let mut unlinked = identity.clone();
        unlinked.attributes = r#"{"wallet":{"apps":[]}}"#.into();
        let mut bare = identity.clone();
        bare.attributes = "{}".into();

        assert!(visible_to(&fungible, Some("anyone.example")), "other token types are unchanged");
        assert!(visible_to(&identity, Some("peerloop.example")));
        assert!(!visible_to(&identity, Some("evil.example")), "another site does not see it");
        assert!(!visible_to(&identity, Some("peerloop.example.evil")), "the domain must match exactly");
        assert!(!visible_to(&unlinked, Some("peerloop.example")));
        assert!(!visible_to(&bare, Some("peerloop.example")));
        for r in [&identity, &unlinked, &bare] {
            assert!(visible_to(r, None), "Hodos's own UI sees every identity");
        }
    }

    #[test]
    fn a_site_is_not_shown_the_wallets_private_notes() {
        // attributes.wallet holds the identity's key IDs and every site it is linked to. A site
        // must not see it (audit H2); the wallet's own UI still does.
        let mut identity = row();
        identity.token_type = "AuthBOLT".into();
        identity.attributes = r#"{"wallet":{"holderKeyId":"authbolt-abc","apps":[{"domain":"peerloop.example","appPubKey":"02aa"}]},"note":"keep"}"#.into();

        let internal = redacted_for_site(identity.clone(), None);
        assert!(internal.attributes.contains("holderKeyId"), "the wallet's own UI keeps its notes");

        let shown = redacted_for_site(identity.clone(), Some("peerloop.example"));
        assert!(!shown.attributes.contains("wallet"), "wallet notes stripped: {}", shown.attributes);
        assert!(!shown.attributes.contains("holderKeyId"));
        assert!(!shown.attributes.contains("peerloop.example"));
        assert!(shown.attributes.contains("keep"), "other attributes stay: {}", shown.attributes);
    }

    #[test]
    fn token_requests_parse_by_op() {
        let put: TokensRequest = serde_json::from_value(serde_json::json!({ "op": "put", "row": serde_json::to_value(row()).unwrap() })).unwrap();
        assert!(matches!(put, TokensRequest::Put { .. }));
        let list: TokensRequest = serde_json::from_str(r#"{"op":"list","type":"AuthBOLT"}"#).unwrap();
        assert!(matches!(list, TokensRequest::List { token_type: Some(t), status: None, .. } if t == "AuthBOLT"));
        let spend: TokensRequest = serde_json::from_str(r#"{"op":"spend","outpoint":"x"}"#).unwrap();
        assert!(matches!(spend, TokensRequest::Spend { .. }));
        assert!(serde_json::from_str::<TokensRequest>(r#"{"op":"delete","outpoint":"x"}"#).is_err(), "there is no delete");
        let annotate: TokensRequest = serde_json::from_str(r#"{"op":"annotate","outpoint":"x","wallet":{"apps":[]}}"#).unwrap();
        assert!(matches!(annotate, TokensRequest::Annotate { .. }));
    }
}
