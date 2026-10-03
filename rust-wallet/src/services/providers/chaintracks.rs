//! Chaintracks header provider — block headers from an Arcade chaintracks server
//! (e.g. the spv-testnet stack at `http://localhost:8083/chaintracks/v2`).
//!
//! The server returns header fields as JSON; this rebuilds the 80-byte header and
//! checks that its double-SHA256 equals the reported `hash`, so a response whose
//! fields disagree with its hash is rejected. It does NOT check proof-of-work,
//! linkage or chain selection: that is the separate header-hardening work (WS4).
//!
//! Enabled by `HODOS_CHAINTRACKS_URL`. When set it is the only provider on the
//! `get_block_header` chain.

use async_trait::async_trait;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::services::provider::{BlockHeader, BlockKey, IndexerError, IndexerProvider, ProviderOp};

const NAME: &str = "chaintracks";
pub const ENV_URL: &str = "HODOS_CHAINTRACKS_URL";

/// Base URL (including the `/chaintracks/v2` path), without a trailing slash.
pub fn configured_base_url() -> Option<String> {
    std::env::var(ENV_URL)
        .ok()
        .map(|u| u.trim().trim_end_matches('/').to_string())
        .filter(|u| !u.is_empty())
}

pub struct ChaintracksProvider {
    client: reqwest::Client,
    base: String,
}

impl ChaintracksProvider {
    pub fn new(client: reqwest::Client, base: String) -> Self {
        Self { client, base }
    }

    pub fn from_env(client: reqwest::Client) -> Option<Self> {
        configured_base_url().map(|base| Self::new(client, base))
    }

