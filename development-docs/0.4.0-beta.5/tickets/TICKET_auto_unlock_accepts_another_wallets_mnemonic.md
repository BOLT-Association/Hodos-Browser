# 🔑 Auto-unlock accepts any valid recovery phrase — it never checks that the phrase belongs to THIS wallet

**Found:** 2026-09-25, by the Wallet-Hardening pre-archive review (source doc named below, now in `archived-docs/Wallet-Hardening/`).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned — suggested: money-path safety · **Filed by:** Claude, at the owner's request (cleanup before archiving)

> ⚠️ **Method note.** **Code reading**, confirmed twice (review agent, then a second read on 2026-09-25): `database/connection.rs :: try_dpapi_unlock` calls `validated_mnemonic`, which checks BIP39 **shape only** (`crypto::mnemonic_guard::is_valid_mnemonic`). Not verified: whether a later layer catches a mismatch (none found).

---

## What happens

On startup the wallet decrypts the stored recovery phrase from the OS credential store and caches it
(`database/connection.rs :: try_dpapi_unlock`). The only check is that the value **is a valid BIP39
phrase** (`validated_mnemonic`, added in `c078423`). A valid phrase from **a different wallet** passes,
and the wallet then signs with keys that are not its own.

## Why it matters

This is the shape of the July 2026 macOS Keychain incident
(`archived-docs/Wallet-Hardening/MAC_KEYCHAIN_CROSSCONTAMINATION_FIX.md`): a dev and a prod wallet shared
a Keychain entry and the wallet **signed with dev keys for three weeks**. The incident's fix (`deff765`,
separate dev/prod service names) removed that one cause. Its **lesson 5 — verify at startup that the
cached phrase derives the DB's identity key — was never done.** A mismatch means wrong signatures, and
addresses derived from the wrong key: funds sent where this wallet cannot see or spend them.

## Proposed fix

Before caching, derive the master public key from the phrase and compare it with the stored identity
key (`users.identity_key`). On mismatch: log the fact (never the value), return `Ok(false)` so the
wallet stays **locked** and the PIN screen repairs the entry — the self-healing path the malformed-value
case already uses. ⚠️ Touches key handling — invariant 3: ask before changing.

## Test and negative control

| | |
|---|---|
| **GREEN** | A valid phrase from a *different* wallet in the credential store → wallet stays locked, PIN screen shown, log line fires |
| **RED** | Remove the comparison → the same setup auto-unlocks (the defect) |
| **SUBJECT** | The identity key the wallet reports after unlock equals the DB's |

**Related:** `../../0.4.0-beta.3/TICKET_credential_store_value_bricks_wallet.md` covers **malformed** values only.
