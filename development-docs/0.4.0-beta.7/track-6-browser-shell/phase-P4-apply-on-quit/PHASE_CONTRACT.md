# B5-T6-P4 — a silent update is applied after the user quits, not while they wait at launch · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (G3, 2026-09-28). ⛔ Documents only — no code has been written for this phase.

**Track:** B5-T6 Browser shell · **Group:** 2 (should ship; cut **last** of the cuttable phases — T6 Q8) · **Tickets:** `../../tickets/TICKET_update_is_visible_when_it_should_not_be.md` — **Tier 2 only** (Tier 1 = B5-T6-P1; Tier 3 deferred) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent for T6 (resumed run) · **Platforms:** Windows only (§9) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **T6 Q3** — Tier 1 in Group 1, **Tier 2 (this phase) in Group 2**. **T6 Q8** — cut order P10 → P9 → P7 → P6 → **P4 last**. **14**. Carried, not reopened.

⚠️ The ticket calls this path *money-adjacent*: *"its acceptance test is two signed builds on a real install, never a rig alone."* Carried as a hard rule (row `P4-A8`).

---

## 1. Goal

A user on automatic updates never watches the *"Hodos is updating…"* splash: the staged update is applied after the last Hodos window closes, and the next launch simply is the new version.

## 2. Done means