    /// The server's own claimed tip height (`GET /height`). A hint for how far to
    /// sync; the wallet's header chain never trusts it for chain selection.
    pub async fn tip_height(&self) -> Result<u32, IndexerError> {
        let resp = self
            .client
            .get(format!("{}/height", self.base))
            .send()
            .await
            .map_err(|e| IndexerError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(IndexerError::ProviderStatus {
                provider: NAME,
                status: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }
        let v: Value = resp
            .json()
            .await
            .map_err(|e| IndexerError::Transport(e.to_string()))?;
        v.get("height")
            .and_then(|h| h.as_u64())
            .and_then(|h| u32::try_from(h).ok())
            .ok_or(IndexerError::InvalidResponse {
                provider: NAME,
                reason: "missing 'height'".into(),
            })
    }
}

#[async_trait]
impl IndexerProvider for ChaintracksProvider {
    fn name(&self) -> &'static str {
        NAME
    }

    fn supports(&self, op: ProviderOp) -> bool {
        matches!(op, ProviderOp::BlockHeader)
    }

    async fn get_block_header(&self, key: BlockKey) -> Result<BlockHeader, IndexerError> {
        let path = match &key {
            BlockKey::Height(h) => format!("header/height/{}", h),
            BlockKey::Hash(h) => format!("header/hash/{}", h),
        };
        let resp = self
            .client
            .get(format!("{}/{}", self.base, path))
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
        let json: Value = serde_json::from_str(&text).map_err(|e| IndexerError::InvalidResponse {
            provider: NAME,
            reason: format!("chaintracks parse error: {}", e),
        })?;
        let header = parse_header(&json).map_err(|reason| IndexerError::InvalidResponse {
            provider: NAME,
            reason,
        })?;
        // The server must answer the question that was asked.
        let asked_ok = match &key {
            BlockKey::Height(h) => header.height == *h,
            BlockKey::Hash(h) => header.block_hash.eq_ignore_ascii_case(h),
        };
        if !asked_ok {
            return Err(IndexerError::InvalidResponse {
                provider: NAME,
                reason: format!(
                    "answered {:?} with header {} at height {}",
                    key, header.block_hash, header.height
                ),
            });
        }
        Ok(header)
    }
}

fn hash_wire_bytes(v: &Value, field: &str) -> Result<Vec<u8>, String> {
    let s = v
        .get(field)
        .and_then(|x| x.as_str())
        .ok_or(format!("missing '{}'", field))?;
    let mut b = hex::decode(s).map_err(|e| format!("bad hex in '{}': {}", field, e))?;
    if b.len() != 32 {
        return Err(format!("'{}' is {} bytes, expected 32", field, b.len()));
    }
    b.reverse(); // display order -> wire order
    Ok(b)
}

fn u32_field(v: &Value, field: &str) -> Result<u32, String> {
    v.get(field)
        .and_then(|x| x.as_u64())
        .and_then(|x| u32::try_from(x).ok())
        .ok_or(format!("missing or out-of-range '{}'", field))
}

/// Rebuild the 80-byte header from chaintracks JSON and verify it hashes to `hash`.
pub(crate) fn parse_header(v: &Value) -> Result<BlockHeader, String> {
    let mut raw = Vec::with_capacity(80);
    raw.extend_from_slice(&u32_field(v, "version")?.to_le_bytes());
    raw.extend_from_slice(&hash_wire_bytes(v, "previousHash")?);
    raw.extend_from_slice(&hash_wire_bytes(v, "merkleRoot")?);
    raw.extend_from_slice(&u32_field(v, "time")?.to_le_bytes());
    raw.extend_from_slice(&u32_field(v, "bits")?.to_le_bytes());
    raw.extend_from_slice(&u32_field(v, "nonce")?.to_le_bytes());
    let height = u32_field(v, "height")?;

    let mut computed: Vec<u8> = Sha256::digest(Sha256::digest(&raw)).to_vec();
    computed.reverse();
    let computed_hex = hex::encode(computed);
    let reported = v
        .get("hash")
        .and_then(|x| x.as_str())
        .ok_or("missing 'hash'")?;
    if !computed_hex.eq_ignore_ascii_case(reported) {
        return Err(format!(
            "header fields hash to {} but server reported {}",
            computed_hex, reported
        ));
    }
    Ok(BlockHeader {
        block_hash: computed_hex,
        height,
        header_hex: hex::encode(raw),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Real header served by the spv-testnet stack (height 5).
    fn sample() -> Value {
        json!({
            "version": 536870912,
            "previousHash": "231deda25164376fdb28cc5490770f35f02ad27919e48903a74bc0ae88f00c0c",
            "merkleRoot": "9d34d4193f6fa3a62e872d4832cd4b16a579541b05daeadee9cbd12a406f1146",
            "time": 1791033023,
            "bits": 545259519,
            "nonce": 3,
            "height": 5,
            "hash": "5ee1c44ffb4c19f3526a5c4fdd9049234123609d1252cb7cf7d5efac215eb6a0"
        })
    }

    #[test]
    fn supports_block_header_only() {
        let p = ChaintracksProvider::new(reqwest::Client::new(), "http://x".into());
        assert!(p.supports(ProviderOp::BlockHeader));
        for op in [
            ProviderOp::RawTx,
            ProviderOp::TxStatus,
            ProviderOp::BroadcastBeef,
            ProviderOp::FetchUtxos,
        ] {
            assert!(!p.supports(op));
        }
    }

    #[test]
    fn valid_header_parses_to_80_bytes() {
        let h = parse_header(&sample()).unwrap();
        assert_eq!(h.height, 5);
        assert_eq!(h.header_hex.len(), 160);
        assert_eq!(
            h.block_hash,
            "5ee1c44ffb4c19f3526a5c4fdd9049234123609d1252cb7cf7d5efac215eb6a0"
        );
    }

    #[test]
    fn tampered_field_is_rejected() {
        let mut j = sample();
        j["nonce"] = json!(4);
        assert!(parse_header(&j).unwrap_err().contains("hash to"));
    }

    #[test]
    fn missing_field_is_rejected() {
        let mut j = sample();
        j.as_object_mut().unwrap().remove("bits");
        assert!(parse_header(&j).is_err());
    }

    /// Live check against the spv-testnet stack: `cargo test --lib chaintracks_live -- --ignored`.
    #[tokio::test]
    #[ignore]
    async fn chaintracks_live_header_by_height_and_hash() {
        let base = configured_base_url()
            .unwrap_or_else(|| "http://localhost:8083/chaintracks/v2".into());
        let p = ChaintracksProvider::new(reqwest::Client::new(), base);
        let by_height = p.get_block_header(BlockKey::Height(5)).await.expect("by height");
        let by_hash = p
            .get_block_header(BlockKey::Hash(by_height.block_hash.clone()))
            .await
            .expect("by hash");
        assert_eq!(by_height.header_hex, by_hash.header_hex);
        assert!(matches!(
            p.get_block_header(BlockKey::Height(99_999_999)).await,
            Err(IndexerError::NotFound)
        ));
    }
}
