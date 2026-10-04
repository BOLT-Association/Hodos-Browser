//! Held merkle proofs (V27 `pending_proofs`): proofs received before the wallet's own header
//! chain could verify them. Never read as a proof: only `pending_proofs::resolve_pending`
//! consumes this table, and a row moves to `proven_txs` only after it has verified.

use crate::cache_errors::CacheResult;
use rusqlite::Connection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingProof {
    pub txid: String,
    pub height: u32,
    pub bump_hex: String,
    pub block_hash: String,
    /// Where it came from: `poll` (tx status) or `push` (SSE event).
    pub source: String,
    pub received_at: i64,
}

pub struct PendingProofRepository<'a> {
    conn: &'a Connection,
}

impl<'a> PendingProofRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PendingProof> {
        Ok(PendingProof {
            txid: r.get(0)?,
            height: r.get::<_, i64>(1)? as u32,
            bump_hex: r.get(2)?,
            block_hash: r.get(3)?,
            source: r.get(4)?,
            received_at: r.get(5)?,
        })
    }

    /// Hold (or replace) the proof for `txid`. One row per tx: a newer proof for the same tx
    /// (e.g. after a reorg moved it) replaces the older one.
    pub fn upsert(&self, txid: &str, height: u32, bump_hex: &str, block_hash: &str, source: &str, now: i64) -> CacheResult<()> {
        self.conn.execute(
            // Same proof again (the poll re-holds an unjudgeable one every tick): keep its age so the
            // expiry can fire. A different proof (e.g. a reorg moved the tx) starts a new age.
            "INSERT INTO pending_proofs (txid, height, bump_hex, block_hash, source, received_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(txid) DO UPDATE SET
                 received_at = CASE WHEN height = excluded.height AND bump_hex = excluded.bump_hex
                                    THEN received_at ELSE excluded.received_at END,
                 height = excluded.height, bump_hex = excluded.bump_hex,
                 block_hash = excluded.block_hash, source = excluded.source",
            rusqlite::params![txid, height as i64, bump_hex, block_hash, source, now],
        )?;
        Ok(())
    }

    pub fn get(&self, txid: &str) -> CacheResult<Option<PendingProof>> {
        let mut stmt = self.conn.prepare(
            "SELECT txid, height, bump_hex, block_hash, source, received_at FROM pending_proofs WHERE txid = ?1",
        )?;
        match stmt.query_row([txid], Self::row) {
            Ok(p) => Ok(Some(p)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// All held proofs, oldest first.
    pub fn list(&self) -> CacheResult<Vec<PendingProof>> {
        let mut stmt = self.conn.prepare(
            "SELECT txid, height, bump_hex, block_hash, source, received_at FROM pending_proofs
             ORDER BY received_at ASC, txid ASC",
        )?;
        let rows = stmt.query_map([], Self::row)?.collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn delete(&self, txid: &str) -> CacheResult<usize> {
        Ok(self.conn.execute("DELETE FROM pending_proofs WHERE txid = ?1", [txid])?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        crate::database::migrations::migrate_v26_to_v27(&c).unwrap();
        c
    }

    #[test]
    fn migration_creates_the_table_and_is_idempotent() {
        let c = db();
        crate::database::migrations::migrate_v26_to_v27(&c).unwrap();
        let cols: Vec<String> = c
            .prepare("PRAGMA table_info(pending_proofs)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        for want in ["txid", "height", "bump_hex", "block_hash", "source", "received_at"] {
            assert!(cols.iter().any(|c| c == want), "missing column {}: {:?}", want, cols);
        }
    }

    #[test]
    fn upsert_get_list_delete_roundtrip() {
        let c = db();
        let r = PendingProofRepository::new(&c);
        r.upsert("bb", 2, "02", "h2", "push", 200).unwrap();
        r.upsert("aa", 1, "01", "h1", "poll", 100).unwrap();
        assert_eq!(
            r.get("aa").unwrap(),
            Some(PendingProof { txid: "aa".into(), height: 1, bump_hex: "01".into(), block_hash: "h1".into(), source: "poll".into(), received_at: 100 })
        );
        assert_eq!(r.get("zz").unwrap(), None);
        let all: Vec<String> = r.list().unwrap().into_iter().map(|p| p.txid).collect();
        assert_eq!(all, vec!["aa", "bb"], "oldest first");
        assert_eq!(r.delete("aa").unwrap(), 1);
        assert_eq!(r.delete("aa").unwrap(), 0);
        assert_eq!(r.get("aa").unwrap(), None);
    }

    #[test]
    fn a_newer_proof_for_the_same_tx_replaces_the_older_one() {
        let c = db();
        let r = PendingProofRepository::new(&c);
        r.upsert("aa", 10, "old", "h-old", "poll", 100).unwrap();
        r.upsert("aa", 12, "new", "h-new", "push", 300).unwrap();
        let all = r.list().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!((all[0].height, all[0].bump_hex.as_str(), all[0].source.as_str(), all[0].received_at), (12, "new", "push", 300));
    }
    #[test]
    fn re_holding_the_same_proof_keeps_its_age_so_it_can_expire() {
        // The poll re-holds a still-unjudgeable proof every tick; that must not make it young again.
        let c = db();
        let r = PendingProofRepository::new(&c);
        r.upsert("aa", 10, "same", "h", "poll", 100).unwrap();
        r.upsert("aa", 10, "same", "h", "poll", 160).unwrap();
        r.upsert("aa", 10, "same", "h", "push", 220).unwrap();
        assert_eq!(r.get("aa").unwrap().unwrap().received_at, 100);
    }
}
