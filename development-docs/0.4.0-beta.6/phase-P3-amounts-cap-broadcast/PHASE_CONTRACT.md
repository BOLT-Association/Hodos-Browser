# B6-P3 — amounts are real numbers, the cap sees the real spend, and a broadcast acknowledgement is ours before we believe it · PHASE CONTRACT

**Release:** `v0.4.0-beta.6` · **Source:** triage G3 (`../ADVISORY_TRIAGE.md`): TSA-148 / -069 negative or overflowing output amounts, TSA-141 spending-cap sum, TSA-192 / -255 broadcast acknowledgement for an empty or different txid. **TSA-250 (weak ARC statuses) deferred to beta.7 T1** (👤 owner, 2026-09-30): `../../0.4.0-beta.7/tickets/TICKET_weak_arc_statuses_count_as_on_the_network.md` · **Status:** ✅ DONE (T1)
**Opened:** 2026-09-30 · **Author:** Claude (Opus 5.5) · **Platforms:** shared Rust + one shared C++ header (`PaymentCost.h`, no platform split)
**Standard:** `../../0.4.0-beta.3/HARNESS.md` + `../../0.4.0-beta.7/HARNESS_DELTA.md`.
**Owner decisions carried (2026-09-30):** fix amounts, cap and broadcast binding; the cap fixture must sum to **≥ 2 sats**, so the old `satoshis > 0` floor cannot be what catches it. **Taken this session:** defer the weak-ARC-status row (see §6).

---

## 1. Goal

A site cannot make a payment look smaller than it is or get an impossible amount into the wallet's books, and the wallet never marks a transaction confirmed, or caches a proof for it, on the strength of an answer about some other transaction.

## 2. Done means

