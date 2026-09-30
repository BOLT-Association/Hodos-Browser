# 🟢 Wallet secrets held in memory are never wiped (no zeroization)

**Found:** 2026-09-30, during the website accuracy audit (`Marston Enterprises/Hodos/Website/AUDIT_2026-09-30.md`, checklist item 20).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned · **Filed by:** Claude (Opus 5.5), Marston-side session, at the owner's request

> ⚠️ **Method note.** **Measured:** no `zeroize` / `Zeroize` in `rust-wallet/Cargo.toml` or
> `rust-wallet/src` at `aaf44aa`. **Code reading:** `database/connection.rs` keeps `cached_mnemonic` as a
> `String` for the process lifetime. Not verified: every place derived private keys are held, and whether
> any dependency (e.g. `secp256k1`) wipes its own key types on drop.

---

## What happens

When the wallet unlocks, the recovery phrase is cached in memory (`cached_mnemonic`) and private keys are
derived from it. Rust frees that memory normally when it drops, but it does not overwrite the bytes, so
copies can remain in freed memory, swap, or a crash dump until something else reuses the space.

## Why it matters

In plain terms: after the wallet is done with a secret, the secret can still be read out of RAM, a page
file or a crash report. This is **defence in depth**. An attacker who can read another process's memory
usually has other routes too. It matters most for crash dumps sent to third parties.

## How exposed are we — answer this first

| If | Then |
|---|---|
| Crash dumps are collected and uploaded | Worth fixing sooner. Whether Hodos uploads wallet-process dumps is **unknown** |
| No dumps leave the machine | Low. Same-machine memory access implies a compromised account |

## Proposed fix

Adopt the `zeroize` crate: wrap `cached_mnemonic` and derived private-key buffers in `Zeroizing<…>`,
clear the cache on lock, and check that key types from dependencies zeroize on drop. The floor is
the cached phrase alone.

**Deliberately out of scope:** OS-level memory locking (`mlock` / `VirtualLock`), and hardware-backed keys.

## Test and negative control

| | |
|---|---|
| **GREEN** | Unit test: after lock or drop, the buffer that held a known test phrase contains no copy of it |
| **RED** | Without `Zeroizing`, the same test finds the phrase bytes |
| **SUBJECT** | Test-only phrase in a unit test; never a real wallet |
| **Tier** | T0 |

## Links

- Audit: `Marston Enterprises/Hodos/Website/AUDIT_2026-09-30.md` (checklist 20)
