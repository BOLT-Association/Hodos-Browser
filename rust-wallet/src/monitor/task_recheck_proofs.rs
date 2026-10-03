//! TaskRecheckProofs — re-evaluate stored merkle proofs after the verified header
//! chain reorganised (WS4), and once at startup after the first successful sync.
//!
//! A stored proof is *orphaned* when its merkle root no longer matches the active
//! chain's header at its height. For each one this task asks the tx-status chain for
//! a fresh proof; if the tx was re-mined and the new proof verifies against the
//! active chain, the `proven_txs` row is swapped for it.
//!
//! ⛔ It never touches `transactions.status`, outputs or balances. An orphaned proof
//! whose tx has not been re-mined yet is left in place and reported on every pass
//! until it resolves; marking things failed from here would be a verdict the header
//! chain alone cannot justify (the tx may well still be in the mempool).
//!
//! Headers we cannot yet judge (no header at that height) are skipped, not flagged.

use actix_web::web;
use log::{info, warn};
use rusqlite::Connection;

use crate::database::{ProvenTxReqRepository, ProvenTxRepository};
use crate::header_sync::{self, HeaderService};
use crate::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Orphaned {
    pub txid: String,
    pub height: u32,
    pub raw_tx: Vec<u8>,
}

/// Proven txs at or above `from_height` whose stored proof no longer matches the
/// active chain. Pure over the connection and the header service.
pub fn find_orphaned(conn: &Connection, svc: &HeaderService, from_height: u32) -> Result<Vec<Orphaned>, String> {
    let mut stmt = conn
        .prepare("SELECT txid, height, merkle_path, raw_tx FROM proven_txs WHERE height >= ?1 ORDER BY height")
        .map_err(|e| format!("prepare: {}", e))?;
    let rows = stmt
        .query_map([from_height as i64], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)? as u32,
                r.get::<_, Vec<u8>>(2)?,
                r.get::<_, Vec<u8>>(3)?,
            ))
        })
        .map_err(|e| format!("query: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("row: {}", e))?;

    let mut out = Vec::new();
    for (txid, height, blob, raw_tx) in rows {
        let Ok(mut tsc) = serde_json::from_slice::<serde_json::Value>(&blob) else {
            warn!("   ⚠️ proof for {} is not valid TSC JSON; skipping", txid);
            continue;
        };
        if tsc.is_array() {
            if let Some(first) = tsc.as_array().and_then(|a| a.first()).cloned() {
                tsc = first;
            }
        }
        let index = tsc["index"].as_u64().unwrap_or(0);
        let Some(nodes) = tsc["nodes"].as_array() else {
            continue;
        };
        let Ok(root) = crate::beef::compute_merkle_root_from_tsc(&txid, height, index, nodes) else {
            warn!("   ⚠️ cannot compute merkle root for stored proof {}; skipping", txid);
            continue;
        };
        match svc.check_merkle_root(height, &root) {
            Ok(true) => {}
            Ok(false) => out.push(Orphaned { txid, height, raw_tx }),
            Err(_) => {} // cannot judge yet
        }
    }
    Ok(out)
}

/// Swap an orphaned `proven_txs` row for a replacement proof (already verified by the
/// caller) in one SQLite transaction: delete + insert, relink the transaction and the
/// proof request, add a history note. Does not touch `transactions.status`, outputs or
/// balances. Returns the new proof's height.
pub fn apply_replacement(
    conn: &Connection,
    o: &Orphaned,
    tsc: &serde_json::Value,
    block_hash: &str,
) -> Result<u32, String> {
    let height = tsc["height"].as_u64().unwrap_or(0) as u32;
    let index = tsc["index"].as_u64().unwrap_or(0);
    let blob = serde_json::to_vec(tsc).map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| format!("begin: {}", e))?;
    let proven = ProvenTxRepository::new(&tx);
    let new_id = proven
        .replace_proof(&o.txid, height, index, &blob, &o.raw_tx, block_hash, "")
        .map_err(|e| format!("replace_proof: {}", e))?;
    proven
        .link_transaction(&o.txid, new_id)
        .map_err(|e| format!("link_transaction: {}", e))?;
    let reqs = ProvenTxReqRepository::new(&tx);
    if let Ok(Some(req)) = reqs.get_by_txid(&o.txid) {
        let _ = reqs.link_proven_tx(req.proven_tx_req_id, new_id);
        let _ = reqs.add_history_note(
            req.proven_tx_req_id,
            "reorg",
            &format!("proof at old height {} orphaned; replaced by verified proof at height {}", o.height, height),
        );
    }
    tx.commit().map_err(|e| format!("commit: {}", e))?;
    Ok(height)
}

