# B5-T6-P10 — pin a tab, and mute a whole site · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (G3, 2026-09-28). ⛔ Documents only — no code has been written for this phase.

**Track:** B5-T6 Browser shell · **Group:** 3 (could ship — **first cut at G5.5**; cut order **P10 first** → P9 → P7 → P6 → P4) · **Tickets:** `../../tickets/TICKET_tab_pin_and_mute_need_model_changes.md` — **pin** and **mute site** only (mute *tab* shipped in beta.3 Phase 4) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent for T6 (resumed run) · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **14** (Group 3). **T6 Q2** (session restore = windows open at quit — pins ride that). Ticket rules: sequence **after** the session-restore fix (B5-T6-P2); mute-site extends `SitePermissionStore` — ⛔ no parallel top-level store (the dual-store mistake, closed `8874232`). Carried, not reopened.

---

## 1. Goal

A user can pin a tab (it moves left, stays small, survives restart with restore on) and can mute a whole website from the tab menu (every tab of that site stays silent, including after navigation and restart) — Chrome's two most-used tab-menu items after Close.

## 2. Done means

- [ ] **Pin:** tab menu → *Pin* / *Unpin*; pinned tabs sort before unpinned in the window's strip and cannot be reordered after an unpinned tab; pinned state saved in `session.json` (both `SaveSession` copies) and restored
- [ ] **Mute site:** tab menu → *Mute site* / *Unmute site*; applies to every tab of that host in every window, now and after navigation / new tabs / restart; stored in `SitePermissionStore` under a new stable id
- [ ] Per-tab mute (beta.3 Phase 4) unchanged; a tab is silent if **tab-muted OR site-muted**; the tab's audio indicator reflects it
- [ ] Session v2 files written before this phase still load (missing `pinned` ⇒ unpinned)

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-GOLD` | Gold pill lands on the originating tab | Pinning reorders tabs; anything indexing the strip by position instead of `Tab::id` misplaces the pill (`Tab::id ≠ CefBrowser::GetIdentifier()`) |
| Session v2 format | Old files load; rollback build reads new files | `pinned` is a new per-tab key |
| Stored ids (`site_permissions.db`) | integers are forever | mute-site takes a new `SitePermissionType` id — **B5-T6-P7 allocates ids too** (§11) |
| `R-CLOSE` | tab menu overlay close | new menu items on the tab-context-menu overlay |
| Right-click "Manage Site Permissions" | quick revoke | a muted site should appear / be resettable there (or explicitly not — §12 Q2) |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. Not a money/schema/crypto phase (browser data) — controls authored here.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P10-P1` | Pin tab 3 of 5 ⇒ it becomes tab 1, pinned style; `tab_json["pinned"]` true | Drop the reorder ⇒ pinned tab stays at 3 | The tab list JSON sent to the header (`Tab::id` order), not pixels | T2 | ⬜ |
| `P10-P2` | Drag an unpinned tab left of a pinned one ⇒ refused / snapped back (pinned-first invariant in `ReorderTabs`) | Remove the constraint ⇒ the unpinned tab lands before the pinned one | `TabManager::ReorderTabs` result + resulting order | T1 + T2 | ⬜ |
| `P10-P3` | Restore on, pin a tab, quit, relaunch ⇒ pinned and first, in the right window | Omit `pinned` from `SaveSession` ⇒ restored unpinned (seen) | `session.json` read **before** relaunch (it is deleted after restore) + restored tab JSON | T2 | ⬜ |
| `P10-P4` | Old `session.json` (no `pinned` key, captured from the pre-change build) loads cleanly, all tabs unpinned | Make the key required ⇒ restore fails on the old file | Restore log + tab count | T1 | ⬜ |
| `P10-P5` | Gold pill on a pinned tab after reordering: payment from pinned tab ⇒ pill on **that** tab | Map the pill by strip index (rig) ⇒ pill on the wrong tab after pinning | `TabManager::GetTabIdForBrowserIdentifier` result + the pill's tab id (R-GOLD SUBJECT) | T2/T3 | ⬜ |
| `P10-M1` | Mute site on `youtube.com` tab ⇒ that tab and every other youtube.com tab (both windows) silent; other sites audible | Store the flag but skip applying to other tabs ⇒ second tab audible | `IsAudioMuted()` per browser, logged, + host of each | T2 | ⬜ |
| `P10-M2` | Navigate a site-muted tab within the site / open a new tab on it / restart ⇒ still muted (re-applied in `OnLoadingStateChange`, as tab-mute is) | Remove the re-apply ⇒ muted state lost after navigation (CEF mute is per-document — beta.3 M14) | `IsAudioMuted()` after each step; store row present | T2 | ⬜ |
| `P10-M3` | Navigate a site-muted tab to **another** site ⇒ audible (unless tab-muted) | M1 ↔ M3: a fix that mutes the tab forever passes M1/M2 and fails M3 | `IsAudioMuted()` after cross-site navigation | T2 | ⬜ |
| `P10-M4` | Tab-mute and site-mute compose: unmute site while the tab is tab-muted ⇒ still silent | Replace OR with last-writer-wins ⇒ tab becomes audible | Both flags + `IsAudioMuted()` | T1 + T2 | ⬜ |
| `P10-M5` | Mute-site stored in `SitePermissionStore` (host-normalised) under the allocated id; frozen-id test extended | Reuse an existing id ⇒ `StoredIntegersAreFrozen` fails | `site_permissions.db` row + `site_permission_mapping_test.cpp` | T1 | ⬜ |
| `P10-H1` | Owner: right-click → Pin / Mute site look and behave as in Chrome; restart keeps both | Today's build ⇒ items absent (seen) | Owner's eyes, restore on | T3 👤 | ⬜ |
| `P10-MAC` | macOS: pin persists through the **mac** `SaveSession` copy; mute site applies | Omit `pinned` from the mac copy only ⇒ pin lost on mac, kept on Windows (the two-copy trap) | mac relay: `session.json` + tab state | T3 (relay) | ⬜ |

