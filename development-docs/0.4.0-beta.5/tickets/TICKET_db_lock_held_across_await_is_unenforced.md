# 🔒 'Never hold the database lock across an await' is followed by convention only

**Found:** 2026-09-25, by the Wallet-Hardening pre-archive review (source doc named below, now in `archived-docs/Wallet-Hardening/`).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned — suggested: instruments & hygiene · **Filed by:** Claude, at the owner's request (cleanup before archiving)

> ⚠️ **Method note.** **Code reading** (review agent): no `clippy::await_holding_lock` lint configured anywhere. Hits not counted.

---

## What happens

Source: `archived-docs/Wallet-Hardening/README.md`, H-register item H-3A. Holding the DB mutex across
an `.await` can stall or deadlock the wallet; nothing enforces the rule.

## Proposed fix

Enable `clippy::await_holding_lock` at **warn**, count and fix the hits, then raise it to **deny**.

**Same register, hygiene only (noted here, not separately ticketed):** H-1 — the schema version in
`rust-wallet/CLAUDE.md` (V23) and `rust-wallet/src/database/CLAUDE.md` (V24) has drifted from the code
(V25); H-2 — startup repair still inline in `main.rs`; H-4 — `handlers.rs` is ~21,600 lines, unsplit.
