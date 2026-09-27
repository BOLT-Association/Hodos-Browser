# B5-T6 — Browser shell — track scope (G2)

**Written:** 2026-09-25 by the G2 research agent for beta.5 Track 6 (Browser shell). **Status:** 📌 PROPOSED — owner review owed.
**Branch / head at time of reading:** `0.4.0`, `4435ba1` (docs: old-address scan ticket). Engine read: `CEF_VERSION "150.0.43-7871.3576+g9ccef04+chromium-150.0.7871.187"` (`cef-binaries/include/cef_version.h`).

> **Labels used on every claim:** 📖 **code reading** · 📏 **measurement** (a grep count or a command run
> today; nothing was run in the browser or wallet) · 📄 **doc says** · 🌐 **web** (URL, fetched 2026-09-25).
> ⛔ Nothing below was measured in a running browser. Every behavioural claim about our code is a code reading
> unless it quotes a ticket's own measurement.
>
> ⚠️ **Reminder from the G1 mission:** this track is **first in line to be cut** if the release runs long. The
> candidate phases in §5 are ordered and grouped so they can be cut **from the back**.

---

## ✅ 0. G2 decisions applied (owner, 2026-09-27)

> The research below is the **evidence**; this block is the **decision**. Where they differ, this block wins.
> Full record: `../README.md` → "✅ Decisions as made".

| Question | Decided |
|---|---|
| §9 **Q1** menu → Exit | ✅ **Quits the whole app** — the industry standard (Chrome, Firefox, Vivaldi, our macOS build). 👤 *"do what people are accustomed to."* The window's X closes one window |
| **Q5** password import · **Q7** split view | ✅ **Deferred** to the next release (decision 14). Passwords return **CSV-only**; split view gets its own scoping after P2 |
| **Q6** store a declined permission | ✅ **No.** The user is simply re-prompted (👤 *"the user's not going to know to go undo it"*). **P8 dropped** |
| Q2 session restore (a) + crash-save item · Q3 Tier 1 G1 / Tier 2 G2 · Q4 claim endpoint (T6 builds, T1 harness rows) · Q8 cut order | Per this doc's recommendations (approved) |

---

## 1. Goal

**The browser around the wallet behaves the way a user of Chrome expects: updates happen out of sight, quitting
and reopening keeps every window, consent screens are Hodos's own and fit on screen, and a user can reach the tools
that recover their own data and payments.**

---

## 2. Telescope — how others handle the same features

All rows 🌐 web, fetched 2026-09-25. Mozilla's support pages refused automated fetches (bot challenge); rows citing
them rest on search-engine excerpts of those pages and are marked *(excerpt)*.

### 2.1 Multi-window session restore — ⭐ this changes how we should read our own ticket

| Browser | What it does | Source |
|---|---|---|
| **Chrome / Chromium** | A window closed **while other windows stay open is not restored.** Source comment: *"If an individual tab is being closed or a secondary window is being closed, just mark the tab as closed now."* Only the **last** window's close is held back (`pending_window_close_ids_`). Menu **Exit** (quit) with several windows open ⇒ all of them come back | https://chromium.googlesource.com/chromium/src/+/main/chrome/browser/sessions/session_service.cc (main) |
| Chrome — *when* it saves | **Continuously**, not only at shutdown: session commands are flushed to disk on a **2.5 s** delay (`kSaveDelay`) and the file is rebuilt periodically | https://chromium.googlesource.com/chromium/src/+/main/chrome/browser/sessions/session_service.h ; https://chromium.googlesource.com/chromium/src/+/main/components/sessions/core/command_storage_manager.cc |
| **Firefox** | Close windows one at a time, then quit ⇒ **only the last window** is restored. Quit with all windows open ⇒ all restored. Saves every **15 s** (`browser.sessionstore.interval`) to `recovery.jsonlz4` | https://support.mozilla.org/en-US/kb/restore-previous-session *(excerpt)* ; https://support.mozilla.org/en-US/questions/1403535 |
| **Vivaldi** | "Last Session" reopens *"the Tabs and Windows that were open last time you closed Vivaldi"*; closed windows go to the **Closed Tabs** list for the current session; asks before closing a window by default | https://help.vivaldi.com/desktop/appearance-customization/starting-vivaldi/ ; https://help.vivaldi.com/desktop/tabs/windows/ |
| **Brave** | Unconfirmed — no Brave page found; presumably inherits Chromium's `SessionService` | — |
| Where a closed window goes (Chrome) | History → "Recently closed", as a group; Ctrl+Shift+T reopens it | https://shiftplus.app/blog/chrome-restore-windows-after-restart-mac/ (third-party, not confirmed on a Google page) |

⭐ **What this means for us.** The session-restore ticket's own "product question" — *if the user deliberately
closes window B and then quits, should B come back?* — is answered the same way by Chrome and Firefox: **no**. And the
ticket's measured run (close both windows, one tab saved) is, by that standard, **what Chrome and Firefox would also
do** if windows are closed one by one. Our real gaps are elsewhere — see §4 tickets 2 and 3.

### 2.2 The menu **Exit / Quit** item

| Browser | What it does | Source |
|---|---|---|
| **Chrome** | Windows/Linux ⋮ → **Exit** (Alt+F, X); Mac **Quit** (Cmd+Q). **Closes the whole application, every window** | https://support.google.com/chrome/answer/2391819 |
| **Firefox** | Menu → Exit / Quit closes all windows; a window's X closes only that window | https://support.mozilla.org/en-US/questions/1277693 ; https://www.ghacks.net/2018/11/27/mozilla-changes-firefoxs-warn-on-quit-logic/ |
| **Vivaldi** | Exit / Alt+F4 / ⌘Q close all windows; optional confirm | https://help.vivaldi.com/desktop/tabs/windows/ |
| **Brave** | Unconfirmed; presumably Chromium's Exit | — |
| **Hodos macOS (ours)** | 📖 `menu_action "exit"` → `ShowQuitConfirmationAndShutdown()` → `ShutdownApplication()` — **quits the app**, like Chrome (`simple_handler.cpp`, `cef_browser_shell_mac.mm :: ShowQuitConfirmationAndShutdown`) | code |
| **Hodos Windows (ours)** | 📖 posts `WM_CLOSE` to `g_hwnd` — closes **the primary window only** | code |