**Two-sided pairs:** M1 ↔ M3 (muted on site / audible elsewhere) · P3 ↔ P4 (new key saved / old files still load).

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `cef-native/include/core/Tab.h :: Tab` (`bool muted`; **no** `pinned`) | ✅ | `pinned` added beside `muted` |
| `cef-native/include/core/TabManager.h :: TabManager::ReorderTabs`, `RecordClosedTab` | ✅ | pinned-first constraint |
| `cef-native/src/handlers/simple_handler.cpp` — tab-context-menu action arms (`mute_toggle`, `duplicate`/`new_tab_right`, `close_others`/`close_right`); `tab_json["muted"]` emit sites | ✅ | new `pin_toggle` / `mute_site_toggle` arms; `pinned` in tab JSON |
| `cef-native/src/handlers/simple_handler.cpp :: SimpleHandler::OnLoadingStateChange` (tab-mute re-apply via `SetAudioMuted`) | ✅ | site-mute re-applies here too |
| `cef-native/include/core/SitePermissionStore.h :: SitePermissionStore` (`GetState`, `SetState`, `NormalizeHost`) | ✅ | the per-host store — extended, not duplicated |
| `cef-native/include/core/SitePermissionType.h :: SitePermissionType` | ✅ | new id (after P7's allocation) |
| `cef-native/cef_browser_shell.cpp :: SaveSession` + `cef_browser_shell_mac.mm :: SaveSession` (static) | ✅ | ⚠️ **both copies** gain `pinned` (SCOPE §3 two-copy finding) |
| `cef-native/src/handlers/simple_app.cpp :: SimpleApp::OnContextInitialized` (v2 restore) | ✅ | reads `pinned` |
| `frontend/src/pages/TabContextMenuOverlayRoot.tsx` (row/divider heights pinned for C++ sizing) | ✅ | ⚠️ adding rows changes the overlay's height — C++ sizing must match |
| `frontend/src/hooks/useTabManager.ts` (tab strip; gold-pill consumer) | ✅ | pinned rendering |

## 6. Out of scope

Pinned tabs surviving when restore is **off** (§12 Q1). Per-site mute UI in Settings. Tab groups. Unifying the two `SaveSession` copies (a kaleidoscope finding, not chased). Changing tab-mute. Split view (deferred).

## 7. Rollback

Two commits: pin (Tab + ReorderTabs + menu + both `SaveSession` copies + restore), mute site (store id + menu + re-apply). Each reverts alone. A rolled-back build ignores the `pinned` key and treats the unknown `SitePermissionType` row as absent (verified once, as in P7 §7).

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| Pin persists on Windows, lost on mac (one `SaveSession` copy missed) | P10-MAC |
| Reordering by pin shifts the gold pill to the neighbouring tab | P5 |
| Site-mute survives a cross-site navigation and the user thinks audio is broken | M3 |
| Mute-site added as a second store "just for audio" — the dual-store mistake again | §5 SitePermissionStore row; review |
| P7 and P10 both write id 8 into users' `site_permissions.db` | §11 serialization; M5 frozen test |
| Tab menu overlay clipped after two new rows at 150% | H1 at a DPI cell; `TabContextMenuOverlayRoot.tsx` height note |
| An old session file fails to load after upgrade and the user loses their session | P4 |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | `P10-MAC` (+ M1–M3 via relay) | shared `simple_handler.cpp`, `TabManager`, React; the mac `SaveSession` copy in `cef_browser_shell_mac.mm` is the platform-specific change |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1** — right-click, restart, verify (SCOPE §7). Mac relay ~0.5 |
| Human-bound rows | `P10-H1` (native right-click), `P10-MAC` |
| Unknowns (K) | none — **K = 0** |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs ← | **B5-T6-P2** | one change to session save at a time — P2 lands first (ticket ordering) |
| ⛔ serialize ↔ | **B5-T6-P7** | both allocate `SitePermissionType` ids; P7 first, P10 takes the next free id |
| gives → | split view (deferred, decision 14) | the tab model's first new per-tab persisted flag |

## 12. Open questions for the owner

1. **Pins with restore off.** Firefox reopens pinned tabs at startup; Chrome's current behaviour with "open the New Tab page" is not confirmed on a Google page (SCOPE §2.4). ⭐ Recommend **pins restore only with "restore session" on** (smallest change, one mechanism). Alternative: always reopen pinned tabs — a second startup path.
2. **Muted sites in "Manage Site Permissions"?** ⭐ Recommend **yes, listed and resettable there** — it is where a user looks for per-site settings, and it reuses the existing store's UI. Alternative: only the tab menu toggles it.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded (R-GOLD, R-CLOSE)
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] C++ touched ⇒ `MAC_RELAY_BETA5.md` round names both `SaveSession` copies and `simple_handler.cpp`
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
