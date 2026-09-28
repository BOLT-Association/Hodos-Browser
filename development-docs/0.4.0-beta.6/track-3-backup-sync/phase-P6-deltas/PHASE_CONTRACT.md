# B5-T3b-P6 — After the first backup, only what changed goes on chain · PHASE CONTRACT

**Track:** B5-T3b Sync & portability · **Tickets:** none; `research/ONCHAIN_BACKUP_REVIEW.md` multi-writer items land in P7 · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 agent for T3b · **Platforms:** both (Rust wallet only) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** **4(a)** 👤 *"sync must never re-broadcast the whole backup — deltas or something equally small"*; **4(b)** efficiency is **tested, not assumed**; **5** (the on-chain format never claims BRC-38; freeze before the first mainnet broadcast of a new token version); **2a** (the money index is derived — never in a delta); **3**. `IMPLEMENTATION_PLAN.md` Phase 5 / D2 / D13 / D14 / H3 / H7 / H19 (survive per SCOPE §5.9, "constants provisional").

> **Scope in one line:** single-device deltas. Two devices writing is **P7**, and P7 is R&D — this phase must not assume its answer (H4 and the D15 tie-break are **not** here).

---

## 0. Fresh read — 2026-09-28

| Source | Finding | Consequence |
|---|---|---|
| BRC-40 (via T3a-P0 §0, read today) | Unchanged since 2026-04-24: sync of 12 entities by watermark, **no deletes, no versioning** | Plan D2 stands: BRC-38 row forms **plus** our own per-table `deletes` and a per-row version counter. Named as a deviation from BRC-40 in P8 |
| `go-private-backup-cache` | Still `ca2136e` (2026-08-21), **still no licence** (`gh api …/license` = null) | D3 stands: borrow two ideas (append-only log, `prev_payload_sha256` link), **no code** |
| wallet-toolbox `WalletStorageManager.ts` | Last change `b3155fa` 2026-09-22 — unchanged since SCOPE §2.5 | — |
| Our code, re-read for this contract | ⚠️ `handlers.rs :: do_onchain_backup` decides "nothing changed" with `settings_repo.get_backup_hash().unwrap_or(None)`, and `settings_repo.rs :: get_backup_hash` itself **logs and returns `None` on a DB error**. An error therefore reads as *"no baseline"* ⇒ a **full** backup is built and broadcast | Trip-wire 2 shape (a verdict where an error is owed), and it is exactly the event decision 4(a) forbids. Owned here (P6-A2's RED), because a missing baseline must mean *snapshot* only when it truly is missing |

## 1. Goal

After a wallet's first on-chain backup, each later backup puts on chain only the rows that changed since the last one — small, bounded and restorable to exactly the same wallet — and nothing short of the declared snapshot rule ever re-broadcasts the whole thing.

## 2. Done means

- [ ] A delta producer exists (plan D2: BRC-38 row forms + per-table deletes + per-row version counter), fed from one consistent DB snapshot, pinned to the last **confirmed-broadcast** baseline (D13)
- [ ] Restore replays snapshot + deltas in parent-topological order and yields `canonical()` byte-identical to the continuous wallet, for ≥ 1,000 seeded op sequences (H3) — money index rebuilt, never carried (decision 2a)
- [ ] ⭐ **Decision 4(a) as a number:** over a P0-E3-shaped workload, zero full snapshots except those the declared snapshot rule fires; a DB/read error on the baseline is an **error**, never "no baseline"
- [ ] The snapshot rule's constants (plan §3.2: 0.5× / 20 deltas / 16 KB — provisional) are **replaced by measured values** from P0-E3 and this phase's distribution, with the owner's P0 acceptance numbers (bytes per backup, six-month slope) met or the miss reported
- [ ] H1 with N ∈ {0, 1, 5, 20} deltas; H7 from every generation boundary; H19's single-device half (crash-ghost successor); H15 — every token version ever broadcast still restores
- [ ] The delta envelope is **frozen and signed by the owner before the first mainnet broadcast of a delta token** (decision 5's gate, applied to the new version this phase creates)

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| Working rule 7, trip-wire 2 | an error is never a verdict | The baseline-hash read above; a delta producer that reads "no rows changed" from a failed query would silently stop backing up |
| Trip-wire 3 / decision 5 freeze | a mainnet token is permanent and must decrypt forever (plan G11) | A delta token broadcast before the freeze is a format we must support forever — P6-A9 + the sign-off gate |
| D14 | nothing deletes or restores on a timer | A delete record emitted by a timer-only monitor transition would erase a row on every future restore |
| `R-RESTORE` | fail-closed survives recovery | Replay must hand every restored output to the classifier path T3a-P2 built, same as a snapshot restore |
| `R-NOSPEND` | no incidental spend | Delta broadcasts are funded like today's backups: smallest-sufficient, never a 1-sat input (`R-PEERPAY-DELIVERY` half 1's existing `a3_backup_funding_is_smallest_sufficient` test is the control) |
| Service fee | backups carry none today | A refactor that routes delta broadcasts through `create_action_internal` would add 1,000 sats per delta — must not happen (SCOPE §6) |
| Invariant 2 | no schema change without asking | ⚠️ The per-row version counter (D2's arbiter) and a delta-baseline record are **new columns/table** ⇒ §12 Q1 |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. Money, schema and crypto rows: RED designed by a **second agent** (`../../../RELEASE_CYCLE.md` §4.2).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P6-A1` (H3 replay) | ≥ 1,000 seeded op sequences (receive, spend, label, basket, certificate op, permission change; snapshots at arbitrary points): `canonical(replay(snapshot + deltas)) == canonical(continuous)` byte-for-byte; money index rebuilt equal | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The **replayed** DB in a fresh process, via T3a-P1's `canonical()` (which includes money-index rows); seed list recorded | T1 (mock) | ⬜ |
| `P6-A2` ⭐ *never the whole backup* (decision 4a) | Over a scripted month of P0-E3-shaped activity: every broadcast is a delta except snapshots the declared rule fires, each logged with the rule clause that fired; a failing baseline read **aborts the cycle with a visible error** and broadcasts nothing | (1) Force `get_backup_hash` to return `Err` ⇒ on **today's** code the cycle broadcasts a full snapshot — the test must go red there (that is the RED observed), and green after the fix. (2) Set the snapshot rule to "always" ⇒ the "only declared snapshots" assertion goes red | The mock chain's broadcast log: each tx's payload type (snapshot/delta) and byte size; the rule clause per snapshot. Never the producer's own claim | T1 | ⬜ |
| `P6-A3` *deterministic and idempotent* | Two runs on identical state ⇒ identical bytes; applying any delta twice == once; producing on identical state ⇒ an empty delta that is **not broadcast** (plan G6: zero no-op broadcasts) | Replace one sorted collection in the producer with hash-map iteration order ⇒ the determinism test must go red within the seed set; skip the "empty ⇒ no broadcast" check ⇒ the no-op counter goes non-zero | Bytes of two runs; the broadcast log's count for an idle interval | T1 | ⬜ |
| `P6-A4` *deletes only on allowlisted, chain-corroborated causes* (D14) | Every delete record carries an allowlisted cause; timer-only monitor transitions (`task_fail_abandoned`, `task_purge`, `task_sweep_reservations`) never emit one | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Each delete record's cause field vs the allowlist; the monitor task that caused the row change, from the log | T1 | ⬜ |
| `P6-A5` *mutation during broadcast lands in the next delta* (D13) | A change committed while a delta is in flight appears in the **next** delta and never in the pinned baseline; the producer reads one consistent snapshot | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The mutated row by id across delta k and k+1; the baseline hash stored after k | T1 | ⬜ |
| `P6-A6` (H1 with deltas) | Seed-only restore + **spend** from chains of 1 snapshot + N ∈ {0,1,5,20} deltas; balance equal to the satoshi both ways; refetch errors 0 | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The restored DB (fresh process) and a signed spend through the **production** selector (T3a-P1-A2's rule) | T1 | ⬜ |
| `P6-A7` (H7) | Restore from every generation boundary t == the true state at t; a healthy tip with `Unknown` spent-status (plain P2PKH — R1-01) restores normally | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Per-t canonical diff; the mock serves real `NoSignal` semantics (T3a-P1-A5) | T1 | ⬜ |
| `P6-A8` (H19, single-device half) | With deltas, the write path never consumes as a sweep input a marker that decrypts under our key and carries `seq` ≥ the local tip — incl. a crash-ghost successor marker (plan R1-05) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The sweep tx's inputs vs the decrypted markers' `seq` values, by outpoint | T1 | ⬜ |
| `P6-A9` (H15) | Every binary payload version ever broadcast to mainnet (starting with the shipped headerless snapshot) and every schema-generation JSON fixture still restores through the current build; the fixture set is append-only | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The checked-in fixtures (never regenerated), restored to a spendable wallet | T1 | ⬜ |
| `P6-A10` *measurement row* | Delta-size distribution per op type and deltas-per-snapshot per profile published beside P0-E3; the §3.2 constants replaced by these numbers; the owner's two acceptance numbers compared, met or missed | The tool fed two **identical** copies must report 0 rows / no broadcast; fed a planted 10 KB label change it must attribute ≥ 9.5 KB to `tx_labels` — a tool that cannot see a planted change cannot measure a real one | Numbers written into `../phase-P0-prove-and-measure/MEASUREMENTS.md` next to E3, with seeds and profile | T1 | ⬜ — **INCOMPLETE without a number** (`HARNESS_DELTA.md` §1.3) |

**Two-sided rows:** `P6-A1` (nothing lost) ⇄ `P6-A2` (nothing re-broadcast) — a producer that "passes" A2 by emitting less than it should fails A1, and one that passes A1 by snapshotting every time fails A2.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/handlers.rs :: do_onchain_backup` | ✅ | Hash-compare skip (`get_backup_hash().unwrap_or(None)`), `compress_for_onchain(…, ref_ts)`, invoice `1-wallet-backup-1`, PushDrop build, previous PushDrop + `marker` lookup — the write path the delta type extends. Same builder, not a parallel one |
| `rust-wallet/src/handlers.rs :: adopt_onchain_backup`, `fetch_onchain_backup`, `wallet_recover_onchain`, `refetch_stripped_data` | ✅ | Discovery + replay; the adopt/sweep region is A8's subject |
| `rust-wallet/src/database/settings_repo.rs :: get_backup_hash`, `set_backup_hash` | ✅ | `get_backup_hash` returns `None` on error — A2's RED |
| `rust-wallet/src/backup.rs :: collect_payload`, `compress_for_onchain`, `compress_payload`, `encrypt_compressed`, `derive_onchain_backup_key`, `serialize_for_onchain`, `deserialize_from_onchain` | ✅ | The producer reuses the collector; the KDF (`SHA-256(master ‖ "hodos-wallet-backup-v1")`, plan D4) is **unchanged** — invariant 3 |
| `rust-wallet/src/monitor/task_backup.rs :: run` | ✅ | The trigger; posts to `/wallet/backup/onchain`, "the handler does hash comparison" |
| `rust-wallet/src/monitor/task_fail_abandoned.rs`, `task_purge.rs`, `task_sweep_reservations.rs` | ✅ (files present) | Timer-driven transitions A4 must keep out of delete records |
| `rust-wallet/src/database/migrations.rs` — `sync_states` table | ✅ | Carried, not activated (plan D9) — the delta baseline must not quietly start using it without §12 Q1 |
| `rust-wallet/src/main.rs :: AppState.utxo_selection_lock`, `create_action_lock` | ✅ | Delta funding takes the same lock as every other spend (BS-C1 is fixed in T3a-P2 — P6 inherits, does not re-fix) |

## 6. Out of scope

Two devices (P7): H4, D15 tie-break, device ids, poll-before-spend. Size-class **padding** (plan Phase 6) — SCOPE §5 P6 makes it conditional on P0's numbers; §12 Q2. Changing the KDF or backup address (D4/D5, owner-settled). Chunking beyond the hard cap (plan: "chunking not defined in v1"). Anything in the BRC text (P8).

## 7. Rollback

Before the first mainnet delta broadcast: revert the phase's commits; the snapshot path is unchanged underneath. ⛔ **After** a delta token is on mainnet, rollback means keeping the delta **decoder** forever (H15 / plan G11) and reverting only the producer — which is why the freeze sign-off comes first.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| A transient SQLite error on the baseline read made the wallet re-broadcast 400 KB, several times a day | A2 (RED 1) |
| Deltas were byte-identical in replay tests because the comparator excluded the columns that changed | T3a-P1-A4 ("sees money") + A1 uses the same `canonical()` |
| A monitor task flipped rows on a timer and the delta log turned those into deletes; every future restore erased them | A4 |
| The delta envelope went to mainnet before anyone froze it, and the next release changed a field | §2 last item, sign-off gate, A9 |
| The snapshot rule's 0.5×/20/16 KB constants shipped unmeasured | A10 is INCOMPLETE without numbers |
| The producer was designed with a single-writer assumption that P7 later has to break (e.g. `seq` without a device dimension) | §11 edge to P7: the envelope reserves the field P7's R4-1 answer needs, or P7's answer is recorded as requiring a new version |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | all (`cargo test`, mock) | Rust-only, platform-neutral; the per-platform difference (DPAPI vs Keychain mnemonic storage) is exercised by A6's restore on each |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~0.5 h**: §12 Q1 (schema) + Q2 (padding), 0.25 h · sign the delta envelope freeze before the first mainnet delta, 0.25 h. The live seed-only restore smoke (H18) is T3a's per-release row, not repeated here |
| Human-bound rows | the freeze sign-off (irreversible outward event) |
| Unknowns (K) — uncertainty, not difficulty | **No** — hard-but-understood (SCOPE §7.2). The constants are unknown until P0-E3 lands, but that is a measurement input, not a design risk |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | **T3a-P0** E1, E3 | real fee rate and delta bytes (the constants and the owner's acceptance numbers) |
| needs | **T3a-P1** | harness, mock, `canonical()`, H2 manifest |
| needs | **T3a-P2** | restore classifies before rebuild; BS-C1 lock; baseline pinning (D13 — "a live silent-loss bug") |
| needs | **T3a-P4** | the frozen envelope/header, intent record (D7) and history discovery (D6) that the delta type rides on |
| needs | **T1-P3** | the money index (excluded from deltas, rebuilt on replay) |
| gives | **P7** | the delta log and its measured sizes — R4-2's per-delta cost input |
| gives | **P8** | the delta format the BRC §7 describes |
| watch | **T4-P1** | stuck `noSend` rows follow BRC-177 (T3a-P2); a delta must never carry a status change T4 did not make |

## 12. Open questions for the owner

1. **Q1 — schema (invariant 2).** Plan D2's per-row version counter needs a column on each carried table (or one side table keyed by table+row), and the pinned baseline needs a durable record of "last delta confirmed broadcast" (today only `settings.backup_hash` + `last_backup_at`). **Recommendation:** one side table (`backup_row_versions`: table, row id, version) plus a baseline row, rather than a column on every table — one migration, one rollback, nothing added to money tables. Your approval is needed before the kickoff.
2. **Q2 — padding this release?** Size-class padding (plan Phase 6) hides payload size from an observer of the backup address, not timing or funding linkage (plan Appendix A item 11). **Recommendation:** decide from P0's numbers at this phase's kickoff; default **no** for beta.6 if the bytes it costs exceed the owner's steady-state bar.
3. No evidence that decisions 4 or 5 are wrong.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed (rows marked ⏳: RED designed by the second agent, name recorded)
- [ ] 👤 Delta envelope freeze signed **before** the first mainnet delta broadcast — date + commit
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
