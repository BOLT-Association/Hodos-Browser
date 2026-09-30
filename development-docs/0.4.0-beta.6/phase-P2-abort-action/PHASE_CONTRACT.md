# B6-P2 — `abortAction` releases only what never left the wallet, and never hangs · PHASE CONTRACT

**Release:** `v0.4.0-beta.6` · **Source:** triage G3 (`../ADVISORY_TRIAGE.md`: TSA-260 re-lock, TSA-023 abort after broadcast, TSA-044 swallowed errors; TSA-042 origin binding **deferred** to `../../0.4.0-beta.8/tickets/TICKET_abortAction_not_bound_to_originating_site.md`) · **Status:** 🚧 IN PROGRESS
**Opened:** 2026-09-30 · **Author:** Claude (Opus 5.5) · **Platforms:** shared Rust, both
**Standard:** `../../0.4.0-beta.3/HARNESS.md` + `../../0.4.0-beta.7/HARNESS_DELTA.md`.
**Owner decisions carried (2026-09-30):** fix the re-lock, the release of inputs and which statuses may be aborted; origin binding deferred (needs a schema change).

---

## 1. Goal

A dApp's `abortAction` answers promptly, releases the coins of an action that never reached the network, and refuses to release the coins of one that might have.

## 2. Done means

- [ ] `abortAction` never re-locks the DB mutex it holds: a call on an abortable action returns within a timeout (`P2-A1`). Today it deadlocks (`handlers.rs :: abort_action` holds `db` while `release_unbroadcast_transaction` locks `state.database` again).
- [ ] Abortable only from **`unsigned`, `unprocessed`, `nonfinal`** (never broadcast by construction) and **`nosend`** (only after the chain says the txid is absent). **Refused** for `sending`, `unproven`, `completed`, `failed`, and for incoming actions — with a named error, nothing changed (`P2-A2`, `P2-A3`).
- [ ] A successful abort: created outputs not spendable, reserved inputs restored, status `failed` (Hodos has no `aborted` status; adding one is a schema/enum change), balance cache invalidated, **and the in-memory `PENDING_TRANSACTIONS` entry removed** so a later `signAction` on that reference cannot sign it (`P2-A4`, `P2-A5`).
- [ ] `nosend` found on chain, or chain check inconclusive ⇒ refused, reservation left for the existing backstops (`TaskCheckForProofs` NOSEND timeout, `TaskSweepReservations`) (`P2-A3`).
- [ ] Found by `reference`, or by txid (as `go-wallet-toolbox` does). Errors are not swallowed: a failed release returns an error, not `aborted:true` (`P2-A6`).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-NODOUBLE` (beta.3 P0.7/8b) | Reservations are never wrongly released | This phase **releases** reservations. The status gate and the `nosend` chain check are what keep a broadcast transaction's inputs spent |
| `R-GOLD` | Gold pill on auto-approved payment | Not touched: abort moves no money and fires no pill |
| `R-INTEXT` | Internal never prompts, external always gates | Not touched: the gate runs before the handler |

## 4. Evidence table

⛔ Money rows: REDs designed by a second agent. *(pending)*

| ID | 🟢 GREEN | 🔴 RED | 🎯 SUBJECT | Tier | Result |
|---|---|---|---|---|---|
| `P2-A1` | Abort of an `unsigned` action returns within 5 s (tokio timeout) | Today's code: the same test **hangs** and hits the timeout | `abort_action` itself, over an in-memory wallet DB | T1 | ⬜ |
| `P2-A2` | `sending` / `unproven` / `completed` / `failed` / incoming ⇒ refused, named code; outputs, inputs, status unchanged | Status gate removed ⇒ an `unproven` action's inputs become spendable | DB rows before/after | T1 | ⬜ |
| `P2-A3` | `nosend` + chain says absent ⇒ aborted; chain says present ⇒ refused; chain error ⇒ refused | Chain check removed ⇒ the "present" case releases inputs | Chain check injected (no network in T1) | T1 | ⬜ |
| `P2-A4` | After abort of `unsigned`: created outputs `spendable=0`, inputs restored, status `failed`, balance cache invalidated | Release call removed ⇒ inputs stay reserved | DB rows | T1 | ⬜ |
| `P2-A5` | After abort, `signAction` on that reference ⇒ not found | `PENDING_TRANSACTIONS` removal dropped ⇒ `signAction` signs | The real `sign_action` handler | T1 | ⬜ |
| `P2-A6` | Lookup by txid works; a DB error yields an error response, never `aborted:true` | — | Handler response | T1 | ⬜ |

## 5. Blast radius (verified 2026-09-30)

| Cited code | Note |
|---|---|
| `rust-wallet/src/handlers.rs :: abort_action` | The change |
| `handlers.rs :: release_unbroadcast_transaction` | Reused unchanged; its contract ("broadcast has not happened") is what the status gate now enforces for this caller |
| `handlers.rs :: PENDING_TRANSACTIONS`, `sign_action` | Entry removed on abort |
| `handlers.rs :: check_tx_exists_on_chain` (or its `internalize_action` copy) | Reused for the `nosend` gate |
| `database/transaction_repo.rs :: get_by_reference`, `get_by_txid` | Lookup |
| `action_storage.rs :: TransactionStatus::to_action_status` | ⚠️ Maps `nosend` → legacy `Aborted`, which is why today's "already aborted" early return fires for **every** `nosend` action. The handler will read `TransactionStatus`, not the legacy enum. The mapping itself belongs to P5 (`listActions` statuses) |

## 6. Out of scope

- Origin binding (TSA-042): beta.8 ticket, schema change.
- A real `aborted` status: schema/enum change.
- `sendWith` partial broadcast (TSA-261): beta.7 T1.

## 7. Rollback

One commit; `git revert`. No schema, no data.

## 8. Pre-mortem

| Failure story | Caught by |
|---|---|
| The deadlock fix drops the lock too early and another request re-reserves the inputs mid-abort | Release runs inside `release_unbroadcast_transaction`'s single SQLite transaction; `P2-A4` |
| A dApp broadcasts its `nosend` bytes, then aborts: we release inputs of an on-chain tx (double-spend risk) | `P2-A3` chain check, fail closed |
| Abort succeeds but `signAction` still signs from memory | `P2-A5` |
| The `nosend` fix breaks BRC-121's paid-retry flow (its tx is `nosend` until `broadcast_nosend`) | ⚠️ Nothing in the 402 path calls `abortAction`; confirm by grep at implementation |

**Deliberate difference from prior art:** `go-wallet-toolbox` aborts `nosend` on its own broadcast evidence alone. A `nosend` transaction's signed bytes were handed to the dApp, which may broadcast them itself, and no evidence of that reaches the wallet. We ask the chain first (working rule 7: the chain is the authority).

## 9. Platforms

Shared Rust; T1 runs on both via `cargo test`.

## 12. Open questions for the owner

None beyond the carried decisions. The `nosend` chain check is stricter than the reference; recorded in §8.

---

## Sign-off

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| negative controls | | | |
| adversarial review | | | |
