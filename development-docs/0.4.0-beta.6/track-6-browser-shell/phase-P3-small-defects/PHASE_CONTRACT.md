# B5-T6-P3 — small startup and consent-screen defects (six items) · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (G3, 2026-09-28). ⛔ Documents only — no code has been written for this phase.

**Track:** B5-T6 Browser shell · **Group:** 1 (keep even if the release runs long) · **Status:** ⬜ NOT STARTED
**Tickets:** `../../tickets/TICKET_profile_lock_misreports_missing_dir.md` (item L) · `../../tickets/TICKET_modal_info_tooltip_overflows_modal.md` (item T) · `../../tickets/TICKET_wallet_quiet_detector_blind_to_long_polls.md` (item Q) · `../../tickets/TICKET_fast_relaunch_attaches_to_dying_wallet_or_fails_port_bind.md` (item F — **measure first** + the port fix) · `../../tickets/TICKET_prompt_opens_behind_another_window_with_two_windows_open.md` (item Z — placed here at G3, see §12 Q1) · `../../tickets/TICKET_find_bar_does_not_follow_tab_switches.md` (item B — placed here at G3, see §12 Q1)
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent for T6 (resumed run) · **Platforms:** both (per item — §9) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **14** (T6 keep set — Group 1). **T6 Q8** cut order (Group 1 is kept). Scope §5 P3 item list, per its recommendations: #14 keeps the 15 s backstop and ⛔ does **not** widen `IsConnectModalType`; #9 is **measure first**. Carried, not reopened.

⭐ **Six independent items, one commit each** (§7). They share a phase because each is small and each makes the user distrust what they see; they share **no code**. An item that grows past a day splits into its own phase at kickoff (RELEASE_CYCLE §4.6).

---

## 1. Goal

A missing profile folder is recreated instead of blaming a second browser; the permission modal's info tooltip stays inside the modal; a site mid-handshake with the wallet is never judged "quiet"; a quick relaunch never leaves the user without a wallet (measured, and the updater stops hard-coding ports); a prompt raised from a second window appears **over that window**; and the find bar follows the tab the user is looking at.

## 2. Done means

