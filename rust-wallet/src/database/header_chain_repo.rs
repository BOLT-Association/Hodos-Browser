//! Persistence for the verified header chain (V26 `header_chain` table).
//!
//! Stores raw 80-byte headers only. Height, chainwork and the active chain are
//! recomputed — and every header re-validated — by `HeaderChain::from_stored` on
//! load, so a tampered row is dropped rather than trusted.

use crate::cache_errors::CacheResult;
use rusqlite::Connection;

pub struct HeaderChainRepository<'a> {
    conn: &'a Connection,
}

impl<'a> HeaderChainRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Insert a header. A duplicate (same network + hash) only refreshes the stored height: the
    /// header bytes are identified by their hash, so only the height can be wrong.
    pub fn insert(&self, network: &str, block_hash: &str, height: u32, header_hex: &str) -> CacheResult<()> {
        self.conn.execute(
            "INSERT INTO header_chain (network, block_hash, height, header_hex, created_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%s','now'))
             ON CONFLICT(network, block_hash) DO UPDATE SET height = excluded.height",
            rusqlite::params![network, block_hash, height, header_hex],
        )?;
        Ok(())
    }

    /// Insert a batch in ONE transaction. The sync used to insert header by header, each its own
    /// fsynced commit, which blocks every other DB user for the length of a long first sync.
    pub fn insert_many(&self, network: &str, rows: &[(String, u32, String)]) -> CacheResult<()> {
        if rows.is_empty() {
            return Ok(());
        }
        let tx = self.conn.unchecked_transaction()?;
        for (block_hash, height, header_hex) in rows {
            self.insert(network, block_hash, *height, header_hex)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// All stored `(height, header_hex)` for a network, lowest height first.
    pub fn load(&self, network: &str) -> CacheResult<Vec<(u32, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT height, header_hex FROM header_chain WHERE network = ?1 ORDER BY height ASC",
        )?;
        let rows = stmt
            .query_map([network], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        crate::database::migrations::migrate_v25_to_v26(&c).unwrap();
        c
    }

    #[test]
    fn insert_load_roundtrip_and_dedup_per_network() {
        let c = db();
        let r = HeaderChainRepository::new(&c);
        r.insert("regtest", "aa", 2, "02").unwrap();
        r.insert("regtest", "bb", 1, "01").unwrap();
        r.insert("regtest", "bb", 1, "01").unwrap();
        r.insert("other", "cc", 9, "09").unwrap();
        assert_eq!(r.load("regtest").unwrap(), vec![(1, "01".into()), (2, "02".into())]);
        assert_eq!(r.load("other").unwrap().len(), 1);
    }

    #[test]
    fn re_inserting_a_known_header_corrects_a_wrong_stored_height() {
        // The height only orders the rows on load; a bad one would make the header (and every
        // descendant) fail to link on every start unless a later sync can overwrite it.
        let c = db();
        let r = HeaderChainRepository::new(&c);
        r.insert("regtest", "aa", 0, "0a").unwrap();
        r.insert("regtest", "aa", 100, "0a").unwrap();
        assert_eq!(r.load("regtest").unwrap(), vec![(100, "0a".into())]);
    }

    #[test]
    fn insert_many_stores_the_batch_and_heals_heights_like_insert() {
        let c = db();
        let r = HeaderChainRepository::new(&c);
        r.insert("regtest", "aa", 0, "0a").unwrap();
        r.insert_many("regtest", &[("aa".into(), 1, "0a".into()), ("bb".into(), 2, "0b".into()), ("cc".into(), 3, "0c".into())]).unwrap();
        assert_eq!(r.load("regtest").unwrap(), vec![(1, "0a".into()), (2, "0b".into()), (3, "0c".into())]);
        r.insert_many("regtest", &[]).unwrap();
    }

    #[test]
    fn migration_is_idempotent() {
        let c = db();
        crate::database::migrations::migrate_v25_to_v26(&c).unwrap();
    }
}
