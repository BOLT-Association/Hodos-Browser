# 🏷️ Two sources for the next address index can drift apart and reuse an address

**Found:** 2026-09-25, by the Wallet-Hardening pre-archive review (source doc named below, now in `archived-docs/Wallet-Hardening/`).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned — suggested: identity & privacy · **Filed by:** Claude, at the owner's request (cleanup before archiving)

> ⚠️ **Method note.** **Code reading** (review agent): receive addresses use `wallet.current_index + 1` (`handlers.rs`), change outputs use `MAX(addresses.index) + 1` (`get_max_index`), with ad-hoc self-heal code between them. Not measured.

---

## What happens

Source: `archived-docs/Wallet-Hardening/FOLLOWUP_NEXT_INDEX_UNIFICATION.md` — planned, never built. When
the two counters disagree, a newly issued address can repeat one already used.

## Why it matters

Address reuse is a **privacy** leak (payments become linkable), not a loss of funds.

## Proposed fix

One source of truth — the doc proposes `MAX(addresses.index >= 0)` — and a decision on the fate of
`current_index`. ⚠️ Any schema change needs owner sign-off (invariant 2). Includes the doc's
address-display UX check.
