# B5-T3a-P2.2 — Making a backup can never double-spend, loop, or quietly skip a change · PHASE CONTRACT

**Track:** B5-T3a Backup you can trust (single device) · **Tickets:** closes `../research/ONCHAIN_BACKUP_REVIEW.md` **BS-C1, BS-M1, BS-L1, BS-L4, BS-SYNC-1, BS-SYNC-2, BS-SYNC-4**, plan D13's baseline bullet (R4-3), and the **baseline-read defect found 2026-09-28** (§0) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 agent for T3a (resumed) · **Platforms:** both (Rust write path + monitor; one wallet-panel status line) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** 2a (the backup's funding selection reads T1-P3's money index and reserves through its `reserved_by` — no parallel reservation), 3, 4(a) (never re-broadcast the whole backup *needlessly* — here: an error must never read as "no baseline"), 7 (nothing timer-restores coins), `SCOPE.md` §0, §4 P2 list, §6 (service fee row).

> Part of **T3a-P2** (P2.1 carry · **P2.2 write path** · P2.3 restore) — split justified in P2.1's header.

---

## 0. The finding this phase fixes, verified 2026-09-28 (working rule 7)

**Claim (from the T3b agent, code reading):** an error reading the backup baseline reads as "no baseline", so a full backup is built and broadcast.

**Verdict: CONFIRMED in code; not observed firing.**

- `rust-wallet/src/database/settings_repo.rs :: SettingsRepository::get_backup_hash` — on any `rusqlite` error it logs `⚠️ get_backup_hash error (returning None)` and returns **`Ok(None)`**.
- `rust-wallet/src/handlers.rs :: do_onchain_backup` Step 2 — `settings_repo.get_backup_hash().unwrap_or(None)`, then `if stored_hash == Some(new_hash) { skip }` — `None` never equals, so the cycle **builds, funds and broadcasts** a full backup.
- Same shape, same function: `get_last_backup_at` returns **`Ok(0)`** on any error (⇒ `ref_ts = now`, so the time-based strips move and the hash differs anyway); after broadcast, `let _ = settings_repo.set_backup_hash(…)` and `let _ = …set_last_backup_at(…)` **discard** their results, and `set_last_backup_at` itself discards its `execute` result — a failed write means the next cycle has no baseline and rebroadcasts. `wallet_recover_onchain` Step 6b discards the same write.
- **Cheap ground-truth check (rule 7):** grep of the retained wallet logs — production `%APPDATA%/HodosBrowser/logs/wallet_r*.log` (11 files, 2026-09-05 → 09-28, warn level) and dev (11 files, 2026-09-19 → 09-28) — for `get_backup_hash error`: **0 hits** in both. The production logger runs at `warn`, so the line would have been kept.
- **Class:** trip-wire 2 *shape* (a verdict where an error is owed). **Not poisoning:** no durable wrong state is written — the cost is one needless full backup (fee + a new tip) per cycle while the error persists, and it repeats every cycle (BS-M1 has no backoff), so a persistent settings-read error would drain fees at the 3-hour cadence. ⇒ **a live latent defect, fixed here (row `P2.2-A3`); no rule-7 stop.** T3b-P6-A2 keeps "force `get_backup_hash` to `Err`" as its RED — after this phase that RED is observed by **reverting this fix**, not on then-current code.

---

## 1. Goal

The periodic on-chain backup never spends a coin a payment is spending, never broadcasts because it could not read its own bookkeeping, never records a baseline that hides a change, and — when it keeps failing — stops, says so in the wallet, and costs nothing until it can succeed.

## 2. Done means

