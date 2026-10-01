# 🟡 The service-fee commission row is deleted whenever an action is signed, so almost no commission is recorded

**Found:** 2026-09-30, beta.6 P5 adversarial review (code reading), **confirmed by measurement** on the dev wallet DB
**Status:** ⬜ UNASSIGNED (beta.8 intake) · **Track:** unassigned · **Filed by:** Claude (Opus 5.5)

> ⚠️ **Method note.** The mechanism is code reading; the count is a read-only query of the dev wallet. Not checked: the owner's installed wallet, and whether any external report relies on this table.

---

## What happens

`create_action_internal` writes a `commissions` row for the 1,000-sat service fee against the **pre-signing** transaction row. Signing calls `database/transaction_repo.rs :: update_txid`, which deletes the old row and its children — including `DELETE FROM commissions WHERE transaction_id = ?1` — and re-inserts the transaction under the signed txid **without** the commission.

## Measured (dev wallet, 2026-09-30)

**3 commission rows across 636 outgoing transactions.** The only recent row with a commission is an action that was never signed (deferred, abandoned on purpose by the P5 probe). A real completed PeerPay (tx 991) has none.

## Why it matters

The fee itself is paid on chain (the output exists); the wallet's **record** of it is lost. Nothing in `rust-wallet/src` reads commission data back (only `create` and `delete` are called), so no wallet decision depends on it — this is accounting loss, not a rule-7 poisoning defect. It matters for any fee reporting, reconciliation, or the backup payload (`backup.rs` exports the table).

## Fix direction

Carry the commission across `update_txid` (re-point `transaction_id` to the new row instead of deleting), as it already does for outputs (`relinked`). Test: sign an action, assert one commission row on the signed transaction; RED = today's delete.
