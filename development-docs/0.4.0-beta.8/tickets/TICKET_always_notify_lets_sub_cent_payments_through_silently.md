# 🟡 "Always notify" lets payments under one cent through with no prompt

**Found:** 2026-09-30, reading code at `aaf44aa` on `0.4.0`, during the website accuracy audit
(`Marston Enterprises/Hodos/Website/AUDIT_2026-09-30.md`, owner checklist item 13).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned · **Filed by:** Claude (Opus 5.5), Marston-side session, at the owner's request

> ⚠️ **Method note.** **Code reading** only; not run. Not verified: the audit's further reading that
> "Always notify" zeros three of four limits (the rate limit may remain) — confirm when fixing.

---

## What happens

1. `frontend/src/components/DomainPermissionForm.tsx` defines "Always notify" as
   `perTxUsd === '0' && perSessionUsd === '0'`. The toggle sets both limits to 0.
2. `cef-native/include/core/PaymentCost.h` converts satoshis to cents with a truncating cast:
   `c.cents = static_cast<int64_t>((satoshis / 1e8) * bsvPriceUsd * 100.0)`. Any payment worth less than
   one US cent becomes **0 cents**.
3. `rust-wallet/crates/hodos_permission_engine/src/matrix_c.rs` prompts only on strict-greater-than:
   `if ctx.requested_cents > ctx.per_tx_limit_cents`, and likewise for the session cap. With a limit of 0 and a
   request of 0, `0 > 0` is false, so the payment proceeds silently.

## Why it matters

The user asked to be asked every time. Micropayments (the case Hodos is built for) are usually
sub-cent, so this is the common case, not an edge case. The setting's name promises something the engine does not do.

## How exposed are we — answer this first

| If | Then |
|---|---|
| "Always notify" is off | No change; sub-cent payments inside normal caps are meant to be silent |
| "Always notify" is on | Every sub-cent payment from that site is silent. The number of users with it on is unknown |

## What already protects us, and how that shapes the fix

The strict-greater-than boundary is deliberate and tested (`PaymentExactlyAtPerTxCapIsSilent`), so
"payment equal to the cap is allowed" must keep holding for non-zero caps. The fix is scoped to the
zero-limit case.

## Proposed fix

Treat "Always notify" as an explicit mode rather than inferring it from two zeros: a per-domain flag the
engine checks first ⇒ always `prompt`. Or, the floor: in `matrix_c.rs`, a per-tx limit of 0 means
"prompt on any fund-moving call", regardless of the cents value.

**Deliberately out of scope:** rounding sub-cent prices up for display; changing the strict boundary for non-zero caps.

## Test and negative control

| | |
|---|---|
| **GREEN** | With "Always notify" on, a 1-satoshi `createAction` from an approved scratch site prompts |
| **RED** | Revert ⇒ the same call completes with no modal (engine decision logged as silent/proceed) |
| **SUBJECT** | Engine unit test on `decide()` with `per_tx_limit_cents = 0`, `requested_cents = 0`, plus one T2 run through the shell |
| **Tier** | T0 (unit) + T2 |

**Standing invariant?** Yes: *"Always notify prompts on every fund-moving call."*

## Links

- Audit rows D-A108, D-B189 in `Marston Enterprises/Hodos/Website/AUDIT_2026-09-30.md`
