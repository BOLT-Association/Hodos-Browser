# 🪦 A dead certificate transaction builder still carries the old fail-open 'treat 404 as spent' check

**Found:** 2026-09-25, by the Wallet-Hardening pre-archive review (source doc named below, now in `archived-docs/Wallet-Hardening/`).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned — suggested: instruments & hygiene · **Filed by:** Claude, at the owner's request (cleanup before archiving)

> ⚠️ **Method note.** **Code reading, grep only** (review agent): `certificate_handlers.rs :: create_certificate_transaction` has no call site found. ⚠️ **Not verified with a compiler dead-code check.**

---

## What happens

Source: `archived-docs/Wallet-Hardening/RECONCILE_PHASE2_DESIGN.md` §2b. The function checks a coin via
WhatsOnChain `/outspend/`, treats a **404 as spent**, and marks the coin spent by `"unknown"` — the
fail-open pattern the WS1 reconcile work (`983655a`, `8903a2b`, `4e7c94f`) replaced everywhere else.

## Why it matters

Only if something revives it. A 404 is not proof of spending; reviving this would mark live coins spent
(root `CLAUDE.md` rule 7, trip-wire 2 — a verdict where an error is owed).

## Proposed fix

Confirm it is unreachable, then delete it or rewire it to the reconcile path. Working rule 3: reported
here, not deleted during cleanup.
