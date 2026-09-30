# 🟠 The Centbee import creates a Hodos wallet with no PIN, storing the recovery phrase in plaintext

**Found:** 2026-09-30, reading code at `aaf44aa` on `0.4.0`, during the website accuracy audit
(`Marston Enterprises/Hodos/Website/AUDIT_2026-09-30.md`, owner checklist item 2).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned · **Filed by:** Claude (Opus 5.5), Marston-side session, at the owner's request

> ⚠️ **Method note.** Everything below is **code reading**. Nothing was run. Not verified: whether any
> path other than the Centbee import also creates a no-PIN wallet, and whether the UI offers a way to
> add a PIN later (the audit's reading says no API sets one).

---

## What happens

`rust-wallet/src/handlers.rs :: recover_external_wallet` (Centbee path, step 9) calls
`db.create_wallet_from_existing_mnemonic(&mnemonic_trimmed, None)`, so the PIN is always `None`.

`rust-wallet/src/database/wallet_repo.rs :: create_wallet_with_mnemonic` then takes the no-PIN branch:

```rust
} else {
    (phrase.clone(), None)
};
```

The `wallets.mnemonic` column receives the **plaintext** phrase, with `pin_salt = NULL`. A DPAPI/Keychain copy
is also written (`mnemonic_dpapi`), but the plaintext column sits beside it.

The 4-digit PIN the import asks for is the **Centbee** BIP39 passphrase. It unlocks the old wallet, not the new one.

## Why it matters

- Anyone who can read `wallet.db` (a backup, a synced folder, another tool running as the user) reads
  the recovery phrase directly. For every other wallet, `wallet.db` holds only PIN-encrypted text.
- `/wallet/reveal-mnemonic` has no PIN to check, so the phrase is revealed with no prompt.
- The website and docs describe "PIN + OS-level encryption at rest". That is false for these wallets.

## How exposed are we — answer this first

| If | Then |
|---|---|
| The user created the wallet in Hodos | Not affected; the creation flow sets a PIN *(reading; confirm the flow requires it)* |
| The user imported from Centbee | Affected for the lifetime of the wallet |
| How many users imported from Centbee | **Unknown.** No telemetry exists. The count decides whether a migration prompt is urgent |

## What already protects us, and how that shapes the fix

The DPAPI/Keychain copy exists, and the PIN encryption code (`crypto::pin::encrypt_mnemonic`) already
exists. So the fix is plumbing: collect a PIN and call the existing function.

## Proposed fix

**The floor:** the Centbee import asks for a **Hodos PIN** (a separate step from the Centbee PIN) and
passes it to `create_wallet_from_existing_mnemonic`.

**The system:** existing wallets with `pin_salt IS NULL` are prompted at unlock to set a PIN. Setting it
encrypts `wallets.mnemonic` in place. Until it is set, `/wallet/reveal-mnemonic` refuses.

**Deliberately out of scope:** changing the PIN scheme itself; the Centbee sweep logic.

## Test and negative control

⛔ Per `../../0.4.0-beta.3/HARNESS.md`: **a fix is not done until the check has been *seen* to fail.**

| | |
|---|---|
| **GREEN** | After a Centbee import on a scratch profile, `wallets.mnemonic` is not a valid BIP39 phrase (it is ciphertext) and `pin_salt` is non-null. Reveal requires the Hodos PIN |
| **RED** | Revert the import to pass `None` ⇒ `SELECT mnemonic FROM wallets` returns 12 English words |
| **SUBJECT** | The scratch profile's `wallet.db`, read with `sqlite3` after the import completes |
| **Tier** | T1 (unit/integration on a scratch DB) |

**Standing invariant?** Yes: *"no code path writes a plaintext phrase to `wallets.mnemonic`."* Propose a
row for `REGRESSION_ADDITIONS.md`.

## Links

- `../../0.4.0-beta.7/tickets/TICKET_auto_unlock_accepts_another_wallets_mnemonic.md`: same storage area
- Audit: `Marston Enterprises/Hodos/Website/AUDIT_2026-09-30.md` (checklist 2; site claim on "PIN + OS-level encryption")
