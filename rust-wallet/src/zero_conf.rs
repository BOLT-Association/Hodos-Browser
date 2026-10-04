//! Zero-conf acceptance of received outputs (spv mode).
//!
//! A received output whose transaction is not mined yet is spendable once the network has
//! seen it, as BSV wallets conventionally allow, provided the SPV evidence is sound:
//!
//!   * the BEEF's ancestry is complete and every BUMP verified against the wallet's own header
//!     chain (both already enforced by `internalize_action` before we get here), and
//!   * every transaction in the BEEF that has no BUMP of its own (the subject, and any unproven
//!     ancestor) is reported by Arcade as `SEEN_ON_NETWORK` / `SEEN_ON_MULTIPLE_NODES` (or already
//!     mined), with no double-spend or rejection.
//!
//! "Spendable" means the output is linked to its incoming transaction row, the same thing that
//! makes the wallet's own unproven change spendable (see `OutputRepository::get_spendable_by_user`).
//! If Arcade later reports a double-spend for that transaction the output is unlinked again, and
//! a rejection fails the transaction, and `mark_failed` disables its outputs (they stay in the table, not selectable). Off-switch: `HODOS_ZERO_CONF=off`.

use std::future::Future;
use std::time::Duration;

use crate::beef::Beef;
use crate::services::{IndexerError, TxState, TxStatus, WalletServices};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// On the network (or already mined), no conflict reported.
    Seen,
    /// Arcade has not got it that far yet (or does not know it, or could not be asked).
    NotYet,
    /// Rejected, conflicting or orphaned: never acceptable.
    Bad,
}

/// Classify one Arcade answer.
pub fn verdict_from_status(r: &Result<TxStatus, IndexerError>) -> Verdict {
    match r {
        Ok(s) => match s.raw_provider_status.as_deref() {
            Some("SEEN_ON_NETWORK") | Some("SEEN_ON_MULTIPLE_NODES") | Some("MINED") | Some("IMMUTABLE") => Verdict::Seen,
            Some("DOUBLE_SPEND_ATTEMPTED")
            | Some("REJECTED")
            | Some("SEEN_IN_ORPHAN_MEMPOOL")
            | Some("MINED_IN_STALE_BLOCK") => Verdict::Bad,
            _ => match s.state {
                TxState::Rejected | TxState::DoubleSpendAttempted => Verdict::Bad,
                TxState::Mined => Verdict::Seen,
                _ => Verdict::NotYet, // RECEIVED, ACCEPTED_BY_NETWORK, SENT_TO_NETWORK, ...
            },
        },
        Err(_) => Verdict::NotYet,
    }
}

/// Ask `fetch` until the tx is `Seen` or `Bad`, or `timeout` passes (then `NotYet`). Arcade usually
/// goes RECEIVED → SEEN_ON_NETWORK within a fraction of a second.
pub async fn wait_until_seen<F, Fut>(mut fetch: F, timeout: Duration, interval: Duration) -> Verdict
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<TxStatus, IndexerError>>,
{
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        match verdict_from_status(&fetch().await) {
            Verdict::NotYet => {}
            v => return v,
        }
        if tokio::time::Instant::now() + interval > deadline {
            return Verdict::NotYet;
        }
        tokio::time::sleep(interval).await;
    }
}

/// Is every transaction in this BEEF that carries no BUMP seen on the network? (BUMPs themselves
/// were verified by the caller.) `Ok(false)` means "not eligible", never an error to surface.
pub async fn beef_zero_conf_ok(services: &WalletServices, beef: &Beef, timeout: Duration) -> bool {
    let mut checked_any = false;
    for (txid, _) in beef.txids_and_hex() {
        if beef.tx_has_proof(&txid) {
            continue;
        }
        checked_any = true;
        let v = wait_until_seen(|| services.tx_status(&txid), timeout, Duration::from_millis(400)).await;
        if v != Verdict::Seen {
            log::info!("   ⏳ zero-conf: {} is {:?} — output stays unconfirmed", &txid[..txid.len().min(16)], v);
            return false;
        }
    }
    checked_any
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn st(raw: Option<&str>, state: TxState) -> Result<TxStatus, IndexerError> {
        Ok(TxStatus {
            txid: "ab".into(),
            state,
            block_height: None,
            block_hash: None,
            merkle_path_bump: None,
            raw_provider_status: raw.map(|s| s.to_string()),
        })
    }

    #[test]
    fn only_seen_or_mined_is_acceptable() {
        for s in ["SEEN_ON_NETWORK", "SEEN_ON_MULTIPLE_NODES", "MINED", "IMMUTABLE"] {
            assert_eq!(verdict_from_status(&st(Some(s), TxState::InMempool)), Verdict::Seen, "{}", s);
        }
    }

    #[test]
    fn early_statuses_are_not_yet_never_acceptable() {
        for s in ["RECEIVED", "STORED", "QUEUED", "SENT_TO_NETWORK", "ACCEPTED_BY_NETWORK", "ANNOUNCED_TO_NETWORK", "REQUESTED_BY_NETWORK"] {
            assert_eq!(verdict_from_status(&st(Some(s), TxState::InMempool)), Verdict::NotYet, "{}", s);
        }
        assert_eq!(verdict_from_status(&Err(IndexerError::NotFound)), Verdict::NotYet);
        assert_eq!(verdict_from_status(&Err(IndexerError::Transport("down".into()))), Verdict::NotYet);
    }

    #[test]
    fn conflicts_and_rejections_are_bad() {
        for s in ["DOUBLE_SPEND_ATTEMPTED", "REJECTED", "SEEN_IN_ORPHAN_MEMPOOL", "MINED_IN_STALE_BLOCK"] {
            assert_eq!(verdict_from_status(&st(Some(s), TxState::Rejected)), Verdict::Bad, "{}", s);
        }
        assert_eq!(verdict_from_status(&st(None, TxState::DoubleSpendAttempted)), Verdict::Bad);
    }

    #[tokio::test]
    async fn waits_through_early_statuses_until_seen() {
        let calls = Arc::new(AtomicUsize::new(0));
        let c = calls.clone();
        let v = wait_until_seen(
            move || {
                let n = c.fetch_add(1, Ordering::SeqCst);
                async move {
                    match n {
                        0 => st(Some("RECEIVED"), TxState::InMempool),
                        1 => Err(IndexerError::NotFound),
                        _ => st(Some("SEEN_ON_NETWORK"), TxState::InMempool),
                    }
                }
            },
            Duration::from_secs(2),
            Duration::from_millis(10),
        )
        .await;
        assert_eq!(v, Verdict::Seen);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn gives_up_as_not_yet_and_stops_early_on_bad() {
        let v = wait_until_seen(|| async { st(Some("RECEIVED"), TxState::InMempool) }, Duration::from_millis(60), Duration::from_millis(20)).await;
        assert_eq!(v, Verdict::NotYet, "timeout is NotYet, not a pass");

        let calls = Arc::new(AtomicUsize::new(0));
        let c = calls.clone();
        let v = wait_until_seen(
            move || {
                c.fetch_add(1, Ordering::SeqCst);
                async { st(Some("DOUBLE_SPEND_ATTEMPTED"), TxState::DoubleSpendAttempted) }
            },
            Duration::from_secs(5),
            Duration::from_millis(10),
        )
        .await;
        assert_eq!(v, Verdict::Bad);
        assert_eq!(calls.load(Ordering::SeqCst), 1, "a conflict is final; no point waiting");
    }
}
