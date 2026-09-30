# 🟡 `abortAction` is not bound to the site that created the action

**Found:** 2026-09-30, beta.6 advisory triage (`../../0.4.0-beta.6/ADVISORY_TRIAGE.md`, G3, TSA-042).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned · **Filed by:** Claude (Opus 5.5)

> **Code reading.** `rust-wallet/src/handlers.rs :: abort_action` looks the action up by reference
> number only. The `transactions` table has no column recording which site created an action.

## What happens

Any approved site that learns (or guesses) an action's reference can abort it, including another
site's pending action.

## Why it matters

Denial of service against another dApp's in-flight payment. Moderate: references are random UUIDs.

## Why not in beta.6

Binding needs the creating origin stored on the transaction row: a **schema change** (invariant 2).
👤 Owner, 2026-09-30: beta.6 fixes the abort re-lock, the release of inputs and which statuses may be
aborted; origin binding is deferred here.

## Fix

Store the originator on creation (a child table or column, mirroring `cert_field_permissions`), and
refuse an abort whose `X-Requesting-Domain` differs. Internal (wallet UI) aborts stay allowed.

## Test and negative control

GREEN: site B aborting site A's reference ⇒ refused, A's action untouched. RED: without the check ⇒
B's abort succeeds.
