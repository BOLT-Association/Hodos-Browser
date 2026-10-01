# 🟡 Privacy Shield's cookie-blocking toggle snaps back ON: `cookie_check_site_allowed` reads the domain from the wrong argument

**Found:** 2026-10-01, beta.6 macOS smoke (relay M-01c), 👤 owner on whatsonchain.com; cause confirmed by reading the code on Windows
**Status:** ⬜ UNASSIGNED (beta.8 intake; not a beta.6 blocker — display only) · **Track:** unassigned · **Filed by:** Claude (Opus 5.5), from the Mac agent's measurement

> ⚠️ **Method note.** The symptom and the empty-domain answer are **measured on macOS** (M-01c §2). The cause is **code reading** of shared C++, so Windows has it too; not reproduced on Windows.

---

## What happens

`cef-native/src/handlers/simple_handler.cpp`, the `cookie_check_site_allowed` arm ("Phase 8c batch 5 — MIGRATED"):

```cpp
std::string domain = (csa_args->GetSize() > 1) ? csa_args->GetString(2).ToString() : "";
```

The bridge sends `[0]=requestId, [1]=domain`, so `[2]` is out of range ⇒ `""` ⇒ `allowed:false` always. Turning cookie blocking OFF **is saved** (`cookie_blocks.db :: allowed_third_party` gets the row) and applied, but the panel re-reads "not allowed" and shows the switch ON again. Siblings (`cookie_allow_third_party`, `cookie_remove_third_party_allow`, `adblock_scriptlet_toggle`) read `[1]`.

## Fix

`GetString(1)`. Regression row: allow → check ⇒ `allowed:true`; negative control: `[2]` ⇒ red. Sweep the other batch-5 migrated arms for the same slip. C++ shared file ⇒ Mac relay note.

## Also seen (M-01c §4, not investigated)

- `fingerprint_settings.json` gained an entry for an **empty domain** (`{"": {"enabled": false}}`) from a toggle pressed while the shield's domain was briefly empty.
- ~419 `cookieResetBlockedCount` / `adblockResetBlockedCount` bridge calls in ~5 min with the panel open 14 times — looks like a polling loop resetting counters.
