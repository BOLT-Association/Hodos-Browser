# SESSION PROMPT — G3: write the phase contracts for `v0.4.0-beta.6`

**Written:** 2026-09-27, at the close of G2. Paste the block below into a **fresh session**.

---

```
We're starting gate G3 for the release planned in development-docs/0.4.0-beta.6/
(it ships as v0.4.0-beta.6; it was called "beta.5" until 2026-09-27 — phase ids
keep the B5- prefix). G0, G1 and G2 are signed off. G3 = one phase contract per
phase, for every track.

Read, in this order:
1. development-docs/0.4.0-beta.6/README.md — "▶️ RESUME HERE", then
   "✅ Decisions as made" (all of it), then "🧭 G2 — Tracks".
2. development-docs/RELEASE_CYCLE.md §2 (gates), §3.7 (feasibility inputs),
   §4.1–§4.2 (contract + where each check belongs), §4.6 (context boundaries).
3. Root CLAUDE.md working rules 1, 2, 3, 5, 7, 8, the phase kickoff workflow, and
   the invariants (2 schema, 3 crypto, 9 macOS parity, 13 test triage).
4. Each track's SCOPE.md — its "✅ 0. G2 decisions applied" block FIRST (it wins
   over the research below it), then "Candidate phases", "Integration check",
   "Feasibility inputs".
5. development-docs/0.4.0-beta.6/HARNESS_DELTA.md and REGRESSION_ADDITIONS.md,
   and the inherited harness ../0.4.0-beta.3/HARNESS.md (read-only).

Then do the following, and stop at the exit condition.
```

---

## What G3 produces

**One `PHASE_CONTRACT.md` per phase**, at `track-<n>-<slug>/phase-P<k>-<slug>/PHASE_CONTRACT.md`.
Start from `../0.4.0-beta.3/PHASE_CONTRACT_TEMPLATE.md` (read-only — ⛔ never edit anything under
`0.4.0-beta.3/`; copy it into this folder as `PHASE_CONTRACT_TEMPLATE.md` first, as its own commit). Each
contract must carry, in addition to the template's sections:

| Required | Why |
|---|---|
| **Every cited file re-verified today** as `file :: symbol` (not line numbers) | CLAUDE.md kickoff step 2 — the scope docs are two days old and code moves |
| **A negative control on every acceptance assertion**, and the **subject** it measures | Hard rule. ⛔ For money, schema and crypto phases, the control is **designed by someone other than the assertion's author** (a second agent) — RELEASE_CYCLE §4.2 |
| **Pre-mortem** (adversarial review *before*): assume it failed — why? | §4.2 |
| **Windows + macOS rows** (or "Windows-only, because …") | Invariant 9; macOS has not started yet and will pick these up |
| **Owner-hours** and which rows are human-bound; **unknowns (K)** | Feeds G5.5 feasibility |
| **Cross-track edges** it gives or needs | Feeds G5 serialization |
| **GitHub issue: *(opened at G6)*** placeholder | §4.1a — ⛔ do **not** open issues in G3 |
| Sign-off line: `- [ ] Context: memory saved · boundary decided (continue / fresh session)` | §4.6 |

## The phases — about 40

Decisions are already made; the contracts **carry** them, they do not reopen them. Where a contract
finds evidence that a decision is wrong, **stop and ask the owner** (working rule 1) — do not route around it.

