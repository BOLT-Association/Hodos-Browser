# B5-T3b-P7 — Two devices, one wallet: can "one active device at a time" meet the bar over the chain? (R&D) · PHASE CONTRACT

> ⚠️ **PROVISIONAL R&D CONTRACT (decision 4).** 👤 *"this decision is very early in the research and development phase… as soon as we kick off that ticket, it's gonna be a lot of work to design it."*
> **"Done" for this contract = written, evidenced answers to R4-1, R4-2 and R4-3, exactly as decision 4 states them — not shipped code.** Building whatever the answers choose is a **separate contract (P7b)**, written after the owner reads them — and it may not fit this release (G5.5 must see that).

**Track:** B5-T3b Sync & portability · **Tickets:** none; the multi-writer `BS-*` items in `../research/ONCHAIN_BACKUP_REVIEW.md` (SCOPE §7.4: ~4) are inputs · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 agent for T3b · **Platforms:** both (R&D on mock + read-only live measurement) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** **4** in full — direction *one active device at a time* (wallet-toolbox model, SCOPE Q3 option B); owner constraints (a) never re-broadcast the whole backup, (b) chain efficiency **tested, not assumed** — *"if this whole method is just too inefficient, then we're gonna have to just start over with using a cloud or something"*, (c) agents are a separate future track but this design must not block them. **5** (the chain copy never claims BRC-38). SCOPE §0 row Q3 and §9 Q3 (⛔ not option D — it links every payment publicly).

---

## 0. Fresh read — 2026-09-28

| Source | Finding | Bearing |
|---|---|---|
| wallet-toolbox `WalletStorageManager.ts :: isActiveEnabled`, `setActive` | Unchanged since SCOPE §2.5's read (last change `b3155fa` 2026-09-22): exactly one **active** storage; if stores disagree about which is active, spending is **disabled** until `setActive` merges | The reference for R4-1 — our marker must have the same "disagreement ⇒ nobody spends" property |
| BRC-155 `wallet/0155.md` §Concurrency (T3a-P0 read today) | Address allocation across writers MUST be single-writer, device-scoped ranges, or compare-and-set | A second device must not hand out receive addresses either — R4-1 covers addresses, not only coins |
| `go-private-backup-cache` | `ca2136e` (2026-08-21), **still no licence** | R4-2's middle ground: read for **semantics only** (per-device append logs; merge unspecified), never code |
| **BRC-181** `wallet/0181.md` (merged, last change `fc03173` 2026-09-22) | An autonomous agent spends from a **dedicated, isolated wallet account** under a signed policy (per-tx / period / lifetime caps, destination allowlist, rate, circuit breaker); *"blast radius = the account balance, never the main wallet"* | R4-3: an agent **off** the active device is exactly BRC-181's isolated account — option C (a partition), not a second writer on the main coins |
| Our code, re-read | No `device_id` anywhere in `rust-wallet/src`; the backup chain tip is a PushDrop (`1-wallet-backup`, suffix `1`) + a P2PKH `marker` (`handlers.rs :: do_onchain_backup`); `reconcile.rs :: check_outpoint_spent` returns `Unknown` for plain P2PKH on both providers (plan R1-01) | R4-1: a device cannot learn "someone spent the tip" from the marker's spent status. Whether the PushDrop output's spent status is observable is **unmeasured** — R4-1's first sub-question |

## 1. Goal

The owner can decide — from measurements, not estimates — whether "one active device at a time, coordinated through the chain" meets his bar (stable, conflict-free, efficient), and if it does not, has the evidence for the cloud/relay question.

## 2. Done means *(each is an answer, written in this folder as `R4_ANSWERS.md`, with its evidence rows green)*