- [ ] **L** — A `profiles.json` entry with no folder launches normally (folder created) with **no** "already in use" dialog and no 3 s stall; a genuinely locked profile still gets today's message. Both platforms.
- [ ] **T** — The ⓘ tooltip never overflows the modal and never adds a scrollbar, at 100% and at DPI cells #4/#6/#9.
- [ ] **Q** — On a **first** connect to a manifest site whose `/waitForAuthentication` is still open, the parked loopback prompt is claimed by the connect modal (one decision, not two stacked prompts). The 15 s backstop still fires for a wedged wallet. `IsConnectModalType` unchanged.
- [ ] **F** — A measured answer to K4 (does a fast relaunch show the user "no wallet", given the beta.3 8d supervisor?) recorded with logs; `hodos-update-helper.exe` no longer hard-codes `31301/31302`; the dead `WalletService` daemon code is **reported** (working rule 3), not deleted here.
- [ ] **Z** — Two windows of one profile: a prompt raised by a tab in window B is placed over **window B**, above it, with activation. Same for right-click → Manage Site Permissions from B.
- [ ] **B** — Find in tab A → switch to B → the bar hides (B has no search); Ctrl+F on B prepopulates A's query, selected, with B's count; back to A restores A's search. No stale highlights left in A after its find closes.

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-CLOSE` | Overlay close guards | Item Z re-parents / re-positions the **shared notification overlay**, the most contended overlay in the app (brand-prompts ticket §4.1: ~60% of Phase 0.9's cost). A wrong owner changes which `WM_ACTIVATE` path closes it |
| **Right-click "Manage Site Permissions"** (kickoff UX safeguard) | Quick revoke flow | Item Z changes where that form appears. Must still open, still revoke, from **either** window |
| **Privacy perimeter / consent surface** | Only a **connect** modal may claim a parked network permission | Item Q must not widen `IsConnectModalType` (ticket: a payment modal claiming loopback would grant local-app access from a screen that never mentions it) |
| `R-INTEXT` | Internal never prompts, external always gates | Item Q adds in-flight counting on the wallet request paths; it must not change how a request is classified |
| `R-GOLD` | Gold pill on the originating tab | Item B introduces per-tab state keyed on tab id — `Tab::id ≠ CefBrowser::GetIdentifier()`; the same confusion would misplace find state, and a shared helper must not be "fixed" on the way |
| `R-UPDATE` | Update apply still works | Item F edits `update-helper/transaction.cpp` (the shutdown POSTs). A wrong port means the helper waits on a wallet it never asked to stop |
| DPI matrix | cells #4/#6/#9 | Items T and Z are consent-surface layout |
| Ports (root CLAUDE.md) | never hard-code | Item F is the fix; nothing new may hard-code |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. Not a money/schema/crypto phase — controls authored here.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P3-L1` | Profile entry with no folder, `--profile=<id>`: browser starts, folder created, `Using profile:` logged, no dialog, startup not delayed by the 6×500 ms retry | The ticket's deterministic repro on **today's** binary ⇒ "Profile … is already in use" dialog after ~3 s (seen, not cited) | The dev `HodosBrowser.exe` for that profile id (path-matched), its log, and the folder's existence before/after | T2 | ⬜ |
| `P3-L2` | A genuinely held lock (second process holding `profile.lock`) still produces today's message | L1 ↔ L2 are each other's control: a fix that ignores every `CreateFile` failure passes L1 and fails L2 | A helper process holding the file with `FILE_SHARE_NONE`; the dialog text | T2 | ⬜ |
| `P3-L3` | Unit: the lock function returns a distinct result for path-not-found vs sharing-violation (both arms — Win32 and POSIX `open`) | Collapse the result back to `bool` ⇒ the not-found test fails | `ProfileLock.cpp :: AcquireProfileLock` return value, per errno / `GetLastError` | T1 | ⬜ |
| `P3-LM` | macOS: same repro ⇒ folder created, no dialog | Today's mac binary ⇒ same misreport (📖 POSIX arm treats every `open` failure alike) | mac relay log | T3 (relay) | ⬜ |
| `P3-T1` | Tooltip opened at every ⓘ site (`BRC100AuthOverlayRoot.tsx` — 3 `InfoIcon` uses today) at 100%: bounding box inside the modal's box; `scrollWidth == clientWidth` on the modal | Today's build ⇒ tooltip right edge beyond the modal and `scrollWidth > clientWidth` (the ticket's dead scrollbar) | DOM rects read **in the notification overlay's browser** (role-log resolved — not a tab, not the header); ⚠️ Vite HMR fakes controls — hard-reload before each measurement (memory) | T2 | ⬜ |
| `P3-T2` | Same at DPI cells #4/#6/#9 **by eye** — no overflow, no scrollbar, text readable | The 100% RED screenshot beside each cell (a cell cannot tell a DPI bug from a broken fix without the 100% control) | Owner's eyes on the real modal (consent surface — gates guard the rule, not the screen) | T3 👤 | ⬜ |
| `P3-Q1` | **First** connect to a manifest site, `/waitForAuthentication` held open > 1.2 s (rig delay), loopback prompt parked ⇒ the **connect modal** claims it; no standalone loopback prompt | Today's binary, same rig ⇒ `No connect modal claimed … (waited 1200ms)` and the standalone prompt appears while the connect modal is still assembling. ⛔ An **already-approved** site cannot fail — the ticket's trap | Shell log lines from `ShowDeferredPermissionTask` + the notification overlay's `type` sequence; the site's domain has **no** `domain_permissions` row before the run | T2 | ⬜ |
| `P3-Q2` | A wedged wallet (request never completes) ⇒ the standalone prompt appears at the **15 s backstop**, not never | Remove the backstop in a rig build ⇒ the prompt never appears (proves Q2 sees the backstop, not the quiet window) | `waitedMs` in the log line at release | T2 | ⬜ |
| `P3-Q3` | In-flight counting covers **both** wallet request paths: the HTTP interception path and the `wallet_call` IPC bridge | Rig request over the IPC bridge only ⇒ on today's code the host reads **quiet** (📖 G3 finding: `NoteWalletRequest` is called only in `HttpRequestInterceptor::GetResourceHandler`; `HandleIpcWalletCall` never notes activity) | Tracker state logged per path; the request's path named in the log | T1 + T2 | ⬜ |
| `P3-Q4` | `IsConnectModalType` unchanged: a payment modal arriving while a loopback prompt is parked does **not** claim it | Temporarily widen it to all modal types ⇒ the payment modal claims the loopback permission (seen once, then reverted) | Overlay `type` + `site_permissions` row for Loopback after the run | T2 | ⬜ |
| `P3-F0` | **K4 measurement:** quit, relaunch within 0.5 / 1 / 2 s, three times each ⇒ record whether the new window ever shows "no wallet" to a dApp or the wallet panel, and for how long | — measurement row; each outcome recorded, "not observed" is a valid result, never upgraded | Dev wallet PIDs (path-matched `rust-wallet\target\release`), wallet log `bind` lines, shell log `WaitForWalletHealth` / supervisor lines, and a dApp page's `isAuthenticated` result | T2 | ⬜ |
| `P3-F1` | The helper's shutdown POSTs use `hodos::WalletPort()` / `AdblockPort()`; under `HODOS_DEV=1` + `HODOS_UPDATE_TEST` they go to **31401/31402** | Today's binary under the same rig ⇒ POST to **31301/31302** — the **installed** wallet's port (the 2026-09-01 wrong-process kill class). ⚠️ Run the RED with no production wallet listening, or against a stub listener on 31301 that logs the hit | A stub listener's log per port; the helper's own log line naming the port | T2 | ⬜ |
| `P3-F2` | Grep row: no literal `31301`/`31302`/`31401`/`31402` left in `cef-native/update-helper/`. The rest of `cef-native` is **counted and reported, not changed** (📏 2026-09-28: literals appear in ~16 files outside `PortConfig.h`, many in comments, origin matchers and tests — each a separate judgement, rule 3) | Add a probe literal to `transaction.cpp` ⇒ the grep reports it | The grep command and its output, recorded in the row | T0 | ⬜ |
| `P3-Z0` | **Reproduce first** on today's build: tab in window B raises a connect prompt ⇒ record the overlay's owner, rect and Z-order relative to B | — this row **is** the RED; if it cannot be reproduced, Z stops and returns to the owner with what was seen (ticket's four capture questions answered) | Win32: `GetWindow(overlay, GW_OWNER)`, `GetWindowRect(overlay)` vs B's and A's rects, Z-order via `GetNextWindow`. 📖 G3 reading: `simple_app.cpp :: CreateNotificationOverlay` sizes **and** owns the overlay by `g_hwnd` (the primary) on both the keep-alive and fresh paths | T2 | ⬜ |
| `P3-Z1` | After the fix: the overlay's rect equals B's, it sits above B, and B's prompt has activation — for connect, domain approval, a protocol grant and Manage Site Permissions | Restore the `g_hwnd` lookup on the same binary ⇒ Z0's measurement returns | Same Win32 reads as Z0, from the **same** process (one non-`--type=` `HodosBrowser.exe`) | T2 | ⬜ |
| `P3-Z2` | A prompt raised in **A** still appears over A (single-window behaviour unchanged) | Z1 ↔ Z2: a fix that always uses "the other window" passes Z1 and fails Z2 | Same reads | T2 | ⬜ |
| `P3-Z3` | Owner, two windows on his machine: prompt visible and clickable over the requesting window; Approve/Deny land (DPI cell #9 with two windows if available) | Z0's recording | Owner's eyes + the resulting `domain_permissions` row | T3 👤 | ⬜ |
| `P3-ZM` | macOS: same two-window run | 📖 `cef_browser_shell_mac.mm :: CreateNotificationOverlay` also takes `g_main_window`'s frame and becomes its child window ⇒ **expected to reproduce on mac** despite the ticket's "no issue on mac" (method unrecorded) — measure, do not assume | mac relay: the overlay's parent window and frame | T3 (relay) | ⬜ |
| `P3-B1` | Find "wallet" in A → switch to B ⇒ bar hidden; Ctrl+F in B ⇒ field shows "wallet", selected, and the count is **B's** | Today's binary ⇒ bar stays open, count empty until typing (ticket RED) | The `find_text` IPC arm's log naming the **target browser id**, mapped to the active tab via `TabManager` (not the previous tab); the count shown | T2 (+T3 keys) | ⬜ |
| `P3-B2` | Back to A ⇒ A's query and count restored; closing A's find clears A's highlights | Drop the per-tab restore ⇒ A's bar reopens empty | Same log + the rendered count | T2 | ⬜ |
| `P3-B3` | Two windows: find in window B never opens or changes the bar in window A (the earlier Phase 3 fix stays fixed) | Key the new per-tab store on the global active tab ⇒ A's bar changes | Two headers' `findBarVisible` state, one process | T2 | ⬜ |

**Two-sided pairs:** L1 ↔ L2 · Q1 ↔ Q4 (claimed when it should be / not claimed when it must not be) · Z1 ↔ Z2 · B1 ↔ B3.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `cef-native/src/core/ProfileLock.cpp :: AcquireProfileLock` (Win32 arm: `CreateFileA` + 6× retry; POSIX arm: `open(O_CREAT)` + retry) | ✅ | both arms in one file — L changes both |
| `cef-native/src/core/ProfileLock.cpp :: IsProfileLockedByAnotherInstance` | ✅ | already distinguishes not-found from sharing — the pattern to copy (SCOPE §4 #8) |
| `cef-native/cef_browser_shell.cpp` — `AcquireProfileLock(profile_cache)` caller + "Profile Locked" `MessageBoxA`; `cef_browser_shell_mac.mm` — its caller | ✅ | ⚠️ `SingleInstance::StartListenerThread` precedes the lock; the stop+join on this failure path must survive (ticket "Related") |
| `frontend/src/pages/BRC100AuthOverlayRoot.tsx :: InfoIcon` | ✅ | `position:'relative'` span, absolutely positioned popup, no clamp; 3 call sites |
| `cef-native/include/core/WalletActivityTracker.h :: WalletActivityTracker::NoteWalletRequest`, `IsTalkingToWallet` | ✅ | last-request time only — no in-flight count |
| `cef-native/src/core/HttpRequestInterceptor.cpp :: HttpRequestInterceptor::GetResourceHandler` (the only `NoteWalletRequest` call) · `HandleIpcWalletCall` (none) | ✅ | Q3's finding |
| `cef-native/src/handlers/simple_handler.cpp :: ShowDeferredPermissionTask`, `kWalletQuietMs = 1200` | ✅ | the deferral; ⛔ do not raise the constant (ticket) |
| `cef-native/src/core/HttpRequestInterceptor.cpp :: IsConnectModalType` | ✅ | unchanged (Q4) |
| `rust-wallet/src/handlers.rs :: health`; `rust-wallet/src/main.rs` — `.bind(("127.0.0.1", wallet_port()))?` | ✅ | stateless `/health`; bind exits on failure — **changed only if F0 shows user impact** (§12 Q3) |
| `cef-native/cef_browser_shell.cpp :: LaunchWalletProcess` (adopt branch on `IsPortListening(hodos::WalletPort())`), `WaitForWalletHealth`, `StartBackendSupervisor` | ✅ | read by F0; ⚠️ **T5-P4 edits the same adopt branch** — serialize (§11) |
| `cef-native/cef_browser_shell_mac.mm :: SpawnWalletServer`, `IsPortListeningMac` | ✅ | mac counterpart, read-only here |
| `cef-native/update-helper/transaction.cpp :: HttpPostShutdown` call sites (5 literals `31301`/`31302`) | ✅ | F1 |
| `cef-native/include/core/PortConfig.h :: hodos::WalletPort`, `AdblockPort` | ✅ | header-only (`inline`), includes only `<cstdlib>`/`<string>` — usable from the helper target |
| `cef-native/src/core/WalletService.cpp :: WalletService::startDaemon`, `cleanupDaemonProcess` | ✅ | nothing calls `startDaemon` — **reported**, not deleted (rule 3) |
| `cef-native/src/handlers/simple_app.cpp :: CreateNotificationOverlay` (keep-alive `SetWindowPos(... GetWindowRect(g_hwnd) ...)`; fresh `CreateWindowEx(..., g_hwnd, ...)`) | ✅ | Z's cause by code reading. Signature carries no window — callers must pass the requesting browser's window |
| Callers: `HttpRequestInterceptor.cpp` (`CreateNotificationOverlayTask`), `simple_handler.cpp :: ShowDeferredPermissionTask` + the permission-prompt path | ✅ | each must pass a window (or browser) |
| `cef-native/cef_browser_shell_mac.mm :: CreateNotificationOverlay` (`[g_main_window frame]`, `addChildWindow`) | ✅ | mac twin |
| `frontend/src/components/FindBar.tsx :: handleInputChange` (`find_text` only on input change); `frontend/src/pages/MainBrowserView.tsx` `findBarVisible` | ✅ | per-window today |
| `cef-native/src/handlers/simple_handler.cpp` — `find_text` / `find_stop` IPC arms | ✅ | target browser choice |

## 6. Out of scope

The Rust half of the fast-relaunch source plan (write intent before broadcast — T3's D7, `track-3a` P4). A not-ready `/health` state and a bind retry **unless** F0 shows user impact (§12 Q3). Deleting the daemon code. Redesigning the tooltip system (owner: *"possible rabbit hole"*). Any other overlay's owner/position (P9). Chromium's find-in-page API (non-functional in our build — root CLAUDE.md). The file-thread ticket (deferred, measure-first). Widening `IsConnectModalType`.

## 7. Rollback

Six commits, one per item (L, T, Q, F, Z, B); each reverts alone. No data format changes; no wallet schema.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| L recreates the folder for a profile whose data was **lost** — the user never learns their data is gone | §12 Q2 (notice or silent); L1 records what the user sees |
| L's stop+join ordering for `SingleInstance` is broken on the new path ⇒ `std::terminate` at exit | L2 exits cleanly (exit code recorded) |
| T "fixed" at 100% by clamping to the viewport, still overflowing the **modal** at 150% | T1 asserts the modal box, T2 by eye at three cells |
| Q counts in-flight only on HTTP; shim dApps (IPC bridge) still read quiet — green on the ticket's site, red on a CWI dApp | Q3 |
| Q's counter leaks (a request that errors never decrements) ⇒ host "busy" forever ⇒ prompt waits the full 15 s every time | Q2 + a Q1 variant where the wallet returns 500 — decrement on every completion path, logged |
| F's port fix passes in dev but the helper runs in **production** with no `HODOS_DEV` ⇒ must still resolve 31301 | F1 run twice: dev (31401) and prod-shaped env (31301 stub) |
| F0 is run once, sees nothing, and "not observed" becomes "fixed" | F0 is 9 runs across three delays; outcome recorded as a measurement, never green |
| Z threads a window into the overlay, but the keep-alive path still re-positions to `g_hwnd` on the **second** prompt | Z1 runs two prompts in a row from B |
| Z's new owner changes which activation path hides the notification overlay ⇒ a prompt closes when the user clicks the requesting window | R-CLOSE at boundary; Z3 clicks inside B while the prompt shows |
| B keys find state on `CefBrowser::GetIdentifier()` in one place and `Tab::id` in another | B1 SUBJECT maps browser id → tab id through `TabManager` explicitly |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | `P3-LM`, `P3-ZM`, plus T/Q/B via relay confirm (shared React + shared `simple_handler.cpp`) | L's POSIX arm lives in the same `ProfileLock.cpp`; Z's mac twin is `cef_browser_shell_mac.mm :: CreateNotificationOverlay`; F is **Windows-only** (`update-helper` is Windows-only; mac uses Sparkle) — its F0 measurement relays once for the mac adopt branch (`SpawnWalletServer`) |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~2** — T2 at three DPI cells (0.5), Z3 two-window sitting (0.5, **shared with B5-T6-P2's sitting**), B keys (0.25), F0 observation once (0.25), L2/Q by agent. Mac relay ~0.75 |
| Human-bound rows | `P3-T2` (DPI by eye), `P3-Z3` (native clicks, two windows), `P3-B1`/`B2` key presses (CDP key events do not reach `OnPreKeyEvent` — memory), `P3-LM`, `P3-ZM` |
| Unknowns (K) | **K4** fast relaunch user impact (F0) · **K-Z** whether "behind" is the `g_hwnd` placement or a real Z-order loss (Z0) · **K-Q** whether `/waitForAuthentication` in the teragun trace went through HTTP or the IPC bridge (Q3 answers) — **K = 3** |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| ⛔ serialize ↔ | **B5-T5-P4** one OS account, one wallet | Both edit `cef_browser_shell.cpp :: LaunchWalletProcess`'s adopt branch and the supervisor. F here only **measures** and edits the helper; if §12 Q3 adds a not-ready `/health`, T5-P4's ownership proof must see it — land T5-P4 first, then F's optional part |
| coordinate ↔ | **B5-T3a-P4** chain/crash safety (decision D7) | a not-ready `/health` during shutdown and the crash-safe reconcile run at the same wallet start/stop — design together if §12 Q3 = yes |
| shares → | **B5-T6-P2** | Z3 runs in P2's two-window sitting |
| gives → | **B5-T6-P7** brand prompts | P7 adds ~21 prompt types to the same shared overlay — Z must land first or every new prompt inherits the wrong-window placement |
| gives → | **B5-T6-P9** window globals | Z converts one `g_hwnd` family by hand; P9 owns the rest and the G11 ratchet |
| needs ← | **B5-T0 Build 1** | browser-level evidence on the refreshed engine |

## 12. Open questions for the owner

1. **Placement (working rule 1 — I did not pick silently).** Two tickets filed after the scope — *prompt opens behind the other window* (Z) and *find bar does not follow tab switches* (B) — are assigned to T6 in the register but to **no phase**. B5-T6-P2 already points at P3 for Z. ⭐ Recommend **both in P3** (Group 1): Z is a public-build consent-surface defect that looks like "the wallet froze"; B is frontend-mostly and owner-raised. Alternative: B into P9 (Group 3, first to cut).
2. **Item L — silent or noticed?** A profile whose folder vanished is recreated empty. ⭐ Recommend **recreate + a one-line, non-alarming notice** ("This profile's data folder was missing and has been recreated") — silent recovery would hide lost data (the ticket's own caution).
3. **Item F — if F0 shows a user-visible "no wallet" after a fast relaunch**, fix it in beta.6 (not-ready `/health` + bounded bind retry — a Rust change coordinated with T3a D7 and T5-P4), or ticket it for beta.7? ⭐ Recommend: **decide on F0's evidence**; if nothing is observed, ticket it.
4. **Found at G3 (code reading, not measured):** the loopback-deferral's activity tracker never hears wallet traffic that arrives over the `wallet_call` IPC bridge — the path the CWI shim uses. Folded into Q as row Q3; flagged so the owner knows the defect may be wider than the ticket's one site.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded (R-CLOSE, Manage Site Permissions, R-INTEXT named above)
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons (G11 must not rise — its pattern does not match `g_hwnd`, so Z does not move it)
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] C++ touched ⇒ `MAC_RELAY_BETA5.md` round names the files — flags `ProfileLock.cpp` (shared `#ifdef`), `simple_handler.cpp`, and the mac `CreateNotificationOverlay` twin
- [ ] Dead daemon code reported (ticket or AAR line), not deleted
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
