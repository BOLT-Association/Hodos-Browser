# 🟡 `live_reservation_placeholders` fails OPEN on a poisoned lock, contrary to its own comment

**Found:** 2026-09-30, beta.6 P5 adversarial review; confirmed by reading the code
**Status:** ⬜ UNASSIGNED (beta.8 intake) · **Track:** unassigned · **Filed by:** Claude (Opus 5.5)

> ⚠️ **Method note.** Code reading only. Not measured: a poisoned `PENDING_TRANSACTIONS` mutex needs a panic while it is held, which has not been observed.

---

## What happens

`rust-wallet/src/handlers.rs :: live_reservation_placeholders`, on `Err` (poisoned lock):

```rust
// Poisoned lock: report "everything is live" so the sweeper releases nothing.
log::error!("... treating all reservations as live", e);
std::collections::HashSet::new()
```

An **empty** set means "nothing is live", the opposite of the comment. `TaskSweepReservations` skips only the placeholders in this set, so after a poisoning it would consider every aged `pending-` reservation abandoned (still gated by its on-chain unspent check), including ones an in-flight action still owns.

## Fix direction

Return a sentinel the sweeper must honour (e.g. `Option<HashSet>` with `None` ⇒ release nothing), or recover the guard with `into_inner()` as `abort_action_with_chain` does. Test: poison the mutex in a test, assert the sweeper releases nothing.
