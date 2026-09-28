# B5-T6-P9 — finish moving window work off process globals, so one window never acts on another · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (G3, 2026-09-28). ⛔ Documents only — no code has been written for this phase.

**Track:** B5-T6 Browser shell · **Group:** 3 (could ship — **first cut at G5.5**; cut order P10 → **P9** → P7 → P6 → P4) · **Tickets:** `../../tickets/TICKET_window_scoped_work_uses_process_globals.md` — **the remainder** (symptoms 1–2 fixed in beta.3 Phase 3; layout cluster in 3.5) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent for T6 (resumed run) · **Platforms:** Windows (DPI part) · both (accessor part) — §9 · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **14** (Group 3, first to cut). The ticket's rule: the test is never "is it global" but **"one value per process, or one per window?"** — ⛔ `g_file_dialog_active` and `g_wallet_overlay_prevent_close` are correctly global and are **not** converted. Working rule 6: G11's pattern/baseline change is **its own commit**, reason in `HARNESS.md` §4, `-NegativeControl` re-run. Carried, not reopened.

---

## 1. Goal

With two windows of one profile open — on monitors with different scaling — every dropdown overlay opened from window B is sized and placed for window B, and no remaining shell code resolves "which window?" to the primary window when it means "this window".

## 2. Done means

