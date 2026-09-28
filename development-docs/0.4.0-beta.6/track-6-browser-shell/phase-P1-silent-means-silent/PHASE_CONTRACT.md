# B5-T6-P1 — silent updates stay silent (no update dialog for users on automatic) · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (G3, 2026-09-28). ⛔ Documents only — no code has been written for this phase.

**Track:** B5-T6 Browser shell · **Group:** 1 (keep even if the release runs long) · **Tickets:** `../../tickets/TICKET_update_is_visible_when_it_should_not_be.md` — **Tier 1 only** (Tier 2 = B5-T6-P4; Tier 3 deferred) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent for T6 · **Platforms:** Windows (fix) · macOS (confirm only) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **14** (T6 keep set), **T6 Q3** (Tier 1 in Group 1, Tier 2 in Group 2 — per the scope's recommendation, approved). Carried, not reopened.

---

## 0. Step 0 — measure WHICH cause fired, before any code (K1)

The owner saw an update dialog on the `0.3.0-beta.29 → 0.4.0-beta.4` self-update. Two candidate causes, both
re-verified by code reading today, **neither measured**:

| # | Candidate | Code (verified 2026-09-28) |
|---|---|---|
| **C-a** | WinSparkle's **own scheduled check** is enabled in **silent** mode, and a scheduled check that finds an update shows WinSparkle's UI | `cef_browser_shell.cpp` (updater init block, after windows are created): `autoCheck = (autoUpdateMode != "off")` → `AutoUpdater::Initialize(…, autoCheck)` → `win_sparkle_set_automatic_check_for_updates(autoCheck)`; `AutoUpdater.cpp :: AutoUpdater::SetUpdateMode` also enables it for `mode != Off`. Its own comment already says a WinSparkle prompt "would defeat silent" — and only guards the **forced** check, not the **scheduled** one |
| **C-b** | The owner's profile is genuinely in **notify** mode: a legacy `autoUpdateEnabled=true` migrates to `"notify"` (`SettingsManager.h`, legacy migration block), and the one-time global collapse takes the **most conservative** mode across all profiles (`cef_browser_shell.cpp`, silent-mirror block: `GlobalUpdateModeWasAbsentAtLoad` → `MoreConservativeMode`) | A `0.3.0-beta.29` upgrader is exactly the legacy case |

**Step 0 (read-only, ~15 min, agent + one owner file):** read the owner's **global** `settings.json` `updateMode`
and each profile's `autoUpdateMode`/`autoUpdateEnabled`; read the beta.4 shell log for `Update mode set to:`,
`Auto-check enabled`, `Silent mirror: one-time global update-mode collapse -> …`, and WinSparkle's
did-find-update callback line.

| Step 0 finds | Then |
|---|---|
| mode was **silent** and WinSparkle found the update on a scheduled check | C-a — proceed with this contract as written |
| mode was **notify** (legacy migration or the collapse) | ⛔ **Stop — §12 Q1.** The dialog was *correct* for notify mode; "fix" means changing the deliberate legacy→notify consent rule, which is the owner's call, not this phase's |
| both | fix C-a here; C-b goes to §12 |
| the log cannot tell | record "not reproduced", run P1-A1's RED rig first to prove C-a can produce the dialog at all |

## 1. Goal

A user whose update mode is **automatic (silent)** never sees an update dialog; a user on **notify** still does; and **Settings → About → Check for updates** still shows an interactive check to anyone who asks.

## 2. Done means

- [ ] Step 0 recorded (which cause, with the log line or setting that shows it)
- [ ] Silent mode, N−1 build, feed offering N: **no WinSparkle window appears** over a full scheduled-check interval **and** at launch, while the silent stager still stages N
- [ ] Notify mode, same rig: the WinSparkle dialog **does** appear
- [ ] Silent mode: Settings → About → Check for updates opens WinSparkle's interactive UI
- [ ] Off mode: no check of any kind (unchanged)
- [ ] macOS: confirmed by reading + one relay observation that silent mode shows no Sparkle dialog (no code change expected)

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-UPDATE` | A staged update still applies N−1 → N; a corrupted installer is refused and rolled back | Turning off WinSparkle's scheduled check in silent mode must not turn off the **stager** (`HODOS_SILENT_AUTOUPDATE` staging thread) — the thing that actually updates silent users |
| Auto-update stability (memory: *auto-update must NEVER force reinstall*) | No user is stranded on an old build | A gate that silences the dialog **and** the only working update path would strand silent users silently |
| `R-CLOSE` | untouched | WinSparkle's shutdown-request callback posts `WM_CLOSE` to `g_hwnd` — not changed here (see P2 §12) |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. Not a money/schema/crypto phase — controls authored here.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P1-S0` | Step 0 names the cause with a quoted log line / setting value | — (a measurement row; "cannot tell" is a valid, recorded outcome, never upgraded) | The **owner's** beta.4 install's global `settings.json` + that install's shell log — not a dev profile | T2 | ⬜ |
| `P1-A1` | Silent mode, rig feed offers N: no WinSparkle window during launch + one forced scheduled check | Same binary with the gate reverted (WinSparkle auto-check left on in silent) ⇒ the WinSparkle dialog **appears** on the same rig. ⚠️ Must be seen, or the rig cannot produce the dialog and A1 is vacuous | Top-level windows of the **dev** `HodosBrowser.exe` PID enumerated (`EnumWindows` + class/title) — a WinSparkle window, not a Hodos overlay; plus WinSparkle's registry `LastCheckTime` moved (proves a check actually ran) | T2 | ⬜ |
| `P1-A2` | Notify mode, same rig: the dialog **appears** | A1 and A2 are each other's control: a fix that simply disables WinSparkle everywhere passes A1 and fails A2 | Same window enumeration; `Update mode set to: notify` in this run's log | T2 | ⬜ |
| `P1-A3` | Silent mode: Settings → About → Check for updates shows WinSparkle's interactive UI | Stub `CheckForUpdatesInteractively` to a no-op ⇒ nothing appears (proves the row sees the interactive path, not the scheduled one) | The click arrives via the `SettingsPage` tab's IPC (`simple_handler.cpp` interactive-check arm) — log line from that arm, then the window | T2 (+T3 click) | ⬜ |
| `P1-A4` | Silent mode still **stages** N while no dialog shows | Disable the staging thread (rig env) ⇒ nothing staged in `AppPaths::GetPendingUpdateDir()` — proves A4 reads the stager | Pending-update dir contents + stager log lines, same run as A1 | T2 | ⬜ |
| `P1-A5` | Off mode: no check, no dialog | Set mode notify on the same profile ⇒ check runs (LastCheckTime moves) | WinSparkle registry `LastCheckTime` unchanged across the run | T2 | ⬜ |
| `P1-A6` | **Real N−1 → N**, owner's machine, silent mode: no dialog; update staged | — cite A1's RED; a real-install RED would need shipping a broken build | The installed (non-dev) app, real public feed, owner's eyes | T3 | ⬜ |
| `P1-M1` | macOS silent mode: no Sparkle dialog (`AutoUpdater_mac.mm :: SetUpdateMode` sets `automaticallyDownloadsUpdates = YES`) | Notify mode on the same mac build ⇒ Sparkle's dialog appears | macOS relay observation on the mac build, mode read from its log | T3 (relay) | ⬜ |

**Two-sided rows:** A1 ↔ A2 (silent hides / notify shows). A1 ↔ A4 (no dialog / still updates).

⚠️ **Rig note (rule 4, WinSparkle docs to be read and cited at kickoff):** the WinSparkle appcast URL is hard-coded
(`https://hodosbrowser.com/appcast.xml`) in the updater init block; the existing `HODOS_UPDATE_TEST_SEAM` /
`HODOS_UPDATE_RIG_URL` seam redirects only the **stager**. A rig for A1/A2 needs WinSparkle pointed at a local
feed under the same test seam (compiled out of production), and WinSparkle's per-user registry `LastCheckTime`
reset so a scheduled check fires inside the run (its minimum interval is 1 h — `AutoUpdater::SetCheckInterval`).

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `cef-native/src/core/AutoUpdater.cpp :: AutoUpdater::Initialize` | ✅ | sets `win_sparkle_set_automatic_check_for_updates(autoCheck)` |
| `cef-native/src/core/AutoUpdater.cpp :: AutoUpdater::SetUpdateMode` | ✅ | `mode != Off` enables the scheduled check; forced check is notify-only (already correct) |
| `cef-native/cef_browser_shell.cpp` — updater init block (`auto& updater = AutoUpdater::GetInstance()`, `autoCheck = … != "off"`) | ✅ | the other place silent mode turns WinSparkle's timer on |
| `cef-native/cef_browser_shell.cpp` — silent-mirror block (`MirrorSilentEligibility`, `GlobalUpdateModeWasAbsentAtLoad`) | ✅ | read-only for this phase; Step 0 reads its log line |
| `cef-native/include/core/SettingsManager.h` — legacy `autoUpdateEnabled` → `autoUpdateMode` migration | ✅ | `legacy ? "notify" : "off"` — deliberate; ⛔ not changed here |
| `cef-native/src/handlers/simple_handler.cpp` — `SetUpdateMode` / `CheckForUpdatesInteractively` arms | ✅ | settings change path + the interactive check |
| `cef-native/src/core/AutoUpdater_mac.mm :: AutoUpdater::SetUpdateMode` | ✅ | macOS: Sparkle silent-downloads in silent mode — confirm only |

## 6. Out of scope

Apply-on-quit (B5-T6-P4). Tier 3 (versioned side-by-side dirs). Changing the legacy→notify migration (§12 Q1).
Any change to the stager, the helper, signatures or the appcast. The WinSparkle shutdown-request `g_hwnd` site
(P2 §12). A UI to explain update modes.

## 7. Rollback

Revert the one commit that gates WinSparkle's automatic check on `mode == Notify`. No data written; settings format unchanged.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| The fix gated the wrong cause (owner was really in notify) — dialog "fixed" in the rig, still appears for him | `P1-S0` first; §0 stop rule |
| Silencing WinSparkle also silenced the only update path for silent users (stager disabled or not compiled) | `P1-A4`; `R-UPDATE` at boundary |
| Rig's "no dialog" because WinSparkle never checked (interval not elapsed) — vacuous green | `P1-A1` SUBJECT requires `LastCheckTime` to move; RED must show the dialog |
| Notify users lose their dialog | `P1-A2` |
| User switches silent → notify at runtime and gets nothing until restart | add to A2: switch mode via Settings in the same session, then force a check |
| A Hodos overlay mistaken for WinSparkle's window (or vice versa) | SUBJECT: window class/title of the dev PID |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | S0, A1–A6 | the fix |
| macOS | M1 | confirm only — Sparkle's silent mode already downloads without UI; no code change expected. If M1 fails, that is a new finding for §12, not silently fixed here |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1** — share his settings/log (5 min) + one real N−1→N observation (A6) |
| Human-bound rows | `P1-A6` (real install), `P1-M1` (mac relay) |
| Unknowns (K) — uncertainty, not difficulty | **K1** which cause fired (Step 0). K1b whether WinSparkle can be pointed at a local feed without a production-visible seam |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| gives → | B5-T6-P4 apply-on-quit | P4 builds on a silent path that is actually silent; P4's rig reuses P1's local-feed seam |
| needs ← | B5-T0 Build 1 (`v0.4.0-beta.5`) | none for code; A6 is observed on whatever real N−1→N pair ships first — if that is the security release, record it there |

## 12. Open questions for the owner

1. **Only if Step 0 finds C-b:** a `0.3.x` upgrader with the old "auto-update on" switch is migrated to **notify**, deliberately (the old switch never meant silent install). Keep that, or promote those users to silent (which installs updates they never explicitly agreed to)? ⭐ Recommend **keep**, and add a one-time notice offering silent — but that is a new feature, so it would be a ticket, not this phase.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] C++ touched ⇒ `MAC_RELAY_BETA5.md` round names the files (branch rule)
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