- [ ] **BS-C1:** funding selection and reservation in `do_onchain_backup` run under `AppState.utxo_selection_lock` in **one** lock scope, through T1-P3's reservation (`money_utxos.reserved_by`) — never a parallel `pending-backup-*` scheme once T1-P3 exists; the reservation result is **checked** (today `let _ = output_repo.mark_multiple_spent(…)`), and a failed reservation aborts before signing
- [ ] **Baseline reads and writes are three-valued:** present / truly absent (no `settings` row, or `backup_hash IS NULL`) / **error** — an error aborts the cycle as `Failed` with a visible reason and **broadcasts nothing**; `set_backup_hash` / `set_last_backup_at` failures are errors, not `let _`
- [ ] **D13 pinned baseline, without a loop:** the stored baseline describes exactly what was broadcast **plus the backup's own known effects** (its spent inputs, its new change, marker and token rows) — a user mutation that lands during the multi-second broadcast window is **not** absorbed into the baseline (today Step 13 re-collects at `now` after broadcast), and an idle wallet's next cycle still **skips** (no self-triggered loop)
- [ ] **BS-M1 backoff:** consecutive failures grow the retry interval (cap and curve recorded in the contract at kickoff); after N failures the wallet panel shows a quiet persistent status *"Backup is failing — last success <date>"*; the failure log is durable across restarts through the existing `monitor_events` table (`Monitor::log_event` already writes `TaskBackup:error`) — **no new table**; a success resets it
- [ ] **Hard pre-broadcast size cap:** an over-cap payload is a typed `Failed(PayloadOverCap{bytes, cap})` before selection, never a warning (today `compress_payload` only warns above `200_000`). The cap value is set by the owner from P0-E1/E2 (§12 Q2)
- [ ] **BS-SYNC-1:** `wallet_sync` with `full_sync` never ingests an output at the backup address (index `-3`) or any other reserved negative index — the exclusion is structural (the address list is filtered at its source), not a row-level check after insert
- [ ] **BRC-177 alignment — no timer restores a coin:** the `task_check_for_proofs.rs :: mark_failed` path reached from *"oracle quorum inconclusive — forcing failure"* stops restoring inputs (it marks the transaction *abandoned-unbroadcast*-pending and holds the inputs); inputs are restored only on **evidence** (a node/ARC rejection, or chain absence past the BRC-177 deadline with the anchor reclaimed). ⚠️ Ownership is §12 Q1 — this path is shared with T1/T4
- [ ] **No service fee on backups** (SCOPE §6): the backup builder is not routed through `create_action_internal`; stated and tested
- [ ] **BS-L1** sub-dust change burned to fee is recorded (an audit/`monitor_events` line with the amount), **BS-L4** a significant-send trigger with a cold price cache still schedules the backup, **BS-SYNC-2** the three stale "reconciles → external-spend" doc claims corrected, **BS-SYNC-4** the dead `reconciled_count` removed from the response or made real

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-NOSPEND` / `R-DUST` | no incidental spend of a 1-sat or held output | Backup funding switches to T1-P3's index reader; a selector that bypasses the index would fund a backup from a held token |
| Working rule 7, trip-wires 1 + 2 | no impossible rows; no verdict where an error is owed | This phase **removes** three trip-wire-2 shapes; the lock change must not create a trip-wire-1 row (reserved coin with no owner) on abort |
| `R-PEERPAY-DELIVERY` half 1 | backup funding is smallest-sufficient (beta.3 P10d-A3) | The selection moves inside a lock and onto the index; the existing test `a3_backup_funding_is_smallest_sufficient` must stay green |
| Service fee (root `CLAUDE.md`) | backups carry none | A refactor "reusing" `create_action_internal` would add 1,000 sats to every backup |
| Gold pill | untouched | Backups are not dApp payments; no IPC change |
| Invariant 2 | no schema change | None: backoff state reuses `monitor_events`; reservations reuse T1-P3's table |

## 4. Evidence table

⛔ Money rows: RED designed by a second agent (`../../../RELEASE_CYCLE.md` §4.2).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P2.2-A1` ⭐ *BS-C1 — no shared input* | 500 randomized interleavings on the mock of `do_onchain_backup` against `create_action_internal` (and `task_consolidate_dust`): no outpoint appears as an input in two signed transactions; each run names the outpoints both paths considered | **Switches:** (1) revert the lock scope so Step 6 selection (DB lock only) and Step 7 `mark_multiple_spent` stay outside `utxo_selection_lock`, and insert a `cfg(test)` yield point between them ⇒ the interleavings must produce a shared input on the mock ARC ⇒ red · (2) ⭐ vacuity: seed the fixture with ONE spendable coin large enough for either path but not both ⇒ without the lock the mock ARC must receive two transactions spending it; WITH the lock the loser must return *Insufficient funds* / `Deferred` — a run where the second path silently builds nothing (0 overlap, 1 tx) never exercised the race · (3) hold the mock ARC's response to `create_action_internal` 2 s so the backup's Step 6 runs inside the window · (4) layer check: one interleaving where the DB reservation is deliberately wrong (reservation written, no tx built) ⇒ the ARC-side assertion still passes while a DB-side one would not — proving which layer the row reads · **Red for the right reason:** the shared `txid:vout` printed with both spending txids; `task_consolidate_dust` interleavings listed separately · **Residue:** none (mock ARC, scratch DB) — designed by controls-C (Fable), 2026-09-28 | The mock ARC's received transactions, input by input (txid:vout), not the DB's reservation columns | T1 | ⬜ |
| `P2.2-A2` *reservation failure aborts* | A reservation write that fails (injected `rusqlite` error) ⇒ the cycle returns `Failed`, nothing is signed or broadcast, and no coin is left reserved | **Switches:** (1) inject a REAL `rusqlite` error into `mark_multiple_spent` (rename `outputs.spending_description` in the scratch DB just before Step 7) and revert the check to today's `let _ = output_repo.mark_multiple_spent(…)` ⇒ signing and broadcast proceed on unreserved coins ⇒ mock broadcast log non-empty ⇒ red · (2) partial failure: fault the write on the SECOND of two outpoints ⇒ the first must be released on abort — one coin left at `pending-backup-*` with no live cycle is a trip-wire-1 row · (3) crash between reserve and abort (`cfg(test)` abort point) ⇒ `task_sweep_reservations` releases it only after 15 min AND an on-chain unspent observation — record the coin as unavailable for that window, not *no coin left reserved* · **Red for the right reason:** outcome variant `Failed(…)` naming the reservation, empty mock log, and the coins' `spendable` / `spending_description` read AFTER the abort returns · **Residue:** case (3) leaves a `pending-backup-*` reservation in the scratch DB until swept — declared — designed by controls-C (Fable), 2026-09-28 | Mock broadcast log (empty) + the coins' reservation state after the abort | T1 | ⬜ |
| `P2.2-A3` ⭐ *baseline read error ≠ no baseline* (§0) | With `get_backup_hash` (and separately `get_last_backup_at`) forced to a DB error: the cycle ends `Failed("baseline unreadable: …")`, **zero** broadcasts, and the error is visible in the status line; with the row truly absent (fresh wallet) the first backup still broadcasts | **Switches:** (1) fault `settings_repo.rs :: get_backup_hash` with a REAL error (rename `settings.backup_hash` in the scratch DB, or `DROP TABLE settings`) and revert to today's `Ok(None)` + Step 2 `.unwrap_or(None)` ⇒ the mock records ONE broadcast ⇒ red; same for `get_last_backup_at` (`Ok(0)` ⇒ `ref_ts = now`) · (2) two-sided: make the fix over-strict (`backup_hash IS NULL` treated as error) ⇒ a fresh wallet NEVER broadcasts its first backup ⇒ the fresh-wallet half goes red · (3) ⭐ `task_backup.rs :: run` classifies the HTTP body by `err.contains("skipped")` — a `Failed("baseline unreadable: … skipped …")` reason is read as `Skipped` and CLEARS `backup_check_needed`; assert the variant AND that the flag is still set after the failed tick · (4) keep the fault for 3 ticks ⇒ broadcast count stays 0 (not 0-then-1 once a retry elapses) · **Red for the right reason:** mock broadcast count, the `BackupOutcome` variant, the flag — never the warn line (today's code prints it too) · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | The mock chain's broadcast log per case, plus the returned `BackupOutcome` variant — never the log line | T1 | ⬜ |
| `P2.2-A4` *baseline write error is an error* | `set_backup_hash` forced to fail after a successful broadcast ⇒ the outcome records the broadcast txid **and** a baseline-write failure; the next cycle does not rebroadcast an identical payload (it recovers the baseline from the recorded broadcast, or stays `Failed` visibly) | **Switches:** (1) fault `set_backup_hash` after a successful mock broadcast (`PRAGMA query_only=1` on the scratch connection from a `cfg(test)` hook after Step 11) and revert to today's `let _ = settings_repo.set_backup_hash(…)` ⇒ the outcome reports a plain `Broadcast(txid)` and the next tick rebroadcasts (mock count 2) ⇒ red · (2) fault `set_last_backup_at` ALONE ⇒ today `ref_ts` falls back to `now`, the time-based strips move, the hash differs ⇒ an idle wallet rebroadcasts — must be seen red, then green · (3) recovery half: with the write failed, over 3 further ticks the mock count is exactly 1 — the cycle either recovers the baseline from the recorded txid or stays `Failed` visibly · **Red for the right reason:** the txid is in both the outcome and the mock; the write failure is in the outcome; `settings.backup_hash` read back equals the pre-broadcast value · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | Broadcast log across two cycles; the second cycle's outcome | T1 | ⬜ |
| `P2.2-A5` ⭐ *pinned baseline, two-sided* | (i) A label written during the broadcast window (mock holds the broadcast response for 2 s) appears in the **next** cycle's payload; (ii) an idle wallet's next cycle after a successful backup **skips** | **Switches:** (i) today's Step 13 (re-collect at `now` after broadcast) IS the red for (i): the label written during the 2 s hold is absorbed into the baseline and never appears in the next payload — run it first, record the red · (ii) pin the baseline to the PRE-broadcast payload with no own-effects ⇒ the next idle cycle's hash differs (own change output, `spent_by`, new `proven_tx_req`) ⇒ broadcast ⇒ red · (3) ⭐ vacuity of (ii): *0 broadcasts over 3 idle cycles* also holds when `task_backup.rs :: run` returns `Deferred("Insufficient funds")` (balance < `MIN_BACKUP_BALANCE_SATS`) or `Deferred("DB busy")` — assert each idle tick's outcome is `Skipped` and the fixture holds ≥ 3,000 sats after the first backup · (4) write the label after the broadcast returns but before Step 12 finishes ⇒ still (i): it must be in the next payload · **Red for the right reason:** (i) the decrypted next payload's `tx_labels`; (ii) the mock count and the `Skipped` variant per tick · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | (i) the decrypted next payload contains the label; (ii) the broadcast count over 3 idle cycles = 0. Each half is the other's control: re-collecting after broadcast passes (ii) and fails (i); pinning to the pre-broadcast payload alone passes (i) and fails (ii) | T1 | ⬜ |
| `P2.2-A6` *backoff converges and is visible* | 10 consecutive forced failures: retry gaps grow per the recorded curve up to its cap; after N the wallet panel status line appears with the last-success date; a restart keeps the counter (read from `monitor_events`); one success clears both | Disable the backoff (constant interval) ⇒ the gap assertion goes red. Second: hide the status line ⇒ the panel assertion (rendered text) goes red | Monitor tick timestamps from the mock clock; the rendered panel text after a hard reload (Vite HMR trap — `reference_vite_hmr_fakes_negative_controls`) | T1 + T3 | ⬜ |
| `P2.2-A7` *size cap fails closed* | A fixture payload one byte over the cap ⇒ `Failed(PayloadOverCap)` before any selection; one byte under ⇒ broadcasts | Raise the cap to `u64::MAX` in the test build ⇒ the over-cap case broadcasts and the assertion goes red | Mock broadcast log + outcome variant; the cap value printed with the result | T1 | ⬜ |
| `P2.2-A8` *BS-SYNC-1* | `POST /wallet/sync?full=true` on a wallet whose backup marker is unspent on the mock ⇒ **no** `outputs` row at the backup address; balance unchanged | **Switches:** (1) revert the source filter (today's unfiltered `address_repo.rs :: get_all_by_wallet`) on a fixture whose marker outpoint is NOT yet in `outputs` (delete the `1-wallet-backup/marker` row; marker unspent on the mock) ⇒ `POST /wallet/sync?full=true` inserts a `basket_id NULL, spendable=1` row at the backup address and the balance rises by 546 ⇒ red — with the marker row already present `INSERT OR IGNORE` hides the defect, so that fixture is vacuous · (2) structural teeth: add an address at index `-4` to the fixture ⇒ must also be excluded — a literal `= -3` filter ⇒ red · (3) reconcile half: with the backup rows present and the mock's address query returning only the marker, full sync must NOT mark any backup row `external-spend` · **Red for the right reason:** the `outputs` row joined by `txid:vout` to the marker; `calculate_balance` before/after · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | `outputs` rows joined to the marker's outpoint (by txid:vout) after sync | T1 | ⬜ |
| `P2.2-A9` *no timer restores a coin* | A `nosend` transaction past `NOSEND_TIMEOUT_SECS` with the oracle **inconclusive** ⇒ inputs stay reserved and the tx is marked pending-abandon; with an explicit rejection ⇒ inputs restored | **Switches:** (1) revert the branch to today's `OracleVerdict::Inconclusive ⇒ mark_failed` (`task_check_for_proofs.rs :: run`, timeout arm) with the three oracle URLs pointed at a mock serving an inconclusive mix ⇒ `restore_spent_by_txid` runs, inputs read `spendable=1` ⇒ red · (2) ⭐ two-sided: make the fix hold inputs on `AllNotFound` / explicit rejection too ⇒ a genuinely dead tx keeps its coins locked past the BRC-177 deadline ⇒ red · (3) both restore paths: `mark_failed` falls back to `restore_by_spending_description(txid)` — one fixture whose inputs are reserved by `spending_description` only (no `spent_by` FK) must stay held on that path too · (4) use a BRC-121 `nosend` row (T4-P1's shape) as one fixture, since the branch is shared · **Red for the right reason:** the input outpoints' `spendable` / `spent_by` / `spending_description` after `run` returns, per served verdict, with the mock oracle's per-call log · **Residue:** the scratch DB keeps one pending-abandon tx with held inputs — declared — designed by controls-C (Fable), 2026-09-28 | The input outpoints' state after `task_check_for_proofs::run` per oracle verdict (mocked `oracle_quorum_check`) | T1 | ⬜ — ⚠️ §12 Q1 |
| `P2.2-A10` *no service fee* | Every backup transaction on the mock has outputs exactly `[PushDrop, marker, change?]` — no output to `HODOS_FEE_ADDRESS`, no `commissions` row | **Switches:** (1) route the backup builder through `create_action_internal` in a test build (the pre-mortem's *reuse*) ⇒ a 1,000-sat output to `HODOS_FEE_ADDRESS` and a `commissions` row appear ⇒ red · (2) ⭐ vacuity: *no commissions row* holds on an empty table — seed one unrelated commission first and assert the count unchanged; assert the decoded output list has exactly 2 or 3 entries (a 4th of any kind is red) · (3) `task_consolidate_dust` runs in the same harness and DOES pay the fee — assert the fee output is found on ITS transaction in the same run, proving the decoder can see one · **Red for the right reason:** the backup tx's decoded output scripts on the mock, the `HODOS_FEE_ADDRESS` P2PKH matched byte-for-byte · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | The signed transaction's outputs, decoded | T1 | ⬜ |
| `P2.2-A11` *BS-L1 / BS-L4* | A backup whose change ≤ 546 sats writes one accounting line with the burned amount; a significant send with both price-cache reads `None` still sets `backup_check_needed` | **Switches:** BS-L1 — (1) ⭐ pin the fixture so `total_in − 1546 − fee` lands in (0, 546] (today's `if change_amount > 546` drops it silently); revert the accounting write ⇒ no `monitor_events` row ⇒ red — a fixture whose change exceeds 546 produces no line either way and is vacuous · (2) change exactly 0 ⇒ no line · (3) the recorded amount must equal the decoded tx's (inputs − outputs − fee at the rate used), not the estimate · BS-L4 — (4) fresh scratch DB (no `bsv_price_cache` row, so `get_stale()` is None), upstream price mocked down, `request_backup_check_if_significant` reverted to today's early return ⇒ `backup_check_needed` stays `None` after a ≥ $3 send ⇒ red; a dev DB with a persisted price never enters the branch — assert both reads returned None in the run · **Red for the right reason:** the `monitor_events` row's amount; `AppState.backup_check_needed` read directly · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | The `monitor_events` row with the amount; `AppState.backup_check_needed` after the send | T1 | ⬜ |
| `P2.2-A12` *BS-SYNC-2 / 4 docs + counter* | No `CLAUDE.md` under `rust-wallet/` claims full sync reconciles to `external-spend`; the sync response has no always-zero field | A grep test for the stale claim, run first on today's tree ⇒ red (the register said three files; on 2026-09-28 `external-spend` appears in `rust-wallet/CLAUDE.md` and `rust-wallet/src/monitor/CLAUDE.md` — which sentences are stale is decided at kickoff, and the grep targets those sentences) — then green. ⚠️ If this becomes a preflight gate, that is its own commit (working rule 6) | The grep's hit list; the sync response JSON keys | T0 | ⬜ |

**Two-sided rows:** A3 (an error never broadcasts) paired with its own fresh-wallet half (a true absence still does); A5 (i)/(ii) as stated.

### 4a. Independent control notes (2026-09-28)

*controls-C (Fable), second agent per `../../../RELEASE_CYCLE.md` §4.2. Findings on GREEN/SUBJECT cells I did not rewrite. One line each: row — problem — suggested fix.*

- P2.2-A3 / A6 — `task_backup.rs :: run` classifies the self-call's body by `err.contains("skipped")`. Any `Failed` reason containing that substring is read as `Skipped`, which clears `backup_check_needed` and advances `last_backup`. Make the classification structural (a `kind` field in the JSON) — its own item, or every new failure reason is one word away from a silent skip.
- P2.2-A5(ii) — *0 broadcasts over 3 idle cycles* is also produced by `Deferred` (insufficient funds, DB busy); assert `Skipped` per tick.
- P2.2-A8 — with the marker row already in `outputs`, `INSERT OR IGNORE` masks BS-SYNC-1 completely; the fixture must lack the row (a restored wallet where `reconcile_backup_tx` failed is the natural shape).
- P2.2-A9 — `oracle_quorum_check` is a private fn with literal oracle URLs; the row's *mocked* oracle needs a base-URL seam (the same class as P1-A7's `WalletServices` routing) or the test rewrites the function it is testing.
- P2.2-A12 — `rust-wallet/CLAUDE.md` §UTXO Sync still says the sync endpoint *reconciles stale outputs … as `external-spend`*; that is one of the sentences the grep should target.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/handlers.rs :: do_onchain_backup` Step 2 (hash compare), Step 6 (selection — DB lock dropped before selecting), Step 7 (`let _ = output_repo.mark_multiple_spent(&utxos_to_reserve, &placeholder_txid)`), Step 12–13 (post-broadcast `post_backup_hash` re-collected at `now`; `let _ = set_backup_hash`, `let _ = set_last_backup_at`) | ✅ | `utxo_selection_lock` is taken only in Step 1.5's c5b sweep and the failure reconcile — confirmed |
| `rust-wallet/src/database/settings_repo.rs :: get_backup_hash`, `set_backup_hash`, `get_last_backup_at`, `set_last_backup_at` | ✅ | `get_backup_hash` Err ⇒ `Ok(None)`; `get_last_backup_at` Err ⇒ `Ok(0)`; `set_last_backup_at` discards its `execute` result and returns `Ok(())` |
| `rust-wallet/src/monitor/mod.rs` TaskBackup block, `monitor/task_backup.rs :: run`, `BackupOutcome::is_current` | ✅ | `Failed` keeps the event flag and retries on the next tick — no backoff (BS-M1) |
| `rust-wallet/src/backup.rs :: compress_payload` | ✅ | Warns above `200_000` bytes; no cap |
| `rust-wallet/src/handlers.rs :: wallet_sync` (`full_sync` ⇒ `address_repo.get_all_by_wallet`) ; `database/address_repo.rs :: get_all_by_wallet` | ✅ | No index filter ⇒ index `-3` included (BS-SYNC-1 live) |
| `rust-wallet/src/monitor/task_check_for_proofs.rs :: run` (timeout branch), `mark_failed` | ✅ | Inconclusive oracle ⇒ `mark_failed` ⇒ `restore_spent_by_txid` / `restore_by_spending_description`. Logs: 0 `oracle quorum inconclusive` lines in retained prod/dev logs (latent) |
| `rust-wallet/src/main.rs :: AppState.utxo_selection_lock`, `backup_check_needed` | ✅ | |
| T1-P3's `money_utxos` / `reserved_by` | ⬜ not yet built | P2.2's selection uses whatever T1-P3 ships; re-verify at kickoff |

## 6. Out of scope

The intent record, three-valued broadcast outcome and crash matrix (P4 — BS-H3/H6/M2). Discovery/recency (P4). What the payload carries (P2.1). Restore (P2.3). Deltas (T3b-P6). Changing the backup address or KDF (D4/D5, owner-settled). A new table for backoff state.

## 7. Rollback

Revert the phase's commits (lock + checked reservation; baseline three-valued; backoff; cap; sync filter; `mark_failed` branch) — each is its own commit and reverts alone. No data migration; nothing written that the old code cannot read.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| The lock was added around selection but reservation stayed outside it — the race window shrank, did not close | A1 (interleavings through the real ARC mock, input-level) |
| Pinning the baseline made every idle cycle rebroadcast because the backup's own change output always differed | A5 (ii) |
| The baseline error became `Failed`, but `Failed` retried every tick with no backoff — the fix moved the loop, did not stop it | A3 + A6 together |
| The backoff status line was added but never shown because the panel reads a cached status | A6's SUBJECT is the rendered text after hard reload |
| The sync filter excluded index `-3` by literal, and a future reserved index (P4's intent, a second marker) was ingested again | §2: filter at the source by "reserved negative index", A8 |
| The `mark_failed` change held inputs forever on a transaction that was genuinely dead | A9's second case (explicit rejection ⇒ restored); BRC-177 deadline + reclaim is the only other release |
| The cap was set from the dev wallet (61 KB) and every production backup (431 KB) started failing | §12 Q2 — the owner sets it from **both** P0 wallets |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | all `cargo test` rows; A6's panel half on the macOS wallet overlay | 🍎 The status line renders in the wallet overlay (borderless `NSWindow`); check it once there — no C++ change |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~0.5 h** — §12 Q1 (who owns `mark_failed`), Q2 (cap value from P0), glance at the failing-backup status line wording |
| Human-bound rows | none for verification; the cap value is a decision |
| Unknowns (K) | **No.** Known fixes; the one design point (A5's loop-free pinned baseline) is understood, not uncertain |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | **T1-P3** | `money_utxos` + `reserved_by` — backup funding reads and reserves through it (decision 2a) |
| needs | **T3a-P0** E1/E2 | the numbers the cap is set from |
| needs | **T3a-P1** | mock chain, ARC mock with held responses (A5), interleaving harness (A1) |
| gives | **T3b-P6** | three-valued baseline read (P6-A2's RED is now "revert P2.2-A3's fix"); BS-C1 lock the delta producer inherits |
| gives | **T4-P1** | ⚠️ the `mark_failed` inconclusive branch stops restoring inputs — touches BRC-121 `nosend` rows T4-P1 describes as "the crash backstop; unchanged by P1". Coordinate at kickoff (§12 Q1) |
| watch | **T1-P2** | "a failure says it failed" — the same no-verdict-on-error rule on the signAction path; keep wording consistent |

## 12. Open questions for the owner

1. **Who owns `task_check_for_proofs.rs :: mark_failed`'s "inconclusive ⇒ force-fail ⇒ restore inputs" branch?** SCOPE put BRC-177 alignment in T3a-P2, but the code is money-path and T4-P1 touches the same rows. Recommendation: **T3a-P2.2 fixes it** (one branch, one row, A9), T4-P1 and T1-P3 review. Latent (0 log hits), so not urgent — but it is the same trip-wire-2 shape as §0.
2. **The size cap.** Recommendation: set it from P0's production E1/E2 numbers with headroom (a number, not a guess), fail closed above it.
3. No evidence that a G2 decision is wrong.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed (A1–A5, A8–A11: RED designed by the second agent, name recorded)
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