pub async fn run(state: &web::Data<AppState>) -> Result<(), String> {
    let Some(from) = header_sync::pending_proof_recheck() else {
        return Ok(());
    };
    let Some(svc) = header_sync::global() else {
        return Ok(());
    };

    let orphaned = {
        let db = state.database.lock().map_err(|e| format!("DB lock: {}", e))?;
        find_orphaned(db.connection(), &svc, from)?
    };

    if orphaned.is_empty() {
        header_sync::clear_proof_recheck_if(from);
        return Ok(());
    }
    warn!("🔀 TaskRecheckProofs: {} stored proof(s) no longer match the active chain", orphaned.len());

    let mut unresolved = 0;
    for o in &orphaned {
        let short = &o.txid[..o.txid.len().min(16)];
        let status = match state.services.tx_status(&o.txid).await {
            Ok(s) => s,
            Err(e) => {
                warn!("   ⚠️ {} orphaned proof, status lookup failed: {}", short, e);
                unresolved += 1;
                continue;
            }
        };
        let (Some(bump_hex), true) = (
            status.merkle_path_bump.clone(),
            status.state == crate::services::TxState::Mined,
        ) else {
            warn!("   ⏳ {} proof orphaned at height {}; not re-mined yet ({:?})", short, o.height, status.state);
            unresolved += 1;
            continue;
        };

        let mut tsc = match crate::beef::parse_bump_hex_to_tsc(&bump_hex) {
            Ok(t) => t,
            Err(e) => {
                warn!("   ⚠️ {} new BUMP unparseable: {}", short, e);
                unresolved += 1;
                continue;
            }
        };
        if let Some(h) = status.block_height {
            tsc["height"] = serde_json::json!(h);
        }
        match crate::cache_helpers::verify_tsc_proof_against_header_chain(&o.txid, &tsc) {
            Ok(true) => {}
            other => {
                warn!("   ⏳ {} replacement proof not accepted yet ({:?})", short, other.map_err(|e| e.to_string()));
                unresolved += 1;
                continue;
            }
        }

        let db = state.database.lock().map_err(|e| format!("DB lock: {}", e))?;
        let height = apply_replacement(db.connection(), o, &tsc, status.block_hash.as_deref().unwrap_or(""))?;
        info!("   ✅ {} proof replaced: height {} -> {} (verified against active chain)", short, o.height, height);
    }

    if unresolved == 0 {
        header_sync::clear_proof_recheck_if(from);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header_chain::tests::{fixture, NOW};
    use crate::header_chain::{compact_to_target, Header, HeaderChain, U256};
    use std::sync::Mutex;

    /// Mine a header with a chosen merkle root on top of `prev`.
    fn mine_with_root(prev: [u8; 32], root_display_hex: &str, time_salt: u32) -> Header {
        let mut root: [u8; 32] = hex::decode(root_display_hex).unwrap().try_into().unwrap();
        root.reverse();
        let mut h = Header { version: 1, prev_hash: prev, merkle_root: root, time: 1_700_000_000 + time_salt, bits: 0x207f_ffff, nonce: 0 };
        let target = compact_to_target(h.bits).unwrap();
        loop {
            let mut x = h.hash();
            x.reverse();
            if U256::from_be_bytes(x) <= target {
                return h;
            }
            h.nonce += 1;
        }
    }

    fn db_with_proof(txid: &str, height: u32, nodes: &[&str]) -> Connection {
        let c = Connection::open_in_memory().unwrap();
        crate::database::migrations::create_schema_v1(&c).unwrap();
        let tsc = serde_json::json!({"height": height, "index": 0, "nodes": nodes, "target": ""});
        c.execute(
            "INSERT INTO proven_txs (txid, height, tx_index, merkle_path, raw_tx, block_hash, merkle_root, created_at, updated_at)
             VALUES (?1, ?2, 0, ?3, x'00', '', '', 0, 0)",
            rusqlite::params![txid, height, serde_json::to_vec(&tsc).unwrap()],
        )
        .unwrap();
        c
    }

    #[test]
    fn proof_is_flagged_only_after_the_chain_reorgs_away_from_it() {
        let txid = "11".repeat(32);
        let sib = "22".repeat(32);
        let root = crate::beef::compute_merkle_root_from_tsc(&txid, 1, 0, &[serde_json::json!(sib)]).unwrap();

        let (mut chain, g) = fixture();
        // Branch A: height 1 carries the proof's root.
        let a1 = mine_with_root(g.hash(), &root, 1);
        chain.add_header(a1.clone(), NOW).unwrap();
        let svc = HeaderService { chain: Mutex::new(chain) };
        let conn = db_with_proof(&txid, 1, &[&sib]);

        assert_eq!(find_orphaned(&conn, &svc, 0).unwrap(), vec![], "proof matches active chain");

        // Branch B overtakes: different roots at heights 1 and 2.
        {
            let mut c = svc.chain.lock().unwrap();
            let b1 = mine_with_root(g.hash(), &"aa".repeat(32), 11);
            let b2 = mine_with_root(b1.hash(), &"bb".repeat(32), 12);
            c.add_header(b1, NOW).unwrap();
            c.add_header(b2, NOW).unwrap();
            assert_eq!(c.tip_height(), Some(2));
        }
        let orphaned = find_orphaned(&conn, &svc, 0).unwrap();
        assert_eq!(orphaned.len(), 1);
        assert_eq!(orphaned[0].txid, txid);
        // A recheck that starts above the proof's height does not look at it.
        assert!(find_orphaned(&conn, &svc, 2).unwrap().is_empty());
    }

    #[test]
    fn proof_above_the_synced_tip_is_skipped_not_flagged() {
        let txid = "33".repeat(32);
        let (chain, _) = fixture();
        let svc = HeaderService { chain: Mutex::new(chain) };
        let conn = db_with_proof(&txid, 50, &[&"44".repeat(32)]);
        assert!(find_orphaned(&conn, &svc, 0).unwrap().is_empty());
    }

    #[test]
    fn replacement_swaps_the_proof_relinks_and_leaves_status_alone() {
        let txid = "77".repeat(32);
        let conn = db_with_proof(&txid, 5, &[&"88".repeat(32)]);
        conn.execute(
            "INSERT INTO transactions (txid, reference_number, status, is_outgoing, satoshis, created_at, updated_at, raw_tx)
             VALUES (?1, 'ref1', 'completed', 1, 1000, 0, 0, '00')",
            [&txid],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO proven_tx_reqs (txid, status, raw_tx, created_at, updated_at) VALUES (?1, 'completed', x'00', 0, 0)",
            [&txid],
        )
        .unwrap();
        let old_id: i64 = conn.query_row("SELECT provenTxId FROM proven_txs WHERE txid=?1", [&txid], |r| r.get(0)).unwrap();

        let o = Orphaned { txid: txid.clone(), height: 5, raw_tx: vec![0] };
        let tsc = serde_json::json!({"height": 9, "index": 3, "nodes": ["99".repeat(32)], "target": ""});
        assert_eq!(apply_replacement(&conn, &o, &tsc, "newhash").unwrap(), 9);

        let (h, idx, bh, new_id): (i64, i64, String, i64) = conn
            .query_row("SELECT height, tx_index, block_hash, provenTxId FROM proven_txs WHERE txid=?1", [&txid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap();
        assert_eq!((h, idx, bh.as_str()), (9, 3, "newhash"));
        assert_ne!(new_id, old_id);
        let (status, linked): (String, Option<i64>) = conn
            .query_row("SELECT status, proven_tx_id FROM transactions WHERE txid=?1", [&txid], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert_eq!(status, "completed", "tx status must not change");
        assert_eq!(linked, Some(new_id));
        let req_link: Option<i64> = conn.query_row("SELECT proven_tx_id FROM proven_tx_reqs WHERE txid=?1", [&txid], |r| r.get(0)).unwrap();
        assert_eq!(req_link, Some(new_id));
        let one: i64 = conn.query_row("SELECT COUNT(*) FROM proven_txs WHERE txid=?1", [&txid], |r| r.get(0)).unwrap();
        assert_eq!(one, 1);
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

    /// End to end against the real stack, through the real code paths:
    /// header sync -> proof verified by `verify_tsc_proof_against_block` (header-chain
    /// mode) -> real reorg -> orphan detection -> verified replacement.
    ///
    /// Needs: stack up with the background miner stopped (`docker stop cb-block-generator`)
    /// and `HODOS_LIVE_RAWTX_HEX` = a signed, unbroadcast spend of a mature coinbase.
    /// `cargo test --bin hodos-wallet live_proof_lifecycle -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn live_proof_lifecycle() {
        use crate::header_sync::{sync_once, ChaintracksSource, HeaderSource};
        use crate::services::providers::{ArcadeProvider, ChaintracksProvider};
        use crate::services::IndexerProvider;

        std::env::set_var("HODOS_CHAINTRACKS_URL", "http://localhost:8083/chaintracks/v2");
        let http = reqwest::Client::new();
        let now = || std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u32;
        let svc = crate::header_sync::init(HeaderChain::new(crate::header_chain::Params::regtest()));
        let src = ChaintracksSource(ChaintracksProvider::new(http.clone(), "http://localhost:8083/chaintracks/v2".into()));
        sync_once(&svc.chain, &src, now(), &mut |_| {}).await.expect("initial sync");

        // 1. Broadcast + mine through Arcade.
        let arcade = ArcadeProvider::new(http.clone(), "http://localhost:8080".into());
        let raw = hex::decode(std::env::var("HODOS_LIVE_RAWTX_HEX").expect("HODOS_LIVE_RAWTX_HEX")).unwrap();
        let sent = arcade.broadcast_beef(&raw).await.expect("broadcast");
        let txid = sent.txid.clone();
        tokio::time::sleep(std::time::Duration::from_secs(8)).await;

        async fn mined_proof(
            arcade: &ArcadeProvider,
            http: &reqwest::Client,
            txid: &str,
        ) -> (serde_json::Value, Option<String>) {
            for _ in 0..6 {
                rpc(http, "generate", serde_json::json!([1])).await;
                for _ in 0..15 {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    let s = arcade.tx_status(txid).await.expect("status");
                    if s.state == crate::services::TxState::Mined {
                        if let Some(b) = s.merkle_path_bump {
                            let mut tsc = crate::beef::parse_bump_hex_to_tsc(&b).unwrap();
                            tsc["height"] = serde_json::json!(s.block_height.unwrap());
                            return (tsc, s.block_hash);
                        }
                    }
                }
            }
            panic!("tx never mined");
        }
        let (tsc, block_hash) = mined_proof(&arcade, &http, &txid).await;
        let height = tsc["height"].as_u64().unwrap() as u32;
        println!("mined at {}", height);

        // 2. Verified by the real function in header-chain mode.
        sync_once(&svc.chain, &src, now(), &mut |_| {}).await.expect("sync after mining");
        assert_eq!(crate::verify_tsc_proof_against_block(&http, &txid, &tsc).await.unwrap(), true);
        let mut bad = tsc.clone();
        bad["nodes"][0] = serde_json::json!("ab".repeat(32));
        assert_eq!(crate::verify_tsc_proof_against_block(&http, &txid, &bad).await.unwrap(), false);
        let mut far = tsc.clone();
        far["height"] = serde_json::json!(height + 500);
        assert!(crate::verify_tsc_proof_against_block(&http, &txid, &far).await.is_err(), "unknown header must be an error");

        // 3. Store it, nothing orphaned yet.
        let conn = Connection::open_in_memory().unwrap();
        crate::database::migrations::create_schema_v1(&conn).unwrap();
        conn.execute(
            "INSERT INTO proven_txs (txid, height, tx_index, merkle_path, raw_tx, block_hash, merkle_root, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, '', 0, 0)",
            rusqlite::params![txid, height, tsc["index"].as_u64().unwrap() as i64, serde_json::to_vec(&tsc).unwrap(), raw, block_hash.clone().unwrap_or_default()],
        ).unwrap();
        assert!(find_orphaned(&conn, &svc, 0).unwrap().is_empty());

        // 4. Real reorg: drop the block holding the tx, mine a longer branch.
        let victim = rpc(&http, "getblockhash", serde_json::json!([height])).await;
        rpc(&http, "invalidateblock", serde_json::json!([victim])).await;
        rpc(&http, "generate", serde_json::json!([3])).await;
        let want = height + 2;
        for _ in 0..60 {
            if src.tip_height().await.unwrap() >= want { break; }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        let report = sync_once(&svc.chain, &src, now() + 10, &mut |_| {}).await.expect("sync after reorg");
        println!("reorg report: {:?}", report);
        assert_eq!(report.reorgs.len(), 1);

        // 5. The stored proof is now orphaned, and says so.
        let orphaned = find_orphaned(&conn, &svc, 0).unwrap();
        assert_eq!(orphaned.len(), 1, "stored proof should be orphaned");
        assert_eq!(orphaned[0].txid, txid);
        assert_eq!(find_orphaned(&conn, &svc, height + 1).unwrap().len(), 0);

        // 6. Re-mined proof is accepted only if it verifies, then swapped in. Right after the
        // reorg Arcade may still serve the OLD proof; the header chain must reject that, which
        // is exactly the "replacement proof not accepted yet" path in `run`.
        let mut accepted = None;
        let mut rejected_stale = 0;
        for attempt in 0..40 {
            if attempt > 0 && attempt % 6 == 0 {
                rpc(&http, "generate", serde_json::json!([1])).await;
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            sync_once(&svc.chain, &src, now() + 20, &mut |_| {}).await.expect("sync");
            let s = arcade.tx_status(&txid).await.unwrap();
            let (true, Some(b)) = (s.state == crate::services::TxState::Mined, s.merkle_path_bump.clone()) else { continue };
            let mut t = crate::beef::parse_bump_hex_to_tsc(&b).unwrap();
            t["height"] = serde_json::json!(s.block_height.unwrap());
            match crate::verify_tsc_proof_against_block(&http, &txid, &t).await {
                Ok(true) => { accepted = Some((t, s.block_hash)); break; }
                other => { rejected_stale += 1; println!("not accepted yet: {:?}", other.map_err(|e| e.to_string())); }
            }
        }
        let (tsc2, bh2) = accepted.expect("a replacement proof that verifies against the active chain");
        println!("stale/unverifiable proofs rejected before success: {}", rejected_stale);
        let o = &orphaned[0];
        let new_h = apply_replacement(&conn, o, &tsc2, bh2.as_deref().unwrap_or("")).unwrap();
        println!("replaced proof: {} -> {}", o.height, new_h);
        assert!(find_orphaned(&conn, &svc, 0).unwrap().is_empty(), "replacement must verify against the active chain");
    }

    #[test]
    fn find_orphaned_never_modifies_the_database() {
        let txid = "55".repeat(32);
        let (chain, _) = fixture();
        let svc = HeaderService { chain: Mutex::new(chain) };
        let conn = db_with_proof(&txid, 0, &[&"66".repeat(32)]);
        let before: i64 = conn.query_row("SELECT COUNT(*) FROM proven_txs", [], |r| r.get(0)).unwrap();
        let _ = HeaderChain::new(crate::header_chain::Params::regtest());
        let _ = find_orphaned(&conn, &svc, 0).unwrap();
        let after: i64 = conn.query_row("SELECT COUNT(*) FROM proven_txs", [], |r| r.get(0)).unwrap();
        assert_eq!(before, after);
    }
}
