# B5-T6-P2 — menu Exit quits the app, and quitting brings back every window that was open · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (G3, 2026-09-28). ⛔ Documents only — no code has been written for this phase.

**Track:** B5-T6 Browser shell · **Group:** 1 (keep even if the release runs long) · **Tickets:** `../../tickets/TICKET_menu_exit_closes_primary_not_the_clicked_window.md` + `../../tickets/TICKET_multiwindow_session_restore_loses_all_but_last_window.md` (merged, as SCOPE §4 #2/#3) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent for T6 · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **T6 Q1** — menu **Exit quits the whole app** (Chrome, Firefox, Vivaldi, our macOS build); a window's **X closes one window**. **T6 Q2** — restore brings back every window **open at the moment of quit**; a window closed earlier stays closed (Chrome/Firefox); **plus** a Windows logoff/restart save; **plus** periodic crash-safety save **as a separate item**. **14** (split view follows this phase). Carried, not reopened.

---

## 1. Goal

On Windows, menu → Exit closes every Hodos window at once (as on macOS), and whenever Hodos quits — by Exit, by closing its last window, or by Windows logging off or restarting — the next launch (with "restore session" on) brings back **every window that was open at that moment**, and a crash loses at most a few seconds of tab changes.

## 2. Done means

- [ ] **Item 1 — Exit quits.** Two windows, Exit from **either** window ⇒ both close; process exits; next launch restores **both** windows with their own tabs and geometry
- [ ] **Item 2 — Session end.** Two windows open, Windows sign-out / restart ⇒ next launch restores both
- [ ] **Item 3 — Crash safety (separate item, own commit).** Two windows, tabs changed, process killed ⇒ next launch restores the state as of ≤ the save interval before the kill
- [ ] A window closed with its **X** while another stays open is **not** restored after a later quit (the Chrome/Firefox rule, unchanged)
- [ ] One window, restore on: behaviour byte-for-byte what it is today (session v2 format unchanged)
- [ ] Restore **off** (the shipped default — `SettingsManager.h` `restoreSessionOnStart = false`): nothing is written to disk by any of the three items
- [ ] macOS: Cmd-Q / menu Quit with two windows restores both (measured on the relay); K3 answered

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-CLOSE` | Overlay close guards (`g_file_dialog_active`, `g_wallet_overlay_prevent_close`) | Quit tears down every overlay; a quit fired while the wallet overlay holds the mnemonic must still go through the normal shutdown, not bypass it. ⛔ Neither flag is converted to per-window (they are correctly global) |
| `R-GOLD` | Gold pill lands on the originating tab | Restored tabs get **new** `Tab::id`s; nothing may cache a pre-quit id. `Tab::id ≠ CefBrowser::GetIdentifier()` |
| `R-UPDATE` | Update apply still works | `ShutdownApplication` Step 0 cleans up the updater; the WinSparkle shutdown-request callback also posts `WM_CLOSE` to `g_hwnd` (§12 Q2) |
| Session v2 format | Old `session.json` still loads | Items 2–3 must not change the file shape; a rollback build must read files this build wrote |
| Wallet shutdown | Wallet backend stops cleanly on quit | Exit now reaches `ShutdownApplication` from a **secondary** window too — the wallet/adblock stop sequence must be identical |

## 4. Evidence table

⛔ Not a money/schema/crypto phase — controls authored here. ⛔ **SUBJECT for every multi-window row:** two windows of the **same** profile in **one** process (confirm via `Win32_Process`: exactly one non-`--type=` `HodosBrowser.exe` from the dev path) — two different profiles are two processes and prove nothing (the window-globals ticket's vacuous-test trap).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P2-A1` | Windows A (primary) + B; **Exit clicked in B's menu** ⇒ both windows gone, process exited, log `🛑 Starting graceful application shutdown...` **once** | Same binary with the `menu_action "exit"` arm reverted to `PostMessage(g_hwnd, WM_CLOSE…)` ⇒ **A** closes, B stays (the ticket's measured K18.1 symptom) | Process list before/after; `ShutdownApplication` log line; B's header browser is the IPC sender (role logged by the arm) | T2 (+T3 click) | ⬜ |
| `P2-A2` | Same, Exit clicked in **A** | Revert as A1 ⇒ A closes, B stays, **no** `SaveSession` line | Same | T2 | ⬜ |
| `P2-A3` | After A1, restore on: next launch restores **2 windows**, each with its own tab URLs and saved geometry (`session.json` holds `windows[2]`) | Revert as A1 ⇒ `📋 Session saved: 1 tabs across 1 windows` (the ticket's K17 measurement) | `session.json` read **before** the relaunch (it is deleted after restore — `simple_app.cpp :: OnContextInitialized` restore block); then the restored windows' tab URLs | T2 | ⬜ |
| `P2-A4` | The `exit` **IPC** arm (`simple_handler.cpp`, `message_name == "exit"`) behaves as A1 | Revert that arm alone ⇒ wrong window closes | The IPC arm's log line, not the menu arm's | T2 | ⬜ |
| `P2-A5` | Window **X** on B with A open ⇒ only B closes; later quit from A ⇒ restore brings back **A only** | A5 ↔ A3 are each other's control: a fix that saves every window ever opened passes A3 and **fails** A5 | `session.json` window count after the later quit | T2 | ⬜ |
| `P2-A6` | One window, restore on, quit ⇒ restore identical to today (URLs, active tab, geometry) | Compare against a baseline `session.json` captured on the pre-change build — a diff in shape fails | Byte-level JSON key comparison, same profile | T2 | ⬜ |
| `P2-A7` | Restore **off**: no `session.json` written by Exit, session-end or the periodic save | Turn restore on ⇒ file appears (proves the check reads the right path) | `<profile>\session.json` mtime/existence | T2 | ⬜ |
| `P2-B1` | **Session end:** two windows, `WM_QUERYENDSESSION`/`WM_ENDSESSION` delivered ⇒ `session.json` holds both windows before the process dies | Build without the handler ⇒ file holds ≤ 1 window or nothing (📖 no handler exists anywhere in `cef-native` today — the RED is today's binary; record what it actually does, K2) | `session.json` content captured by a watcher process; the handler's log line | T2 (synthetic messages) | ⬜ |
| `P2-B2` | Real Windows **sign-out** with two windows ⇒ next sign-in + launch restores both | Today's build, same gesture ⇒ observe and record (K2 — this is also the K2 measurement) | Owner's machine, dev build, restore on | T3 | ⬜ |
| `P2-C1` | **Periodic save:** two windows, change tabs, wait > interval, `taskkill /F` ⇒ next launch restores the post-change state | Periodic save disabled (item 3 reverted) ⇒ next launch restores nothing / the stale pre-change file | `session.json` mtime before the kill; restored URLs | T2 | ⬜ |
| `P2-C2` | A kill **during** a save never leaves an unreadable `session.json` (write-temp-then-rename) | Fault-inject a kill between write and rename on a build that writes in place ⇒ truncated JSON, restore logs a parse failure | File validity after 20 randomised kills; restore log | T2 | ⬜ |
| `P2-C3` | Periodic save does not run on the UI thread for longer than one frame budget and never on the shared `TID_FILE_*` thread | Put the save on `TID_FILE_USER_BLOCKING` with a stalled task queued ⇒ save latency balloons (proves the measurement sees thread choice) | Save timing log; thread id logged | T1/T2 | ⬜ |
| `P2-M1` | **macOS:** two windows, Cmd-Q ⇒ restore brings back both | Revert: remove the second window from `WindowManager` before `SaveSession` ⇒ 1 window | mac relay: `session.json` + restored windows | T3 (relay) | ⬜ |
| `P2-M2` | **macOS K3:** second window's close button closes **that** window and its tabs (📖 `MainWindowDelegate windowShouldClose:` hard-codes `window_id == 0`) | — measurement row; outcome recorded either way | mac relay log: which `window_id`'s tabs closed | T3 (relay) | ⬜ |

**Two-sided pairs:** A1/A2 ↔ A5 (quit restores all / X-closed window stays closed). A7 ↔ A3 (off writes nothing / on writes all).

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `cef-native/src/handlers/simple_handler.cpp` — `menu_action`, `action == "exit"` arm (Windows: `PostMessage(g_hwnd, WM_CLOSE…)`; macOS: `ShowQuitConfirmationAndShutdown()`) | ✅ | Windows arm changes to the quit path |
| `cef-native/src/handlers/simple_handler.cpp` — `message_name == "exit"` IPC arm | ✅ | same fix; its macOS arm is a comment only (`NSApp terminate`) — leave |
| `cef-native/cef_browser_shell.cpp :: ShutdownApplication` | ✅ | the single quit path; calls `SaveSession()` at Step 0a. ⭐ Exit = call this, **not** a new routine (SCOPE §3). Has a double-entry guard |
| `cef-native/cef_browser_shell.cpp :: ShellWindowProc` `WM_CLOSE` (last / primary / secondary arms) | ✅ | unchanged; X behaviour must not move. Last-window arm sets `g_app_shutting_down` + `SingleInstance::SetShuttingDown()` **before** `ShutdownApplication` — the Exit path must do the same (F5 gap-a) |
| `cef-native/cef_browser_shell.cpp :: SaveSession` | ✅ | unchanged shape; items 2–3 call it. ⚠️ Writes with no temp-rename today |
| `cef-native/cef_browser_shell_mac.mm :: SaveSession` (static) | ✅ | ⚠️ **second copy** — item 3 lands in both or neither |
| `cef-native/cef_browser_shell_mac.mm :: ShowQuitConfirmationAndShutdown`, `ShutdownApplication`, `MainWindowDelegate windowShouldClose:` | ✅ | macOS quit already = all windows; `windowShouldClose` hard-codes window 0 (K3) and deliberately leaves the process alive after the last window (Chromium `ScopedKeepAlive` convention) |
| `cef-native/src/handlers/simple_app.cpp :: SimpleApp::OnContextInitialized` — v2 `windows[]` restore + `session.json` delete-after-restore | ✅ | unchanged |
| `cef-native/cef_browser_shell.cpp` — `WM_QUERYENDSESSION` / `WM_ENDSESSION` | ✅ **absent** (grep, whole `cef-native`) | new handler, main WndProc only. ⛔ Rule 4: Win32 docs read and cited at kickoff (session-end timing, `ENDSESSION_LOGOFF`, `ShutdownBlockReasonCreate`) |
| `cef-native/src/handlers/simple_handler.cpp :: SimpleHandler::GetOwnerWindow` | ✅ | not needed for quit-all; used only if §12 Q2 is answered "include" |
| `TICKET_cef_file_thread_ids_share_one_thread.md` | ✅ open, deferred | item 3 must not add load to the shared file thread |

## 6. Out of scope

Split view (decision 14 — its own scoping run after this phase). Pin persistence (B5-T6-P10). A "closed windows"
list / Ctrl+Shift+T for windows. A Windows "quit with N tabs?" confirmation (Chrome on Windows does not ask; macOS
keeps its existing dialog). Changing the session file format. Restoring internal pages (NTP, settings) — still
filtered as today. Changing the X button.

## 7. Rollback

Three commits, one per item (Exit, session-end, periodic save); each reverts alone. Session format is unchanged, so
files written by this build load on the previous build.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| Exit reaches `ShutdownApplication` without `g_app_shutting_down` / `SetShuttingDown`, so a concurrent relaunch attaches to a dying process | A1 log order check (add: both lines precede `SaveSession`) |
| Periodic save writes a window the user just closed back into the file after it was removed (race between close and save) | A5 repeated with the save interval forced to 1 s |
| Crash mid-write corrupts `session.json` ⇒ the one crash we meant to survive now loses **everything** | C2 |
| Periodic save put on `TID_FILE_*` stalls behind a slow task (the shared-thread ticket) — or blocks the UI thread | C3 |
| Session-end handler runs after CEF has begun tearing down, or Windows kills us before the write — green in the synthetic test, red in a real logoff | B1 is synthetic; **B2 is the real one** and cannot be skipped |
| Quit while the wallet overlay shows the mnemonic silently discards the user's in-progress backup | R-CLOSE at boundary; A1 run once with the wallet overlay open in unsafe state — record what happens (shutdown is today's behaviour for last-window close too) |
| macOS: closing both windows then Cmd-Q saves **nothing** (last window removed before quit) — "fixed" on Windows, still losing on Mac | M1 + M2; if measured, recorded as a finding (Chrome-on-Mac parity question), not silently fixed |
| Per-window active tab wrong after restore (📖 `SaveSession` compares every window's tabs to the **global** `GetActiveTabId()`, so non-focused windows restore with tab 0 active) | noticed at G3 — §12 Q1 |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | A1–A7, B1–B2, C1–C3 | the fix |
| macOS | M1, M2, and C1–C2 via relay if item 3 lands in the mac `SaveSession` copy | Exit already quits on mac; `WM_QUERYENDSESSION` has no mac twin — the mac equivalent is `applicationShouldTerminate`/`NSWorkspaceWillPowerOffNotification` (📖 neither referenced in `cef-native` today) — **K2-mac**, measured on the relay, not built here unless M1 shows loss |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1.5** — one multi-window sitting (Exit from A and from B, X-then-quit) + one real sign-out (B2). ⭐ Run B5-T6-P3's two-window prompt row in the **same** sitting |
| Human-bound rows | `P2-B2` (real sign-out), the A-rows' native menu clicks (CDP key/click events do not reach native input — memory), `P2-M1`/`M2` (mac relay ~1 h) |
| Unknowns (K) | **K2** what a real logoff does today and whether our save beats teardown · **K3** mac second-window close + Cmd-Q restore · K2b the periodic-save interval (Chrome 2.5 s debounce vs Firefox 15 s) — agent picks at kickoff with a measured save cost |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| gives → | B5-T6-P10 pin/mute | P10 edits the same `SaveSession` pair — P2 lands first (ticket ordering) |
| gives → | split view (deferred, decision 14) | its scoping run starts after this phase |
| gives → | B5-T6-P4 apply-on-quit | P4's trigger is "the app quit"; P2 makes Exit a real quit on Windows |
| needs ← | B5-T0 Build 1 | browser-level evidence gathered on the refreshed engine |
| shares → | B5-T6-P3 (two-window prompt Z-order) | same two-window sitting |

## 12. Open questions for the owner

1. **Found at G3 (code reading):** `SaveSession` marks the active tab by the **process-wide** active tab, so every window except the focused one restores with its **first** tab selected instead of the one you had open. It is one line in the function this phase already calls. ⭐ Recommend **fold it into item 1** (it is part of "brings back every window"). Otherwise it becomes a ticket.
2. **Found at G3:** the WinSparkle "shut down so the installer can run" callback (`cef_browser_shell.cpp`, `updater.SetShutdownCallback`) also posts `WM_CLOSE` to the **primary window only** — with two windows open, a notify-mode install would leave window B running with files locked. ⭐ Recommend **include** (same one-line change as Exit: call the quit path). Otherwise ticket it under P4.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons (G11 must not rise)
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] C++ touched ⇒ `MAC_RELAY_BETA5.md` round names the files, flags the shared `simple_handler.cpp` `#ifdef` split and the mac `SaveSession` copy
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
