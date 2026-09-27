# 🧺 Five efficiency and ecosystem-alignment items from the Final MVP sprint were planned and never scheduled

**Found:** 2026-09-25, while reviewing `Final-MVP-Sprint/` before archiving it (now
`archived-docs/Final-MVP-Sprint/`).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned — suggested: instruments & hygiene, or split to the tracks named below · **Filed by:** Claude, at the owner's request (cleanup before archiving)

> ⚠️ **Method note.** **Doc reading only.** The checkbox state was read from the two plans below; I did
> not re-verify against code that the unchecked items are still undone. The backup-related items in
> the same plans are captured in `../track-3-backup-sync/BACKUP_HISTORY_OVERVIEW.md` and are
> **not** repeated here.

---

## What is open

Sources: `archived-docs/Final-MVP-Sprint/wallet-efficiency-and-bsv-alignment.md` (the parent
checklist) and `archived-docs/Final-MVP-Sprint/bsv-ecosystem-alignment-plan.md`. Everything else in
those plans is done (constant-time comparisons, lazy consolidation, auto dust consolidation, BEEF
compaction — commits `afd4d0a`, `1bce17f`) or deliberately skipped with reasons recorded.

| # | Item | What it is | Effort (plan's estimate) | Fits |
|---|---|---|---|---|
| 1 | **`1b` Adaptive service timeouts** | Per-provider timeouts that learn from response times, instead of fixed ones | 2–3 days | money-path safety (resilience) |
| 2 | **`1d` WhatsOnChain BUMP endpoint** | Fetch merkle proofs in BUMP form directly, removing a format conversion | 1–2 days | hygiene |
| 3 | **Strategy 5.2 — same-counterparty consolidation** | One-click "combine 23 payments from one sender" | ~1 day | wallet UI |
| 4 | **Strategies 5.4 / 5.5 — basket and cross-counterparty consolidation** | Power-user consolidation, behind a privacy-warning modal | not estimated | wallet UI · ⚠️ privacy |
| 5 | **`3b` Fuzz testing for parsers** (BEEF, transaction, script) | Deferred with the trigger **"post-launch"** — ⭐ **that trigger has now fired** (`v0.4.0-beta.4` is public) | 1–2 days setup | instruments |

`1a` (broadcast-failure classification) was deferred because `arc_status.rs` already covers it — not
carried here.

## Why it matters

None of these is a defect today. Item 5 is the one with a live argument: the wallet parses untrusted
BEEF and transactions from the network, and the plan's own condition for doing it has been met.

## Proposed handling

Triage at `G2`: item 5 into the instruments work, item 1 into money-path safety, items 3–4 into
whichever track owns wallet UI, item 2 as a background item — or defer each **with a re-check
condition**.
