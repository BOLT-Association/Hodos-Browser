# B6-P2 — `abortAction` releases only what never left the wallet, and never hangs · PHASE CONTRACT

**Release:** `v0.4.0-beta.6` · **Source:** triage G3 (`../ADVISORY_TRIAGE.md`: TSA-260 re-lock, TSA-023 abort after broadcast, TSA-044 swallowed errors; TSA-042 origin binding **deferred** to `../../0.4.0-beta.8/tickets/TICKET_abortAction_not_bound_to_originating_site.md`) · **Status:** ✅ DONE (T1)
**Opened:** 2026-09-30 · **Author:** Claude (Opus 5.5) · **Platforms:** shared Rust, both
**Standard:** `../../0.4.0-beta.3/HARNESS.md` + `../../0.4.0-beta.7/HARNESS_DELTA.md`.
**Owner decisions carried (2026-09-30):** fix the re-lock, the release of inputs and which statuses may be aborted; origin binding deferred (needs a schema change).

---

## 1. Goal

A dApp's `abortAction` answers promptly, releases the coins of an action that never reached the network, and refuses to release the coins of one that might have.

## 2. Done means

- [x] `abortAction` never re-locks the DB mutex it holds: a call on an abortable action returns (`P2-A1`). It used to deadlock: `handlers.rs :: abort_action` held `db` while `release_unbroadcast_transaction` locked `state.database` again (a `std::sync::Mutex`).
- [x] Abortable only from **`unsigned`, `unprocessed`, `nonfinal`** (never fully signed) and **`nosend`** (only after every chain provider says the txid is unknown). **Refused** for `sending`, `unproven`, `completed`, `failed`, an unrecognised status, and incoming actions — named error, nothing changed (`P2-A2`, `P2-A3`).
- [x] A successful abort: created outputs not spendable, reserved inputs restored, status `failed` **with `failed_at`** (so `TaskUnFail` re-checks it; Hodos has no `aborted` status), balance cache invalidated, and the in-memory `PENDING_TRANSACTIONS` entry removed so a later `signAction` on that reference is not found (`P2-A4`, `P2-A5`).
- [x] `nosend` known to any provider, or any provider not answering ⇒ refused; the reservation stays for the existing backstops (`TaskCheckForProofs` NOSEND timeout, `TaskSweepReservations`) (`P2-A3`).
- [x] Found by `reference`, or by txid (as `go-wallet-toolbox` does). A failed release returns an error, never `aborted:true`, and rolls everything back (`P2-A6`).
- [x] A `signAction` that read the pending transaction before the abort committed cannot sign after it: it refuses once the row is `failed` (`P2-A7`). `sign_action`'s update block calls `update_txid` (delete + re-insert of the row) and then sets `sending`/`nosend`, which would have un-aborted the row and broadcast it.
- [x] An abort cannot land in the middle of the wallet's own broadcast of a `nosend` row (`P2-A8`, from the adversarial review): it waits on `create_action_lock`, which `create_action_internal` holds from coin selection through broadcast (including `sendWith`), and which `broadcast_nosend` now also takes.

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-NODOUBLE` (beta.3 P0.7/8b) | Reservations are never wrongly released | This phase **releases** reservations. The status gate, the unanimous chain check, the status compare-and-set and `create_action_lock` are what keep a broadcast transaction's inputs spent |
| `R-GOLD` | Gold pill on auto-approved payment | Not touched: abort moves no money and fires no pill. `broadcast_nosend` (BRC-121) only gains a lock wait |
| `R-INTEXT` | Internal never prompts, external always gates | Not touched: the gate runs before the handler |

## 4. Evidence table

⛔ Money rows: REDs designed by a second agent (Claude Opus 5.5) before the tests were final; A3c, A4j and A8 added by the author and the adversarial review.

| ID | 🟢 GREEN | 🔴 RED (mutation in `negative_controls.py`) | 🎯 SUBJECT | Tier | Result |
|---|---|---|---|---|---|
| `P2-A1` | The real `abort_action` handler, on its own thread, returns `aborted:true` for an `unsigned` action within 5 s and releases it | **A1** re-take the DB mutex while holding it (the pre-fix shape) ⇒ "abortAction HUNG" · **A1b** hold the DB lock across the chain call ⇒ "DB lock was held during the chain call" (the fake chain `try_lock`s) | `abort_action` (web handler) over a real `WalletDatabase` on a temp file | T1 | 🟢🔴 |
| `P2-A2` | `sending`/`unproven`/`completed`/`failed`/`not-a-status`, and an **abortable-status** incoming action ⇒ 400 `ERR_NOT_ABORTABLE`; a full snapshot of every `transactions` and `outputs` column is unchanged; pending entry kept | **A2a** `unproven` abortable ⇒ "status unproven" · **A2b** gate through `TransactionStatus::from_str` (maps unknown → `unprocessed`) ⇒ only "status not-a-status" · **A2c** incoming allowed ⇒ "incoming" | DB snapshot before/after | T1 | 🟢🔴 |
| `P2-A3` | `nosend`: mempool / mined / rejected / unknown state ⇒ 400; transport error ⇒ 503 `ERR_ABORT_CHAIN_UNVERIFIED`; all refused with rows unchanged. Absent ⇒ released, the chain asked **exactly once**, never under the DB lock; reservation under the txid and under the placeholder both restored. Status moved on during the chain call ⇒ refused by the compare-and-set, nothing released. `abort_chain_verdict`: only `NotFound` is absent, over all 5 `TxState`s and 5 error variants. `call_unanimous_not_found`: `NotFound` only when **every** provider said so (`[2,1,1,1]`, `[3,1,1,1]`… refused); `call` itself unchanged | **A3a** any error = absent ⇒ "chain error" · **A3b** rejected/unknown = absent (`check_tx_exists_on_chain`'s reading) ⇒ "rejected" · **A3c** `nosend` without the chain check ⇒ "in mempool" + the race test · **A3d** last-error semantics ⇒ "[2, 1, 1, 1]" · **A3e** a timed-out provider counts as "no" ⇒ "[3, 1, 1, 1]" | Chain injected (`abort_action_with_chain`); the collection with its own `TestProvider` | T1 | 🟢🔴 |
| `P2-A4` | `unsigned`/`unprocessed` (placeholder) and `nonfinal` (txid): created output `spendable=0`, input `spendable=1`, `failed`, `failed_at` set, balance cache cleared, chain **not** consulted. Second abort refused with the DB unchanged, although the input was re-reserved by a new action. Unrelated action untouched. No pending entry (restart): aborted, placeholder reservation left for the sweeper. A created output already reserved (`spending_description`) or spent (`spent_by`) by another action ⇒ refused, unchanged | **A4a** created outputs left spendable · **A4b** placeholder not restored · **A4c** txid not restored · **A4d** `failed_at` not set ⇒ "failed_at not set: TaskUnFail would never re-check" · **A4e** cache not invalidated · **A4h** CAS result ignored · **A4i** CAS without the status · **A4j** child check removed · **A4k** `spent_by` arm removed | DB rows | T1 | 🟢🔴 |
| `P2-A5` | After abort, the real `sign_action` ⇒ 404 "Transaction reference not found", pending entry gone | **A5** entry kept ⇒ "pending entry survived the abort" (5 tests) | `sign_action` (web handler) | T1 | 🟢🔴 |
| `P2-A6` | Found by txid (and its pending entry, keyed by reference, removed); unknown ⇒ 404; a fault injected into the input restore (SQLite trigger) ⇒ 500 `ERR_ABORT_FAILED`, never `aborted:true`, status/rows rolled back, pending entry kept | **A6a** no txid lookup ⇒ "by txid" · **A6b** restore error swallowed (`unwrap_or(0)`) ⇒ `aborted:true` | Handler response + rows | T1 | 🟢🔴 |
| `P2-A7` | Pending entry present, row already `failed` ⇒ `sign_action` 409 `ERR_ACTION_ABORTED`, row still `failed` with its txid. Control arm: the same fixture with the row `unsigned` signs (200), proving the call reaches the update block | **A7** guard removed ⇒ 200, row rewritten | `sign_action` (web handler) | T1 | 🟢🔴 |
| `P2-A8` | While `create_action_lock` is held: `abort` does not run (300 ms) and changes nothing; `broadcast_nosend` does not run; after release the abort succeeds | **A8a** abort without the lock · **A8b** `broadcast_nosend` without the lock | Both handlers | T1 | 🟢🔴 |

**Controls run:** `negative_controls.py` (this folder) restores both files, applies one mutation, runs the P2 tests (only `p2_a1` / `p2_a3_nosend` for the two lock mutations, whose other tests would hang rather than fail; each run has a 15-minute cap), checks exactly which tests failed and that the stated text appears, and restores. **25/25 RED OK**, green again after restore (2026-09-30).

**Not falsifiable in T1 (declared):** (1) `abort_action` → `WalletServices::tx_status_unanimous` is a three-line wrapper; no test goes through the real providers, so swapping it back to `tx_status` would pass T1. The unanimity rule itself is covered at the collection. (2) Release order (created outputs before inputs) happens inside one SQLite transaction, so no single-threaded test can observe it.

**Live check (2026-09-30, one HTTP call each, random txid):** WhatsOnChain, JungleBus, Bitails and ARC GorillaPool all answer **404** for an unknown txid, so the unanimous rule can say "absent" in production (a provider that answered 500 for unknown txids would have made every `nosend` abort a 503).

## 5. Blast radius (verified 2026-09-30)

| Cited code | Note |
|---|---|
| `rust-wallet/src/handlers.rs :: abort_action`, new `abort_action_with_chain`, `abort_chain_verdict`, `ABORTABLE_NEVER_SENT` | The change |
| `handlers.rs :: sign_action` (update block) | One guard: refuse if the row is `failed` (`P2-A7`). ⚠️ Behaviour change: it also refuses rows failed by `TaskFailAbandoned` (unsigned > 300 s) or by the NOSEND timeout; both have already released their inputs, so signing them was the bug |
| `handlers.rs :: broadcast_nosend` | Takes `create_action_lock` (`P2-A8`). Its only caller is the C++ paid-retry chain, after `pay_402` has returned, so no nesting |
| `services/collection.rs :: call` → `call_tracking`; new `call_unanimous_not_found`; `services/mod.rs :: tx_status_unanimous` | `call` returns `call_tracking(..).0`: same order, stats, demotion and result for every existing caller (`p2_call_keeps_its_last_error_semantics`) |
| `handlers.rs :: check_tx_exists_on_chain` | **Not reused**: it reads `Rejected`/`Unknown`/orphan as "does not exist" (right for "is it on chain?", wrong for "did it ever leave?") |
| `handlers.rs :: release_unbroadcast_transaction` | **Not reused**: it swallows every DB error (`unwrap_or(0)`, `let _ =`) and returns `(0,0)` on a failed commit, indistinguishable from "nothing to release" (trip-wire 2). Abort does the same steps with `?`, in one SQLite transaction with the status compare-and-set. **Reported, not changed** (rule 3); its two callers are the refusal paths |
| `action_storage.rs :: TransactionStatus::to_action_status` | Maps `nosend` → legacy `Aborted`, which made the old "already aborted" early return fire for every `nosend` action. Abort now reads the raw column. The mapping itself is P5's (`listActions`) |

## 6. Out of scope

- Origin binding (TSA-042): beta.8 ticket, schema change.
- A real `aborted` status: schema/enum change.
- `sendWith` partial broadcast (TSA-261): beta.7 T1.

## 7. Rollback

One commit; `git revert`. No schema, no data.

## 8. Pre-mortem and residuals

| Failure story | Caught by |
|---|---|
| The deadlock fix drops the lock and another request re-reserves or changes the action mid-abort | Status compare-and-set in the same SQLite transaction as the release; `P2-A3` race, `P2-A4` second abort |
| A dApp broadcasts its `nosend` bytes, then aborts | `P2-A3` unanimous chain check, fail closed |
| Abort lands while the **wallet** is broadcasting a `nosend` row (every ordinary send is signed `nosend` then broadcast; `sendWith`; `broadcast_nosend`) — "not indexed yet" reads as absent | `P2-A8` (found by the adversarial review) |
| Abort succeeds but `signAction` still signs, from memory or mid-flight | `P2-A5`, `P2-A7` |
| An unknown status string reads as abortable through `from_str` | Raw-string gate; `P2-A2` |
| A parent is aborted while a child spends its output; aborting the child later resurrects the parent's output | Refused (`P2-A4`). **Deliberate difference:** go-wallet-toolbox disables created outputs unconditionally (its reservations live in a separate table) |

**Residuals (not closed, stated):**
1. **Propagation window.** A dApp that holds `nosend` bytes can broadcast them seconds before, or any time after, its own abort; "every provider says not found" cannot see a broadcast that has not propagated. The coins are then released while the transaction lands. `failed_at` is set, so `TaskUnFail` re-marks the inputs spent and re-enables the outputs **if it is mined within 6 h**. This is the same exposure the existing 10-minute NOSEND timeout already has (it force-fails even on an inconclusive oracle); abort only makes it reachable sooner. Origin binding (beta.8) narrows *who* can abort, not *when*.
2. `pay_402`'s reuse check reads `nosend`, drops the lock, then hands the bytes to C++. An abort in that gap leaves the row `failed`; `broadcast_nosend` then skips it, and `TaskUnFail` heals it if the server broadcasts and it is mined.
3. An `unsigned` action aborted after a restart (no pending entry) answers `aborted:true` with its inputs still reserved under the placeholder; `TaskSweepReservations` frees them. `aborted:true` means "this action will never be sent", not "every coin is back already".
4. A `nosend` abort needs all four providers to answer; one outage ⇒ 503 every time (fail closed, by design).

**Deliberate difference from prior art:** `go-wallet-toolbox` aborts `nosend` on its own broadcast evidence ("park the known tx"). A `nosend` transaction's signed bytes were handed to the dApp, which may broadcast them itself; we ask the chain first (working rule 7). Its "park" role for the wallet's own broadcasts is played here by `create_action_lock` (`P2-A8`).

## 9. Platforms

Shared Rust; T1 runs on both via `cargo test`. No C++.

## 12. Open questions for the owner

None. Reviewer claims **checked and rejected**, with evidence: "two-phase `nosend` inputs stay under the phase-1 txid, so abort reports success over a leak" — `transaction_repo.rs :: update_txid` re-keys `spending_description` from the old txid to the new one (`relinked_spent`), so the inputs follow the signed txid.

---

## Sign-off

| Item | Result | Date | By |
|---|---|---|---|
| preflight `-Full` | PASS — all checks ran | 2026-09-30 | Claude (Opus 5.5) |
| preflight `-NegativeControl` | PASS — every gate seen to fail | 2026-09-30 | Claude (Opus 5.5) |
| `cargo test --workspace` | 0 failed across 20 targets | 2026-09-30 | Claude (Opus 5.5) |
| P2 negative controls | 25/25 RED OK (`negative_controls.py`), green after restore | 2026-09-30 | Claude (Opus 5.5); REDs designed by a second agent |
| adversarial review (after) | Second agent (Claude Opus 5.5). **Closed:** abort during the wallet's own `nosend` broadcast (HIGH → `P2-A8`, lock in abort and `broadcast_nosend`); CAS race answered 500 instead of a refusal (now 400); `spent_by` arm untested (→ A4k); a poisoned pending lock skipped the removal (now `into_inner`); wrong test comments; contract drift. **Rejected with evidence:** two-phase input leak (§12). **Declared:** wrapper wiring and release order not falsifiable in T1 (§4); residuals (§8). Lock order DB → `PENDING_TRANSACTIONS` checked at every site: no inversion | 2026-09-30 | second agent + author |
