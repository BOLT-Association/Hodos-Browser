//! Opens (and therefore migrates) the database file named by `HODOS_MIGRATE_DB`.
//! Ignored by default: it exists to run the schema migrations against a COPY of a real
//! wallet database and check the result from outside, e.g.
//!   HODOS_MIGRATE_DB=/path/to/copy/wallet.db cargo test --test migrate_copy -- --ignored --nocapture
//! Never point it at a live database: it migrates in place.

use hodos_wallet::database::WalletDatabase;
use std::path::PathBuf;

#[test]
#[ignore]
fn migrate_the_database_named_by_env() {
    let path = PathBuf::from(std::env::var("HODOS_MIGRATE_DB").expect("HODOS_MIGRATE_DB"));
    assert!(path.exists(), "database file must already exist: {}", path.display());
    let db = WalletDatabase::new(path).expect("open + migrate");
    drop(db);
}
