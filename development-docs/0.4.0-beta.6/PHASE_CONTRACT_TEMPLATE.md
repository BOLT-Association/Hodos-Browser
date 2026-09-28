# B5-T<t>-P<k> — <plain-language title> · PHASE CONTRACT

> Copied 2026-09-28 from `../0.4.0-beta.3/PHASE_CONTRACT_TEMPLATE.md` (read-only; ⛔ never edit that folder).
> Changes from the original: paths re-pointed to the inherited harness + this release's deltas, and the
> G3 additions from `SESSION_PROMPT_G3_phase_contracts.md` ("What G3 produces") appended as §8–§13.

**Track:** B5-T<t> <track name> · **Tickets:** `../../tickets/<TICKET_*.md>` · **Status:** ⬜ NOT STARTED / 🚧 IN PROGRESS / ✅ SIGNED OFF
**Opened:** <date> · **Author:** <agent + model> · **Platforms:** Windows / macOS / both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** <numbers from `../../README.md` "✅ Decisions as made" — carried, not reopened>

---

## 1. Goal

<One sentence, user-observable. What is true for a user afterwards that is not true now.>

## 2. Done means

<Results, not activities. Each line measurable. "Deleted the writes" is an activity;
"`{app}` contains no volatile files after a 2-hour session" is a result.>

- [ ]
- [ ]

## 3. Invariants preserved

<Named, from `../../../0.4.0-beta.3/REGRESSION_SET.md` and `../../REGRESSION_ADDITIONS.md`. Say which apply
and why. Generic reassurance is banned.>

| ID | Invariant | Why this phase could break it |
|---|---|---|

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. A green result is reported with its red half or not at all.
⛔ Money, schema and crypto rows: the RED (negative control) is **designed by someone other than the
assertion's author** — a second agent (`../../../RELEASE_CYCLE.md` §4.2). Record who designed it.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P<k>-A1` |  |  |  | T0–T4 | ⬜ |

**Two-sided rows:** where the requirement is "A must happen AND B must not", write both and make each
the other's control. Note the pairing here.

## 5. Blast radius

<What this touches that it is not about. Cite code as `file :: symbol`, **re-verified on the contract's
date** (CLAUDE.md kickoff step 2) — never line numbers. Be specific enough that a reviewer can check you looked.>

| Cited code (`file :: symbol`) | Verified <date> | Note |
|---|---|---|

## 6. Out of scope

<Explicit, so scope creep is visible in the diff. Include the things you were tempted by.>

## 7. Rollback

<How to undo in one commit. If you cannot say it in two lines, the phase is too big — split it.>

## 8. Pre-mortem (adversarial review — before)

<Assume this phase shipped and failed. Why? List the failure stories, and which evidence row catches each.
A story no row catches is a missing row.>

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | | |
| macOS | | <or "Windows-only, because …" — invariant 9> |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | |
| Human-bound rows | |
| Unknowns (K) — uncertainty, not difficulty | |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|

## 12. Open questions for the owner

<Only what the decisions above do not already settle. Evidence that a G2 decision is wrong goes here,
and stops the phase (working rule 1).>

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