| Track | Phases | Decisions that shape them |
|---|---|---|
| **T0 Engine** | Build 1 (P1 → P2 → P5) is governed by `track-0-engine/SECURITY_RELEASE_PLAN.md` — ships **alone** as `v0.4.0-beta.5`. G3 writes contracts for **Build 2**: P3 ad-block pull (**b3** shape), P4 `"Hodos"` brand, P5-lite verify | 1 |
| **T1 Money path** | P1 reqwest 0.12 · P2 a failure says it failed (refused `signAction` → retries then error) · P3 reservations **+ the Go-style money-index table** · P4 ingest truth (real scripts) · P5 classifier + Unknown display · P6 late payments to generated addresses (+ BIP-32 hits swept to BRC-42) · P7 records true over time (narrow reorg) | 2, 2a, 7, T1 Q2/Q5 |
| **T2 Ordinals** | **P2 token-spend permission first** (per action, BRC-165) · P1 classify & file (`1sat` / `bsv21`) · P3 receive & transfer an ordinal · P4 hold & show incl. BSV-21 balances. ⛔ No BSV-21 transfer | 8 |
| **T3a Backup you can trust** | P0 prove the DB correct + measure · P1 round-trip harness · P2 trustworthy backup (restore **classifies before rebuilding** the money index) · P3 re-fetchable bytes off chain · P4 freeze format | 3, 5, 7, SCOPE §6a |
| **T3b Sync & portability** | P5 BRC-38/39 export/import + tiered phrase import · P6 deltas · P7 two devices — ⚠️ **PROVISIONAL R&D**, opens with R4-1/2/3 · P8 publish the BRC | 4, 5, 6 |
| **T4 402** | P1 never lose track of whether it paid (Step 0 already run — zero hits) · P2 the 431 path. ⛔ No x402 adapter | 9, 10 |
| **T5 Identity & privacy** | P1 servers prove who they are (incl. `/.well-known/auth` **server-role fix to the protocol**) · P2 dApp-reachable surface · P3 derived keys parts 1+2 (detector as a **child table** — SCOPE T3 §6a) + optional counterparty · P4 one OS account, one wallet · P5 usage ping (opt-out, three conditions) | 11, 12, 13, T5 Q2 |
| **T6 Browser shell** | Group 1: P1 silent updates · P2 Exit quits the app + session restore · P3 small defects. Group 2: P4 apply-on-quit · P5 Tools tab claim · P6 Chrome bookmarks/history. Group 3: P7 brand prompts · P9 window globals · P10 pin/mute. ⛔ P8 dropped; split view + password import deferred | 14, T6 Q1/Q6 |

## How to run it — ✅ decided by the owner 2026-09-28

1. **One agent per track, in parallel** (8 agents: T0 build 2, T1, T2, T3a, T3b, T4, T5, T6), each writing
   its track's contracts — then **one integration pass** by the orchestrating session over the
   cross-track edges below. 👤 Owner: *"Yes, run one agent per track in parallel."* This is the explicit
   opt-in for the multi-agent run.
2. **Model split — hardest tracks on Fable:** **Fable** for **T1, T3a, T3b, T5** (money, backup, sync,
   identity — schema and signing, invariants 2 and 3); **Opus** for **T0 build 2, T2, T4, T6**. 👤 Owner:
   *"Shouldn't Fable be the hardest stuff?"* — the earlier Opus-on-hard proposal had no evidence behind
   it (G2's only data point: "comparable on a sample of two"). ⭐ Record in `AAR_NOTES.md` how each
   model's contracts fared at the owner's later review — that is the comparison.
3. The owner expects to **review all contracts again** before G6, so optimise for correct and checkable,
   not for polish.

## Cross-track edges the integration pass must close

- **T1 → T3a:** money index is derived ⇒ omitted from backups and rebuilt; **old backups carry `change=0`
  on received payments** ⇒ restore classifies first (T3 SCOPE §6a). T1 lands before T3 format work.
- **T1-P6 ↔ T3a restore path** (`reconcile_backup_tx`, the address strip): T1 writes, T3 reviews (T1 Q6).
- **T1-P4 → T2-P1:** the classifier needs **real** scripts (≥ ~256 B for the BSV-21 JSON body).
- **T5-P3 → T3:** `derived_key_cache` is not backed up; the detector restarts empty after restore.
- **T6-P5 claim endpoint:** T6 builds it, **T1's money-path harness rows** apply.
- **T0 Build 1 before** any track gathers browser-level evidence (gathered once, on the new engine).
- **T4-P1** reuses `check_tx_exists_on_chain`; `Err` must never become a verdict (rule 7, trip-wire 2).

## Rules for this session

- ⛔ **No code, no schema, no GitHub issues.** G3 is documents. Commit per track; push after
  `git fetch && git rebase origin/0.4.0` (CLAUDE.md branch rules). Nothing under `0.4.0-beta.3/`.
- ⛔ `NOTES_parallel_work.md` and any other untracked file you did not write: **never commit**. Another
  session may be writing in this folder — stage only your own paths.
- 👤 **Talking to the owner:** a direct question gets the direct answer first (yes / no / which), then a few
  short lines; offer detail, don't lead with it. Every id in chat carries its subject (working rule 8).
  When the owner explains how a protocol works, say which parts match the spec — never call it "his design".
- ⭐ Default fix direction is **follow the protocol / reference SDK**; name any deviation as a deviation.

## Exit condition — stop here

Every phase above has a contract; the integration pass has closed every edge in the list; the README's
"▶️ RESUME HERE" shows **G3 certified** (agent-certified gate — RELEASE_CYCLE §2) with a one-line
state per track; a relay round in `MAC_RELAY_BETA5.md` names the new contracts macOS will execute.
Then **stop and tell the owner** what G4 (logistics — nothing we depend on expires inside the cycle) needs.
