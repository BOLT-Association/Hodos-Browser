//! WhatsOnChain provider — broadest API coverage (all 7 ops). Mainnet-only.
//!
//! Lifted from existing call sites in `cache_helpers.rs`, `utxo_fetcher.rs`,
//! `handlers.rs`. Phase 1.6d.B step 4.

use async_trait::async_trait;
use serde_json::Value;

use crate::header_chain::Header;
use crate::services::collection::ProviderCollection;
use crate::services::provider::{
    verified_block_header, BlockHeader, BlockKey, BroadcastResult, IndexerError, IndexerProvider,
    OutspendStatus, ProviderOp, TxState, TxStatus,
};
use crate::utxo_fetcher::UTXO;

const NAME: &str = "whatsonchain";
const BASE: &str = "https://api.whatsonchain.com/v1/bsv/main";

pub struct WhatsOnChainProvider {
    client: reqwest::Client,
}

impl WhatsOnChainProvider {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    fn url(path: &str) -> String {
        format!("{}{}", BASE, path)
    }

    async fn get_text(&self, path: &str) -> Result<String, IndexerError> {
        let resp = self
            .client
            .get(Self::url(path))
            .send()
            .await
            .map_err(|e| IndexerError::Transport(e.to_string()))?;
        let status = resp.status();
        if status.as_u16() == 404 {
            return Err(IndexerError::NotFound);
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(IndexerError::ProviderStatus {
                provider: NAME,
                status: status.as_u16(),
                body,
            });
        }
        resp.text()
            .await
            .map_err(|e| IndexerError::Transport(e.to_string()))
    }

    async fn get_json(&self, path: &str) -> Result<Value, IndexerError> {
        let text = self.get_text(path).await?;
        serde_json::from_str(&text).map_err(|e| IndexerError::InvalidResponse {
            provider: NAME,
            reason: format!("JSON parse error: {}", e),
        })
    }
}