- [ ] **Item D — DPI sites:** the `ScalePx(x, g_hwnd)` calls in `simple_app.cpp`'s `Create*Overlay` functions (📏 **57** today) take their scale from the **target** window; the `Show*Overlay(…, targetWin)` re-show path re-scales too (SCOPE §4 #4: not checked)
- [ ] **Item S — backward-compat accessors:** each of the static `Get*Browser()` accessors that route through `GetPrimaryWindow()` (📏 **17** in `simple_handler.cpp`) is either retired (callers moved to `GetOwnerWindow()`) or kept with a one-line reason why the primary is correct
- [ ] **Item J — judgement-call sites:** every remaining `PostMessage(g_hwnd, WM_CLOSE…)` (📏 3 in `simple_handler.cpp`, 1 in `cef_browser_shell.cpp`) read individually and labelled *close my window* / *quit* / *correct as is* — the Exit and WinSparkle ones are **B5-T6-P2's**, not re-done here
- [ ] **Item G — the ratchet:** G11 baseline lowered to the count the script measures after S (own commit); a **separate** commit, if the owner agrees (§12 Q1), widens G11's pattern to see `g_hwnd` in window-scoped code, baselined by the script
- [ ] Two-window, mixed-DPI evidence on real hardware — or the DPI half recorded as **not run** with the reason (K10)

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-CLOSE` | Close guards | ⛔ the two correct globals stay global; converting accessors changes which overlay a close IPC reaches |
| `R-GOLD` | Gold pill on the originating tab | Accessor changes touch window→tab mapping; `Tab::id ≠ CefBrowser::GetIdentifier()` |
| DPI matrix | cells #4/#6/#9 | Item D **is** a DPI change; cell #9 **with two windows** has never been run |
| CEF lifecycle (invariant 8) | window creation timing | Moving a lookup must not move when a window/browser is created |
| G11 | ratchet cannot rise | the gate this phase drives down |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. Not a money/schema/crypto phase — controls authored here. ⛔ **SUBJECT for every two-window row:** two windows of the **same** profile in **one** process (`Win32_Process`: exactly one non-`--type=` dev `HodosBrowser.exe`) — two profiles are two processes and prove nothing (the ticket's vacuous-test trap).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P9-D1` | Unit/T1: the overlay geometry helper, given a target window at 150% while the primary is at 100%, returns 150%-scaled offsets | Pass `g_hwnd` (today) ⇒ 100%-scaled offsets | The helper's return for a synthetic pair of `GetDpiForWindow` values | T1 | ⬜ |
| `P9-D2` | Mixed-DPI hardware (cell #9, two windows): each dropdown overlay (menu, downloads, cookies, bookmarks, profile, site-info, tab-list, settings-menu, omnibox) opened from window B on the 150% monitor is sized and anchored to B's toolbar icon | Same binary with one overlay reverted to `g_hwnd` ⇒ that overlay is mis-sized / mis-anchored on B (seen, photographed) — the ticket's latent symptom, first observation | Overlay HWND rect vs B's icon rect (Win32), per overlay; one process | T3 👤 | ⬜ |
| `P9-D3` | Re-show path: open in A, close, open in B ⇒ scaled for B (keep-alive overlays reuse the HWND) | Skip re-scaling on `Show*Overlay` ⇒ second open keeps A's scale | Same reads, second open | T2 (+T3 on hardware) | ⬜ |
| `P9-S1` | Every retired accessor's former callers act on **their own** window: for each, an action in B changes nothing in A (header visibility, client rect, overlay shown) | Restore that accessor's `GetPrimaryWindow()` lookup on the same binary ⇒ the action in B lands on A | Two windows, one process; A's state before/after | T2 | ⬜ |
| `P9-S2` | Each kept accessor has a written reason (primary is correct because …) and a test that its callers are primary-only | — a review row; RED = reviewer finds a kept accessor with a caller reachable from a secondary window | Grep of callers + the reason line | T0 (review) | ⬜ |
| `P9-J1` | Each judgement-call `WM_CLOSE` site labelled; any "close my window" site now closes the **owner** window | Revert one converted site ⇒ the primary closes instead (ticket's type case) | Which HWND received `WM_CLOSE` (logged) | T2 | ⬜ |
| `P9-G1` | G11 baseline lowered to the script's measured count after S; `preflight.ps1` PASS | `-NegativeControl` ⇒ G11 FAILs on its probe (the gate can fail) | `preflight.ps1` output, both runs | T0 | ⬜ |
| `P9-G2` | (If §12 Q1 = yes) widened G11 pattern baselined **by the script**, in its own commit, `-NegativeControl` re-run | A probe line `ScalePx(x, g_hwnd)` in a matched path ⇒ G11 FAILs (proves the widened pattern sees it) | `preflight.ps1` output | T0 | ⬜ |
| `P9-W1` | Two-sided: B's own action still **works** (overlay opens in B, fullscreen/find act on B) | P9-S1 ↔ W1: a fix that ignores B's action passes S1 and fails W1 | B's state after the action | T2 | ⬜ |
| `P9-M1` | macOS: accessor changes in shared `simple_handler.cpp` do not regress mac two-window overlays | Revert as S1 on mac ⇒ wrong-window effect (if the accessor is reachable on mac) | mac relay, two windows | T3 (relay) | ⬜ |

**Two-sided pairs:** S1 ↔ W1 (A untouched / B works) — the ticket's own pairing.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `cef-native/src/handlers/simple_app.cpp` — `Create*Overlay` functions (`CreateSettingsOverlayWithSeparateProcess`, `CreateSettingsMenuOverlay`, `CreateDownloadPanelOverlay`, cookie/bookmarks/profile/site-info/tab-list/menu/omnibox/wallet creators) | ✅ | 📏 57 `ScalePx(…, g_hwnd)` lines, all in this file; 22 `GetPrimaryWindow()` lines |
| `cef-native/src/handlers/simple_handler.cpp` — "Static getters — redirect to WindowManager window 0 for backwards compatibility" block (`SimpleHandler::GetOverlayBrowser` …) | ✅ | 📏 17 accessor lookups via `WindowManager::GetInstance().GetPrimaryWindow()`; 18 `GetPrimaryWindow()` lines in the file |
| `cef-native/src/handlers/simple_handler.cpp :: SimpleHandler::GetOwnerWindow` | ✅ | the correct API — already used widely |
| `cef-native/include/core/WindowManager.h :: WindowManager::GetWindowForBrowser`, `GetPrimaryWindow`; `TabManager::GetActiveTabForWindow` | ✅ | reuse; no second registry |
| `scripts/preflight.ps1` — gate `G11` (baseline **58**, target 0, pattern `GetPrimaryWindow *\(\)` \| `TabManager::GetInstance().GetActiveTab *\(\)`, paths `src/handlers`, `src/core`) | ✅ | ⚠️ pattern does **not** match `g_hwnd` / `ScalePx` — item D cannot move G11 |
| `cef-native/cef_browser_shell_mac.mm` — overlay geometry comment "⚠️ NO `ScalePx` COUNTERPART, deliberately" (overlays sized in points) | ✅ | ⇒ item D is **Windows-only** by design |
| `development-docs/DevOps-CICD/DPI_RESOLUTION_TEST_MATRIX.md` cell #9 | ✅ | two-window variant never run |

## 6. Out of scope

Retiring `WindowManager`/`BrowserWindow`. The `Tab`/`CefBrowser` identity mapping (`R-GOLD` territory). Converting `g_file_dialog_active` / `g_wallet_overlay_prevent_close`. The Exit and WinSparkle `WM_CLOSE` sites (B5-T6-P2). The notification overlay's window (B5-T6-P3 item Z). Split view (deferred). Any mac `ScalePx` equivalent (deliberately none).

## 7. Rollback

Commits per item: D (one per overlay family or one for all, decided at kickoff by diff size), S, J, then G1 (baseline), then G2 (pattern) — G1/G2 each their own commit per rule 6. Each reverts alone; reverting S after G1 fails G11 — revert G1 with it.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| Mechanical conversion turns a correct global into a per-window one (the ticket's warning) | §6 exclusions + S2 review |
| Two-window tests run with two profiles — green, meaningless | SUBJECT: one process |
| Item D green by unit test, never seen on hardware; mixed-DPI users still get mis-sized overlays | D2 is the real row; if K10 has no hardware, D2 is **not run** and the phase reports it, never upgraded |
| G11 lowered in the same commit as the code | Sign-off rule 6 check |
| The widened pattern's baseline is hand-counted | G2 SUBJECT: script output only (HARNESS §9) |
| A retired accessor had a caller in a render-process path or an IPC arm nobody exercised | S1 per caller; grep list attached to the row |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | `P9-M1` | Item D is **Windows-only because** mac overlays are sized in points with no scale step (`cef_browser_shell_mac.mm` comment); items S/J touch shared `simple_handler.cpp` and relay once. SCOPE §5: mac unassessed beyond this |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1.5** — one mixed-DPI two-window sitting (D2/D3), if the hardware exists |
| Human-bound rows | `P9-D2`, `P9-D3` on hardware; `P9-M1` relay |
| Unknowns (K) | **K10** does the owner have two monitors at different scaling? (if not, D2 is not run, and says so) — **K = 1** |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs ← | **B5-T6-P2** | P2 fixes the Exit/WinSparkle `WM_CLOSE` sites; J excludes them |
| needs ← | **B5-T6-P3** item Z | the notification overlay's window is fixed there; not repeated |
| ⚠️ does not apply | **B5-T0-P4** echo page | T0's report says its echo page "can record T6-P9's window globals in the same load". **It cannot:** this phase's globals are C++ process-globals (`g_hwnd`, `GetPrimaryWindow()`), invisible to any web page. No action; recorded so nobody plans on it |
| needs ← | **B5-T0 Build 1** | browser-level evidence on the refreshed engine |

## 12. Open questions for the owner

1. **Widen the G11 gate to see `g_hwnd` in window-scoped code?** Today it counts only `GetPrimaryWindow()` and the global active-tab call, so the 57 DPI sites are invisible to it. ⭐ Recommend **yes, as its own commit after S** (rule 6), baselined by the script — otherwise nothing stops new `ScalePx(…, g_hwnd)` lines.
2. **No mixed-DPI hardware?** ⭐ Recommend ship D with its unit row and a **not-run** D2 recorded honestly, rather than cutting D — or cut P9 whole at G5.5 per the order.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded (R-CLOSE, R-GOLD)
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] G11 lowered in `../../../0.4.0-beta.3/HARNESS.md` §4 in its **own** commit — ⚠️ that file is read-only to beta.6 sessions; record the lowering where the harness owner directs (HARNESS_DELTA §4) and ask if unclear
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] C++ touched ⇒ `MAC_RELAY_BETA5.md` round names `simple_handler.cpp` (shared) and states item D is Windows-only
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