⇒ Every browser checked, **and our own macOS build**, treat Exit as *quit the application*. 👤 The owner's stated
expectation on 2026-08-31 was *"close the window it was clicked in"*. That is a real disagreement — §9 Q1.

### 2.3 Split view

| Browser | What it does | Source |
|---|---|---|
| **Chrome** | **Exactly two** tabs, one divider; one pane active, the address bar and back/forward act on it; entry from the tab menu, dragging a tab to the edge, the link menu, Shift+Alt+N. On by default in **Chrome 145** (Feb 2026, press-reported); the `splitViewId` extension API dates from 140 | https://support.google.com/chrome/answer/16971124 ; https://winbuzzer.com/2026/02/20/google-chrome-145-split-view-pdf-annotation-drive-save-xcxwbn/ |
| **Brave** | Own split view, two pages, tab menu → "New Split View"; default-on targeted at 1.78.x | https://github.com/brave/brave-browser/issues/43803 |
| **Firefox** | Two tabs side by side, shipped **Firefox 149 (24 Mar 2026)**; 150 added "Open Link in Split View" | https://blog.mozilla.org/en/firefox/split-view/ ; https://ubuntuhandbook.org/index.php/2026/03/firefox-149-0-split-view-tab-note/ |
| **Vivaldi** | **Tab Tiling** — unlimited tiles, grid/vertical/horizontal, draggable dividers; flexibility improved in 7.8 | https://help.vivaldi.com/desktop/tabs/tab-tiling/ |

⇒ Three of four settle on **two panes**. That is the smallest honest target if we ever build it.

### 2.4 Pin and mute

| Browser | What it does | Source |
|---|---|---|
| **Chrome — mute** | The tab menu's item is **"Mute site"** — **per website**, stored as the Sound site setting. Per-*tab* mute exists only behind a flag | https://www.ghacks.net/2018/03/18/the-complete-google-chrome-audio-muting-guide/ ; https://www.ghacks.net/2022/03/30/chrome-100-tab-audio-muting-is-back/ |
| **Chrome — pin** | Pin/Unpin in the tab menu; the help page does **not** state pins survive restart (they do in practice via session restore — unconfirmed on a Google page) | https://support.google.com/chrome/answer/2391819 |
| **Firefox** | Pinned tabs always reopen at startup (they belong to a window, so quit with it open). Mute is **per tab** | https://support.mozilla.org/en-US/kb/pinned-tabs-keep-favorite-websites-open *(excerpt)* |
| Brave / Vivaldi | Not researched | — |

