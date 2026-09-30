# G3 — brief for the independent negative-control round

**Why this round exists:** `../RELEASE_CYCLE.md` §4.2 — for money, schema and crypto phases, *the negative control is
designed by someone other than the assertion's author.* The contract authors (per-track agents, 2026-09-28) wrote
GREEN and SUBJECT for those rows and left the RED cell as `⏳ independent control — second agent (RELEASE_CYCLE §4.2)`.
You are the second agent. 🚨 The dominant defect class here, three releases running, is **"the test did not measure its
subject"** — a control validates only the mechanism its designer thought of. Your job is to think of the others.

## Read first
1. Root `CLAUDE.md` — the ⛔ NEGATIVE CONTROL section (the three farbling-harness failures), working rule 7 (trip-wires;
   *a deliberate fault-injection test must DECLARE its residue*), invariant 13.
2. `../RELEASE_CYCLE.md` §4.2. `HARNESS_DELTA.md` (§1.1 fail closed, §1.2 destructive tests need a scratch profile),
   `REGRESSION_ADDITIONS.md`, `../0.4.0-beta.3/HARNESS.md` (tiers T0–T4, the four-column evidence table) — read-only.
3. `G3_INTEGRATION.md` and your tracks' section of `G3_RESUME_NOTES.md`.
4. Each assigned contract **in full** before touching it — the pre-mortem (§8) is the author's list of failure stories;
   your controls should catch stories the author did *not* list, too.

## For every `⏳ independent control` cell in your assigned contracts
Replace the cell with a RED that says:
- **How it is made to fail** — the specific switch: revert/stub a named `file :: symbol`, a config toggle, a fault
  injection (DB error, indexer 5xx/timeout, crash point, malformed input), or a fixture that must be rejected.
  Re-verify any symbol you name against today's code (Read/Grep). Never a line number.
- **What red looks like, and why it is red for the right reason** — the observable, and what distinguishes it from the
  test failing for an unrelated reason (the harness, the wrong browser/process/DB, a flaky provider).
- **Residue**, if the control leaves state behind (a broken row, a spent coin, a DB in an impossible state): name it, and
  where it lives (scratch profile/regtest/mock chain). Rule 7: undeclared residue looks like a live defect later.
- End the cell with `— designed by <your agent label>, 2026-09-28`.

Keep each cell compact (it is a table cell — no line breaks; use `·` between parts).

## Also check, per row — and record findings, don't silently fix
- **Can the GREEN pass with the feature absent?** (asserting "no error", asserting a value changed rather than the value,
  reading a field our own code synthesised, a count that is zero both ways, an Err mapped to a default verdict).
- **Is the SUBJECT the production call** — same function, same config, same arguments, same process/browser/DB?
- **Two-sided pairs:** does each side really control the other?
If a GREEN or SUBJECT is wrong or cannot fail, **do not rewrite the author's cell.** Add a section at the end of §4:
`### 4a. Independent control notes (2026-09-28)` with one line per finding: `row id — problem — suggested fix`.
Also add a row there if you see a failure story no row catches (proposed id `<phase>-X<n>`, GREEN + RED + SUBJECT).

## Hard rules
- ⛔ Edit ONLY the `⏳ independent control` cells and add §4a, in your assigned contracts. Nothing else in them, no other files.
- ⛔ No code, no schema, no running tests against real wallets, no git writes, no GitHub issues. Read-only DB access
  only if a design question genuinely needs it (SQLite `mode=ro`), and never the production wallet unless essential — say so.
- Never touch `NOTES_parallel_work.md` or anything under `0.4.0-beta.3/`.
- If you find evidence of a **poisoning** defect (rule 7), stop designing, verify cheaply, and put it first in your report.
- Be economical with reading: the contracts cite what you need.

## Final message (tight)
1. Per contract: cells filled (count) and any left unfilled, with why.
2. §4a findings: rows whose GREEN can pass with the feature absent / wrong SUBJECT / missing rows — the most serious first.
3. Residue declared (which controls leave state, where).
4. Anything that should go to the owner.
