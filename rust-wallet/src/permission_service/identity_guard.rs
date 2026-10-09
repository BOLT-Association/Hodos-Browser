//! AuthBOLT identity keys are the wallet's own.
//!
//! A person's AuthBOLT identities are minted and presented only by Hodos's own prompt (the
//! frontend's BOLT identity overlay, which calls the wallet as an internal origin). Each identity
//! is a token under keys derived with the BRC-43 protocol `authbolt identity`. A site must never
//! derive, sign or encrypt with those keys: if it could, it could build a presentation itself and
//! skip the prompt. So any call from a site (a request with `X-Requesting-Domain`) that names that
//! protocol is refused, whatever grants the site holds. Hodos's own UI sends no domain and passes.
//!
//! Kept to the protocol's name: the level does not matter, and BRC-43 names compare after
//! lower-casing and collapsing whitespace.

use actix_web::{HttpRequest, HttpResponse};

use super::request_gate::X_REQUESTING_DOMAIN;
use crate::crypto::brc43::normalize_protocol_id;

/// The BRC-43 protocol AuthBOLT identity keys are derived under (ChainBrowsers packages/bolt
/// `IDENTITY_PROTOCOL`).
pub const IDENTITY_PROTOCOL_NAME: &str = "authbolt identity";

/// Whether a BRC-43 protocol name is the identity protocol.
pub fn is_identity_protocol(name: &str) -> bool {
    name.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase() == IDENTITY_PROTOCOL_NAME
}

/// The refusal for a site that names the identity protocol; `None` for the wallet's own UI or
/// any other protocol.
pub fn refuse_identity_protocol(http_req: &HttpRequest, protocol_name: &str) -> Option<HttpResponse> {
    let from_site = http_req
        .headers()
        .get(X_REQUESTING_DOMAIN)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|d| !d.is_empty());
    if from_site && is_identity_protocol(protocol_name) {
        log::warn!("🛡️ a site named the AuthBOLT identity protocol: refused");
        return Some(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "AuthBOLT identity keys are the wallet's own: a site cannot use them. Ask with BOLT.requestPresentation.",
            "code": "IDENTITY_PROTOCOL",
            "status": "error"
        })));
    }
    None
}

/// The protocol name a BRC-43 invoice is built from: normalised (lower-cased, single spaces, and
/// only letters, digits and spaces, as BRC-43 requires), then refused for a site that names the
/// identity protocol. Normalising first is what closes the collision in audit H1: a name such as
/// `authbolt identity-authbolt` would otherwise build the identity key's invoice, but the hyphen
/// is not a legal BRC-43 character, so it is refused here before any key is derived. Every key
/// operation (getPublicKey, createSignature, …) must resolve its name through this, so the derived
/// key can never depend on an un-normalised name.
pub fn guarded_protocol_name(http_req: &HttpRequest, name: &str) -> Result<String, HttpResponse> {
    let normalized = normalize_protocol_id(name).map_err(|e| {
        HttpResponse::BadRequest().json(serde_json::json!({
            "error": e,
            "status": "error",
        }))
    })?;
    if let Some(refused) = refuse_identity_protocol(http_req, &normalized) {
        return Err(refused);
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn the_identity_protocol_is_recognised_however_it_is_spaced_or_cased() {
        for name in ["authbolt identity", "AuthBOLT Identity", "  authbolt   identity "] {
            assert!(is_identity_protocol(name), "{name}");
        }
        for name in ["bolt token", "authbolt", "authbolt identity2", ""] {
            assert!(!is_identity_protocol(name), "{name}");
        }
    }

    #[test]
    fn the_invoice_name_is_normalised_so_the_identity_key_cannot_be_reached_by_a_collision() {
        // The identity invoice is `2-authbolt identity-<keyId>`. A site that asks to sign under
        // protocol name "authbolt identity-authbolt" with a crafted keyId would build the same
        // invoice and so the same key. Normalising the name refuses the `-`, closing audit H1.
        let site = TestRequest::default().insert_header((X_REQUESTING_DOMAIN, "peerloop.example")).to_http_request();
        let collision = guarded_protocol_name(&site, "authbolt identity-authbolt");
        assert_eq!(collision.err().expect("a hyphenated name is refused").status(), 400);
        // Naming the identity protocol outright is refused for a site (403).
        assert_eq!(guarded_protocol_name(&site, "authbolt identity").err().expect("refused").status(), 403);
        // The wallet's own UI derives identity keys; a general protocol normalises to lower case.
        let own = TestRequest::default().to_http_request();
        assert_eq!(guarded_protocol_name(&own, "authbolt identity").ok().as_deref(), Some("authbolt identity"));
        assert_eq!(guarded_protocol_name(&site, "Bolt  Token").ok().as_deref(), Some("bolt token"));
    }

    #[test]
    fn a_site_is_refused_and_the_wallet_ui_is_not() {
        let site = TestRequest::default().insert_header((X_REQUESTING_DOMAIN, "peerloop.example")).to_http_request();
        let refused = refuse_identity_protocol(&site, "authbolt identity").expect("a site is refused");
        assert_eq!(refused.status(), 403);
        assert!(refuse_identity_protocol(&site, "bolt token").is_none(), "other protocols are the gate's business");
        let own_ui = TestRequest::default().to_http_request();
        assert!(refuse_identity_protocol(&own_ui, "authbolt identity").is_none());
        let empty = TestRequest::default().insert_header((X_REQUESTING_DOMAIN, "")).to_http_request();
        assert!(refuse_identity_protocol(&empty, "authbolt identity").is_none(), "an empty domain is the wallet's own UI");
    }
}