- [x] `create_action_internal` (every builder: dApp `createAction`, send, PeerPay, Paymail, `pay_402`) refuses a negative output, or outputs summing past 21,000,000 BSV (2.1 × 10¹⁵ sats; this also bounds each output), with 400 `ERR_INVALID_OUTPUT_AMOUNT`, **before** any coin is selected or reserved (`P3-A1`). `create_action`'s defence-in-depth cap sum uses the same check (its `.sum()` panicked on overflow in debug builds and, in release, wrapped negative and skipped the cap).
- [x] The browser-side price (`PaymentCost.h :: ExtractOutputSatoshis`) is **not derivable**, so the engine prompts instead of pricing, when any **numeric** amount is negative, fractional, above the maximum, an unsigned value that would wrap, or the outputs sum past the maximum. Fixture `[1000000, -999998]` (2 sats; the old code priced it at 2 sats). Non-number fields keep their old handling (ignored, pinned by `MalformedAndEmptyBodiesAreZeroNotCrash`; Rust refuses them at parse) (`P3-A2`).
- [x] `broadcast_transaction`: an acknowledgement is **bound** only if it names our txid. From an unbound one nothing is cached and our row is never marked `confirmed` (`P3-A3`). **Different txid:** our txid is looked up across every provider — known ⇒ success; unknown to every provider or rejected ⇒ error; lookup inconclusive ⇒ success, unproven. **Empty txid** (WoC/mAPI "already known", the reply to our own bytes): success, unproven, no lookup.

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-GOLD` | Gold pill on auto-approved payment | `P3-A2` changes which payments are priced; an ordinary payment wrongly "not derivable" would prompt instead of auto-approving. `OrdinaryAmountsStillPrice` covers createAction, 0-sat, max-exact, send, PeerPay and 402 shapes |
| `R-CAP` | Over-cap payments prompt | Strengthened; ordinary amounts price identically |
| `R-NODOUBLE` | A failed broadcast releases its inputs; a successful one does not | `P3-A3` turns some old successes into errors. Only when **every** provider says our txid is unknown (the `NotFound` variant), or it is rejected; an inconclusive lookup and an empty-txid ack stay successes |

## 4. Evidence table

⛔ Money rows: REDs designed by a second agent (Claude Opus 5.5) before the tests were final; A3f/A3g from the adversarial review.

| ID | 🟢 GREEN | 🔴 RED (mutation in `negative_controls.py`) | 🎯 SUBJECT | Tier | Result |
|---|---|---|---|---|---|
| `P3-A1` | Validator: `[0,1,546]` ⇒ 547; max-exact ⇒ ok; no-amount (sendMax) output skipped; negative, `i64::MIN`, max+1, `i64::MAX`, sum past max, 5×max ⇒ refused. Builder: `[1000000,-999998]`, `[max,1]`, `[i64::MIN]` ⇒ 400 `ERR_INVALID_OUTPUT_AMOUNT` with a full DB snapshot unchanged; control: 1,000 sats gets past the check | **A1a** negatives allowed · **A1c** sum unbounded ⇒ "above max accepted" (the sum bound is the per-output bound) · **A1d** 0-sat refused · **A1e** max-exact refused · **A1f** builder skips the check | `validate_output_amounts`; the real `create_action_internal` over a temp wallet DB | T1 | 🟢🔴 |
| `P3-A2` | `[1000000,-999998]` ⇒ not derivable, `priceAvailable=false`; fractional, max+1, sum past max, `18446744073709551614` (wraps to −2 as int64), `INT64_MIN`, `1e300`, negative / over-max single shapes ⇒ not derivable; ordinary amounts price as before | **A2a** negatives read ⇒ "priced 2 sats" · **A2b** floats read · **A2c** unsigned not range-checked (also fails the single-amount test: positive JSON integers are unsigned in nlohmann) · **A2d** sum unbounded · **A2e** single `amount` unchecked | `hodos::ExtractOutputSatoshis` / `ComputePaymentCost` (gtest, `hodos_tests`) | T1 | 🟢🔴 |
| `P3-A3` | Control: bound + `MINED` + a real BUMP ⇒ proof cached, ours `completed`, no lookup. Different txid + `MINED` + BUMP: never confirms ours, caches nothing, looks up **ours** exactly once; known ⇒ Ok, unknown ⇒ Err, inconclusive ⇒ Ok. Empty txid (even with `MINED` + BUMP) ⇒ Ok, no lookup, nothing cached or confirmed. Lookup verdict: absent only on the `NotFound` variant — an error whose text says "not found" is inconclusive | **A3a** every ack bound · **A3b** lookup asks about the provider's txid · **A3c** unbound proof cached · **A3d** unknown accepted · **A3e** inconclusive ⇒ error (releases inputs) · **A3f** lookup by error text · **A3g** empty-txid ack looked up and refused | `accept_broadcast_result` (the post-network half of `broadcast_transaction`, lookup injected) over a temp DB; `broadcast_lookup_verdict` | T1 | 🟢🔴 |

**Controls run:** `negative_controls.py` (this folder) — Rust rows run `cargo test p3_`, C++ rows rebuild and run `hodos_tests`; each mutation restored before the next. **17/17 RED OK**, green after restore (2026-09-30).

**Not falsifiable in T1 (declared):** (1) the production lookup wiring `broadcast_lookup_verdict(tx_status_unanimous(..))` inside `broadcast_transaction` (no injectable broadcaster); its two halves are each covered. (2) `checked_add` in the validator: with every term ≤ max the sum cannot overflow `i64`; it is kept as a guard, not claimed as evidence. (3) `create_action`'s cap-sum call site: everything it refuses is also refused by `create_action_internal`; only the debug-build panic differs.

## 5. Blast radius (verified 2026-09-30)

| Cited code | Note |
|---|---|
| `rust-wallet/src/handlers.rs :: validate_output_amounts`, `MAX_SATOSHIS`; `create_action_internal`; `create_action` (cap sum) | The amount check |
| `cef-native/include/core/PaymentCost.h :: ReadSatoshis`, `ExtractOutputSatoshis`, `kMaxSatoshis` | C++, header-only, shared by both platforms ⇒ Mac relay |
| `handlers.rs :: broadcast_transaction` → `accept_broadcast_result`, `broadcast_lookup_verdict` | All 18 callers checked by the review: 17 pass `Some(db), Some(txid)`; `wallet_recover_external`'s sweep passes `None, None` (no binding possible, unchanged). `task_send_waiting` treats "TX collision" as transient (retries) |
| `handlers.rs :: check_tx_exists_on_chain` | Body moved verbatim into `tx_exists_verdict`; unchanged for `/wallet/cleanup`, `internalize_action`, `TaskCheckPeerPay` |
| `services/collection.rs :: call_unanimous_not_found` | Error text reworded to "not every provider answered" (it contained "not found", which text matchers read as absent) |

## 6. Out of scope

- ⏸️ **Weak ARC statuses counted as on the network (TSA-250)** — deferred to beta.7 T1 by the owner (2026-09-30). Found at implementation: tightening the money-in check alone changes nothing, because our own broadcast accepts `QUEUED`/`RECEIVED`/`STORED` as success, and the code base already draws the line three ways. Ticket above.
- Merkle-proof verification before storing (TSA-188/-249): beta.7 T1. This phase only stops caching an **unbound** proof.
- `/wallet/cleanup` chain error ⇒ "spent" (TSA-306): beta.7 T1-P2. ARC 200 with an unknown status (TSA-415, P2).

## 7. Rollback

Rust and C++ in one commit; `git revert`. No schema, no data.

## 8. Pre-mortem and residuals

| Failure story | Caught by |
|---|---|
| A legitimate "already known" rebroadcast now fails and releases a mempool tx's inputs | Empty-txid acks are accepted as unproven (`P3-A3`, A3g) |
| An inconclusive lookup ("some providers did not answer") reads as absent | Lookup by variant (`P3-A3`, A3f — found by the adversarial review) |
| The amount check refuses a 0-sat data output or a sendMax request | `P3-A1` |
| `PaymentCost` stops pricing ordinary payments ⇒ every payment prompts | `OrdinaryAmountsStillPrice` |

**Residuals (stated):**
1. **Inconclusive lookup ⇒ success also applies to money in.** `internalize_action` and `TaskCheckPeerPay` broadcast the sender's transaction; if a provider names a different txid and our lookup is inconclusive, the payment is credited. The row is stored `unproven`, and `TaskCheckForProofs` (which scans `unproven`) fails it and disables its outputs after all oracles return 404 for 5 minutes. Before this phase the same case **with** a merkle path was credited unconditionally.
2. An empty-txid ack is still not correlated beyond "it answered our request"; `TaskCheckForProofs` is the correlation.

## 9. Platforms

Rust shared. `PaymentCost.h` is header-only and shared; gtest runs on both. **Mac relay:** `cef-native/include/core/PaymentCost.h`, `cef-native/tests/payment_cost_test.cpp` (round W-30c).

## 12. Open questions for the owner

None open. Answered 2026-09-30: TSA-250 → defer to beta.7 T1.

---

## Sign-off

| Item | Result | Date | By |
|---|---|---|---|
| preflight `-Full` | PASS — all checks ran | 2026-09-30 | Claude (Opus 5.5) |
| preflight `-NegativeControl` | PASS — every gate seen to fail | 2026-09-30 | Claude (Opus 5.5) |
| `cargo test --workspace` | 0 failed across 20 targets | 2026-09-30 | Claude (Opus 5.5) |
| `hodos_tests` (C++) | PASS | 2026-09-30 | Claude (Opus 5.5) |
| P3 negative controls | 17/17 RED OK (`negative_controls.py`), green after restore | 2026-09-30 | Claude (Opus 5.5); REDs designed by a second agent |
| adversarial review (after) | Second agent (Claude Opus 5.5). **Closed:** HIGH — the inconclusive unanimous lookup read as "absent" through a text match and would release a mempool tx's inputs (→ `broadcast_lookup_verdict`, A3f; error text reworded); empty-txid ack refused on index lag (→ accepted unproven, A3g); docstring settle time (5 min, not 6 h); contract drift. **Recorded:** money-in inconclusive ⇒ credited, settled by `TaskCheckForProofs` (§8). **Checked clean:** legitimate flows (0-sat, max, sendMax, every builder), `/wallet/cleanup` unchanged, C++/Rust agree on what an amount is, all 18 broadcast callers | 2026-09-30 | second agent + author |
