//! Arcade provider — broadcast, tx status and merkle proofs from a self-hosted
//! [Arcade](https://github.com/bsv-blockchain/arcade) instance (e.g. the local
//! spv-testnet stack). Arcade is ARC-compatible, so this reuses the GorillaPool ARC
//! response types and status mapping.
//!
//! Differences from ARC GorillaPool: paths have no `/v1` prefix (`POST /tx`,
//! `GET /tx/{txid}`), a new submission answers `202`, and there is no API key.
//!
//! Enabled by setting `HODOS_ARCADE_URL` (e.g. `http://localhost:8080`). When set it
//! replaces ARC GorillaPool and TAAL in the broadcast, tx-status and proof chains.

use async_trait::async_trait;
use serde_json::Value;

use super::arc_gorillapool::{arc_response_to_tx_status, interpret_broadcast_response, ArcResponse};
use crate::services::provider::{
    BroadcastResult, IndexerError, IndexerProvider, ProviderOp, TxStatus,
};

const NAME: &str = "arcade";
pub const ENV_URL: &str = "HODOS_ARCADE_URL";

/// Base URL of the configured Arcade instance, without a trailing slash.
pub fn configured_base_url() -> Option<String> {
    crate::chain_mode::spv_url(ENV_URL)
}

pub struct ArcadeProvider {
    client: reqwest::Client,
    base: String,
}

impl ArcadeProvider {
    pub fn new(client: reqwest::Client, base: String) -> Self {
        Self { client, base }
    }

    pub fn from_env(client: reqwest::Client) -> Option<Self> {
        configured_base_url().map(|base| Self::new(client, base))
    }

    /// Re-submit an Extended Format tx Arcade may already know, only to subscribe this wallet's
    /// callback token to its status updates (Arcade treats a re-submit as idempotent). Best-effort:
    /// the proof task polls whether or not this works. Returns the HTTP status.
    pub async fn register_for_push(&self, ef_hex: &str) -> Result<u16, String> {
        let Some(token) = crate::arcade_push::token() else {
            return Err("no callback token yet".to_string());
        };
        let resp = self
            .client
            .post(format!("{}/tx", self.base))
            .header("Content-Type", "text/plain")
            .header("X-CallbackToken", token)
            .body(ef_hex.to_string())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Ok(resp.status().as_u16())
    }
}

#[async_trait]
impl IndexerProvider for ArcadeProvider {
    fn name(&self) -> &'static str {
        NAME
    }

    fn supports(&self, op: ProviderOp) -> bool {
        matches!(
            op,
            ProviderOp::TxStatus | ProviderOp::MerkleProof | ProviderOp::BroadcastBeef
        )
    }

    async fn tx_status(&self, txid: &str) -> Result<TxStatus, IndexerError> {
        let resp = self
            .client
            .get(format!("{}/tx/{}", self.base, txid))
            .send()
            .await
            .map_err(|e| IndexerError::Transport(e.to_string()))?;
        let status = resp.status();
        if status.as_u16() == 404 {
            return Err(IndexerError::NotFound);
        }
        let text = resp
            .text()
            .await
            .map_err(|e| IndexerError::Transport(e.to_string()))?;
        if !status.is_success() {
            return Err(IndexerError::ProviderStatus {
                provider: NAME,
                status: status.as_u16(),
                body: text,
            });
        }
        let arc: ArcResponse =
            serde_json::from_str(&text).map_err(|e| IndexerError::InvalidResponse {
                provider: NAME,
                reason: format!("Arcade parse error: {}", e),
            })?;
        Ok(arc_response_to_tx_status(txid, &arc))
    }

    async fn get_merkle_proof_tsc(&self, txid: &str) -> Result<Value, IndexerError> {
        let status = self.tx_status(txid).await?;
        let bump_hex = status.merkle_path_bump.ok_or(IndexerError::NotFound)?;
        let mut tsc = crate::beef::parse_bump_hex_to_tsc(&bump_hex).map_err(|e| {
            IndexerError::InvalidResponse {
                provider: NAME,
                reason: format!("BUMP parse error: {}", e),
            }
        })?;
        if let Some(h) = status.block_height {
            tsc["height"] = serde_json::json!(h);
        }
        Ok(tsc)
    }

    async fn broadcast_beef(&self, beef: &[u8]) -> Result<BroadcastResult, IndexerError> {
        // Arcade parses the body as a raw or Extended Format transaction; it does not
        // accept BEEF. Convert (the parents inside the BEEF supply the EF source data).
        let body = arcade_body_from_beef(beef).map_err(|reason| IndexerError::InvalidResponse {
            provider: NAME,
            reason,
        })?;
        let mut req = self
            .client
            .post(format!("{}/tx", self.base))
            .header("Content-Type", "text/plain");
        // Subscribe this wallet's callback token so Arcade pushes this tx's status over SSE.
        if let Some(token) = callback_token_header(crate::chain_mode::push_enabled(), crate::arcade_push::token()) {
            req = req.header("X-CallbackToken", token);
        }
        let resp = req
            .body(body)
            .send()
            .await
            .map_err(|e| IndexerError::Transport(e.to_string()))?;
        let http_status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        // Rejections come back as `{"error": ...}` or ARC problem JSON; every
        // `ArcResponse` field is optional, so either parses and the status code
        // decides in `interpret_broadcast_response`.
        let arc: ArcResponse =
            serde_json::from_str(&text).map_err(|e| IndexerError::InvalidResponse {
                provider: NAME,
                reason: format!(
                    "Arcade parse error: {} — body: {}",
                    e,
                    &text[..text.len().min(200)]
                ),
            })?;
        interpret_broadcast_response(http_status, &arc, &text).map(|mut r| {
            r.provider = NAME;
            r
        })
    }
}

