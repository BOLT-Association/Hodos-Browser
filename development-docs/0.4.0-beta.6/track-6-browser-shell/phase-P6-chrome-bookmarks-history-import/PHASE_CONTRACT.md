# B5-T6-P6 — import bookmarks and history from Chrome (and a bookmarks file) from the live Settings page · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (G3, 2026-09-28). ⛔ Documents only — no code has been written for this phase.

**Track:** B5-T6 Browser shell · **Group:** 2 (should ship) · **Tickets:** `../../tickets/TICKET_chrome_import_bookmarks_history_passwords.md` — **slices A + B only** · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent for T6 (resumed run) · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **14** — Chrome bookmarks + history import (slices A + B) **stays**; ⛔ **password import (slice C) is deferred to the next release, CSV only**, never touching Chrome's keys. Ticket constraints 1–4 (never defeat Chrome's encryption; no cookie/session import; imports write to the user's real data with no undo ⇒ a restore story; extend `ProfileImporter`, do not duplicate). Carried, not reopened.

⛔ **No passwords, no cookies, no `Login Data`, no `Network/Cookies`, no `Local State` key reads — in code, tests, or rigs.** A row that would need one is out of scope.

---

## 1. Goal

A user can reach **Settings → Import** on the Settings page they actually use, bring over their Chrome (or Brave / Edge) bookmarks and history on this computer, or import a bookmarks file exported on another computer — and can undo an import that went wrong.

## 2. Done means