- [ ] Silent mode, update staged, user quits (Exit, last-window X): the helper applies after **every** Hodos browser process of this install has exited; next launch shows **no** splash and runs version N
- [ ] Multi-profile: with two profiles open, quitting the first does **not** apply; quitting the last does
- [ ] A failed apply (corrupt installer, locked file, killed mid-apply) rolls back to N−1 and the next launch runs N−1 healthy — `R-UPDATE` holds with the apply moved
- [ ] A launch **during** an apply-on-quit waits behind the update lock with the splash (the only time anyone sees it) and never starts against half-swapped files
- [ ] Windows sign-out / restart does **not** start an apply (the OS would kill the installer); the startup apply remains the fallback
- [ ] Notify and Off modes: unchanged (no apply-on-quit)
- [ ] The startup apply (`MaybeApplyStagedUpdate`) stays as the fallback for anything staged but not applied at quit

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-UPDATE` | N−1 → N applies; a corrupted installer is refused and rolled back | The apply moves from a supervised launch to an unsupervised quit — the helper's health proof today depends on relaunching the browser (`transaction.cpp :: WaitForHealthy` waits for the relaunched process to write healthy). Nobody relaunches on quit |
| Auto-update stability (memory: *never force reinstall, never brick*) | No user stranded, no broken install | A half-applied install found at the next launch, with no helper running, is a brick |
| Code-signature chain (CEF 150 bootstrap: `HodosBrowser.exe` verifies `HodosBrowser.dll` + `chrome_elf.dll`) | Never a mixed-signer set | An apply interrupted between files leaves exactly that |
| Wallet shutdown | Wallet backend stops cleanly on quit | The helper also stops wallet/adblock (`HttpPostShutdown`) — the quit path already did; a double stop must be harmless |
| `R-CLOSE` / P2's quit path | Exit = `ShutdownApplication` | P4's trigger lives in the quit path P2 rewires |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. Controls authored here (update path — not money/schema/crypto by the §4.2 definition; the ticket's "money-adjacent" caution is honoured by `P4-A8` and by running every RED on a scratch install).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P4-S0` | **Step 0 (read, rule 4):** the helper's resume/health machinery read and written down — what proves an apply healthy today, what `ArmRunOnce` / `isResume` do, and what the next launch does with an `ApplyRecord` left in each phase | — a reading row; the design in §12 Q1 is chosen **after** it | `transaction.cpp :: RunApplyTransaction`, `WaitForHealthy`, `DoRollback`, `ArmRunOnce`; `cef_browser_shell.cpp :: MaybeApplyStagedUpdate` | T0 | ⬜ |
| `P4-A1` | Rig (N−1 → N, local signed pair, silent): quit ⇒ helper applies after exit; next launch: **no splash window**, version N in `About` and in the log's version line | Same binaries with apply-on-quit disabled (rig env) ⇒ next launch shows the splash and applies at startup (today's behaviour, seen) | Top-level windows at next launch enumerated (the helper's splash window class/title) + `ApplyRecord` phase before and after + installed file versions | T2 | ⬜ |
| `P4-A2` | The helper does not act until **all** browser processes of this install have exited | Start the apply while a second profile's `HodosBrowser.exe` is alive (skip the wait in a rig build) ⇒ `PollUnlocked` / `CountSiblingBrowsers` refuse (`app-locked` / `sibling-present`) — proves the check sees siblings | `CountSiblingBrowsers` result logged; process list (path-matched) at apply start | T2 | ⬜ |
| `P4-A3` | Two profiles open: quit profile 1 ⇒ **no** apply starts; quit profile 2 ⇒ apply starts | A2 ↔ A3 pair: a trigger that fires on any quit passes A1, fails A3 | Helper spawn log line with the PID that spawned it | T2 | ⬜ |
| `P4-A4` | Corrupt the staged installer after signing ⇒ apply-on-quit **refuses** (integrity gate) and N−1 launches healthy next time | `R-UPDATE`'s own RED, run on the quit path: disable the integrity gate in a rig build ⇒ the corrupt installer runs (seen on a scratch VM only) | `IntegrityGate` log line; installed file hashes; next launch healthy marker | T2 | ⬜ |
| `P4-A5` | Kill the helper mid-install (`taskkill /F` during `Installing`) ⇒ next launch completes rollback or resume **before** starting the browser; never a mixed-signer set | Same kill with the next-launch recovery disabled ⇒ bootstrap `LOG(FATAL)` / mixed versions (seen on a scratch install) | `ApplyRecord` phase at kill; signer of the three bootstrap binaries after recovery | T2 | ⬜ |
| `P4-A6` | User relaunches while apply-on-quit runs ⇒ the new launch waits on the update lock with the splash, then starts N | Remove the lock wait in a rig build ⇒ launch starts against half-swapped files (crash or mixed version) | `UpdateLock` acquire/wait log lines from both processes | T2 | ⬜ |
| `P4-A7` | Sign-out / restart (`WM_ENDSESSION`, from B5-T6-P2) ⇒ **no** apply starts; the next launch's startup apply handles it | Trigger apply-on-quit from the session-end path in a rig build ⇒ the installer starts during logoff (seen in the log; outcome recorded) | Helper spawn log absent on the session-end path; next launch shows the startup apply | T2 (+T3 real sign-out, shared with `P2-B2`) | ⬜ |
| `P4-A8` | **Two real signed builds, real install** (owner's machine, public-style feed): silent mode, quit ⇒ next launch is N, no splash seen by the user | — cite A1's RED; a real-install RED would require shipping a broken build | The **installed** app (not dev), its installed version, owner's eyes on the next launch | T3 👤 | ⬜ |
| `P4-A9` | Notify and Off modes: quit ⇒ no apply-on-quit | Force mode silent on the same profile ⇒ apply starts (proves the mode check is read) | `Update mode set to:` log line + helper spawn absent | T2 | ⬜ |
| `P4-A10` | Nothing staged ⇒ quit is exactly as fast as today (no helper spawned) | Stage a marker without an installer ⇒ helper spawns and aborts `no-installer` (proves the "staged?" check is what gates) | Helper spawn log; quit duration logged | T2 | ⬜ |

**Two-sided pairs:** A2 ↔ A3 (waits for all / still applies at the last) · A1 ↔ A9 (silent applies / other modes do not) · A4 ↔ A1 (refuses bad / applies good).

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `cef-native/cef_browser_shell.cpp :: MaybeApplyStagedUpdate` (static; inert in dev unless `HODOS_UPDATE_TEST`; `state.silent` required) | ✅ | the startup apply — **kept** as the fallback |
| `cef-native/cef_browser_shell.cpp :: ShutdownApplication` | ✅ | where the quit trigger would go (after `SaveSession` and the backend stop) |
| `cef-native/update-helper/transaction.cpp :: RunApplyTransaction` (splash first, `PollUnlocked`, `CountSiblingBrowsers` ⇒ `sibling-present` abort, installer `/VERYSILENT`) | ✅ | the machinery — Tier 2 moves the **trigger**, not the machinery (SCOPE §3) |
| `cef-native/update-helper/transaction.cpp :: CountSiblingBrowsers`, `PollUnlocked`, `WaitForHealthy`, `DoRollback`, `ArmRunOnce`, `IntegrityGate`, `HttpPostShutdown` | ✅ | K5 is partly answered already: siblings are **detected** (abort), not **waited for** |
| `cef-native/update-helper/splash.h` | ✅ | stays; seen only by A6's relauncher |
| `cef-native/src/core/UpdateApply.cpp`, `UpdateFs.cpp`, `UpdateStager.cpp`; `cef-native/include/core/UpdateLock.h` | ✅ | stager + lock; unchanged unless S0 says otherwise |
| `cef-native/src/core/AutoUpdater.cpp :: AutoUpdater::SetShutdownCallback` path (WinSparkle notify install) | ✅ | notify mode — untouched (A9) |
| B5-T6-P2's `WM_ENDSESSION` handler (does not exist yet) | ⬜ owed by P2 | A7 needs it |

## 6. Out of scope

Tier 3 (versioned side-by-side directories, delta updates). macOS (Sparkle already installs on quit — ticket, relay 23j). WinSparkle's notify-mode install path. The update-mode UI. Changing the installer. A "Restart to update" nudge.

## 7. Rollback

One commit adding the quit trigger (plus the helper mode it needs); reverting it restores the startup apply, which stays in place throughout. `ApplyRecord`/`UpdateState` formats unchanged — or, if S0 needs a new phase value, older helpers must treat it as "not applied" (tested by A5).

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| The apply succeeds on disk but N is broken; nobody relaunched, so no health check ran and no rollback happened ⇒ the user's next launch crashes with no helper watching | S0 + §12 Q1 design; A5 variant: ship a rig N that exits immediately ⇒ next launch must roll back |
| Second profile still open ⇒ helper aborts `sibling-present` silently every quit ⇒ never applies, and the startup apply also aborts ⇒ user stranded on N−1 | A3 + A10; stranding is caught by `R-UPDATE` "no user stranded" |
| Installer runs during Windows sign-out and is killed ⇒ mixed-signer set | A7 (never trigger on session end) + A5 (recovery if it happens anyway) |
| The quit path now waits for the helper ⇒ quitting feels slow — the complaint moved, not removed | A10 records quit duration; the helper must be spawned **detached** |
| A rig pair passes, the real signed pair does not (signature, AV, installer UAC) | A8 is mandatory; the ticket's rule |
| Dev rig's helper sends `/shutdown` to the **installed** wallet's port | B5-T6-P3 `P3-F1` (port helpers) lands first |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | none — **Windows-only**, because Sparkle already installs on quit on macOS (📄 ticket Tier 2; 📏 macOS relay 23j measured install-on-quit completing even when the host aborts). No `_mac` counterpart is written |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **4–5** — 2–3 rounds × two signed builds on a real install, incl. one multi-profile round (SCOPE §7). The costliest owner-hours in the track |
| Human-bound rows | `P4-A8` (real install, signed pair); `P4-A7` real sign-out (shared with B5-T6-P2 `P2-B2`) |
| Unknowns (K) | **K5** "all processes exited" across profiles (partly answered: `CountSiblingBrowsers` exists and aborts) · **K5b** who proves an apply-on-quit healthy when nothing relaunches (S0 → §12 Q1) — **K = 2** |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs ← | **B5-T6-P1** silent means silent | P4's rig reuses P1's local-feed seam; P4 assumes silent mode is actually silent |
| needs ← | **B5-T6-P2** Exit quits + session end | the quit trigger lives in the path P2 rewires; A7 needs P2's `WM_ENDSESSION` handler |
| needs ← | **B5-T6-P3** item F | helper ports through `PortConfig.h` before any rig run |
| needs ← | **B5-T0** signed builds | every A8 round needs two signed builds from the release pipeline — schedule with T0's build windows |

## 12. Open questions for the owner

1. **Who proves the new version healthy?** Today the helper relaunches the browser and waits for it to write "healthy", rolling back if it does not. On quit, nothing relaunches. ⭐ Recommend: **the next launch completes the proof** — the apply leaves a "pending health" record, the next launch runs under the helper's supervision as today's resumed apply does, and rolls back if unhealthy. Alternative: the helper relaunches the browser hidden and closes it — rejected as surprising (a browser starting after the user quit). Final shape after S0.
2. **Cut line reminder:** if P4 is cut at G5.5, B5-T6-P1 alone still removes the dialog; the splash stays. Confirmed as the scope's order — no action unless you want P4 protected.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded (**R-UPDATE with a real N−1 → N**, not only the scripts)
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] C++ touched ⇒ `MAC_RELAY_BETA5.md` round names the files (Windows-only — say so, so mac does not wait for it)
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