/// The `X-CallbackToken` to send with a submission: only when push is on and a token exists.
pub(crate) fn callback_token_header(push_enabled: bool, token: Option<String>) -> Option<String> {
    if push_enabled { token } else { None }
}

/// Hex body for Arcade's `POST /tx` (`text/plain`) from the bytes Hodos broadcasts.
///
/// BEEF (Atomic, V1 or V2) becomes the main transaction in Extended Format, built from the
/// parents carried in the BEEF; it fails if a parent is missing, because Arcade cannot
/// validate scripts without the source outputs. Bytes that are not a BEEF are assumed to
/// already be a raw/EF transaction and are sent as they are.
pub(crate) fn arcade_body_from_beef(bytes: &[u8]) -> Result<String, String> {
    let parsed = if bytes.len() >= 4 && bytes[0..4] == crate::beef::ATOMIC_BEEF_MARKER {
        crate::beef::Beef::from_atomic_beef_bytes(bytes).map(|(_, b)| b)
    } else {
        crate::beef::Beef::from_bytes(bytes)
    };
    match parsed {
        Ok(beef) => beef
            .to_ef_hex()
            .map_err(|e| format!("cannot build Extended Format for Arcade: {}", e)),
        Err(_) => Ok(hex::encode(bytes)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_tx(prev_txid_wire: [u8; 32], prev_vout: u32, out_sats: u64) -> Vec<u8> {
        let mut t = Vec::new();
        t.extend_from_slice(&1u32.to_le_bytes());
        t.push(1); // one input
        t.extend_from_slice(&prev_txid_wire);
        t.extend_from_slice(&prev_vout.to_le_bytes());
        t.push(0); // empty scriptSig
        t.extend_from_slice(&0xffff_ffffu32.to_le_bytes());
        t.push(1); // one output
        t.extend_from_slice(&out_sats.to_le_bytes());
        t.push(1);
        t.push(0x51); // OP_TRUE
        t.extend_from_slice(&0u32.to_le_bytes());
        t
    }

    #[test]
    fn beef_is_converted_to_extended_format_with_the_parents_outputs() {
        use sha2::{Digest, Sha256};
        let parent = raw_tx([7u8; 32], 0, 5000);
        let parent_txid_wire: [u8; 32] = Sha256::digest(Sha256::digest(&parent)).into();
        let child = raw_tx(parent_txid_wire, 0, 4000);

        let mut beef = crate::beef::Beef::new();
        beef.add_parent_transaction(parent);
        beef.set_main_transaction(child);
        let bytes = beef.to_bytes().expect("beef bytes");

        let ef = arcade_body_from_beef(&bytes).expect("ef");
        let raw = hex::decode(&ef).unwrap();
        assert_eq!(&raw[4..10], &[0, 0, 0, 0, 0, 0xEF], "EF marker after the version");
        // the source output (5000 sats = 0x1388 LE) sits after the input's sequence
        assert!(raw.windows(8).any(|w| w == 5000u64.to_le_bytes()), "source satoshis present");
    }

    #[test]
    fn a_missing_parent_is_an_error_not_a_silent_fallback() {
        let child = raw_tx([9u8; 32], 0, 1000);
        let mut beef = crate::beef::Beef::new();
        beef.set_main_transaction(child);
        let bytes = beef.to_bytes().expect("beef bytes");
        assert!(arcade_body_from_beef(&bytes).unwrap_err().contains("Extended Format"));
    }

    #[test]
    fn non_beef_bytes_pass_through_unchanged() {
        let raw = raw_tx([1u8; 32], 0, 10);
        assert_eq!(arcade_body_from_beef(&raw).unwrap(), hex::encode(&raw));
    }

    #[test]
    fn callback_token_is_sent_only_when_push_is_on_and_a_token_exists() {
        let t = Some("tok".to_string());
        assert_eq!(callback_token_header(true, t.clone()).as_deref(), Some("tok"));
        assert_eq!(callback_token_header(false, t), None, "push off: no subscription");
        assert_eq!(callback_token_header(true, None), None, "no token yet (wallet locked): plain submit");
    }

    #[test]
    fn supports_broadcast_status_and_proof_only() {
        let p = ArcadeProvider::new(reqwest::Client::new(), "http://localhost:8080".into());
        for op in [ProviderOp::TxStatus, ProviderOp::MerkleProof, ProviderOp::BroadcastBeef] {
            assert!(p.supports(op), "arcade must support {:?}", op);
        }
        for op in [
            ProviderOp::RawTx,
            ProviderOp::BlockHeader,
            ProviderOp::Outspend,
            ProviderOp::FetchUtxos,
        ] {
            assert!(!p.supports(op), "arcade must not claim {:?}", op);
        }
    }

    #[test]
    fn accepted_202_is_success() {
        let arc: ArcResponse =
            serde_json::from_str(r#"{"txid":"ab","status":202,"txStatus":"RECEIVED"}"#).unwrap();
        let r = interpret_broadcast_response(202, &arc, "").unwrap();
        assert_eq!(r.txid, "ab");
        assert_eq!(r.tx_status, "RECEIVED");
    }

    #[test]
    fn error_body_with_400_is_provider_status_error() {
        let body = r#"{"error":"fee too low"}"#;
        let arc: ArcResponse = serde_json::from_str(body).unwrap();
        match interpret_broadcast_response(400, &arc, body) {
            Err(IndexerError::ProviderStatus { status: 400, .. }) => {}
            other => panic!("expected ProviderStatus 400, got {:?}", other),
        }
    }

    /// Live check against the spv-testnet stack (`stack.ps1 up`). Run with
    /// `HODOS_LIVE_RAWTX_HEX=<signed tx hex> cargo test --lib arcade_live -- --ignored --nocapture`.
    /// Needs a mature coinbase spend that has not been broadcast yet.
    #[tokio::test]
    #[ignore]
    async fn arcade_live_broadcast_status_and_proof() {
        use crate::services::provider::TxState;
        let raw = std::env::var("HODOS_LIVE_RAWTX_HEX").expect("HODOS_LIVE_RAWTX_HEX");
        let base = std::env::var(crate::chain_mode::ENV_ARCADE).unwrap_or_else(|_| "http://localhost:8080".into());
        let client = reqwest::Client::new();
        let p = ArcadeProvider::new(client.clone(), base);

        let sent = p.broadcast_beef(&hex::decode(raw).unwrap()).await.expect("broadcast");
        println!("broadcast: {:?}", sent);
        assert_eq!(sent.provider, "arcade");

        // Let block assembly pick the tx up, then mine until Arcade reports it MINED.
        tokio::time::sleep(std::time::Duration::from_secs(8)).await;
        let mut status = None;
        for _ in 0..5 {
            let r = client
                .post("http://localhost:29292")
                .basic_auth("bitcoin", Some("bitcoin"))
                .json(&serde_json::json!({"method": "generate", "params": [1]}))
                .timeout(std::time::Duration::from_secs(200))
                .send()
                .await
                .expect("mine");
            println!("generate: {}", r.status());
            for _ in 0..15 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                let s = p.tx_status(&sent.txid).await.expect("status");
                println!("status: {:?} {:?}", s.state, s.raw_provider_status);
                if s.state == TxState::Mined {
                    status = Some(s);
                    break;
                }
            }
            if status.is_some() {
                break;
            }
        }
        let status = status.expect("tx never MINED");
        assert!(status.merkle_path_bump.is_some());

        let tsc = p.get_merkle_proof_tsc(&sent.txid).await.expect("proof");
        let root = crate::beef::compute_merkle_root_from_tsc(
            &sent.txid,
            tsc["height"].as_u64().unwrap() as u32,
            tsc["index"].as_u64().unwrap(),
            tsc["nodes"].as_array().unwrap(),
        )
        .expect("root");
        let hdr: serde_json::Value = client
            .get(format!("http://localhost:8083/chaintracks/v2/header/height/{}", tsc["height"]))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        println!("computed root {} / header root {}", root, hdr["merkleRoot"]);
        assert_eq!(hdr["merkleRoot"].as_str().unwrap(), root);
    }
}
