//! BOLT tokens held by this wallet (V28 `bolt_tokens`).
//!
//! The token logic (b017) runs in the page's `window.BOLT` handler, not here: this table is where
//! that handler keeps what the wallet holds, so tokens belong to the wallet and not to one site's
//! storage. A row is what the handler reports; the wallet cannot check it against the BEEF.
//!
//! Because the writer is page code, the table is append-and-retire, never destroy:
//! - a row's token data (`type`, `issuer`, `owner_pkh`, `amount`, `attributes`, `beef`) is written
//!   once and never overwritten, so a later `put` for the same outpoint cannot replace a BEEF;
//! - a spent token is marked `spent`, not deleted, and never returns to `held`.

use crate::cache_errors::CacheResult;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// One row, in the column names the page-side store uses (`packages/bolt/src/store.js`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoltTokenRow {
    pub outpoint: String,
    #[serde(rename = "type")]
    pub token_type: String,
    pub issuer: String,
    #[serde(default)]
    pub owner_pkh: Option<String>,
    #[serde(default = "held")]
    pub status: String,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default = "empty_object")]
    pub attributes: String,
    pub beef: String,
    #[serde(default)]
    pub anchor_txid: Option<String>,
    #[serde(default)]
    pub anchor_kind: Option<String>,
    #[serde(default)]
    pub anchor_network: Option<String>,
    #[serde(default)]
    pub anchor_proven: i64,
    #[serde(default)]
    pub anchor_height: Option<i64>,
    #[serde(default)]
    pub anchor_merkle_root: Option<String>,
    #[serde(default)]
    pub provenance: Option<String>,
    /// Set by the wallet; whatever the page sends is ignored.
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

fn held() -> String {
    "held".to_string()
}

fn empty_object() -> String {
    "{}".to_string()
}

const COLUMNS: &str = "outpoint, type, issuer, owner_pkh, status, amount, attributes, beef, anchor_txid, \
     anchor_kind, anchor_network, anchor_proven, anchor_height, anchor_merkle_root, provenance, created_at, updated_at";

pub struct BoltTokenRepository<'a> {
    conn: &'a Connection,
}