- [ ] **R4-1 — how the active device is recorded and switched.** A written design covering: where "active" lives (starting sketch from decision 4: a marker riding in the backup chain tip; a device not named is **read-only**), how a device takes over (proposed: spending the current tip is a chain-level compare-and-set — two takeovers cannot both confirm), how a demoted device **learns** it is demoted (given R1-01), what "read-only" blocks in code (spends, reservations, **receive-address issuance** — BRC-155), and the **residual race window** measured on the mock with the P0-contracted lag distribution. Evidence: P7-R1a–R1d
- [ ] **R4-2 — the chain as the sync channel.** Poll vs push (which push sources exist for BSV today, and what each costs), indexer lag p50/p95 **measured live** (read-only), per-delta fee and bytes (from P0 E1/E3 and P6-A10), and the device-to-device round-trip time (write on A → visible on B). Compared against **the owner's numeric bar** (P7-R0). **Exit:** meets the bar ⇒ proceed to a P7b build contract; **cannot** ⇒ evaluate the middle ground first (off-chain relay + chain as authority, `go-private-backup-cache` semantics), then return the cloud/relay question **to the owner with the evidence**. Evidence: P7-R2a–R2d
- [ ] **R4-3 — agents as writers.** A written answer: an agent on the active device runs under that device's authority; an agent elsewhere gets **its own coin allowance** (option C / BRC-181's isolated account); plus a check that R4-1's design does not bake in an assumption that blocks the future agents track. Evidence: P7-R3
- [ ] ⛔ Any spike lives on `spike/b5-t3b-p7`, runs only on the mock and read-only live calls, **never broadcasts on mainnet**, and is **deleted** the day this contract closes — its only surviving output is `R4_ANSWERS.md` and the numbers

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| Production isolation | the installed wallet is never contended | Two-device experiments run on two **scratch** profiles on the mock; live measurement is read-only (WoC/ARC status reads, no broadcast) |
| Working rule 7 | escalate on evidence | A lag measurement that looks like "the chain lost our backup" is checked against the chain before anyone is told |
| Invariant 2 / 3 | no schema / crypto change without asking | R&D writes none. If the answer needs a device table or a new marker derivation, it is a **P7b** owner ask, named in `R4_ANSWERS.md` |
| `R-NOSPEND` | no incidental spend | The mock's two devices use the production selectors; nothing here spends real coins |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. Money rows: RED designed by a **second agent** (`../../../RELEASE_CYCLE.md` §4.2). R&D measurement rows: a measurement with no number is **INCOMPLETE**, not green (`HARNESS_DELTA.md` §1.3).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P7-R0` 👤 *the bar, as numbers, before measuring* | The owner has set, in writing: max acceptable delay before device B can spend after A stops (switch time); max acceptable added latency per spend on the active device; max bytes and fee per sync event; tolerated residual race window (e.g. "zero" or "≤ N s with a visible warning") | If R4-2's measurement runs before these exist, its conclusion is the agent's opinion — the row is **void**, not green. (Recorded so nobody sets the bar after seeing the numbers) | The owner's written numbers with date, in `R4_ANSWERS.md` §0 | T4 (owner decision) | ⬜ |
| `P7-R1a` *the race exists without the mechanism* (the instrument can fail) | On the mock, two scratch devices on one phrase, mechanism **off**, lag drawn from the P0-A11 distribution: a same-coin double-spend occurs within N rounds (SCOPE §5 P7's control), N recorded | **Switches (this row is the instrument's own control):** (1) make the race impossible — one device only, or the two devices strictly alternated with zero lag ⇒ the detector must report **zero** double-spends; a detector that reports conflicts there is counting something else · (2) define a double-spend as *two different txids spending one outpoint*: an identical re-broadcast (T3a-P4 allows one) or two txs sharing a parent must not count · (3) ⭐ subject: two separate processes and data dirs on one phrase, each using the production selector and backup builder — two handles on one DB serialize through SQLite and the in-process locks and hide the race · (4) lag drawn from the P0-A11 distribution with the seed recorded · **Red for the right reason:** the mock's per-outpoint spend log for the zero-race configuration · **Residue:** none (mock, spike branch) — designed by controls-E (Opus), 2026-09-28 | The mock's accepted/rejected tx log by outpoint (txid:vout, sats) — the coin both devices tried to spend | T1 (mock) | ⬜ |
| `P7-R1b` *the mechanism holds* | Mechanism **on**: zero same-coin double-spends in ≥ 500 randomized rounds incl. the 30 s–5 min tail; two simultaneous takeovers ⇒ exactly one confirms; a demoted device refuses spends **and** receive-address issuance | **Switches:** (1) mechanism off on **the same seed list** that made R1a red ⇒ the double-spends reappear ⇒ red · (2) takeover by a flag write instead of spending the tip ⇒ two simultaneous takeovers both *confirm* ⇒ red · (3) remove the read-only check from receive-address issuance only ⇒ the demoted device issues an address ⇒ red — tests BRC-155's half apart from spends · (4) ⭐ print the lag histogram and the demoted device's attempt counts: zero rounds in the 30 s–5 min tail, or zero issuance attempts, void the corresponding claim · **Red for the right reason:** the spend log by outpoint and each refusal reason per attempt · **Residue:** none (mock; spike branch deleted at close) — designed by controls-E (Opus), 2026-09-28 | Same log as R1a; the demoted device's refusal reason per attempt | T1 (mock) | ⬜ |
| `P7-R1c` *the window, measured* | The residual race window (both devices believe they are active) as a distribution: p50/p95/max over the rounds, and what the user sees in it | Plant a known 90 s lag on B only ⇒ the tool must report a window ≥ 90 s for B's rounds; a tool reporting ~0 there measures nothing | The window per round from both devices' logs, aligned on mock time | T1 | ⬜ |
| `P7-R1d` *how a demoted device learns* | Measured, not assumed: for the backup **PushDrop** output and the **marker**, what WoC (and a second provider) reports once spent — status value and delay. States plainly whether the device can learn "demoted" from spent status, from address history (P0-A11), or only by walking the chain from its last known tip | Query a **known-spent** and a **known-unspent** PushDrop outpoint first (positive control, decision 10's method); if both read the same, the method cannot tell them apart and the row is void | Recorded HTTP bodies with outpoints and dates; read-only | T2 (live, read-only) | ⬜ |
| `P7-R2a` *indexer lag, live* | p50/p95/max seconds from broadcast (by someone else's tx we observe, or our own dev backup when one happens naturally) to visibility in each endpoint the design reads (history, unspent, tx), ≥ 30 samples | Replay the sampler against the mock with a planted fixed lag ⇒ it must report that lag ± 1 s | Per-sample txid, broadcast time source, first-seen time per endpoint | T2 (read-only) | ⬜ |
| `P7-R2b` *poll vs push* | A table of push sources available for BSV today (e.g. ARC callbacks, JungleBus-style subscriptions, WoC sockets — **each verified to exist by a fetch, dated**) with cost, auth, privacy exposure (who learns our backup address), and failure mode; vs poll cost at the interval R0 implies | A source listed without a dated fetch of its current docs is struck from the table — the row reviewer checks every link | The dated fetches | T2 (read-only) | ⬜ |
| `P7-R2c` *per-sync cost* | Bytes and fee per sync event from P0-E1/E3 and P6-A10, projected to the R0 interval over six months; compared with R0's bar | If P0-E1/E3 or P6-A10 are still INCOMPLETE the projection is **not computed** — no numbers are invented to fill it | The source rows' numbers, cited by id | T1 | ⬜ |
| `P7-R2d` *verdict + exit* | One of: **meets the bar** (every R0 number met, with margins) ⇒ draft a P7b contract; **does not** ⇒ the middle ground (off-chain relay + chain authority, `go-private-backup-cache` semantics) evaluated against the same R0 numbers, then the cloud/relay question goes to the owner with R2a–R2c attached | A verdict that cites no R0 number, or reaches "cloud" without having evaluated the middle ground, fails this row — decision 4's exit is followed exactly or not at all | `R4_ANSWERS.md` §R4-2, each claim linked to a row | — | ⬜ |
| `P7-R3` *agents are not blocked* | Written: agent on the active device = that device's authority (its spends go through the same permission engine and the same active check); agent elsewhere = its own coin allowance (BRC-181 isolated account, option C partition), never a second writer on the main coins; plus a list of every assumption in R4-1's design, each marked "does / does not block an agent writer" | A second agent tries to write one concrete agent scenario (e.g. a headless agent on a server paying per query while the user's laptop is active) that R4-1's design makes impossible without a redesign; if it finds one, the row is red until the design leaves room | The scenario text and the design clause it tests | — (design review) | ⬜ |

**Two-sided rows:** `P7-R1a` (the race is visible without the mechanism) ⇄ `P7-R1b` (and absent with it) — R1b is meaningless unless R1a has been seen red.

### 4a. Independent control notes (2026-09-28)

*controls-E (Opus), second agent per `../../../RELEASE_CYCLE.md` §4.2. Findings on GREEN/SUBJECT cells I did not rewrite. One line each: row — problem — suggested fix. Code facts are from reading, not from a run.*

- P7-R1a — the GREEN counts *a same-coin double-spend*; it must exclude identical re-broadcasts and txs that merely share a parent, and it must run two processes with two data dirs — two handles on one DB serialize and hide the race.
- P7-R1b — must reuse R1a's exact seed list, and must count the demoted device's receive-address issuance **attempts** (zero attempts ⇒ the BRC-155 clause is untested).

## 5. Blast radius

Nothing shipped changes. Code **read** to ground the design:

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/handlers.rs :: do_onchain_backup` (PushDrop `1-wallet-backup`/`1`, `marker`, Step 5c unspent check) | ✅ | Where a marker would ride; Step 5c's `unwrap_or_default()` read of WoC is P0-A11's subject |
| `rust-wallet/src/handlers.rs :: adopt_onchain_backup`, `fetch_onchain_backup` | ✅ | How a device finds the tip today (`max_by_key(height, 0 ⇒ i64::MAX)`) — plan H10's litter hazard applies to any active marker |
| `rust-wallet/src/reconcile.rs :: check_outpoint_spent`, `SpentStatus` | ✅ | `Unknown` for plain P2PKH — R1d |
| `rust-wallet/src/main.rs :: AppState.create_action_lock`, `utxo_selection_lock` | ✅ | The in-process serializers; a cross-device "read-only" check would sit before them |
| `rust-wallet/src/monitor/task_backup.rs :: run` | ✅ | Today's only periodic chain writer; a poll loop would be a sibling task in `monitor/` |
| `rust-wallet/src/permission_service/` | ✅ (dir) | R4-3: an on-device agent's spends pass the same engine |

## 6. Out of scope

Building anything (P7b). Option A (detect and heal — fails the no-conflict bar, decision 4), option D (payments spend the tip — ⛔ links every payment). Choosing a cloud provider (the owner's call after R2d). Agent spend policy itself (the future agents track; BRC-181). Multi-device **address** allocation beyond stating what read-only blocks (T1 owns the counter).

## 7. Rollback

Nothing ships; `git branch -D spike/b5-t3b-p7` if the spike exists. `R4_ANSWERS.md` is kept.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| The two-device mock never produced a race even with the mechanism off, so "zero double-spends" proved nothing | R1a must be seen red first |
| Lag was measured on a quiet afternoon and the design assumed 30 s; the real tail is 5 min | R2a p95/max, ≥ 30 samples; R1b uses the tail |
| The demoted device was assumed to notice via spent status — which reads `Unknown` for P2PKH | R1d measured with a positive control |
| The bar was set after seeing the numbers, so the chain "passed" | R0 precedes R2 |
| "Too slow ⇒ cloud" was concluded without looking at the relay-plus-chain middle ground decision 4 names | R2d |
| The active-device design keyed "authority" to a human's device and quietly made a server-side agent impossible | R3 |
| The spike became the implementation | §2 last item; P7b is a separate contract |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | R1a–R1c, R2c (mock) | 🍎 A real two-device sitting across **Windows + macOS** is the natural P7b row (the owner has both); not owed by this R&D contract |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1.5 h**: set the numeric bar (R0), 0.5 h · read `R4_ANSWERS.md` and choose P7b / middle ground / cloud, 1.0 h. (SCOPE §7.1's 2.0 h two-device sitting moves to **P7b**, if P7b happens) |
| Human-bound rows | R0 (owner decision); the R2d decision |
| Unknowns (K) — uncertainty, not difficulty | **Yes — T3b's second design-invalidating unknown, and the largest in the track: the conflict mechanism and whether the chain can carry sync at all.** Its answer can void P7b entirely and move sync off chain |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | **T3a-P0** E1, E3, E5, A11 | fee rate + slope, delta bytes, restore fetch/latency, the address-history contract (R2c, R1d) |
| needs | **T3a-P1** | the mock chain with scriptable lag (two-device runs) |
| needs | **P6** A10 | measured delta sizes (R2c). ⚠️ If P6 is scheduled after P7, R2c uses P0-E3 only and says so |
| gives | **P6** | if R4-1 needs a device dimension in the delta envelope (e.g. `seq` per device), P6's freeze must reserve it — raise at P6 kickoff |
| gives | **P8** | whether the BRC specifies multi-device behaviour at all this release |
| gives | **T1** | read-only must block receive-address issuance (BRC-155) — the high-water mark T1 settles is what a takeover reads |
| gives | future **agents track** | R4-3's answer (BRC-181 isolated account); nothing built |

## 12. Open questions for the owner

1. **R0 — your bar, in numbers,** before any measuring: switch time, added latency per spend, bytes/fee per sync, tolerated race window. Without them R4-2 cannot conclude.
2. **G5.5 flag:** P7 is R&D in a fixed-scope release. If R4-2 says the chain meets the bar, is the **P7b build** in beta.6, or does beta.6 ship P7's answers and the build moves to beta.7? **Recommendation:** decide at G5.5 with the answers in hand; plan beta.6 as answers-only.
3. No evidence that decision 4's direction is wrong; the fresh read (toolbox unchanged, BRC-181's isolated account) supports it.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed (rows marked ⏳: RED designed by the second agent, name recorded)
- [ ] `R4_ANSWERS.md` written; R4-1, R4-2, R4-3 each answered with row links; owner has read it
- [ ] `spike/b5-t3b-p7` deleted (or never created) — recorded
- [ ] `scripts/preflight.ps1` run — result + date recorded below (no product code expected; confirms the spike did not leak)
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` — n/a unless product code changed; state which
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] `../../../PRIOR_ART.md` rows added (push sources checked; BRC-181 isolated account)
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
