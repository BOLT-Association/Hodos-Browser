# 💥 A crash between broadcast and record would permanently lose the wallet's record of an ordinal

**Found:** 2026-09-25, by the Wallet-Hardening pre-archive review (source doc named below, now in `archived-docs/Wallet-Hardening/`).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned — suggested: 1Sat ordinals — settle before ordinals ship · **Filed by:** Claude, at the owner's request (cleanup before archiving)

> ⚠️ **Method note.** **Code reading** of the ordering (review agent); the source doc's own revisit trigger is now live. Not measured.

---

## What happens

Outputs are recorded **after** broadcast. That is safe only for outputs the wallet can **re-derive**
from its own keys. Source: `../track-4-onchain-backup-sync/research/FOLLOWUP_RECORD_BEFORE_BROADCAST_TOKENS.md`,
whose stated revisit trigger was *"when we add non-self-derivable token outputs"*.

## Why it matters

**beta.5's ordinals work is that trigger.** If the process dies in that window, the wallet loses its
record of the ordinal for good. The ordinals track docs have no crash-ordering or write-ahead
discussion today.

## Proposed fix

Define a per-token-type crash contract — a write-ahead record before broadcast, with a cleanup that
**never deletes on mere absence** — before ordinals ship. Likely a phase or item inside the ordinals
track. Related: the backup track's decision D7 (write intent before broadcast, for backups).
