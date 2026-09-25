# 🔁 A fast relaunch can attach to a wallet that is shutting down, or lose the port race and start with no wallet

**Found:** 2026-09-25, by the Wallet-Hardening pre-archive review (source doc named below, now in `archived-docs/Wallet-Hardening/`).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned — suggested: browser shell / wallet supervision · **Filed by:** Claude, at the owner's request (cleanup before archiving)

> ⚠️ **Method note.** **Code reading** (review agent): `/health` takes no state and cannot report 'shutting down'; the wallet binds with a plain `.bind(...)?` and exits on failure; dead `WalletService` daemon code (`startDaemon`, `cleanupDaemonProcess`) remains; the updater hardcodes ports `31301/31302` in `update-helper/transaction.cpp`. Not measured.

---

## What happens

The non-backup half of `../track-3-backup-sync/research/FIX_B_CRASH_SAFETY_SHUTDOWN_PLAN.md`:
(1) `/health` reports healthy while the wallet is exiting, so a quick relaunch can adopt a wallet that
is about to disappear; (2) a new wallet that loses the port race exits instead of retrying; (3) the
updater hardcodes release ports — ⚠️ root `CLAUDE.md`: *never hardcode ports; use the `PortConfig.h` helpers*.

## Why it matters

Availability — the user relaunches and gets "no wallet". Partly mitigated by the beta.3 Phase 8d
wallet supervisor (bounded relaunch); check against that before designing a fix.

## Proposed fix

A not-ready health state once shutdown starts; a bounded bind retry; the dead daemon code removed
(reported first — working rule 3); the updater's ports routed through the port helpers.
**The Rust half** of the source plan (write intent before broadcast, for backups) is owned by the
backup track, decision D7.