⇒ Hodos shipped per-**tab** mute in beta.3 Phase 4 (Firefox's model). Chrome's default is per-**site**. The
open ticket asks for per-site mute; that is Chrome parity, not a new idea.

### 2.5 Importing from Chrome — ⛔ passwords

| Item | Finding | Source |
|---|---|---|
| **App-Bound Encryption (ABE)** | Chrome 127 (Windows) binds decryption to Chrome's identity via a SYSTEM service, *"causing other apps to fail when attempting to decrypt"*; started with cookies, announced to extend to passwords | https://security.googleblog.com/2024/07/improving-security-of-chrome-cookies-on.html (body did not load; quoted via https://www.bleepingcomputer.com/news/security/google-chrome-adds-app-bound-encryption-to-block-infostealer-malware/) — 30 Jul 2024 |
| **Firefox** | **No longer imports Chrome passwords directly on Windows — CSV only** (Fx 139 English, 140 all locales). Settings → Import Data → "Passwords from CSV file" | https://support.mozilla.org/en-US/kb/import-data-another-browser *(excerpt)* ; https://support.mozilla.org/en-US/kb/import-login-data-file |
| **Brave** | **Removed** "Saved passwords" from its Chrome import dialog (issue #46206, opened 20 May 2025, milestone 1.80.x): Chrome does a step *"before saving to disk… We do not have access to that code"*. Re-enable open as #54463. CSV password import added (1.84) | https://github.com/brave/brave-browser/issues/46206 ; https://github.com/brave/brave-browser/issues/54463 ; https://brave.com/whats-new/import-passwords-csv/ |
| **Edge / Vivaldi** | Edge still documents direct "Import from Google Chrome"; whether passwords come across today — **no official statement found**. Vivaldi unconfirmed | https://support.microsoft.com/en-us/edge/import-your-favorites-and-passwords-in-microsoft-edge |
| **macOS** | Chrome's key lives in the Keychain item **"Chrome Safe Storage"** — reading it prompts the user and is the same "defeat Chrome's encryption" route our ticket forbids | Chromium `components/os_crypt/common/keychain_password_mac.mm` (via GitHub code search) |
| Bookmarks / history | Still import directly in every browser; nothing found to the contrary | — |

⇒ **Two Chromium-based browsers that ship Chrome's own code (Brave) and a non-Chromium one (Firefox) have both
retreated to CSV for passwords.** Our ticket's position (CSV only, never touch Chrome's keys) is the industry
position, not a Hodos limitation.

### 2.6 Silent vs visible updates

| Browser | What it does | Source |
|---|---|---|
| **Chrome (Windows)** | Background download, then a coloured "Relaunch to update" nudge (green <2 d, orange ~4, red ≥7). New build goes in `Application\<version>` beside the running one with a `new_chrome.exe` renamed on relaunch; relaunch restores tabs and windows | https://support.google.com/chrome/answer/95414 ; `new_chrome.exe` detail: https://support.lynchburg.edu/TDClient/51/Portal/KB/Article/17895/ (third-party) |
| **Brave (Windows)** | Fork of Google's Omaha 3 (`BraveUpdate.exe`), a separate service — updates while Brave is closed; Omaha 4 move in progress | https://github.com/brave/brave-browser/wiki/Brave-omaha ; https://github.com/brave/brave-core/pull/39372 |
| **Firefox (Windows)** | Maintenance Service applies without UAC; a Background Update scheduled task checks every 7 h while Firefox is closed; staged updates finish on restart | https://support.mozilla.org/en-US/kb/what-mozilla-maintenance-service ; https://firefox-source-docs.mozilla.org/toolkit/mozapps/update/docs/BackgroundUpdates.html |
| **Vivaldi** | Own updater; downloads in the background, **installs on next restart**, "Restart Required" button; all-users installs show a Windows notification | https://help.vivaldi.com/desktop/install-update/update-vivaldi/ |
| **Sparkle (macOS, ours)** | With automatic download on, installs **on quit**; prompts only if the app has not quit within `SUScheduledImpatientCheckInterval` (1 week) | https://sparkle-project.org/documentation/customization/ |

⇒ Nobody makes the user **watch** an apply. Chrome and Vivaldi apply at relaunch but make relaunch cheap (versioned
folders); Sparkle applies on quit. The update ticket's **Tier 2 (apply on quit)** is Sparkle's model on Windows.

### 2.7 Permission prompts

| Item | Finding | Source |
|---|---|---|
| Chromium prompt types | `components/permissions/request_type.h` lists **31** request types (≈21 on every desktop platform; the rest desktop-only, Android-only or ChromeOS-only). ⚠️ The fetch tool reported 28 and listed 31 — treat the count as approximate | https://chromium.googlesource.com/chromium/src/+/main/components/permissions/request_type.h |
| Our engine's view | 📖 `cef_types.h` (CEF 150.0.7871.187) exposes **28** `CEF_PERMISSION_TYPE_*` bits (1<<0 … 1<<28, with 1<<25 aliased) | `cef-binaries/include/internal/cef_types.h` |
| **Brave** | Uses Chromium's prompt UI for almost everything; swaps in its own prompt **only** for its wallet requests (Ethereum/Solana/Cardano), and adds expiring grants ("permission lifetime") | github.com/brave/brave-core: `patches/components-permissions-request_type.h.patch`, `chromium_src/chrome/browser/ui/views/permissions/permission_prompt_factory.cc`, `components/permissions/permission_lifetime_*` (master) |

⇒ Brave does **not** brand Chromium's generic prompts; our "no stock Chrome bubble ever" goal is stricter than
Brave's. Worth knowing when sizing it — nobody has shown us the whole job done.

---

## 3. Kaleidoscope (open) — do we already have this shape?

| Need | Already exists — reuse it | Shape we must NOT duplicate |
|---|---|---|
| "Which window am I acting on?" | 📖 `SimpleHandler::GetOwnerWindow()`, `WindowManager::GetWindowByHwnd`, `TabManager::GetActiveTabForWindow(int)`, and inside a WndProc `GetWindowLongPtr(hwnd, GWLP_USERDATA)` (`cef_browser_shell.cpp :: ShellWindowProc`) | A second window registry. `BrowserWindow` is the per-window record; new per-window state (fullscreen, panes) goes there |
| Close one window vs quit | 📖 `ShellWindowProc`'s `WM_CLOSE` already splits last / primary / secondary; `ShutdownApplication()` is the single quit path and already calls `SaveSession()` first; `ReleaseOverlaysOwnedBy` hands overlays to the survivor | A new quit routine. Exit-as-quit is "call `ShutdownApplication()` from the menu", not a new function |
| Session file | 📖 `SaveSession()` already writes v2 `windows[]` for N windows; `simple_app.cpp :: OnContextInitialized` already restores N windows with geometry | ⚠️ **Two copies already exist**: `cef_browser_shell.cpp :: SaveSession` and `cef_browser_shell_mac.mm :: SaveSession` (static). Any session change (restore fix, pins, panes) must land in **both** or be unified first — kaleidoscope finding, noted not chased |
| Recently closed | 📖 `TabManager::RecordClosedTab` / `recently_closed_` (tab-level) | A separate "closed windows" store — extend this one if a closed window should be reopenable |
| Tab flags | 📖 `Tab::muted` + re-apply in `OnLoadingStateChange` (beta.3 Phase 4) is the template for `Tab::pinned` | — |
| Per-site settings | 📖 `SitePermissionStore` (host-normalised), `MirrorNetworkPermissionToChromium` write-through | A parallel top-level store for "mute site" (the dual-store mistake, closed `8874232`) |
| Overlays | 📖 `<Name>OverlayRoot.tsx` + `Create*/Show*/Hide*` in `simple_app.cpp`; shared `notification_browser_` multiplexed by `BRC100AuthOverlayRoot.tsx`'s type dispatch — which **already has** the generic permission fallback `PERM[permCode] \|\| { label: 'access a device feature' }` | New HWNDs for new prompt types — add cases |
| Chrome import | 📖 `ProfileImporter` (`include/core/ProfileImporter.h`, bookmarks + history), `useImport.ts`, IPC `import_detect_profiles` / `import_bookmarks` / `import_history` (`simple_handler.cpp`). The Import UI lives only on the **orphaned** `SettingsOverlayRoot` (`/settings`); the live `SettingsPage` has sections General / Privacy & Security / Downloads / Wallet / About and **no Import** | A second importer. ⚠️ And a possible bigger reuse: Chromium's own password manager (live in Hodos per the ticket) has its **own CSV import** in its settings page — whether it is reachable in our CEF build is **unknown (K6)**. If it is, slice C needs no parser of ours at all |
| Update apply | 📖 `hodos-update-helper.exe` transactional apply + rollback (`cef-native/src/core/UpdateApply.cpp`, `UpdateFs.cpp`, `update-helper/transaction.cpp`), picker-exit wait (`ae5beb6`, 2026-07-09, *silent-update picker-gate v2*) | A new updater. Tier 2 moves the **trigger**, not the machinery |
| Wallet supervision | 📖 beta.3 Phase 8d supervisor (`StartBackendSupervisor`, bounded relaunch 3× at 2/4/8 s) and `WaitForWalletHealth` (`cef_browser_shell.cpp`) | A second supervisor for the fast-relaunch ticket |
| Denied-grant memory (edit-limits item 4) | 📖 All three V18 grant tables (`domain_protocol_permissions`, `domain_basket_permissions`, `domain_counterparty_permissions`) **already carry `revoked_at`**, revoke is a **soft** update (`domain_permission_repo.rs`), the upsert clears `revoked_at` on re-grant, and list-all queries that include revoked rows exist. ⇒ *"user said no"* may be expressible as a soft-revoked row **without a new column**. ⚠️ Still a semantics change to wallet data — invariant 2 asks first either way | A new "denials" table |
| Tools-tab card 1 | 📖 `handlers.rs :: payment_claim_block` (writer, golden-keys test), `internalize_action` (hardened in beta.3 10a). 📏 **No claim endpoint exists** in `main.rs`'s route table today | A frontend-built BEEF — the Rust wallet must fetch and build it (invariant 1's spirit: money logic stays in Rust) |
| Tools-tab card 2 | 📖 `/wallet/rescan` route exists (`main.rs`) but scans BIP32 only (T1's ticket). 📏 **No `.tsx` calls it**; the UI was removed in `15abffd` (2026-03-27); `.wd-rescan-*` CSS remains in `WalletDashboard.css` | — |

**Stale doc confirmed (report only, working rule 3):** 📏 `frontend/src/components/wallet/CLAUDE.md` lines 19, 88
and 104 say `SettingsTab` has a *Wallet Rescan* section calling `/wallet/rescan`. No `.tsx` does; `git show 15abffd`
shows the section being deleted from `SettingsTab.tsx`. The layer doc is stale. Whoever builds card 2 corrects it.

---

## 4. Ticket review — all 14

Verdict key: **KEEP** (still true) · **KEEP, re-shaped** · **MEASURE FIRST** · **DEFER** · **CLOSE**.

| # | Ticket (plain subject) | Still true today? — evidence | Verdict · lands in | Why |
|---|---|---|---|---|
| 1 | `TICKET_update_is_visible_when_it_should_not_be.md` — *the user watches the update happen, twice* | ✅ 📖 `MaybeApplyStagedUpdate` at `cef_browser_shell.cpp:5588`, `CefInitialize` at `:6089` — the ticket's line numbers are still exact. ⭐ **New, likely cause of the dialog** (📖, not measured): `AutoUpdater::SetUpdateMode` enables WinSparkle's own scheduled check for **both** notify **and silent** (`mode != Off`), and WinSparkle's scheduled check shows its UI when it finds an update. Second candidate: a legacy profile with `autoUpdateEnabled=true` **migrates to `"notify"`** (`SettingsManager.h`, legacy migration) — plausible for a **0.3.0-beta.29** upgrader. Default for a fresh profile is `"silent"` | **KEEP** · Tier 1 → **P1**; Tier 2 → **P4**; Tier 3 **DEFER** | Every user, every update, owner-raised. Tier 1 must first **measure which** of the two causes fired, or the fix may remove the wrong one |
| 2 | `TICKET_menu_exit_closes_primary_not_the_clicked_window.md` — *Exit closes the primary window, not the one clicked* | ✅ 📖 Windows: both sites still `PostMessage(g_hwnd, WM_CLOSE…)` — `menu_action "exit"` and the `exit` IPC (`simple_handler.cpp`). **macOS resolved by code reading:** Exit → `ShowQuitConfirmationAndShutdown()` = quit all (Chrome's behaviour) | **KEEP, merged** with #3 → **P2** | Prior art and our own macOS say Exit = quit. Making Windows do the same fixes this ticket **and** the Exit half of #3 in one change |
| 3 | `TICKET_multiwindow_session_restore_loses_all_but_last_window.md` — *session restore keeps only the last window* | ✅ 📖 `SaveSession()` is reached only from `ShutdownApplication()`, which only the last-window `WM_CLOSE` calls. **Its open question answered:** 📖 shipped default `restoreSessionOnStart = false` (`SettingsManager.h:15`), so only users who turned it on are exposed. ⭐ **Re-shaped by prior art:** closing windows one by one and losing the earlier ones is what **Chrome and Firefox also do**. The real gaps are: (a) Windows has **no quit-all path** (ticket #2); (b) 📖 **no `WM_QUERYENDSESSION`/`WM_ENDSESSION` handling** anywhere in `cef-native` — a Windows logoff/restart with several windows open may save nothing or one window (unverified); (c) we save **only at shutdown** — a crash loses the session (Chrome saves every 2.5 s, Firefox every 15 s) | **KEEP, re-shaped** → **P2** (with #2) | The ticket's proposed "save on every window close" would make a deliberately closed window come back — the behaviour the ticket itself calls a bug. §9 Q2 |
| 4 | `TICKET_window_scoped_work_uses_process_globals.md` — *one window acts on another* | ✅ remainder true. The two observed symptoms (Ctrl+F, video fullscreen) were fixed in beta.3 Phase 3; 📖 `OnFullscreenModeChange` now uses the browser. 📏 **New count:** `ScalePx(…, g_hwnd)` appears on **57 lines**, **all in `simple_app.cpp`**, **all inside `Create*Overlay` functions** (10 in tab-list, site-info, bookmarks; 6 profile; 4 each in settings, settings-menu, menu, downloads, cookies; 1 wallet) — **zero** left in `cef_browser_shell.cpp`. The ticket's "12 sites" were the ones Phase 3.5 converted; these 57 were never counted. Whether the `Show*Overlay(…, targetWin)` path re-scales to the target window is **not checked**. G11 gate baseline **58** (`scripts/preflight.ps1`) and ⚠️ its pattern does not match `g_hwnd` at all (HARNESS §G11) | **KEEP** → **P9** | Latent, needs mixed-DPI hardware to observe. Ratchet protects it while it waits |
| 5 | `TICKET_split_view_needs_multi_visible_tab_model.md` — *split view* | ✅ 📖 `TabManager::SwitchToTab` still `SW_SHOW`s one and `SW_HIDE`s every other visible tab in the window | **DEFER to beta.6** with its own scoping run | Convenience, not a defect; crosses six subsystems and both platforms; must follow #3. Recheck condition: #3 landed and owner still wants it |
| 6 | `TICKET_tab_pin_and_mute_need_model_changes.md` — *pin and mute-site have no data model* | ✅ 📖 no `pinned` anywhere in `cef-native` Tab/TabManager; mute-tab done (Phase 4). Mute-site is Chrome's **default** mute model (§2.4) | **KEEP** → **P10** (after P2) | Parity feature; session-restore ordering constraint stands |
| 7 | `TICKET_chrome_import_bookmarks_history_passwords.md` — *Chrome import* | ✅ 📖 Slice A gap real: no Import section in `SettingsPage.tsx`; `useImport` used only by `SettingsOverlayRoot.tsx`. No password code anywhere. Prior art (§2.5) confirms CSV-only for passwords | **KEEP slices A+B** → **P6**; **slice C (passwords) DEFER** | Slice C is blocked by the un-brandable save-password bubble (no CEF hook — an engine patch, T0's queue) and is security-sensitive. Owner said *"don't pull forward without an explicit ask"* — §9 Q5 |
| 8 | `TICKET_profile_lock_misreports_missing_dir.md` — *a missing profile folder is reported as "in use"* | ✅ 📖 Windows `AcquireProfileLock` treats every `CreateFileA` failure alike, 6×500 ms retry. ⭐ **macOS has it too** (📖 `open(O_CREAT)` fails on a missing parent the same way, same retry, same message in `cef_browser_shell_mac.mm`) — the ticket said nothing about macOS. Note `IsProfileLockedByAnotherInstance` in the same file **already** distinguishes not-found from sharing-violation — the pattern to copy | **KEEP** → **P3** | Small, both platforms, reproducible by an agent (the ticket's repro needs no human) |
| 9 | `TICKET_fast_relaunch_attaches_to_dying_wallet_or_fails_port_bind.md` — *fast relaunch finds a dying wallet or loses the port* | ✅ all four claims 📖: `/health` is stateless (`handlers.rs :: health`); `.bind(("127.0.0.1", wallet_port()))?` exits on failure (`main.rs`); `WalletService::startDaemon` / `cleanupDaemonProcess` exist and **nothing calls `startDaemon`**; updater hardcodes `31301/31302` (`update-helper/transaction.cpp`, 5 sites). **But** the Phase 8d supervisor (bounded relaunch 2/4/8 s) should recover both failure modes after a delay | **MEASURE FIRST** → **P3** | Only a measurement against the 8d supervisor tells us whether a user sees anything. The port helper fix is certain and tiny. The Rust "not-ready" health state touches T3's decision D7 (write intent before broadcast / crash-safe reconcile) — coordinate, don't duplicate |
| 10 | `TICKET_brand_remaining_permission_prompts.md` — *21 prompts still stock Chrome* | ✅ 📖 `SitePermissionType.h` still has **7** ids (Camera…Loopback). ⭐ **Its blocker is gone:** §2 "blocked on Phase 1 (DPI)" — the modal-buttons ticket it waited on was **closed 2026-09-25** with evidence `0a7d43b` (2026-08-26, *DPI-correct mouse input for the OSR overlays*). The two related tickets it defers to (prompt denials, dual store) are also closed | **KEEP** → **P7** | Consent surface; the design (one mechanism, generic fallback, six high-risk types with specific copy) is sound. ⚠️ T0's engine refresh may change the type list — do it **after** T0 |
| 11 | `TICKET_cef_file_thread_ids_share_one_thread.md` — *all file-thread tasks share one thread* | ✅ 📖 still 29 `TID_FILE_*` post sites; `send_transaction` alone on `hodos-wallet-send` | **DEFER — measure first** (option A stays) | The ticket's own recommendation. Re-check condition: a measured stall on the residual (e.g. P3's fast-relaunch run with a hung wallet) |
| 12 | `TICKET_edit_limits_modal_usability.md` — leftover **item 4 only**: *store denials so a declined item can be re-approved* | ✅ 📖 items 1/2/3/5 landed (`a1712e9`, `6338208`, filter in `DomainPermissionsTab.tsx`); item 4 not done. ⭐ Kaleidoscope: `revoked_at` soft-revoke rows may carry it without a new column (§3) | **KEEP** → **P8** | Owner-requested; needs an invariant-2 decision — §9 Q6 |
| 13 | `TICKET_modal_info_tooltip_overflows_modal.md` — *tooltip overflows the modal* | ✅ likely — 📖 `InfoIcon` (`BRC100AuthOverlayRoot.tsx`) is a `position:relative` span with an absolutely positioned popup and no edge clamping visible in the component head; not re-observed | **KEEP** → **P3** | Consent surface, small, needs eyes at DPI cells |
| 14 | `TICKET_wallet_quiet_detector_blind_to_long_polls.md` — *"is the wallet busy?" misses a long poll* | ✅ 📖 `WalletActivityTracker` records only `lastRequestMs_` per host — no in-flight count; `kWalletQuietMs = 1200` unchanged | **KEEP** → **P3** | Consent surface; the fix (count in-flight) is small but the negative control needs a **first** connect to a manifest site |
| — | **Tools tab outline** (`TOOLS_TAB_claim_a_payment.md`) — card 1 *Claim a payment*, card 2 *Scan my old addresses* | 📖 no Tools entry in `WalletSidebar.tsx` (Dashboard / Activity / Certificates / Tokens / Approved Sites / Settings); no claim endpoint | **KEEP** → **P5**; card 2 **blocked on T1** | Owner decided 2026-09-15 it ships in beta.5 |

Nothing closes outright. Nothing merges across tracks.

---

## 5. Candidate phases — ordered, grouped to cut from the back

⛔ These are **candidates** for the microscope pass, not contracts. Owner-hours are rough (§7).

### Group 1 — keep even if the release runs long

**B5-T6-P1 — Silent means silent (update Tier 1)**
- **Objective:** a user on automatic updates never sees an update dialog; notify-mode users still do; Settings → About → Check for updates still works.
- **Tickets:** #1 Tier 1.
- **Unknown (K1):** *which* cause produced the owner's dialog — WinSparkle's scheduled check running in silent mode, or a legacy profile migrated to notify. Step 0 reads the owner's `settings.json` and the beta.4 install log.
- **Negative control:** re-enable WinSparkle auto-check in silent mode on the same binary ⇒ dialog returns on a staged N−1 feed.
- **macOS:** confirm only (Sparkle already silent-downloads; check the dialog path is gated the same way).

**B5-T6-P2 — Quit keeps every window (Exit + session restore)**
- **Objective:** Exit quits the application on Windows as on macOS; quitting — by Exit, by closing the last window, or by Windows logoff/restart — restores every window that was open **at that moment**; a window the user closed earlier does not return (Chrome/Firefox model).
- **Tickets:** #2 + #3 (merged).
- **Items:** Exit → `ShutdownApplication()` (or the owner's alternative, §9 Q1); `WM_QUERYENDSESSION`/`WM_ENDSESSION` save; optional — periodic save for crash safety (§9 Q2).
- **Unknowns:** K2 does CEF/Chromium's own session-end handling already run before our process dies on logoff; K3 macOS `windowShouldClose` hard-codes `window_id == 0` (📖) — does a second mac window close correctly and does Cmd-Q restore both?
- **Negative control:** two windows, distinct tabs, Exit ⇒ 2 windows restored; revert Exit to `g_hwnd` on the same binary ⇒ 1. Subject: one process (the same-profile trap in #4's ticket).
- **macOS:** yes — measure Cmd-Q with two windows (relay).

**B5-T6-P3 — Small startup and consent-screen defects**
- **Objective:** four small defects that each make the user distrust what they see.
- **Tickets:** #8 profile lock (both platforms) · #13 tooltip overflow · #14 quiet detector (in-flight counting, keep the 15 s backstop, do **not** widen `IsConnectModalType`) · #9 fast relaunch **measurement** plus the certain fix (updater ports through the port helpers) and a report on the dead daemon code.
- **Unknowns:** K4 does the 8d supervisor already hide #9 from users?
- **Negative controls:** #8 — the ticket's deterministic repro (profile entry, no folder) goes red on today's binary; #14 — **first** connect with a slowed `/waitForAuthentication` (an already-approved site cannot fail); #13 — by eye at 100% and at DPI cells #4/#6/#9.
- **macOS:** #8 yes (same defect); #13/#14 relay confirm.

### Group 2 — should ship

**B5-T6-P4 — Apply the update on quit (Tier 2)**
- **Objective:** the file swap runs after the last Hodos process exits; next launch is simply the new version.
- **Tickets:** #1 Tier 2.
- **Unknowns:** K5 "the browser quit" with several profiles = several processes — the helper must wait for **all**; a failed apply now surfaces a launch later, so rollback carries more weight.
- **Negative control:** same two signed builds with apply-on-quit disabled ⇒ the startup splash returns.
- **macOS:** no (Sparkle already does this).
- ⚠️ Every round needs **two signed builds and a real install** — the costliest owner-hours in the track.

**B5-T6-P5 — Tools tab and "Claim a payment"**
- **Objective:** the outline's card 1, rows T-A1…T-A5; card 2 added when T1's scan functions land.
- **Unknowns:** K7 the claim needs a **new Rust endpoint** (fetch tx + proof, build BEEF, derive, internalize). Is that endpoint T6's or T1's? Money-path rigour applies either way (§9 Q4).
- **Negative control:** as the outline's RED column; T-A4 (edited amount) is the key one.
- **macOS:** frontend only; relay confirm.

**B5-T6-P6 — Chrome import, bookmarks and history (slices A + B)**
- **Objective:** Settings → Import on the live settings page, driving the existing importer; plus a bookmarks-HTML file import (visible file input).
- **Unknowns:** K8 does the existing importer still work (last exercised through the orphaned overlay); what is the restore story for a bad import.
- **Negative control:** remove the section ⇒ no import reachable (today's state); import into a scratch profile and diff bookmark/history counts.
- **macOS:** yes — Chrome's paths differ; the importer's mac arm must be exercised, not assumed.

### Group 3 — could ship

**B5-T6-P7 — Brand the remaining permission prompts** · ticket #10 · *after T0's engine refresh*. Unknown K9: which of the 21 types can fire in our build (the ticket's §4.5). Negative control per the ticket §7. macOS: yes (every mac arm from Phase 0.9 is written-but-unrun).

**B5-T6-P8 — A declined site permission can be re-approved** · ticket #12 item 4 · needs the §9 Q6 decision first. Negative control: untick an item at connect ⇒ it appears with **Approve**; today it is absent.

**B5-T6-P9 — Finish moving window work off process globals** · ticket #4 remainder · the 57 `Create*Overlay` DPI sites, the 18 backward-compat accessors, G11 toward 0 (⚠️ widening G11's pattern is its **own** commit per working rule 6). Needs **two monitors at different DPI** — hardware the owner may or may not have (K10). macOS: unassessed.

**B5-T6-P10 — Pin tab and mute site** · ticket #6 · after P2. Both session-save copies (§3) change. macOS: yes.

### Group 4 — recommend deferring to beta.6

Split view (#5) · password CSV import (#7 slice C) · file-thread option B (#11) · update Tier 3 (#1).

---

## 6. Integration check (RELEASE_CYCLE §3.3)

**1. What existing invariant could this violate?**

| Guarantee | Which phases touch it | Watch |
|---|---|---|
| **Overlay close / lifecycle** (`R-CLOSE`) | P2 (quit tears down overlays), P3 (#13, #14 on the notification overlay), P7 (shared overlay contention — ~60% of Phase 0.9's cost), P8 | ⛔ `g_file_dialog_active` and `g_wallet_overlay_prevent_close` are **correctly** global — P9 must not convert them |
| **The gold pill** (`R-GOLD`) | P9 (window→tab mapping), P10 (tab model), P2 (restored tabs get new ids) | `Tab::id ≠ CefBrowser::GetIdentifier()` |
| **Right-click "Manage Site Permissions"** | P8 (same `DomainPermissionForm`), P7 (Site-controls write-through) | two entry points, one component — fix once, test twice |
| **Privacy perimeter / consent surface** | P3 #14 (only connect modals may claim a parked network permission — keep it), P7 (label never vaguer than Chrome's), P8 (a declined item must not silently re-grant) | a prompt **denial** stays temporary; an **overt untick** is a settled preference (ticket #12's reconciliation) |
| **DPI matrix** | P3 #13, P7, P9 | cells #4/#6/#9; P9 needs cell #9 **with two windows**, never run |
| **macOS parity** (invariant 9) | P2, P3 #8, P6, P7, P10 | two `SaveSession` copies; relay rounds per C++ commit (branch rule) |
| **Wallet DB schema** (invariant 2) | P8, P5 (if a claim record is added) | ask first |
| **Money path** | P5 | claim credits only what the chain says (T-A4) |
| **Ports** | P3 #9 | never hardcode — `PortConfig.h` |

**2. What does it touch that we did not write?** ⭐

| Dependency | Phase | Risk |
|---|---|---|
| **WinSparkle** scheduled check (shows its own UI) | P1 | our "silent" setting does not govern a third-party DLL's timer — the likely cause of the dialog |
| **Windows session-end messages** and CEF's own shutdown on logoff | P2 | ordering between our `SaveSession` and Chromium's teardown is not ours |
| **Code-signature chain** under CEF 150's bootstrap (`HodosBrowser.exe` = `bootstrap.exe`, verifies `HodosBrowser.dll` + `chrome_elf.dll`) | P4 (and Tier 3) | the helper must never leave a mixed-signer set |
| **Chromium permission request types**, changing per engine bump | P7 | sequence after T0 |
| **Chromium's password manager and its CSV import** | slice C (deferred), K6 | save-password bubble has **no CEF hook** |
| **Chrome's on-disk profile formats** (Bookmarks JSON, History SQLite while Chrome runs) | P6 | Chrome can lock History; copy-then-read |
| **OS keychain / DPAPI / ABE** | nothing — ⛔ the ticket forbids touching Chrome's keys, and prior art (§2.5) agrees | — |
| **`/waitForAuthentication`** long-poll shape used by dApps | P3 #14 | a dApp we do not control decides the timing |

**3. What would we have to un-ship if we are wrong?**
- P4 wrong ⇒ a failed apply found at next launch; rollback must be proven **before** ship (two signed builds, real install).
- P2 wrong ⇒ users lose windows on quit, or deliberately closed windows reappear. Reversible by build; session files written in between must still load (keep v2 format).
- P8 wrong ⇒ a declined item treated as granted — a consent-surface regression in stored wallet data. Needs a migration-safe reading, and must fail closed.
- P7 wrong ⇒ a prompt that grants more than its label says. The ticket's "fail closed without the disclosure rendering" rule applies.
- P6 wrong ⇒ corrupted bookmarks/history in the user's own profile with no undo — needs a restore story.

---

## 7. Feasibility inputs (G5.5)

⚠️ **This track is human-bound.** Native input, multi-window gestures, visual judgement and real installs cannot be
driven from the agent environment (📄 memory: SendInput clicks are dropped; CDP key events never reach
`OnPreKeyEvent`). First-pass estimate; record the actual in the AAR.

| Phase | Owner-hours (rough) | Why | Mac relay |
|---|---|---|---|
| P1 Silent means silent | 1 | one real N−1→N install to confirm no dialog | confirm |
| P2 Quit keeps every window | 1.5 | multi-window Exit / X / logoff sitting | ~1 (Cmd-Q, two windows) |
| P3 Small defects | 1.5 | tooltip by eye at 3 DPI cells; first-connect quiet test; fast relaunch observed once | ~0.5 |
| P4 Apply on quit | 4–5 | 2–3 rounds × two signed builds on a real install, incl. multi-profile | — |
| P5 Tools tab card 1 | 2 | a real undelivered PeerPay payment; T-A1…A4 | frontend confirm |
| P6 Import A+B | 1.5 | real Chrome profile on the owner's machine; bookmarks-HTML file | ~1 |
| P7 Branding | 3 | six high-risk types triggered live; DPI cells | ~1.5 |
| P8 Re-approve declined | 1 | connect, untick, re-approve, verify effect | — |
| P9 Window globals | 1.5 | needs two different-DPI monitors | unassessed |
| P10 Pin + mute site | 1 | right-click, restart, verify | ~0.5 |
| **Group 1 (P1–P3)** | **≈ 4** | | ≈ 1.5 |
| **Groups 1+2 (P1–P6)** | **≈ 12–13** | | ≈ 3.5 |
| **All ten** | **≈ 18–20** | split view and passwords not counted | ≈ 6 |

**Unknowns (K) — uncertainty, not difficulty:**
K1 which cause produced the update dialog (P1) · K2 logoff/restart session behaviour (P2) · K3 macOS second-window
close and Cmd-Q restore (P2) · K4 whether the 8d supervisor already hides fast-relaunch (P3) · K5 multi-profile
"all processes exited" for apply-on-quit (P4) · K6 is Chromium's own password CSV import reachable in our CEF build
(deferred slice C) · K7 who owns the claim endpoint (P5) · K8 does the dormant importer still work (P6) · K9 which
of the 21 prompt types can fire in our build (P7) · K10 mixed-DPI two-monitor hardware (P9).
⇒ **K = 8 phases with a genuine unknown** (every candidate except P8 and P10; K6 belongs to deferred slice C). Front-load the cheap ones: K1, K4 and K8 are
each a single look.

**Dependencies:**
- **T1 (money path) → P5 card 2**: blocked on T1's BIP32 and BRC-42 self-scan functions, including the phantom-coin
  root cause (`TICKET_rescan_cannot_find_payments_to_generated_addresses.md`).
- **T1 → P5 card 1**, possibly: if the claim endpoint is judged money-path work, T1 owns it and P5 is its screen.
- **T3 (backup & sync) decision D7** (intent written before broadcast; crash-safe startup reconcile) ↔ **#9 fast relaunch**:
  a not-ready `/health` during shutdown and the reconcile both run at wallet start/stop — design them together.
- **beta.3 Phase 8d wallet supervisor** ↔ #9: measure against it first.
- **T0 (engine refresh) → P7**: new Chromium may add or rename permission types; P7 goes after T0.
- **P2 → P10** (and → split view): one change to session save at a time.

---

## 8. Prior-art rows, ready for `development-docs/PRIOR_ART.md` §3 ledger

Columns match the ledger: `| Date | Question | Source(s) read | What we learned | Verdict | Landed in |`.
⚠️ Licence discipline: **Vivaldi's UI is source-available, not open source** (only its help pages were read);
Brave and Firefox are MPL-2.0; Chromium BSD-3. **Read, never copy.** Nothing here was copied.

| Date | Question | Source(s) read | What we learned | Verdict | Landed in |
|---|---|---|---|---|---|
| 2026-09-25 | Should a window closed mid-session come back after quit? | Chromium `session_service.cc` / `.h`, `command_storage_manager.cc` (main); Firefox restore-session KB *(excerpt)*; Vivaldi help | **No**, in Chrome and Firefox; only windows open **at quit** return. Both save continuously (2.5 s / 15 s), not only at shutdown | 🟢 changed the design — the ticket's "save on every close" would contradict both | B5-T6 SCOPE §2.1, P2 |
| 2026-09-25 | What does the menu Exit item mean? | Chrome help 2391819; Firefox support Q1277693; Vivaldi help | **Quit the application** in all three (and in our own macOS build) | 🟢 surfaced an owner decision | B5-T6 SCOPE §2.2, §9 Q1 |
| 2026-09-25 | Split view shape | Chrome help 16971124; Mozilla blog (Fx 149); Brave #43803; Vivaldi tab tiling help | Chrome/Firefox/Brave: **two panes**; Vivaldi: unlimited tiling | 🟡 informs a deferred item | B5-T6 SCOPE §2.3 |
| 2026-09-25 | Mute tab or mute site? | ghacks (2018, 2022); Chrome help; Firefox pinned-tabs KB | Chrome's default is **per-site**; Firefox per-tab | 🟡 confirms ticket #6's scope | B5-T6 SCOPE §2.4 |
| 2026-09-25 | Can we import Chrome passwords directly? | Google security blog (ABE, Jul 2024, via BleepingComputer); Brave #46206/#54463 + CSV page; Firefox import KB *(excerpt)*; Chromium `keychain_password_mac.mm` | **No** — Firefox (139/140) and Brave (1.80) both moved to **CSV only** | 🟢 confirms the ticket's constraint | B5-T6 SCOPE §2.5 |
| 2026-09-25 | How do others avoid a visible update apply? | Chrome help 95414; Brave Omaha wiki; Firefox maintenance-service + BackgroundUpdates docs; Vivaldi update help; Sparkle customization docs | Nobody makes the user watch the swap; Sparkle applies **on quit**; Chrome/Vivaldi on relaunch with versioned folders | 🟢 supports Tier 2 | B5-T6 SCOPE §2.6, P4 |
| 2026-09-25 | Does anyone brand all of Chromium's permission prompts? | Chromium `request_type.h`; brave-core patches + `permission_prompt_factory.cc` | **Brave does not** — only its own wallet prompts; ~31 request types upstream | 🟡 sizes P7 as uncharted | B5-T6 SCOPE §2.7 |

---

## 9. Open questions for the owner — each with a recommendation

**Q1 — What should menu → Exit do on Windows?**
(a) quit the whole application, as Chrome, Firefox, Vivaldi and **our own macOS build** do; or (b) close the window
it was clicked in, as you said on 2026-08-31.
⭐ **Recommend (a).** It matches every browser checked and our Mac build, the window's X already does (b), and (a)
also fixes the session-restore loss for the Exit path in the same change.

**Q2 — What should session restore bring back?**
(a) every window open **at the moment of quit** (Chrome/Firefox), a window closed earlier stays closed; or (b) every
window ever open in the session.
⭐ **Recommend (a)**, plus saving on Windows logoff/restart. Optional extra: save periodically so a **crash** does not
lose the session (Chrome 2.5 s, Firefox 15 s) — recommend **yes** but as a separate item, because it changes how
often we write the file.

**Q3 — Update: Tier 2 (apply on quit) in beta.5, or Tier 1 only?**
⭐ **Recommend Tier 1 in Group 1, Tier 2 in Group 2.** Tier 2 is the real fix but costs ~4–5 owner-hours of
signed-build installs. If the cut line falls above it, Tier 1 alone still removes the dialog.

**Q4 — Who owns the Tools-tab claim endpoint (card 1)?**
It needs new Rust: fetch the transaction and proof, build BEEF, derive, internalize.
⭐ **Recommend: T6 builds the screen and the endpoint together, but the endpoint's phase contract carries T1's
money-path harness rows** (chain amount only, never overwrite, negative controls designed by someone other than the
author). Alternative: T1 owns the endpoint and T6 waits on it.

**Q5 — Password import (slice C) in beta.5?**
⭐ **Recommend defer to beta.6.** Firefox and Brave both went CSV-only; the save-password bubble our import would
feed **cannot be branded without an engine patch** (add it to `DevOps-CICD/NEXT_CHROMIUM_BUILD.md` PART 2 if wanted);
and you deferred it deliberately. Ship slices A+B (bookmarks, history, bookmarks file) instead.

**Q6 — Edit-limits item 4: may a declined permission be stored?**
It is a change to wallet data (invariant 2). The existing `revoked_at` soft-revoke column may express *"the user said
no"* without a new column.
⭐ **Recommend: yes, as a soft-revoked row** — no schema change — but only for an **overt untick** on the connect
screen or in the panel, never for a prompt denial (which stays temporary, per the closed prompt-denials ticket).

**Q7 — Split view in beta.5?**
⭐ **Recommend no — beta.6**, with its own scoping run after session restore is fixed. You said *"we might not want to
build that much right now"*; the code agrees.

**Q8 — ⭐ The cut line, if the release runs long.**
⭐ **Recommend: keep Group 1 (P1–P3, ≈ 4 owner-hours) and P5 (Tools tab card 1, ≈ 2) — you already decided the claim
tool ships in beta.5. Cut the rest from the back in this order:** P10 pin/mute-site → P9 window globals → P8
re-approve → P7 branding → P6 import → P4 apply-on-quit. P4 goes last because it removes the most-felt complaint,
but it is also the most expensive in your hours; if you have to cut it, Tier 1 still removes the dialog.

---

*End of scope. Next: owner review of §9, then the microscope pass turns the confirmed candidates into phase contracts (G3).*
