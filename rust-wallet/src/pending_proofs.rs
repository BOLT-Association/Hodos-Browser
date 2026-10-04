//! Held (not yet verifiable) merkle proofs, spv mode.
//!
//! A proof Arcade hands us can arrive before the wallet's own header chain has the block it
//! points at (header sync trails Arcade's MINED event by seconds). Throwing it away and
//! re-fetching works, but loses the SSE payload and depends on Arcade serving it again. Instead
//! such a proof is **held** in the separate `pending_proofs` table (V27), never in `proven_txs`:
//! every reader of `proven_txs` treats a row as a verified proof, so an unverified row there
//! would look proven to BEEF building, the `completed` status and output promotion.
//!
//! `resolve_pending` is the pure core: given the held proofs and a header check, it says which
//! proofs verified (the caller stores them through the normal store-and-promote path and then
//! deletes the held row), which are wrong (root does not match the verified chain), which are
//! stale, and which must keep waiting for a header. A held proof is never used for anything
//! until it has been verified.

use rusqlite::Connection;
use serde_json::Value;

use crate::database::PendingProofRepository;

/// What to do with a proof once it has been checked against the header chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofDecision {
    /// Verified against the wallet's header chain: store and promote.
    StoreVerified,
    /// Its root contradicts the verified chain: reject, never store.
    RejectBad,
    /// Cannot be judged yet (no header at that height) and a header chain is configured: hold it.
    HoldPending,
    /// No header chain configured (public mode): the historical "store anyway" behaviour.
    StoreUnverifiedLegacy,
}

/// Map a header-chain check result to a decision. `Ok(true/false)` are verdicts; `Err` means the
/// check could not be made.
pub fn classify_proof(check: &Result<bool, String>, header_chain_configured: bool) -> ProofDecision {
    match check {
        Ok(true) => ProofDecision::StoreVerified,
        Ok(false) => ProofDecision::RejectBad,
        Err(_) if header_chain_configured => ProofDecision::HoldPending,
        Err(_) => ProofDecision::StoreUnverifiedLegacy,
    }
}

/// Outcome of re-checking one held proof.
#[derive(Debug, Clone, PartialEq)]
pub enum Resolution {
    /// Verified now. The caller stores it, then deletes the held row (`delete_held`).
    Verified { txid: String, height: u32, block_hash: String, tsc: Value },
    /// The tx already has a stored proof; the held row was dropped.
    AlreadyProven { txid: String },
    /// Corrupt, or its root does not match the verified chain; the held row was dropped.
    Bad { txid: String, reason: String },
    /// Older than `max_age_secs` and still unjudgeable; the held row was dropped.
    Expired { txid: String },
    /// No header at that height yet; kept.
    StillWaiting { txid: String },
}

/// Re-check every held proof. `check(height, merkle_root_display_hex)` is the header-chain check
/// (`Err` = no header yet). Deletes the rows it resolves as Bad / Expired / AlreadyProven; leaves
/// Verified and StillWaiting rows in place.
pub fn resolve_pending(
    conn: &Connection,
    check: &dyn Fn(u32, &str) -> Result<bool, String>,
    now: i64,
    max_age_secs: i64,
) -> Result<Vec<Resolution>, String> {
    let repo = PendingProofRepository::new(conn);
    let held = repo.list().map_err(|e| format!("list held proofs: {}", e))?;
    let mut out = Vec::with_capacity(held.len());

    for p in held {
        let txid = p.txid.clone();
        let drop_row = |why: &str| {
            let _ = repo.delete(&txid);
            log::debug!("   held proof for {} dropped: {}", &txid[..txid.len().min(16)], why);
        };

        // A stored proof already exists: nothing to verify, nothing to keep.
        let proven: i64 = conn
            .query_row("SELECT COUNT(*) FROM proven_txs WHERE txid = ?1", [&txid], |r| r.get(0))
            .map_err(|e| format!("proven_txs lookup: {}", e))?;
        if proven > 0 {
            drop_row("already proven");
            out.push(Resolution::AlreadyProven { txid });
            continue;
        }

        // Decode the BUMP and compute the merkle root it claims for this tx.
        let mut tsc = match crate::beef::parse_bump_hex_to_tsc(&p.bump_hex) {
            Ok(t) => t,
            Err(e) => {
                drop_row("unparseable BUMP");
                out.push(Resolution::Bad { txid, reason: format!("unparseable BUMP: {}", e) });
                continue;
            }
        };
        // The BUMP is authoritative for its own height; fall back to what we recorded.
        let height = tsc["height"].as_u64().filter(|h| *h > 0).map(|h| h as u32).unwrap_or(p.height);
        tsc["height"] = serde_json::json!(height);
        let index = tsc["index"].as_u64().unwrap_or(0);
        let nodes = tsc["nodes"].as_array().cloned().unwrap_or_default();
        let root = match crate::beef::compute_merkle_root_from_tsc(&txid, height, index, &nodes) {
            Ok(r) => r,
            Err(e) => {
                drop_row("merkle root not computable");
                out.push(Resolution::Bad { txid, reason: format!("merkle root not computable: {}", e) });
                continue;
            }
        };

        match check(height, &root) {
            Ok(true) => out.push(Resolution::Verified { txid, height, block_hash: p.block_hash, tsc }),
            Ok(false) => {
                drop_row("root contradicts the verified chain");
                out.push(Resolution::Bad { txid, reason: format!("root {} does not match the verified header at {}", root, height) });
            }
            Err(_) if now - p.received_at > max_age_secs => {
                drop_row("expired without a header");
                out.push(Resolution::Expired { txid });
            }
            Err(_) => out.push(Resolution::StillWaiting { txid }),
        }
    }
    Ok(out)
}