impl<'a> BoltTokenRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<BoltTokenRow> {
        Ok(BoltTokenRow {
            outpoint: r.get(0)?,
            token_type: r.get(1)?,
            issuer: r.get(2)?,
            owner_pkh: r.get(3)?,
            status: r.get(4)?,
            amount: r.get(5)?,
            attributes: r.get(6)?,
            beef: r.get(7)?,
            anchor_txid: r.get(8)?,
            anchor_kind: r.get(9)?,
            anchor_network: r.get(10)?,
            anchor_proven: r.get(11)?,
            anchor_height: r.get(12)?,
            anchor_merkle_root: r.get(13)?,
            provenance: r.get(14)?,
            created_at: r.get(15)?,
            updated_at: r.get(16)?,
        })
    }

    /// Keep a token as `held`. A new outpoint is inserted whole. For an outpoint already stored only
    /// what is known about its anchor is refreshed: the token data and its status stay as first
    /// written (see the module note).
    pub fn put(&self, t: &BoltTokenRow, now: i64) -> CacheResult<()> {
        self.conn.execute(
            &format!(
                "INSERT INTO bolt_tokens ({COLUMNS})
                 VALUES (?1, ?2, ?3, ?4, 'held', ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)
                 ON CONFLICT(outpoint) DO UPDATE SET
                     anchor_network = excluded.anchor_network,
                     anchor_proven = excluded.anchor_proven,
                     anchor_height = excluded.anchor_height,
                     anchor_merkle_root = excluded.anchor_merkle_root,
                     provenance = COALESCE(excluded.provenance, provenance),
                     updated_at = excluded.updated_at"
            ),
            rusqlite::params![
                t.outpoint,
                t.token_type,
                t.issuer,
                t.owner_pkh,
                t.amount,
                t.attributes,
                t.beef,
                t.anchor_txid,
                t.anchor_kind,
                t.anchor_network,
                if t.anchor_proven != 0 { 1 } else { 0 },
                t.anchor_height,
                t.anchor_merkle_root,
                t.provenance,
                now,
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, outpoint: &str) -> CacheResult<Option<BoltTokenRow>> {
        let mut stmt = self.conn.prepare(&format!("SELECT {COLUMNS} FROM bolt_tokens WHERE outpoint = ?1"))?;
        match stmt.query_row([outpoint], Self::row) {
            Ok(t) => Ok(Some(t)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Tokens with `status`, oldest first, optionally of one issuer and/or type.
    pub fn list(&self, status: &str, issuer: Option<&str>, token_type: Option<&str>) -> CacheResult<Vec<BoltTokenRow>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM bolt_tokens
             WHERE status = ?1 AND (?2 IS NULL OR issuer = ?2) AND (?3 IS NULL OR type = ?3)
             ORDER BY created_at ASC, outpoint ASC"
        ))?;
        let rows = stmt
            .query_map(rusqlite::params![status, issuer, token_type], Self::row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Retire a held token (it was transferred, split, merged or melted). The row and its BEEF are
    /// kept. Returns whether a held row was retired.
    pub fn mark_spent(&self, outpoint: &str, now: i64) -> CacheResult<bool> {
        let n = self.conn.execute(
            "UPDATE bolt_tokens SET status = 'spent', updated_at = ?2 WHERE outpoint = ?1 AND status = 'held'",
            rusqlite::params![outpoint, now],
        )?;
        Ok(n > 0)
    }

    /// Every row, held and spent, for a backup.
    pub fn all(&self) -> rusqlite::Result<Vec<BoltTokenRow>> {
        let mut stmt = self.conn.prepare(&format!("SELECT {COLUMNS} FROM bolt_tokens ORDER BY created_at ASC, outpoint ASC"))?;
        let rows = stmt.query_map([], Self::row)?.collect();
        rows
    }

    /// Put back a row from a backup exactly as it was (status and timestamps included); a row the
    /// wallet already has is left alone.
    pub fn restore(&self, t: &BoltTokenRow) -> rusqlite::Result<()> {
        self.conn.execute(
            &format!(
                "INSERT OR IGNORE INTO bolt_tokens ({COLUMNS})
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)"
            ),
            rusqlite::params![
                t.outpoint, t.token_type, t.issuer, t.owner_pkh, t.status, t.amount, t.attributes, t.beef,
                t.anchor_txid, t.anchor_kind, t.anchor_network, t.anchor_proven, t.anchor_height,
                t.anchor_merkle_root, t.provenance, t.created_at, t.updated_at,
            ],
        )?;
        Ok(())
    }

    /// The wallet's own notes on a token (`attributes.wallet`: an AuthBOLT identity's keys and the
    /// apps it is linked to). The only change to `attributes` after a row is first written, and
    /// only the `wallet` key: the token data stays as stored. Hodos takes it from its own UI only
    /// (`bolt::bolt_tokens`). False when the outpoint is unknown.
    pub fn annotate(&self, outpoint: &str, wallet: &serde_json::Value, now: i64) -> CacheResult<bool> {
        let Some(row) = self.get(outpoint)? else { return Ok(false) };
        let mut attrs: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&row.attributes).unwrap_or_default();
        attrs.insert("wallet".to_string(), wallet.clone());
        let n = self.conn.execute(
            "UPDATE bolt_tokens SET attributes = ?2, updated_at = ?3 WHERE outpoint = ?1",
            rusqlite::params![outpoint, serde_json::Value::Object(attrs).to_string(), now],
        )?;
        Ok(n > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::database::migrations::migrate_v27_to_v28(&conn).unwrap();
        conn
    }

    fn token(outpoint: &str, amount: Option<&str>) -> BoltTokenRow {
        BoltTokenRow {
            outpoint: outpoint.to_string(),
            token_type: "SimpleMultiBOLT".to_string(),
            issuer: format!("02{}", "ab".repeat(32)),
            owner_pkh: Some("cd".repeat(20)),
            status: "held".to_string(),
            amount: amount.map(str::to_string),
            attributes: "{}".to_string(),
            beef: "0101010100beef".to_string(),
            anchor_txid: Some("11".repeat(32)),
            anchor_kind: Some("settle".to_string()),
            anchor_network: Some("accepted".to_string()),
            anchor_proven: 0,
            anchor_height: None,
            anchor_merkle_root: None,
            provenance: Some("{\"txid\":\"aa\",\"kind\":\"mint\"}".to_string()),
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn annotate_sets_the_wallets_notes_and_nothing_else() {
        let conn = db();
        let repo = BoltTokenRepository::new(&conn);
        let a = format!("{}.0", "11".repeat(32));
        let mut t = token(&a, None);
        t.token_type = "AuthBOLT".to_string();
        t.attributes = "{\"kind\":\"x\"}".to_string();
        repo.put(&t, 10).unwrap();
        let notes = serde_json::json!({ "issuerKeyId": "authbolt-1", "apps": [{ "domain": "peerloop.example" }] });
        assert!(repo.annotate(&a, &notes, 20).unwrap());
        let got = repo.get(&a).unwrap().unwrap();
        let attrs: serde_json::Value = serde_json::from_str(&got.attributes).unwrap();
        assert_eq!(attrs["wallet"], notes);
        assert_eq!(attrs["kind"], "x", "the token's own attributes stay");
        assert_eq!((got.beef.as_str(), got.issuer.as_str(), got.updated_at), (t.beef.as_str(), t.issuer.as_str(), 20));
        assert!(!repo.annotate("nope", &notes, 30).unwrap(), "an unknown outpoint changes nothing");
    }

    #[test]
    fn all_and_restore_carry_every_row_exactly_through_a_backup() {
        let from = db();
        let repo = BoltTokenRepository::new(&from);
        let a = format!("{}.0", "11".repeat(32));
        let b = format!("{}.1", "22".repeat(32));
        let mut identity = token(&a, None);
        identity.token_type = "AuthBOLT".to_string();
        identity.attributes = r#"{"wallet":{"issuerKeyId":"authbolt-1","apps":[]}}"#.to_string();
        repo.put(&identity, 10).unwrap();
        repo.put(&token(&b, Some("5")), 20).unwrap();
        repo.mark_spent(&b, 30).unwrap();
        let saved = repo.all().unwrap();
        assert_eq!(saved.len(), 2, "held and spent rows are both kept");

        let to = db();
        let back = BoltTokenRepository::new(&to);
        for t in &saved {
            back.restore(t).unwrap();
            back.restore(t).unwrap(); // twice: a row already there is left alone
        }
        assert_eq!(back.all().unwrap(), saved, "every column, status and timestamp as it was");
    }

    #[test]
    fn put_get_list_round_trip() {
        let conn = db();
        let repo = BoltTokenRepository::new(&conn);
        let a = format!("{}.0", "11".repeat(32));
        let b = format!("{}.1", "22".repeat(32));
        repo.put(&token(&a, Some("1000")), 10).unwrap();
        repo.put(&token(&b, None), 20).unwrap();

        let got = repo.get(&a).unwrap().unwrap();
        assert_eq!(got.amount.as_deref(), Some("1000"));
        assert_eq!(got.status, "held");
        assert_eq!((got.created_at, got.updated_at), (10, 10)); // the wallet's clock, not the page's
        assert_eq!(repo.get("nope").unwrap(), None);

        let listed = repo.list("held", None, None).unwrap();
        assert_eq!(listed.iter().map(|t| t.outpoint.as_str()).collect::<Vec<_>>(), vec![a.as_str(), b.as_str()]);
        assert_eq!(repo.list("held", Some("03nope"), None).unwrap().len(), 0);
        assert_eq!(repo.list("held", None, Some("SimpleMultiBOLT")).unwrap().len(), 2);
    }

    #[test]
    fn a_second_put_refreshes_the_anchor_but_never_the_token_data() {
        let conn = db();
        let repo = BoltTokenRepository::new(&conn);
        let op = format!("{}.0", "11".repeat(32));
        repo.put(&token(&op, Some("1000")), 10).unwrap();

        let mut again = token(&op, Some("999999"));
        again.beef = "deadbeef".to_string();
        again.issuer = format!("03{}", "ff".repeat(32));
        again.anchor_proven = 1;
        again.anchor_height = Some(1234);
        again.provenance = None;
        repo.put(&again, 50).unwrap();

        let got = repo.get(&op).unwrap().unwrap();
        assert_eq!(got.beef, "0101010100beef", "the BEEF is written once");
        assert_eq!(got.amount.as_deref(), Some("1000"));
        assert!(got.issuer.starts_with("02"));
        assert_eq!((got.anchor_proven, got.anchor_height), (1, Some(1234)), "the anchor state is refreshed");
        assert!(got.provenance.is_some(), "a missing provenance does not erase the stored one");
        assert_eq!((got.created_at, got.updated_at), (10, 50));
    }

    #[test]
    fn a_spent_token_is_kept_and_never_returns_to_held() {
        let conn = db();
        let repo = BoltTokenRepository::new(&conn);
        let op = format!("{}.0", "11".repeat(32));
        repo.put(&token(&op, Some("1000")), 10).unwrap();

        assert!(repo.mark_spent(&op, 20).unwrap());
        assert!(!repo.mark_spent(&op, 30).unwrap(), "already spent");
        assert!(!repo.mark_spent("nope", 30).unwrap());

        let got = repo.get(&op).unwrap().unwrap();
        assert_eq!(got.status, "spent");
        assert_eq!(got.beef, "0101010100beef", "the row and its BEEF survive");
        assert_eq!(repo.list("held", None, None).unwrap().len(), 0);
        assert_eq!(repo.list("spent", None, None).unwrap().len(), 1);

        repo.put(&token(&op, Some("1000")), 40).unwrap(); // a page putting it again
        assert_eq!(repo.get(&op).unwrap().unwrap().status, "spent");
    }
}
