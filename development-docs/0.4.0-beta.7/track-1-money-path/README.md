# B5-T1 — Money path

**Status:** 📌 PROPOSED at G2 (2026-09-25) · **Scope doc:** owed at G2 — goal, integration check, telescope (re-fetch current BRCs/SDKs first), kaleidoscope

**Goal.** No path — automatic, user-triggered or recovery — can spend an output the wallet has not positively classified as money, and the wallet's coin and transaction records stay true to the chain.

## Tickets

Tickets stay in `../tickets/` (planning note D11); this list mirrors the register's Track column.

- [`TICKET_reservation_ownership_converge_on_spent_by.md`](../tickets/TICKET_reservation_ownership_converge_on_spent_by.md) — UTXO reservations should be owned by a transaction row, not by a placeholder string
- [`TICKET_auto_unlock_accepts_another_wallets_mnemonic.md`](../tickets/TICKET_auto_unlock_accepts_another_wallets_mnemonic.md) — 🔑 Auto-unlock accepts any valid recovery phrase — it never checks that the phrase belongs to THIS wallet
- [`TICKET_confirmed_tx_never_rechecked_after_reorg.md`](../tickets/TICKET_confirmed_tx_never_rechecked_after_reorg.md) — ⛓️ A confirmed transaction is never re-checked, so a chain reorg leaves the wallet trusting a block that no longer exists
- [`TICKET_dead_cert_tx_builder_marks_coins_spent_on_404.md`](../tickets/TICKET_dead_cert_tx_builder_marks_coins_spent_on_404.md) — 🪦 A dead certificate transaction builder still carries the old fail-open 'treat 404 as spent' check
- [`TICKET_bulk_utxo_sync_truncates_at_20_per_address.md`](../tickets/TICKET_bulk_utxo_sync_truncates_at_20_per_address.md) — The wallet never sees more than 20 UTXOs per address
- [`TICKET_loopback_host_form_wallet_routing.md`](../tickets/TICKET_loopback_host_form_wallet_routing.md) — TICKET — wallet routing ignores the `127.0.0.1` host form on BRC-100 compatibility ports
- [`TICKET_signaction_response_not_brc100_shape.md`](../tickets/TICKET_signaction_response_not_brc100_shape.md) — TICKET — `/signAction` returns `rawTx` (hex) instead of BRC-100's `tx` (AtomicBEEF bytes)
- [`TICKET_synced_outputs_store_a_fabricated_locking_script.md`](../tickets/TICKET_synced_outputs_store_a_fabricated_locking_script.md) — 🚨 Every output found by address sync stores a **fabricated** locking script
- [`TICKET_transaction_row_can_sit_at_created_while_its_coin_is_on_chain.md`](../tickets/TICKET_transaction_row_can_sit_at_created_while_its_coin_is_on_chain.md) — A transaction row can sit at status `created` while its coin is on chain -- and nothing reconciles it
- [`TICKET_wallet_cannot_shed_large_parents.md`](../tickets/TICKET_wallet_cannot_shed_large_parents.md) — The wallet has no way to shed a large parent, so a coin can stay unsendable forever
- [`TICKET_rescan_cannot_find_payments_to_generated_addresses.md`](../tickets/TICKET_rescan_cannot_find_payments_to_generated_addresses.md) — 🔎 The wallet cannot find a late payment to an old generated address

## Existing material

- `utxo-safety-guard/` — the classification guard (was beta.4 Track 1)
- `reqwest-tls-bump/` — the TLS-validator library bump (was beta.4 Track 0; now a phase here)
- `../README.md` §"Research questions owed" — RQ-1 (what a classified output is stored as) and RQ-2 (restore behaviour) sit between this track and T2/T3