/// Hold the proofs carried by push events (`source = "push"`). Only a MINED frame that has both a
/// merkle path and a block height, for a tx this wallet knows about, is held; everything else is
/// ignored. Returns how many were held.
pub fn hold_events(conn: &Connection, events: &[crate::arcade_push::StatusEvent], now: i64) -> usize {
    let repo = PendingProofRepository::new(conn);
    let mut held = 0;
    for e in events {
        if e.tx_status != "MINED" {
            continue;
        }
        let (Some(bump), Some(height)) = (e.merkle_path.as_deref(), e.block_height) else {
            continue;
        };
        // Only txs this wallet knows about (the token scopes events to us, but never trust that
        // alone to put rows in the database).
        let known: i64 = conn
            .query_row("SELECT COUNT(*) FROM transactions WHERE txid = ?1", [&e.txid], |r| r.get(0))
            .unwrap_or(0);
        if known == 0 {
            continue;
        }
        let height = u32::try_from(height).unwrap_or(0);
        if height == 0 {
            continue;
        }
        if repo
            .upsert(&e.txid, height, bump, e.block_hash.as_deref().unwrap_or(""), "push", now)
            .is_ok()
        {
            held += 1;
        }
    }
    held
}

/// Drop a held row once its proof has been stored (or is no longer wanted).
pub fn delete_held(conn: &Connection, txid: &str) -> Result<(), String> {
    PendingProofRepository::new(conn)
        .delete(txid)
        .map(|_| ())
        .map_err(|e| format!("delete held proof: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::cell::RefCell;

    const MAX_AGE: i64 = 6 * 3600;

    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        crate::database::migrations::create_schema_v1(&c).unwrap();
        crate::database::migrations::migrate_v26_to_v27(&c).unwrap();
        c.execute_batch("PRAGMA foreign_keys = OFF").unwrap();
        c
    }

    fn dsha(b: &[u8]) -> Vec<u8> {
        Sha256::digest(Sha256::digest(b)).to_vec()
    }

    /// A tx, and a BUMP (hex) for it in a two-tx block at `height`.
    fn tx_and_bump(height: u8) -> (String, String) {
        let txid_natural = dsha(&[height, 1, 2, 3]);
        let sibling = vec![0x77u8; 32];
        let mut bump = vec![height, 1, 2]; // height (varint), tree height 1, two nodes at level 0
        bump.extend([0u8, 0x02]);
        bump.extend(&txid_natural);
        bump.extend([1u8, 0x00]);
        bump.extend(&sibling);
        let txid_display: String = txid_natural.iter().rev().map(|b| format!("{:02x}", b)).collect();
        (txid_display, hex::encode(bump))
    }

    fn hold(c: &Connection, txid: &str, height: u32, bump: &str, at: i64) {
        PendingProofRepository::new(c).upsert(txid, height, bump, "blockhash", "poll", at).unwrap();
    }

    fn held(c: &Connection, txid: &str) -> bool {
        PendingProofRepository::new(c).get(txid).unwrap().is_some()
    }

    // ---- classify_proof ----------------------------------------------------------------

    #[test]
    fn classify_maps_every_combination() {
        assert_eq!(classify_proof(&Ok(true), true), ProofDecision::StoreVerified);
        assert_eq!(classify_proof(&Ok(true), false), ProofDecision::StoreVerified);
        assert_eq!(classify_proof(&Ok(false), true), ProofDecision::RejectBad);
        assert_eq!(classify_proof(&Ok(false), false), ProofDecision::RejectBad);
        assert_eq!(classify_proof(&Err("no header".into()), true), ProofDecision::HoldPending);
        assert_eq!(
            classify_proof(&Err("api down".into()), false),
            ProofDecision::StoreUnverifiedLegacy,
            "public mode keeps its existing behaviour"
        );
    }

    // ---- resolve_pending: the three cases -----------------------------------------------

    #[test]
    fn a_proof_held_before_its_header_verifies_later_and_is_handed_back_to_be_stored() {
        let c = db();
        let (txid, bump) = tx_and_bump(120);
        hold(&c, &txid, 120, &bump, 1_000);

        // No header yet: it waits and the row stays.
        let r = resolve_pending(&c, &|_, _| Err("no header at height 120".into()), 1_010, MAX_AGE).unwrap();
        assert_eq!(r, vec![Resolution::StillWaiting { txid: txid.clone() }]);
        assert!(held(&c, &txid));

        // The header arrives and matches: Verified, with the proof ready to store.
        let seen = RefCell::new(Vec::new());
        let r = resolve_pending(&c, &|h, root| { seen.borrow_mut().push((h, root.to_string())); Ok(true) }, 1_020, MAX_AGE).unwrap();
        match &r[..] {
            [Resolution::Verified { txid: t, height, block_hash, tsc }] => {
                assert_eq!((t.as_str(), *height, block_hash.as_str()), (txid.as_str(), 120, "blockhash"));
                assert_eq!(tsc["height"].as_u64(), Some(120));
                assert_eq!(tsc["index"].as_u64(), Some(0));
            }
            other => panic!("expected Verified, got {:?}", other),
        }
        let seen = seen.borrow();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].0, 120, "checked at the proof's own height");
        assert_eq!(seen[0].1.len(), 64, "with the merkle root computed from the BUMP");
        assert!(held(&c, &txid), "the row stays until the caller has stored the proof");

        delete_held(&c, &txid).unwrap();
        assert!(!held(&c, &txid));
    }

    #[test]
    fn a_held_proof_whose_root_contradicts_the_verified_chain_is_dropped_as_bad() {
        let c = db();
        let (txid, bump) = tx_and_bump(121);
        hold(&c, &txid, 121, &bump, 1_000);
        let r = resolve_pending(&c, &|_, _| Ok(false), 1_005, MAX_AGE).unwrap();
        assert!(matches!(&r[..], [Resolution::Bad { txid: t, .. }] if t == &txid), "{:?}", r);
        assert!(!held(&c, &txid), "a wrong proof is never kept");
    }

    #[test]
    fn a_held_proof_that_never_gets_a_header_expires() {
        let c = db();
        let (txid, bump) = tx_and_bump(122);
        hold(&c, &txid, 122, &bump, 1_000);
        let still = |now| resolve_pending(&c, &|_, _| Err("no header".into()), now, MAX_AGE).unwrap();
        assert_eq!(still(1_000 + MAX_AGE), vec![Resolution::StillWaiting { txid: txid.clone() }], "exactly at the limit: still waiting");
        assert_eq!(still(1_000 + MAX_AGE + 1), vec![Resolution::Expired { txid: txid.clone() }]);
        assert!(!held(&c, &txid), "expired rows are removed");
    }

    // ---- hold_events: what a push event may leave behind ----------------------------------

    fn known_tx(c: &Connection, txid: &str) {
        c.execute(
            "INSERT INTO transactions (txid, reference_number, status, is_outgoing, satoshis, created_at, updated_at)
             VALUES (?1, ?2, 'unproven', 0, 1, 0, 0)",
            rusqlite::params![txid, format!("ref-{}", txid)],
        )
        .unwrap();
    }

    fn mined(txid: &str, height: Option<u64>, bump: Option<&str>) -> crate::arcade_push::StatusEvent {
        crate::arcade_push::StatusEvent {
            txid: txid.into(),
            tx_status: "MINED".into(),
            block_height: height,
            block_hash: Some("hash-from-event".into()),
            merkle_path: bump.map(|s| s.to_string()),
        }
    }

    #[test]
    fn a_mined_event_for_a_known_tx_is_held_with_its_payload() {
        let c = db();
        let (txid, bump) = tx_and_bump(140);
        known_tx(&c, &txid);
        assert_eq!(hold_events(&c, &[mined(&txid, Some(140), Some(&bump))], 5_000), 1);
        let p = PendingProofRepository::new(&c).get(&txid).unwrap().expect("held");
        assert_eq!((p.height, p.bump_hex.as_str(), p.block_hash.as_str(), p.source.as_str(), p.received_at), (140, bump.as_str(), "hash-from-event", "push", 5_000));
    }

    #[test]
    fn events_that_cannot_carry_a_proof_or_are_not_ours_are_ignored() {
        let c = db();
        let (txid, bump) = tx_and_bump(141);
        known_tx(&c, &txid);
        let mut seen = mined(&txid, None, None);
        seen.tx_status = "SEEN_ON_NETWORK".into();
        let events = vec![
            seen,                                              // not MINED
            mined(&txid, Some(141), None),                     // MINED but no merkle path
            mined(&txid, None, Some(&bump)),                   // MINED but no height
            mined(&"ee".repeat(32), Some(141), Some(&bump)),   // not a tx this wallet knows
        ];
        assert_eq!(hold_events(&c, &events, 5_000), 0);
        assert!(PendingProofRepository::new(&c).list().unwrap().is_empty());
    }

    #[test]
    fn a_later_event_for_the_same_tx_replaces_the_held_proof() {
        let c = db();
        let (txid, bump) = tx_and_bump(142);
        known_tx(&c, &txid);
        hold_events(&c, &[mined(&txid, Some(142), Some(&bump))], 100);
        hold_events(&c, &[mined(&txid, Some(143), Some(&bump))], 200);
        let all = PendingProofRepository::new(&c).list().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!((all[0].height, all[0].received_at), (143, 200));
    }

    // ---- resolve_pending: edge cases -----------------------------------------------------

    #[test]
    fn a_tx_that_already_has_a_stored_proof_drops_its_held_row() {
        let c = db();
        let (txid, bump) = tx_and_bump(123);
        hold(&c, &txid, 123, &bump, 1_000);
        c.execute(
            "INSERT INTO proven_txs (txid, height, tx_index, merkle_path, raw_tx, block_hash, merkle_root, created_at, updated_at)
             VALUES (?1, 123, 0, x'00', x'00', '', '', 0, 0)",
            [&txid],
        )
        .unwrap();
        let r = resolve_pending(&c, &|_, _| panic!("must not even check it"), 1_001, MAX_AGE).unwrap();
        assert_eq!(r, vec![Resolution::AlreadyProven { txid: txid.clone() }]);
        assert!(!held(&c, &txid));
    }

    #[test]
    fn a_corrupt_held_proof_is_dropped_as_bad() {
        let c = db();
        hold(&c, &"ab".repeat(32), 124, "zz-not-hex", 1_000);
        let r = resolve_pending(&c, &|_, _| Ok(true), 1_001, MAX_AGE).unwrap();
        assert!(matches!(&r[..], [Resolution::Bad { .. }]), "{:?}", r);
        assert!(!held(&c, &"ab".repeat(32)));
    }

    #[test]
    fn each_held_proof_is_judged_on_its_own() {
        let c = db();
        let (t1, b1) = tx_and_bump(130);
        let (t2, b2) = tx_and_bump(131);
        hold(&c, &t1, 130, &b1, 1_000);
        hold(&c, &t2, 131, &b2, 1_000);
        // Only height 130 has a header and it matches; 131 has none yet.
        let r = resolve_pending(&c, &|h, _| if h == 130 { Ok(true) } else { Err("no header".into()) }, 1_002, MAX_AGE).unwrap();
        assert_eq!(r.len(), 2);
        assert!(r.iter().any(|x| matches!(x, Resolution::Verified { txid, .. } if txid == &t1)));
        assert!(r.iter().any(|x| matches!(x, Resolution::StillWaiting { txid } if txid == &t2)));
    }
}