- [ ] **Slice A:** `SettingsPage` gains an **Import** section (sibling of General / Privacy & Security / Downloads / Wallet / About) that drives the **existing** importer and IPC (`import_detect_profiles`, `import_bookmarks`, `import_history`, `import_all`); detected browsers listed; per-item import; a result line with counts
- [ ] **Slice B:** a **visible** file input (CEF input pattern) accepts a Chrome/Netscape bookmarks HTML file and imports its folders and links
- [ ] **Restore story:** before any import writes, the bookmarks and history stores are copied aside; the section offers **Undo last import** (or a documented one-step restore — §12 Q1); an observed RED shows it restoring
- [ ] Importing the same source twice does **not** duplicate bookmarks (or duplicates are recorded as today's behaviour and the owner decides — K8b)
- [ ] The importer is exercised end-to-end on **both** platforms with a real Chrome profile (it was last run through the orphaned overlay)
- [ ] The orphaned `SettingsOverlayRoot` Import tab: reported, not deleted here (working rule 3)
- [ ] **Review, not build:** T5-P5's usage-ping switch (`frontend/src/components/settings/PrivacySettings.tsx`) and first-run notice reviewed by T6 for settings-page and first-run overlay conflicts (edge, §11)

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| Ticket constraint 3 — user data with no undo | A bad import corrupts the user's real bookmarks/history | This phase is the first to write into them from a user action at scale; the restore story is a Done item, not a nicety |
| Ticket constraint 1 — never defeat Chrome's encryption | ABE / DPAPI / Keychain untouched | A "while we're here" read of `Login Data` or `Local State` would breach it |
| Browser data vs wallet data (root CLAUDE.md glossary) | Browser data lives in the C++ layer, separate from the wallet | The import must not touch the wallet DB; profile scoping must hold (import into the **current** profile only) |
| CEF input patterns | Visible file input; native inputs | Slice B's file input; hidden `.click()` inputs are unreliable in CEF |
| `R-CLOSE` a | `g_file_dialog_active` guards overlays during a native file dialog | Slice B opens a native file dialog from the settings page |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. Not a money/schema/crypto phase — controls authored here. ⛔ Every import run targets a **scratch Hodos profile**; the owner's real Chrome profile is **read-only** input (copy-then-read is already the importer's shape for History).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P6-S0` | **Step 0 (K8):** today's importer, driven by its IPC from a dev build, imports a known Chrome profile's bookmarks and history into a scratch profile; counts recorded | — measurement row; "does not work" is a valid outcome and resizes the phase before code | Source counts (Chrome `Bookmarks` JSON node count; `History` `urls` row count via a read-only copy) vs Hodos store counts after | T2 | ⬜ |
| `P6-A1` | Settings → Import visible on the live `SettingsPage` (`/settings-page/import`), lists detected browsers | Today's build ⇒ no Import section (the gap, seen); remove the section entry ⇒ route falls back to General | The **settings page's** browser (role-log resolved), not the orphaned `/settings` overlay | T2 | ⬜ |
| `P6-A2` | Import bookmarks from a detected Chrome profile ⇒ Hodos bookmark count rises by the source's count; folder structure preserved (3 spot-checked folders) | Stub `import_bookmarks` to return success without writing ⇒ counts unchanged while the UI says success — the row must go red on **counts**, not on the UI line | Source node count vs Hodos store rows before/after, same scratch profile | T2 | ⬜ |
| `P6-A3` | Import history ⇒ Hodos history rows rise by the source's `urls` count (or the importer's documented cap); visit times preserved on 3 spot checks | Same stub pattern on `import_history` | Source row count (read-only copy) vs Hodos history store | T2 | ⬜ |
| `P6-A4` | Import while Chrome is **running** ⇒ succeeds via copy-then-read (History locked by Chrome) | Disable the copy (read in place) in a rig build ⇒ `SQLITE_BUSY` / sharing violation, import fails (seen) | Chrome's PID alive during the run; importer log shows the temp copy path | T2 | ⬜ |
| `P6-A5` | Import the same source twice ⇒ no duplicate bookmarks (or recorded, K8b) | — if dedupe is built: remove it ⇒ duplicates appear (seen). If not built: this row records today's count doubling and goes to §12 Q2 | Bookmark row count after run 1 vs run 2 | T2 | ⬜ |
| `P6-A6` | **Undo last import** ⇒ bookmarks and history stores return exactly to their pre-import state | Skip the pre-import copy in a rig build ⇒ Undo cannot restore (seen) | Row counts **and** a checksum of each store before import vs after undo | T2 | ⬜ |
| `P6-A7` | Slice B: a Chrome-exported bookmarks HTML file (visible input) ⇒ folders and links imported; counts match the file's `<A HREF>` count | Feed a file with nested folders to a parser that flattens ⇒ structure assertion fails; feed a non-bookmarks HTML ⇒ refused with a sentence, nothing written | Parsed link/folder counts vs the file; store rows after | T1 (parser) + T2 | ⬜ |
| `P6-A8` | Slice B bounds: a > cap file (e.g. 20 MB) or a `javascript:` / `data:` HREF ⇒ refused / skipped with a count, never imported as a clickable bookmark | Remove the scheme filter ⇒ a `javascript:` bookmark is stored (seen, scratch profile) | Store rows' URL schemes after import | T1 + T2 | ⬜ |
| `P6-A9` | Imports land in the **current** Hodos profile only | Import from profile 2's settings page while profile 1 is open (separate processes) ⇒ profile 1's stores unchanged — a wrong-profile write shows here | Both profiles' store counts before/after | T2 | ⬜ |
| `P6-A10` | ⛔ No read of `Login Data`, `Cookies`, `Local State` during any import | Instrument: a rig that fails if those files are opened (Process Monitor capture on Windows / `fs_usage` on mac) ⇒ seeded read in a probe build shows up (proves the capture sees it) | File-open capture filtered to the Chrome profile directory | T2 | ⬜ |
| `P6-A11` | Owner: real Chrome profile on his machine ⇒ import looks right (a few known bookmarks and recent history present); file dialog doesn't close the settings page (R-CLOSE a) | Today's build ⇒ nowhere to click (A1's RED) | Owner's eyes, scratch Hodos profile | T3 👤 | ⬜ |
| `P6-M1` | macOS: A2/A3/A7 with Chrome's mac profile (`~/Library/Application Support/Google/Chrome/Default`) | Point detection at a missing path ⇒ "no browsers found" (proves the mac arm is exercised, not assumed) | mac relay: counts before/after | T3 (relay) | ⬜ |

**Two-sided pairs:** A2 ↔ A10 (reads what it should / never reads what it must not) · A6 ↔ A2 (import writes / undo unwrites).

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `cef-native/include/core/ProfileImporter.h :: ProfileImporter` (`DetectProfiles`, `ImportBookmarks`, `ImportHistory`, `ImportAll`, `GetFirefoxProfilePath`, `ImportBookmarkNode`) | ✅ | extend, not duplicate. Slice B = a new `ImportBookmarksHtml`-style member |
| `cef-native/src/core/ProfileImporter.cpp` — Chrome/Brave/Edge path arms (`_WIN32` `LOCALAPPDATA`, `__APPLE__` `~/Library/Application Support/...`), `CopyFilePortable` (History + `-wal`/`-shm` copied), `BookmarkManager::AddBookmark` writes | ✅ | ⚠️ detects only each browser's **`Default`** profile — a Chrome user on `Profile 1` sees nothing (K8c, §12 Q3) |
| `cef-native/src/core/BookmarkManager.cpp`, `HistoryManager.cpp` | ✅ | the stores A6 snapshots |
| `cef-native/src/handlers/simple_handler.cpp` — `import_detect_profiles`, `import_bookmarks`, `import_history`, `import_all` IPC arms | ✅ | driven as-is; slice B adds one arm |
| `frontend/src/hooks/useImport.ts :: useImport` | ✅ | today used only by `SettingsOverlayRoot.tsx` |
| `frontend/src/pages/SettingsPage.tsx` — section list (`general`, `privacy`, `downloads`, `wallet` external, `about`) | ✅ | Import section added |
| `frontend/src/pages/SettingsOverlayRoot.tsx` (orphaned `/settings`) | ✅ | reported, not deleted |
| `cef-native/src/handlers/simple_handler.cpp :: SimpleHandler::OnFileDialog` (sets `g_file_dialog_active = true`) | ✅ | slice B's dialog path — R-CLOSE a |

## 6. Out of scope

⛔ Passwords (slice C — decision 14), cookies, sessions, autofill, extensions. Firefox import (stub stays a stub). Google Takeout history JSON. Deleting the orphaned overlay. Branding the save-password bubble. A general "import from any browser" wizard. Building the usage-ping switch (T5-P5 builds it; T6 reviews).

## 7. Rollback

Two commits: (A) Import section + restore snapshot/undo, (B) bookmarks-HTML import. Each reverts alone; imported data stays (the Undo action is the data rollback, tested by A6).

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| The UI reports success, nothing was written (dormant importer broke since beta.3) | S0 + A2/A3 assert on counts, not UI text |
| Importing twice doubles every bookmark and the user has no undo | A5 + A6 |
| History import fails whenever Chrome is open (most of the time) | A4 |
| The owner's Chrome profile is `Profile 1`, detection finds nothing, and the feature looks broken | S0 on his machine; §12 Q3 |
| A crafted bookmarks file plants `javascript:` bookmarks | A8 |
| Someone "helpfully" adds password import behind a flag | A10 + §6; review rejects |
| Import writes into the wrong profile's stores | A9 |
| The file dialog steals focus and the settings page or another overlay closes mid-import | A11 + R-CLOSE a |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | `P6-M1`, A10 (`fs_usage`), A11 via relay | the importer's `__APPLE__` arms exist but must be **run**, not assumed (SCOPE §5 P6). Chrome on mac uses Keychain — irrelevant here because no secret is read |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1.5** — real Chrome profile on his machine (A11), a bookmarks file export. Mac relay ~1 |
| Human-bound rows | `P6-A11`, `P6-M1` |
| Unknowns (K) | **K8** does the dormant importer still work (S0) · K8b dedupe behaviour today · K8c non-`Default` Chrome profiles — **K = 1 phase-level** (K8; the others are sub-questions S0 answers) |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| reviews → | **B5-T5-P5** usage ping | T5-P5 builds the settings switch (`frontend/src/components/settings/PrivacySettings.tsx`) and first-run notice; **T6 reviews** for settings-page layout and first-run overlay conflicts (T5 recommendation — T6's scope lacked the item). Recorded as a sign-off item here because this phase edits the same settings page |
| needs ← | **B5-T0 Build 1** | browser-level evidence on the refreshed engine |
| gives → | beta.7 slice C (deferred) | the Import section is where CSV password import will live |

## 12. Open questions for the owner

1. **Restore story shape.** ⭐ Recommend **copy the two stores aside before each import + an "Undo last import" button** (one level). Alternative: copy aside only, and document the restore — cheaper, but a user cannot find it.
2. **Duplicates on re-import** (if S0 shows today's importer duplicates): skip URLs already bookmarked in the same folder (⭐ recommend), or leave as is.
3. **Chrome profiles other than `Default`:** today only `Default` is detected. ⭐ Recommend **list every Chrome profile folder** (`Default`, `Profile N`) in detection — small, and without it many users see "no browsers found". Say no if you want the phase kept to the existing importer exactly.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded (R-CLOSE a)
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] T5-P5 switch + first-run notice reviewed (result + date recorded; review only — no T6 code)
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] C++ touched ⇒ `MAC_RELAY_BETA5.md` round names `ProfileImporter.cpp` (shared `#ifdef`) and `simple_handler.cpp`
- [ ] `frontend/src/pages/CLAUDE.md` section roster updated (invariant 11)
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
