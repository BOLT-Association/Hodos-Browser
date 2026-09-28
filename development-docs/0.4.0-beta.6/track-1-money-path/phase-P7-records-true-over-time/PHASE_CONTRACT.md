# B5-T1-P7 — a coin's record stays true after the chain moves, and no coin is stuck forever · PHASE CONTRACT

**Track:** B5-T1 Money path · **Tickets:** `../../tickets/TICKET_confirmed_tx_never_rechecked_after_reorg.md` · `../../tickets/TICKET_wallet_cannot_shed_large_parents.md` · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **T1 Q4** (agent-level, approved per `../SCOPE.md` §11): **narrow reorg handling** — *detect* (a stored proof's block is no longer on the main chain) and **flag and re-fetch**; **do not auto-revert statuses**. Rationale: the references disagree — `ts-stack` has `TaskReorg` / `reproveHeader` with an **open** bug (issue #545: stale proofs stay and spends fail `merged Beef failed validation`), `go-wallet-toolbox` v0.187.1 has **no** reorg task (issue #1035 open) — so we build only the part both would agree on. **2 / 2a** (shedding spends only money-index coins, through the guard). Root `CLAUDE.md` "Wallet Service Fee" (the consolidator already pays the fee on a schedule — flagged there as not user-initiated).

---

## 1. Goal

If a block the wallet relied on stops being part of the main chain, the wallet **notices, says so, and fetches a fresh proof** instead of trusting the old one forever; and a coin whose parent is too large to deliver over MessageBox or a 402 header is, once the user agrees, turned into a coin that can be delivered — reported done only when it actually is.

## 2. Done means

- [ ] **Reorg detection, bounded window.** A monitor task re-checks proofs for transactions proven within a recent window (proposed: the last **144 blocks** ≈ 1 day — §12 Q1): for each `proven_txs` row in the window, the stored `block_hash` is compared with the **main chain's** header hash at that height, fetched **fresh** from the Services chain (`services/mod.rs :: WalletServices::get_block_header`) — **never** from `database/block_header_repo.rs :: BlockHeaderRepository::get_by_height`, which returns the cached header the stale proof was checked against.
- [ ] **On a mismatch: flag and re-fetch, nothing else.** The transaction is flagged (a log line at `warn` naming txid, height, old and new hash; the row's existing `proven_tx_reqs` path is re-opened so `TaskCheckForProofs` fetches a new proof and re-verifies it with `cache_helpers.rs :: verify_tsc_proof_against_block`). **No** status is reverted, **no** input is un-spent, **no** output is deleted — the one automatic write is the re-opened proof request. If the transaction never re-confirms, that is visible (the row stays waiting for a proof) and becomes an owner decision, not an automatic one.
- [ ] **A header fetch failure is not a verdict:** `Err` / unknown ⇒ that row is skipped this pass and counted "unchecked" (rule 7, trip-wire 2).
- [ ] **Large-parent shedding, through the guard.** A pass finds **money-index** coins (P3/P5) whose parent transaction exceeds the tightest delivery budget (`handlers.rs :: large_parent_bytes` — the budget the selector already uses for MessageBox/BRC-121), and — **after the user confirms** (§12 Q2) — spends them back to a fresh self address in one transaction. Rules the ticket requires: the 1-sat floor, token-reserved values, never a non-default basket (all automatic once the spend reads only the money index).
- [ ] ⛔ **Not done until proven:** a shed is reported *"pending — usable in about 10–15 minutes"* until `TaskCheckForProofs` has a merkle proof for the consolidating transaction (BRC-62 stops the ancestry walk at a BUMP only then); the new coin's BEEF size is then re-measured.
- [ ] Shedding pays the **1,000-sat service fee** like every other outgoing transaction **unless the owner says otherwise** (§12 Q2) — and root `CLAUDE.md`'s fee table gains the row in the same commit (invariant 11).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| rule 7 trip-wire 1/3 | no impossible money-path row; durable state | An auto-revert on a false reorg signal would un-spend spent inputs — the reason the decision forbids reverts |
| `R-NOSPEND` / `R-DUST` | no automatic spend of a token / 1-sat coin | Shedding is a spend path; it reads only the money index, and it is user-confirmed |
| `R-BEEFOUT` | a BEEF we hand a counterparty stands on its own | Shedding exists so BEEFs get smaller; the re-measure after proof is `R-BEEFOUT`'s shape. State whether `MAX_BEEF_ANCESTORS` was reached |
| `R-PEERPAY-DELIVERY` | a PeerPay delivers or never leaves | The point of shedding; half 2 runs at this boundary with a previously-undeliverable coin |
| service fee | every outgoing tx pays 1,000 sats | Shedding is a new builder unless Q2 exempts it |
| memory: ARC txStatus ladder | ANNOUNCED ≠ success | "Shed" is success only at MINED + proof, not at broadcast |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. A green result is reported with its red half or not at all.
⛔ Money, schema and crypto rows: the RED (negative control) is **designed by someone other than the assertion's author** — a second agent (`../../../RELEASE_CYCLE.md` §4.2). Record who designed it.
⚠️ **A real reorg cannot be produced.** Every reorg row is T1 against a fake header source; the contract says so rather than implying a live test.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P7-A1` | A `proven_txs` row inside the window whose stored `block_hash` ≠ the fake main chain's hash at that height ⇒ flagged, proof request re-opened, **no** status/input/output changed | Three switches, one per clause: ① unregister the task in `monitor/mod.rs` ⇒ the stale row is not flagged and no `proven_tx_reqs` row is re-opened (SCOPE's sketch) · ② make the task also revert `transactions.status` / un-spend inputs on a mismatch (the auto-revert decision Q4 forbids) ⇒ the before/after diff shows a change — proves the "nothing else changed" diff can see one · ③ compare the stored `block_hash` against the fake header's hash in the **other byte order** ⇒ every row flagged, including the matching one (the two-sided half) — the format trap `cache_helpers.rs :: fix_tsc_byte_order` exists for; a check that flags everything passes the first clause alone · Right reason: the flag log names txid, height, old/new hash; a fake header source returning `Err` yields "unchecked", which is `P7-A3`, not this red · Residue: none (fake header source, in-memory DB) — designed by controls-B (Fable), 2026-09-28 | The row's flag + `proven_tx_reqs` state; every `transactions.status` and `outputs.spendable` for that txid unchanged (diffed) | T1 (money records) | ⬜ |
| `P7-A2` | ⭐ The check reads a **fresh** header: with the cache pre-seeded with the **stale** header and the fake chain serving the new one, the mismatch is still detected | Pre-condition asserted: `block_header_repo.rs :: get_by_height` returns the stale hash before the run — a cache never seeded cannot fail this row · Switch ①: point the check's lookup at `BlockHeaderRepository::get_by_height` instead of `services/mod.rs :: WalletServices::get_block_header` ⇒ zero mismatches while the fake chain differs (the instrument-cannot-fail shape) · Switch ②: keep the fresh call but let a provider `Err` fall back to the cache ⇒ with the fake chain erroring, the row reads the stale header and reports "matched" — it must count "unchecked" (`P7-A3`) · Right reason: the per-lookup log names `provider:<name>` or `cache`; the fake must sit **inside** the Services provider chain, not around the task, or that log line lies · Residue: none — designed by controls-B (Fable), 2026-09-28 | Which source answered (log line per lookup names provider vs cache). A check that reads the cache compares the stale proof with itself — an instrument that cannot fail (trip-wire 4) | T1 | ⬜ |
| `P7-A3` | Header source `Err` ⇒ row skipped and counted "unchecked"; nothing flagged | Map `Err` to "mismatch" in a scratch build ⇒ every row flagged (seen) | Flag count + unchecked count for the pass | T1 | ⬜ |
| `P7-A4` | A row **outside** the window is not re-checked (bounded cost) | Set the window to all rows ⇒ the out-of-window row is looked up (log line appears) | Lookup log lines per pass | T1 | ⬜ |
| `P7-A5` | After a flagged row gets a fresh proof that verifies, the flag clears and the row's proof is the new one | ⚠️ `cache_helpers.rs :: verify_tsc_proof_against_block` fetches the block **by height from a hard-coded WhatsOnChain URL with the raw client** — not through `WalletServices`, not the cache — so a T1 fixture cannot hand it a fake fresh proof; either the phase routes that lookup through `WalletServices::get_block_header(BlockKey::Height)` or this row is T2 (§4a) · Switches: ① do not call `proven_tx_repo.rs :: replace_proof` after a verified re-fetch ⇒ `block_hash`/`merkle_path` unchanged and the flag still set · ② clear the flag on fetch, before verification, with a fresh proof that **fails** to verify ⇒ the flag clears on a bad proof (two-sided: a clear that does not require a verified proof is not a clear) · Right reason: the row's `block_hash` before/after and the verifier's `Merkle root verified` line for the new hash · Residue: none — designed by controls-B (Fable), 2026-09-28 | The `proven_txs` row's `block_hash`/`merkle_path` before and after | T1 | ⬜ |
| `P7-A6` | Shedding (user-confirmed): a scratch-wallet coin with a parent > `large_parent_bytes` (e.g. the ticket's ~400 KB class) is spent to self; inputs are **only** that coin (and fee coins from the index); outputs = self + 1,000-sat fee (unless Q2) + change | Switches, each asserted on the serialised tx with broadcast stubbed before the one real run: ① let the builder select from the confirmed pool instead of the named coin ⇒ other outpoints appear among the inputs · ② remove the fee output ⇒ no `HODOS_FEE_ADDRESS` output (if Q2 exempts the fee, this switch inverts) · ③ remove the confirmation gate ⇒ `P7-A9`'s shape · Pre-condition: the coin's parent size is measured (`parent_transactions` byte length) and stated to exceed `handlers.rs :: large_parent_bytes` — a coin below the line is the wrong subject · Right reason: inputs/outputs by outpoint; WhatsOnChain for the real one · Residue: the real shed pays 1,000 sats to the treasury plus a network fee (irreversible) on the scratch wallet named here — designed by controls-B (Fable), 2026-09-28 | The **serialised transaction** and WhatsOnChain; the coin named by outpoint and parent size before the run | T2 (money) | ⬜ |
| `P7-A7` | The shed is reported *pending* until a merkle proof exists; after it, the new coin's BEEF is small and a PeerPay over MessageBox with it **succeeds** where the old coin was refused | Two switches: ① report "done" at broadcast (ANNOUNCED) instead of at proof, then send the PeerPay **inside the pre-proof window** ⇒ refused again with the same size error — the BEEF still carries the grandparent (ANNOUNCED ≠ success); schedule the send before the block arrives · ② disable shedding ⇒ the same PeerPay is refused (SCOPE's sketch) · Measure, do not estimate: the "after" byte count comes from the bytes actually built for the recipient (`beef_helpers`), with the `MAX_BEEF_ANCESTORS` cap state logged — a figure from the selector's `parent_sizes` map is the estimate, not the measurement · Right reason: the recipient's `peerpay_received` row appears only after the proof; the MessageBox size error is the refusal, not an auth error · Residue: one real PeerPay between two scratch wallets — designed by controls-B (Fable), 2026-09-28 | The BEEF byte count before/after (and whether `MAX_BEEF_ANCESTORS` was reached); the recipient's `peerpay_received` row | T2 (money) | ⬜ |
| `P7-A8` | Shedding never selects a token, a 1-sat coin or a non-default-basket coin even if its parent is large | Fixtures (parents above the line, sizes stated): a ≥2-sat ordinal filed in `1sat` (non-default basket), a ≥2-sat BRC-162-shaped Unknown row (basket NULL, `change=0`), and a 1-sat coin — three guards, three fixtures; the 1-sat one tests only the floor and its survival is not evidence for the other two · Switch: revert the shed's coin source from the money index to `output_repo.rs :: get_spendable_confirmed_by_user` (basket filter kept) ⇒ the Unknown row is selected; additionally drop the basket predicate ⇒ the filed ordinal is selected · ⛔ Assert on the built tx with broadcast stubbed — never broadcast the RED · Right reason: the three outpoints in/out of the serialised inputs · Residue: none on chain; scratch DB with reserved-but-unbroadcast inputs, discarded — designed by controls-B (Fable), 2026-09-28 | A large-parent **ordinal** fixture (outpoint named) — untouched after the pass | T1 + T2 | ⬜ |
| `P7-A9` | Shedding does not run without the user's confirmation (no timer-driven spend) | Wire it to a monitor timer in a scratch build ⇒ a spend happens with no prompt (seen) | Broadcast log over a day of dev use; zero shed transactions without a confirmation record | T1 + T2 | ⬜ |

**Two-sided rows:** `P7-A1` (stale proof flagged) ⟷ a row whose hash matches is **not** flagged (in the same fixture — a task that flags everything passes A1 alone). `P7-A6`/`A7` (large-parent coin shed and usable) ⟷ `P7-A8` (a token is never shed).

### 4a. Independent control notes (2026-09-28)

*By controls-B (Fable). Findings only — the author's GREEN/SUBJECT cells are untouched. Most serious first.*

- `P7-A5` — **SUBJECT is not reachable at T1.** `cache_helpers.rs :: verify_tsc_proof_against_block` resolves the block by height from a hard-coded `api.whatsonchain.com/.../block/height/{h}` URL using the raw `reqwest::Client`, not `WalletServices` and not the header cache (read today). A "fake header source" therefore cannot drive the re-verification; the row as written is T2 against the live chain, where no reorg can be produced. Fix: route that height lookup through `WalletServices::get_block_header(BlockKey::Height(h))` — the same fresh source `P7-A2` demands — so one fake serves both rows; otherwise mark A5 as not runnable and say so (HARNESS §8).
- `P7-A2` — the same finding bears on the SUBJECT's "log line per lookup names provider vs cache": if the phase reuses the verifier's direct URL for the task's own height lookup, there is no provider name to log — it is a raw URL. The task must go through the Services chain for the log to mean anything.
- `P7-A1` — the byte-order trap is live: `computed_root == actual_root` in the verifier is a string compare of two hex encodings, and `fix_tsc_byte_order` exists because the two orders have already been confused once. The new task compares `proven_txs.block_hash` (written from the TSC `target`) against a provider hash — state which order each is in, in the contract, before the first run.
- `P7-A8` — three guards (index membership, basket, 1-sat floor) need three fixtures; a row run with only a 1-sat ordinal proves the beta.3 floor and nothing this phase adds.
- `P7-A9` — the author's RED ("wire it to a timer ⇒ a spend happens with no prompt") is a real spend that pays the 1,000-sat fee; it should run with broadcast stubbed on the scratch build and the cell should say so (rule 7: declare residue).
- `P7-A6`/`A7` — the fee clause inverts if the owner answers Q2 "no fee"; both REDs are written for the recommended answer and must be re-read at kickoff if it changes.
- Not added: a row for "the fresh proof WoC serves right after a reorg is still the old one (provider lag)". `P7-A5` ② (a bad proof must not clear the flag) already covers the consequence; the row would only re-measure it.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/database/proven_tx_repo.rs :: ProvenTxRepository`; `migrations.rs` — `proven_txs(txid, height, merkle_path, raw_tx, block_hash, merkle_root)` | ✅ | The stored proof and its block hash |
| `rust-wallet/src/database/block_header_repo.rs :: BlockHeaderRepository::get_by_height`, `get_by_hash`, `upsert` | ✅ | ⚠️ The cache the reorg check must **not** read |
| `rust-wallet/src/services/mod.rs :: WalletServices::get_block_header` (providers: `whatsonchain.rs`, `junglebus.rs`) | ✅ | Fresh header source |
| `rust-wallet/src/cache_helpers.rs :: verify_tsc_proof_against_block` (callers: `monitor/task_check_for_proofs.rs` ×2, `handlers.rs` ×1) | ✅ | Runs only when a proof is first acquired today (ticket) — reused for the re-fetch |
| `rust-wallet/src/monitor/task_check_for_proofs.rs :: run`; `monitor/mod.rs` schedule (`check_for_proofs: 60`, `consolidate_dust: 86400`) | ✅ | New task registered here (reorg); shedding is **not** scheduled |
| `rust-wallet/src/handlers.rs :: large_parent_bytes`, `large_parent_bytes_for_budget`, `select_utxos_with_preference` (parent-size aware) | ✅ | The existing budget |
| `rust-wallet/src/monitor/task_consolidate_dust.rs :: run_inner` | ✅ | The closest existing self-spend builder — reused shape, not a second consolidator |
| Tools tab / wallet panel (T6) | — | Where the user confirms a shed |

## 6. Out of scope

- Automatic status reversal after a reorg (decision Q4 — both references would not agree on it; TS's own is broken).
- Reorg handling for the **backup chain** (T3's H17).
- A header-follower that tracks the tip continuously — the window check on the monitor cadence is enough for the narrow scope.
- Automatic (timer-driven) shedding — §12 Q2.

## 7. Rollback

Reorg: revert the task registration — no data written except flags and re-opened proof requests, which the old code ignores. Shedding: revert the endpoint and card; shed transactions already made are ordinary self-sends.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Row that catches it |
|---|---|
| The check compares the stored proof with the cached header it was validated against — it can never find a reorg, and "no reorgs found" looks like health | `P7-A2` |
| An indexer outage looks like "block not on main chain" and every recent transaction is flagged — or worse, reverted | `P7-A3`; no reverts by decision |
| The window grows with the wallet and every pass fetches thousands of headers | `P7-A4` |
| Shedding reports success at broadcast; the user retries the PeerPay and it fails again (BEEF still carries the grandparent) | `P7-A7` (pending until proof) |
| Shedding spends a large-parent **ordinal** (inscriptions have large parents by nature) | `P7-A8` — this is the likeliest real victim |
| Shedding runs on a timer and quietly pays the treasury 1,000 sats a day on a wallet full of large-parent coins | `P7-A9`; §12 Q2 |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | A1–A9 | Primary |
| macOS | A1–A5, A8 (T1) + `cargo test` | Rust only; the money rows run once on Windows. If the confirm step lands in the wallet panel, a macOS glance at it. Relay round names this contract |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1 h** — one shed of a real large-parent coin on a scratch wallet, wait for the block, one PeerPay with the new coin. (A reorg cannot be produced by hand.) |
| Human-bound rows | A6, A7 (real money, a block's wait), A9's day of dev use |
| Unknowns (K) — uncertainty, not difficulty | **K = 1.** (a) The window size and how often the task runs (header fetch cost vs detection lag); (b) the fee question (Q2). The reorg **signal source** is resolved: fresh `get_block_header` by height vs the stored `block_hash` |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs ← | B5-T1-P3, P5 | Shedding reads only the money index; the guard |
| ↔ | **B5-T4** (402 payments) | The BEEF budget ⇄ large parents: a shed coin is what makes a 402 payable again when every coin was too big (`TICKET_brc121_beef_header_exceeds_100kb…` is T4's). T4-P2 (the 431 path) and this phase measure the same size |
| gives → | B5-T6 | The confirm surface for a shed (Tools tab or wallet panel) |
| gives → | B5-T3a | A flagged (reorged) transaction is carried in a backup as-is; T3 must not treat the flag as a new field unless it chooses to (the flag reuses existing proof-request state) |
| gives → | root `CLAUDE.md` fee table | A shedding row, if Q2 keeps the fee |

## 12. Open questions for the owner

| # | Question | Recommendation |
|---|---|---|
| **Q1** | Reorg window — how far back to re-check proofs? | **144 blocks (≈ 1 day)**, checked hourly. BSV reorgs deeper than a few blocks are not expected; a day is cheap and generous |
| **Q2** | Shedding: user-confirmed or automatic, and does it pay the 1,000-sat service fee? | **User-confirmed, and it pays the fee** like every other outgoing transaction. Offer it where the failure happens (*"this coin is too large to send this way — shed it? costs ~1,000 sats + network fee, usable in ~10–15 min"*). Automatic would repeat the consolidator's "fee on a schedule the user never triggered" |

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
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
