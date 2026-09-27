# TICKET — the user watches the update happen, twice, and neither moment needs to be visible

**Filed:** 2026-09-24, from the 👤 owner's live `0.3.0-beta.29 → 0.4.0-beta.4` self-update test —
the first real N−1→N run on the public feed. ⭐ **The update WORKED.** This is about what it looked
like, not whether it functioned.

👤 *"It kind of sucks having to watch it update and take forever… that doesn't happen with Chrome or
Brave."* Correct, and the reason is architectural rather than anything we lack permission to do.

## What the owner actually saw, in his words

1. **A dialog** offering the update. He **closed it without clicking** — and the update downloaded
   anyway (correct: staging is the silent path, the dialog is the notify path).
2. On the **next launch**, a splash: *"this could take a minute, don't power off"* — **1–2 minutes**
   of watching before the browser opened.

## Why each one happens

| | |
|---|---|
| the dialog | the notify path runs regardless of the user's update-mode setting |
| the splash | `MaybeApplyStagedUpdate` (`cef_browser_shell.cpp:5588`) runs at **startup**, ~500 lines before `CefInitialize` (`:6089`), and `hodos-update-helper.exe` shows `splash.h` while it swaps files. ⭐ It runs there because that is the only moment the browser's own files are **not locked** |

⚠️ Note: release builds **do** ship `-DHODOS_SILENT_AUTOUPDATE=ON` (`release.yml`). Silent staging is
real and working. What is not silent is the **apply**.

---

## Tier 1 — suppress the dialog when the user chose automatic ⭐ cheap

👤 Owner: *"if the user setting is automatic updates, then don't even have that dialogue pop up."*
Agreed — asking someone who already said "just do it" is noise.

Gate the notify dialog on the update-mode setting; leave it for the manual/notify modes.
**Effort: hours.** ⚠️ Keep a discoverable path (Settings → About → Check for updates) so a user who
*wants* to know still can.

## Tier 2 — apply on QUIT, not on next launch ⭐⭐ the one to actually do

The files are unlocked the moment the browser **exits**, exactly as much as before it starts. So run
the apply at shutdown: the user closes the browser, the helper swaps while they walk away, and the
next launch is simply the new version. ⇒ **the splash still exists, but nobody is looking at it.**

⭐ This is precisely what Sparkle does on macOS, which is why the Mac side does not have this
complaint. 📏 And macOS relay 23j measured that install-on-quit completes even when the host process
*aborts* — the pattern is robust.

**We already own the hard parts:** `hodos-update-helper.exe` does a transactional apply with rollback
(`UpdateApply.cpp` / `UpdateFs.cpp`), and "wait for the exact process to exit before acting" was
solved for the profile picker (`ae5beb6`, beta.26).

⚠️ **What needs care:** multiple profiles = multiple processes, so "the browser quit" must mean *all*
of them; and a failed apply now surfaces at the *next* launch rather than immediately, so the rollback
path carries more weight. **Effort: ~1 week**, most of it testing — every round needs two signed
builds and real installs.

## Tier 3 — Chrome's actual model ⛔ weeks, and probably not worth it

Chrome/Brave are invisible because of **two** things, not one:
1. an **external updater** (Omaha / BraveUpdate) that runs whether or not the browser does;
2. ⭐ **side-by-side versioned directories** — `Application\<version>\`, with a small launcher that
   picks the newest. A new version is a **new folder** written while the old one runs, so nothing is
   ever locked and there is no swap to wait for. Plus **delta updates**, so a typical update is a few
   MB rather than 130.

⛔ **The complication that is specific to us:** under CEF 150's bootstrap model `HodosBrowser.exe` is
CEF's `bootstrap.exe`, and it **verifies the code signature** of `HodosBrowser.dll` and
`chrome_elf.dll` — all three must be unsigned or share one signer, or it `LOG(FATAL)`s at launch
(`cef-native/CLAUDE.md`). A root launcher shim puts a **fourth** binary in that chain. The installer,
the uninstaller, and the anti-rollback high-water mark all change too.

⇒ **Possible. Nothing blocks it.** But its gain *over Tier 2* is mostly **download size**, not
experience. ⭐ **Recommendation: do 1 + 2, revisit 3 only if 130 MB downloads become a real
complaint.** Together 1+2 give Chrome's experience for roughly a tenth of Chrome's engineering.

## First step

Tier 1 standalone. Then Tier 2 with its own contract — ⛔ it is a money-adjacent path and its
acceptance test is *two signed builds on a real install*, never a rig alone.