#[async_trait]
impl IndexerProvider for WhatsOnChainProvider {
    fn name(&self) -> &'static str {
        NAME
    }

    fn supports(&self, op: ProviderOp) -> bool {
        // WoC supports everything — broadcast goes through tx/raw on the main tx.
        match op {
            ProviderOp::RawTx
            | ProviderOp::MerkleProof
            | ProviderOp::BlockHeader
            | ProviderOp::TxStatus
            | ProviderOp::Outspend
            | ProviderOp::FetchUtxos
            | ProviderOp::BroadcastBeef => true,
        }
    }

    async fn get_raw_tx(&self, txid: &str) -> Result<Vec<u8>, IndexerError> {
        let hex_str = self.get_text(&format!("/tx/{}/hex", txid)).await?;
        let hex_str = hex_str.trim();
        hex::decode(hex_str).map_err(|e| IndexerError::InvalidResponse {
            provider: NAME,
            reason: format!("hex decode error: {}", e),
        })
    }

    async fn get_merkle_proof_tsc(&self, txid: &str) -> Result<Value, IndexerError> {
        // WoC may return `null` (proof not yet available), an object, or an array.
        // Normalize array → first element; null → NotFound (treat as "no proof yet").
        let json = self.get_json(&format!("/tx/{}/proof/tsc", txid)).await?;
        parse_tsc_proof(json).ok_or(IndexerError::NotFound)
    }

    async fn get_block_header(&self, key: BlockKey) -> Result<BlockHeader, IndexerError> {
        // One hop: `/block/height/{h}` and `/block/hash/{hash}` carry the header fields, and the
        // 80 bytes are rebuilt from them and must hash to the reported hash (`parse_block_json`).
        // The former second hop, `/block/hash/{hash}/header`, is no longer served (404 since
        // 2026-10).
        let path = match &key {
            BlockKey::Height(h) => format!("/block/height/{}", h),
            BlockKey::Hash(hash) => format!("/block/hash/{}", hash),
        };
        let json = self.get_json(&path).await?;
        let header = parse_block_json(&json)
            .map_err(|reason| IndexerError::InvalidResponse { provider: NAME, reason })?;
        // The answer must be the block that was asked for.
        let asked_ok = match &key {
            BlockKey::Height(h) => header.height == *h,
            BlockKey::Hash(hash) => header.block_hash.eq_ignore_ascii_case(hash),
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

    async fn tx_status(&self, txid: &str) -> Result<TxStatus, IndexerError> {
        let json = self.get_json(&format!("/tx/hash/{}", txid)).await?;
        Ok(parse_tx_status(txid, &json))
    }

    async fn outspend(&self, txid: &str, vout: u32) -> Result<OutspendStatus, IndexerError> {
        // WoC `/tx/{txid}/outspend/{vout}`: 200 `{"spent":bool}` → status. A 404 is AMBIGUOUS
        // (spent OR the tx/output doesn't exist), so treat it as Unspent (FAIL-CLOSED) — never
        // claim Spent without proof. The old `404 → Spent` was fail-open and could mark good
        // UTXOs spent. For an authoritative spent-check use `reconcile::check_outpoint_spent`,
        // which cross-validates the correct `/tx/{txid}/{vout}/spent` endpoint across providers.
        match self
            .get_json(&format!("/tx/{}/outspend/{}", txid, vout))
            .await
        {
            Ok(json) => Ok(parse_outspend(json)),
            Err(IndexerError::NotFound) => Ok(OutspendStatus::Unspent),
            Err(e) => Err(e),
        }
    }

    async fn fetch_utxos(&self, address: &str) -> Result<Vec<UTXO>, IndexerError> {
        let json = self
            .get_json(&format!("/address/{}/unspent/all", address))
            .await?;
        parse_woc_utxos(&json, address).map_err(|reason| IndexerError::InvalidResponse {
            provider: NAME,
            reason,
        })
    }

    async fn broadcast_beef(&self, beef: &[u8]) -> Result<BroadcastResult, IndexerError> {
        // WoC's broadcast endpoint expects raw tx hex of the *main* tx, not BEEF.
        // Extract the main tx from the BEEF wrapper.
        let parsed = crate::beef::Beef::from_bytes(beef).map_err(|e| {
            IndexerError::InvalidResponse {
                provider: NAME,
                reason: format!("BEEF parse error: {}", e),
            }
        })?;
        let main_tx = parsed.main_transaction().ok_or(IndexerError::InvalidResponse {
            provider: NAME,
            reason: "BEEF has no main transaction".to_string(),
        })?;
        let raw_tx_hex = hex::encode(main_tx);

        let body = serde_json::json!({ "txhex": raw_tx_hex });
        let resp = self
            .client
            .post(Self::url("/tx/raw"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| IndexerError::Transport(e.to_string()))?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        if status.is_success() {
            let txid = text.trim().trim_matches('"').to_string();
            Ok(BroadcastResult {
                provider: NAME,
                txid,
                tx_status: "SEEN_ON_NETWORK".to_string(),
                merkle_path_bump: None,
                block_height: None,
            })
        } else {
            // Treat "already in mempool" / "already known" as success per the existing
            // handlers.rs:8992 logic.
            let lower = text.to_lowercase();
            if lower.contains("already in")
                || lower.contains("already known")
                || lower.contains("duplicate")
                || lower.contains("txn-already-in-mempool")
                || lower.contains("txn-already-known")
            {
                Ok(BroadcastResult {
                    provider: NAME,
                    txid: String::new(), // WoC error path doesn't return the txid.
                    tx_status: "ALREADY_KNOWN".to_string(),
                    merkle_path_bump: None,
                    block_height: None,
                })
            } else {
                Err(IndexerError::ProviderStatus {
                    provider: NAME,
                    status: status.as_u16(),
                    body: text,
                })
            }
        }
    }
}

// --- Pure parse helpers (unit-tested below) ---

/// Normalize a WoC TSC proof response: null → None; array → first element wrapped;
/// object → wrapped as-is.
pub(crate) fn parse_tsc_proof(json: Value) -> Option<Value> {
    if json.is_null() {
        return None;
    }
    if json.is_array() {
        return json.as_array().and_then(|a| a.get(0).cloned());
    }
    Some(json)
}

/// Parse WoC `/tx/hash/{txid}` response into a normalized `TxStatus`. WoC reports
/// `confirmations` and `blockheight`/`blockhash` — confirmations=0 means mempool;
/// missing `blockheight` also means mempool.
pub(crate) fn parse_tx_status(txid: &str, json: &Value) -> TxStatus {
    let confirmations = json.get("confirmations").and_then(|v| v.as_u64()).unwrap_or(0);
    let block_height = json.get("blockheight").and_then(|v| v.as_u64()).map(|h| h as u32);
    let block_hash = json
        .get("blockhash")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let state = if confirmations >= 1 || block_height.is_some() {
        TxState::Mined
    } else {
        TxState::InMempool
    };

    TxStatus {
        txid: txid.to_string(),
        state,
        block_height,
        block_hash,
        merkle_path_bump: None, // WoC doesn't return BUMP here.
        raw_provider_status: None, // WoC has no rich status vocabulary beyond TxState.
    }
}

/// Parse WoC `/tx/{txid}/outspend/{vout}` response.
pub(crate) fn parse_outspend(json: Value) -> OutspendStatus {
    let spent = json.get("spent").and_then(|v| v.as_bool()).unwrap_or(false);
    if spent {
        let spending_txid = json
            .get("txid")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();
        OutspendStatus::Spent {
            spending_txid,
            spending_vin: json
                .get("vin")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
        }
    } else {
        OutspendStatus::Unspent
    }
}

/// Parse WoC `/address/{addr}/unspent/all` response. Accepts both the wrapped
/// `{result: [...], error: ""}` shape and the legacy flat array.
pub(crate) fn parse_woc_utxos(json: &Value, address: &str) -> Result<Vec<UTXO>, String> {
    let script = generate_p2pkh_script_from_address(address)?;

    let entries: Vec<Value> = if let Some(obj) = json.as_object() {
        if let Some(err) = obj.get("error").and_then(|v| v.as_str()) {
            if !err.is_empty() {
                return Err(format!("WoC error: {}", err));
            }
        }
        obj.get("result")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
    } else if let Some(arr) = json.as_array() {
        arr.clone()
    } else {
        return Err("response was neither object nor array".into());
    };

    let utxos = entries
        .into_iter()
        .filter_map(|e| {
            let txid = e.get("tx_hash").and_then(|v| v.as_str())?.to_string();
            let vout = e.get("tx_pos").and_then(|v| v.as_u64())? as u32;
            let satoshis = e.get("value").and_then(|v| v.as_i64())?;
            let height = e.get("height").and_then(|v| v.as_i64()).unwrap_or(0);
            Some(UTXO {
                txid,
                vout,
                satoshis,
                script: script.clone(),
                address_index: -1,
                custom_instructions: None,
                confirmed: height > 0,
            })
        })
        .collect();

    Ok(utxos)
}

/// P2PKH script generation lifted from `utxo_fetcher::generate_p2pkh_script_from_address`.
/// Duplicated here to avoid bringing a `pub` on the original; this trait impl is the
/// only out-of-module caller and the function is small.
fn generate_p2pkh_script_from_address(address: &str) -> Result<String, String> {
    let decoded = bs58::decode(address)
        .with_check(None)
        .into_vec()
        .map_err(|e| format!("Invalid base58 address: {}", e))?;
    if decoded.len() != 21 {
        return Err(format!("Invalid decoded address length: {}", decoded.len()));
    }
    if decoded[0] != 0x00 {
        return Err(format!("Not a mainnet P2PKH address (version: 0x{:02x})", decoded[0]));
    }
    let pubkey_hash = &decoded[1..21];
    let mut script = Vec::new();
    script.push(0x76); // OP_DUP
    script.push(0xa9); // OP_HASH160
    script.push(0x14); // Push 20 bytes
    script.extend_from_slice(pubkey_hash);
    script.push(0x88); // OP_EQUALVERIFY
    script.push(0xac); // OP_CHECKSIG
    Ok(hex::encode(&script))
}

// Suppress unused-warning on ProviderCollection import — it's used transitively by
// downstream WalletServices construction in step 11.
#[allow(dead_code)]
fn _ensure_collection_in_scope() -> Option<ProviderCollection<dyn IndexerProvider>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_tsc_proof_null_returns_none() {
        assert!(parse_tsc_proof(Value::Null).is_none());
    }

    #[test]
    fn parse_tsc_proof_object_returns_object() {
        let v = json!({"height": 800000, "index": 0, "nodes": []});
        assert_eq!(parse_tsc_proof(v.clone()), Some(v));
    }

    #[test]
    fn parse_tsc_proof_array_unwraps_first_element() {
        let inner = json!({"height": 800000, "index": 0, "nodes": []});
        let outer = json!([inner.clone()]);
        assert_eq!(parse_tsc_proof(outer), Some(inner));
    }

    #[test]
    fn parse_tsc_proof_empty_array_returns_none() {
        let outer = json!([]);
        assert!(parse_tsc_proof(outer).is_none());
    }

    #[test]
    fn parse_tx_status_mined_with_confirmations() {
        let j = json!({"confirmations": 6, "blockheight": 800123, "blockhash": "abc"});
        let s = parse_tx_status("txid1", &j);
        assert_eq!(s.state, TxState::Mined);
        assert_eq!(s.block_height, Some(800123));
        assert_eq!(s.block_hash.as_deref(), Some("abc"));
    }

    #[test]
    fn parse_tx_status_mempool_when_zero_confirmations_and_no_blockheight() {
        let j = json!({"confirmations": 0});
        let s = parse_tx_status("txid1", &j);
        assert_eq!(s.state, TxState::InMempool);
        assert!(s.block_height.is_none());
    }

    #[test]
    fn parse_outspend_spent() {
        let j = json!({"spent": true, "txid": "ffff", "vin": 2});
        match parse_outspend(j) {
            OutspendStatus::Spent {
                spending_txid,
                spending_vin,
            } => {
                assert_eq!(spending_txid, "ffff");
                assert_eq!(spending_vin, Some(2));
            }
            _ => panic!("expected Spent"),
        }
    }

    #[test]
    fn parse_outspend_unspent() {
        let j = json!({"spent": false});
        assert!(matches!(parse_outspend(j), OutspendStatus::Unspent));
    }

    #[test]
    fn parse_utxos_wrapped_with_result() {
        // Satoshi's genesis address — pick something with a valid P2PKH form.
        let address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
        let j = json!({
            "result": [
                {"tx_hash": "aa", "tx_pos": 0, "value": 5000, "height": 800000},
                {"tx_hash": "bb", "tx_pos": 1, "value": 1000, "height": 0}
            ],
            "error": ""
        });
        let utxos = parse_woc_utxos(&j, address).expect("should parse");
        assert_eq!(utxos.len(), 2);
        assert_eq!(utxos[0].txid, "aa");
        assert!(utxos[0].confirmed);
        assert_eq!(utxos[1].txid, "bb");
        assert!(!utxos[1].confirmed, "height=0 should be unconfirmed");
    }

    #[test]
    fn parse_utxos_flat_array_legacy_format() {
        let address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
        let j = json!([{"tx_hash": "cc", "tx_pos": 0, "value": 9999, "height": 800001}]);
        let utxos = parse_woc_utxos(&j, address).expect("should parse");
        assert_eq!(utxos.len(), 1);
        assert_eq!(utxos[0].satoshis, 9999);
    }

    #[test]
    fn parse_utxos_returns_err_on_wrapped_error() {
        let address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
        let j = json!({"result": [], "error": "rate limited"});
        assert!(parse_woc_utxos(&j, address).is_err());
    }
}

/// The 80-byte header rebuilt from WhatsOnChain's block JSON (`/block/height/{h}` and
/// `/block/hash/{hash}` carry `version`, `previousblockhash`, `merkleroot`, `time`, `bits`,
/// `nonce`, `height`, `hash`), accepted only when it hashes to `hash`. `bits` arrives as a hex
/// string (as Bitcoin's RPC reports it) or a number; the genesis block has no
/// `previousblockhash`, which means zeros.
pub(crate) fn parse_block_json(v: &Value) -> Result<BlockHeader, String> {
    let str_field = |field: &str| -> Result<&str, String> {
        v.get(field)
            .and_then(|x| x.as_str())
            .ok_or_else(|| format!("missing '{}'", field))
    };
    let u32_field = |field: &str| -> Result<u32, String> {
        v.get(field)
            .and_then(|x| x.as_u64())
            .and_then(|x| u32::try_from(x).ok())
            .ok_or_else(|| format!("missing or out-of-range '{}'", field))
    };
    let bits = match v.get("bits") {
        Some(Value::String(s)) => u32::from_str_radix(s.trim(), 16)
            .map_err(|e| format!("bad hex in 'bits' {:?}: {}", s, e))?,
        _ => u32_field("bits")?,
    };
    let prev_hash = match v.get("previousblockhash") {
        None | Some(Value::Null) => "00".repeat(32),
        Some(_) => str_field("previousblockhash")?.to_string(),
    };
    let header = Header::from_fields(
        u32_field("version")?,
        &prev_hash,
        str_field("merkleroot")?,
        u32_field("time")?,
        bits,
        u32_field("nonce")?,
    )
    .map_err(|e| format!("block JSON: {}", e))?;
    verified_block_header(u32_field("height")?, header, str_field("hash")?)
}

#[cfg(test)]
mod block_json_tests {
    use super::*;
    use serde_json::json;

    /// WhatsOnChain's block JSON shape with the fields of the spv-testnet block at height 5 (the
    /// chaintracks tests use the same block).
    fn woc_block() -> Value {
        json!({
            "hash": "5ee1c44ffb4c19f3526a5c4fdd9049234123609d1252cb7cf7d5efac215eb6a0",
            "confirmations": 1000, "size": 200, "height": 5, "version": 536870912, "versionHex": "20000000",
            "merkleroot": "9d34d4193f6fa3a62e872d4832cd4b16a579541b05daeadee9cbd12a406f1146",
            "time": 1791033023, "mediantime": 1791032000, "nonce": 3, "bits": "207fffff", "difficulty": 4.6565e-10,
            "previousblockhash": "231deda25164376fdb28cc5490770f35f02ad27919e48903a74bc0ae88f00c0c",
            "nextblockhash": "00", "nTx": 1
        })
    }

    #[test]
    fn block_json_rebuilds_the_header_and_checks_its_hash() {
        let h = parse_block_json(&woc_block()).unwrap();
        assert_eq!(
            (h.height, h.block_hash.as_str(), h.header_hex.len()),
            (5, "5ee1c44ffb4c19f3526a5c4fdd9049234123609d1252cb7cf7d5efac215eb6a0", 160)
        );
        let mut numeric = woc_block();
        numeric["bits"] = json!(545259519u64);
        assert_eq!(parse_block_json(&numeric).unwrap().header_hex, h.header_hex, "bits as a number is accepted too");
    }

    #[test]
    fn block_json_with_a_tampered_or_missing_field_is_rejected() {
        let mut j = woc_block();
        j["nonce"] = json!(4);
        assert!(parse_block_json(&j).unwrap_err().contains("hash"));
        let mut j = woc_block();
        j.as_object_mut().unwrap().remove("merkleroot");
        assert!(parse_block_json(&j).unwrap_err().contains("merkleroot"));
        let mut j = woc_block();
        j["bits"] = json!("not hex");
        assert!(parse_block_json(&j).unwrap_err().contains("bits"));
    }

    #[test]
    fn the_genesis_block_has_no_previousblockhash() {
        // The genesis as regtest serves it (height 0): the field is absent and means zeros.
        let g = json!({
            "hash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206", "height": 0, "version": 1,
            "merkleroot": "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
            "time": 1296688602, "nonce": 2, "bits": "207fffff"
        });
        let h = parse_block_json(&g).unwrap();
        assert_eq!(h.height, 0);
        assert!(h.header_hex[8..72].chars().all(|c| c == '0'), "prev_hash is all zeros");
    }
}
