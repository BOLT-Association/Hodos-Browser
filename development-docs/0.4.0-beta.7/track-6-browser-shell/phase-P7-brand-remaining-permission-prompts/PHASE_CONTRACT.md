# B5-T6-P7 — no stock Chrome permission bubble: every site-permission prompt is Hodos's own · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (G3, 2026-09-28). ⛔ Documents only — no code has been written for this phase.

**Track:** B5-T6 Browser shell · **Group:** 3 (could ship — **first cut at G5.5**; cut order P10 → P9 → **P7** → P6 → P4) · **Tickets:** `../../tickets/TICKET_brand_remaining_permission_prompts.md` (inventory: `../../../0.4.0-beta.3/PROMPT_BRANDING_INVENTORY.md`, read-only) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent for T6 (resumed run) · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **14** (Group 3 first to cut). **T6 Q6** — a declined permission is **not** stored (P8 dropped): prompt denials stay temporary (`DISMISS`), overt Site-controls actions persist — the ticket's §8 rule, unchanged. The ticket's design (§3: one mechanism, generic fallback, six high-risk types with specific copy **in the same commit**) is carried. **Sequenced after T0** (SCOPE §5 P7: the engine refresh may change the type list).

⭐ **Its old blocker is gone:** the ticket's §2 "blocked on Phase 1 (DPI)" waited on the modal-buttons ticket, closed 2026-09-25 with `0a7d43b` (DPI-correct mouse input for the OSR overlays) — SCOPE §4 #10.

---

## 1. Goal

No Hodos user ever sees a stock Chrome permission bubble — including for a permission type a future Chromium adds — and no Hodos prompt ever says less than Chrome's would have; and the wallet's protocol-permission prompt reads correctly when a site asks to **share a key**, not only to sign.

## 2. Done means

- [ ] Every `CEF_PERMISSION_TYPE_*` bit in the shipped engine (📏 today: **29 bits**, `1<<0 … 1<<28`, `1<<25` aliased — `cef-binaries/include/internal/cef_types.h`) maps to a stable `SitePermissionType` id and a wire code; `OnShowPermissionPrompt` never returns `false` for a mapped type
- [ ] An **unmapped** type (a future Chromium addition) reaches the Hodos **generic** prompt, not Chrome's bubble (the fail-safe direction — ticket §3a)
- [ ] The six high-risk types — File system access · Window management · Sensors · Local fonts · Idle detection · Protected media identifier — carry **specific** copy in the same commit that routes them (ticket §3c)
- [ ] Two buttons on the `OnShowPermissionPrompt` path (CEF has no "grant once"); three only on the media path
- [ ] Every type added to `kSitePermCaps` writes through to Chromium's setting (`MirrorSitePermissionToChromium`)
- [ ] Stored ids frozen: the frozen-id test extended for every new id; no id reused or renumbered
- [ ] Each type is either **demonstrated** (prompt seen, allow ⇒ works, deny ⇒ does not) or explicitly marked **mapped but unexercised** — never "branded because it compiles" (ticket §4.5)
- [ ] **Share-a-key wording (T5-P3 edge):** the `protocol_permission_prompt` modal, which T5-P3 makes fire for derived-key fetches, names the action correctly for a key fetch and for a signature, and names the counterparty (memory: *derived keys — always name the counterparty*)

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-CLOSE` | Overlay close guards | 21+ new prompt types on the **shared** notification overlay — the ticket's §4.1 hazard (~60% of Phase 0.9's cost) |
| `R-PERIM` + consent surface | A prompt never grants more than its label says; fails closed without the disclosure rendering | The generic label must never be vaguer than Chrome's (§3c); a prompt that grants before its text renders is a defect (P0.9 §7.2) |
| `R-ONE-CLICK-ONE-SPEND` | One Approve signs exactly what it was shown | A permission prompt pre-empting a payment modal, or vice versa, on the same overlay (ticket §5 criterion 3) |
| `R-INTEXT` | Internal never prompts | Hodos's own pages must not start raising Chromium permission prompts through the new mapping |
| **Right-click "Manage Site Permissions"** | quick revoke | New types appear in Site controls; each must genuinely revoke (ticket §5 criterion 5) |
| T6 Q6 | Prompt denials are temporary | A new type must not persist a DISMISS as a Block |
| Stored ids (`site_permissions.db`) | Integers are forever | ⚠️ they have been reassigned once before (ticket §4.4); **B5-T6-P10 also allocates an id** (mute site) — one allocation, one commit order (§11) |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. Not a money/schema/crypto phase by §4.2 (browser `site_permissions.db`, not the wallet DB) — controls authored here. ⛔ Before any live row: `reset_test_state.py verify` exits 0 (ticket §4.7); a fresh profile is **not** a fresh test.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P7-S0` | **Step 0 (K9), on the T0-refreshed engine:** the shipped `cef_types.h` bit list recorded; for each type, can a test page trigger it in our build (Windows + mac)? Table: triggerable / not / unknown | — measurement row | The engine actually shipped by T0 Build 1 (and Build 2 if it lands first) — `CEF_VERSION` quoted | T2 | ⬜ |
| `P7-U1` | Unit: every bit maps to a unique id + non-empty wire code; ids for the 7 existing types unchanged | Delete one mapping arm ⇒ exactly that type's test fails; renumber an existing id ⇒ `StoredIntegersAreFrozen` fails | `site_permission_mapping_test.cpp` | T1 | ⬜ |
| `P7-U2` | Unit: the six high-risk types never resolve to the generic label | Map File system access to generic ⇒ this test fails | The label lookup the overlay uses (C++ wire code → `PERM` table), tested on the codes C++ sends | T1 | ⬜ |
| `P7-U3` | Unit: an **unknown** bit (simulated future type, e.g. `1<<29`) ⇒ routed to the generic prompt, not `return false` | Today's code with the simulated bit ⇒ `false` (Chrome's bubble) | `OnShowPermissionPrompt`'s decision for the mask, via a testable helper | T1 | ⬜ |
| `P7-A1` | Live, each **triggerable** high-risk type from a real page ⇒ Hodos prompt with its specific copy; **allow ⇒ capability works; deny ⇒ it does not** | Today's build ⇒ Chrome's stock bubble for the same page (seen, screenshot) | The **effect** on the page (API result), not the dialog; the prompt in the notification overlay's browser (role-log) | T2 + T3 | ⬜ |
| `P7-A2` | Deny ⇒ re-prompted next time (temporary); Site controls Block ⇒ persists and **the site's behaviour** changes (write-through) | Remove the mirror call for one type ⇒ Site controls shows Block, the site keeps working (ticket §4.2 — the dual-store defect) | The page's API result after the toggle; Chromium's content setting read back | T2 | ⬜ |
| `P7-A3` | Callback answered **exactly once** across: tab close, navigation, browser close, the 60 s watchdog, `OnDismissPermissionPrompt`, overlay pre-emption, modal closed without a decision | Skip the answer on one path (rig) ⇒ the page hangs / jams subsequent prompts (the P0.9 jam) — seen, then reverted | Callback counter per request id, logged | T1 + T2 | ⬜ |
| `P7-A4` | A permission prompt never destroys a live wallet modal (payment/connect) and never strands an overlay; the P0.9 pre-emption latch still re-shows a pre-empted prompt | Remove `markPreempted` ⇒ the pre-empted prompt is lost (seen) | Overlay `type` sequence + the payment modal's approval outcome | T2 | ⬜ |
| `P7-A5` | Two-button rule on the prompt path; three only on the media path | Give a prompt-path type `noOnce: false` ⇒ an "Allow once" that actually persists (CEF `ACCEPT` persists — seen in `site_permissions.db`) | The stored row after clicking the would-be "once" | T2 | ⬜ |
| `P7-A6` | DPI cells #4/#6/#9: the new prompts render, and Allow/Deny land where aimed | The same prompt at 100% (the ticket's control — a cell cannot tell a DPI bug from a broken prompt otherwise) | Owner's clicks + resulting stored state | T3 👤 | ⬜ |
| `P7-A7` | **Share-a-key wording:** a derived-key fetch (T5-P3 part 1) and a signature request each show wording that names **what** is asked (share a key / sign) and **with whom** (the counterparty) | Render the key-fetch case with today's sign-only copy ⇒ owner reads it as "sign" (the unjudgeable-popup failure) | Owner reads both modals in the notification overlay; the request payload's method (`getPublicKey` vs `createSignature`) logged beside it | T3 👤 | ⬜ |
| `P7-M1` | macOS: the six high-risk types (where triggerable) show the Hodos prompt | Today's mac build ⇒ stock bubble; Phase 0.9's mac arms are written-but-unrun (ticket §4.6) | mac relay: prompt + effect | T3 (relay) | ⬜ |

**Two-sided pairs:** A1 allow ↔ deny (works / does not) · U2 ↔ U3 (specific where required / generic where unknown).

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `cef-native/include/core/SitePermissionType.h :: SitePermissionType` (Camera=1 … Loopback=7; comment reserves `Midi=8, Usb=9, Bluetooth=10` as "v2") | ✅ | new ids appended; ⚠️ the comment's reserved numbers are not a registry — P7 writes the allocation down (§11 with P10) |
| `cef-native/include/core/SitePermissionMapping.h :: hodos::siteperm::PermCode` | ✅ | wire codes |
| `cef-native/src/handlers/simple_handler.cpp :: SimpleHandler::OnShowPermissionPrompt`, `SimpleHandler::OnRequestMediaAccessPermission`, `FireHodosPermissionPrompt` (static), `kSitePermCaps` | ✅ | routing, prompt fire, revocable caps |
| `cef-native/src/handlers/simple_handler.cpp :: MirrorSitePermissionToChromium` (static) | ✅ | write-through. ⚠️ SCOPE §3/§6 name it `MirrorNetworkPermissionToChromium` — that name does not exist (doc fix) |
| `cef-native/src/handlers/simple_handler.cpp :: ShowDeferredPermissionTask` | ✅ | loopback/local-network deferral — unchanged; new types do not defer |
| `cef-native/src/handlers/simple_app.cpp :: CreateNotificationOverlay` (+ `PendingPermissionManager::markPreempted`) | ✅ | shared overlay; B5-T6-P3 item Z changes its window first |
| `frontend/src/pages/BRC100AuthOverlayRoot.tsx` — `PERM` table (`noOnce`), generic fallback `PERM[permCode] \|\| { … 'access a device feature' }`, `protocol_permission_prompt` shared modal | ✅ | copy table + A7 wording |
| `cef-native/tests/site_permission_mapping_test.cpp` (`StoredIntegersAreFrozen`, `MaskToTypes…`) | ✅ | extended |
| `cef-binaries/include/internal/cef_types.h` — `CEF_PERMISSION_TYPE_*` (29 bits, `1<<25` aliased) at `CEF_VERSION 150.0.43-7871.3576` | ✅ | re-read on the T0 engine at S0 |
| `cef-native/cef_browser_shell_mac.mm :: CreateNotificationOverlay` | ✅ | mac overlay |

## 6. Out of scope

`OnJSDialog` and `GetAuthCredentials` (different machinery — ticket §6). Group B Chrome-UI bubbles (save password, save card, translate — no CEF hook; an engine patch would be a `NEXT_CHROMIUM_BUILD.md` PART 2 row). Changing the existing 7 types' behaviour. Storing declined permissions (T6 Q6, P8 dropped). Permission lifetimes / expiring grants (Brave's feature — not requested). The wallet's own permission engine (Rust) — A7 is copy only.

## 7. Rollback

Commits: (1) ids + mapping + generic route + the six specific labels (one commit, by the §3c rule), (2) remaining copy table, (3) share-a-key wording. Reverting (1) returns unmapped types to Chrome's bubble; stored rows with new ids remain in `site_permissions.db` and must be **ignored**, not misread, by the older build (tested once at kickoff: an old build reading a row with an unknown id).

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| A type is "branded" in code but never fires in our build; the release notes claim all 29 | S0 + "mapped but unexercised" marking |
| File system access ships on "access a device feature" for one commit — a consent regression | U2 + the same-commit rule |
| New prompts jam the shared overlay and every permission browser-wide falls to Chrome (P0.9's jam) | A3 + A4 |
| Site controls shows Block while the site keeps its access | A2 |
| A future Chromium type silently reverts to Chrome's UI | U3 |
| P7 and P10 both take id 8 | §11 id allocation, U1's frozen test |
| Key-fetch prompts read as "sign" and the user cannot judge them | A7 (human reads) |
| New prompts appear over the wrong window with two windows open | B5-T6-P3 item Z lands first |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | `P7-M1`, A7 via relay | mapping/test/React are shared; `simple_handler.cpp` prompt path is shared C++ with mac arms from Phase 0.9 (written-but-unrun); mac overlay twin `cef_browser_shell_mac.mm :: CreateNotificationOverlay`. Budget a relay round (ticket §4.6) |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~3** — six high-risk types triggered live, DPI cells, A7 wording read (SCOPE §7). Mac relay ~1.5 |
| Human-bound rows | `P7-A1` (effect confirmation per type), `P7-A6` (DPI), `P7-A7` (wording), `P7-M1` |
| Unknowns (K) | **K9** which types can fire in our build (S0) — **K = 1** |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs ← | **B5-T0 Build 1** (and Build 2 if earlier) | the type list is read on the shipped engine (S0) |
| needs ← | **B5-T6-P3** item Z | prompts placed over the requesting window before 21 more types use the overlay |
| needs ← | **B5-T5-P3** derived keys | T5-P3 makes `protocol_permission_prompt` fire for key fetches; A7's wording lands **with or after** it (T5-P3 §11) |
| ⛔ serialize ↔ | **B5-T6-P10** pin/mute site | both allocate `SitePermissionType` ids; P7 lands first and records the allocation; P10 takes the next free id |

## 12. Open questions for the owner

None beyond the decisions carried. (If S0 finds a type that fires and has no sensible specific copy, it goes on the generic label **only** if it is not one of the six — the ticket's rule already settles it.)

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded (R-CLOSE, R-PERIM, R-ONE-CLICK-ONE-SPEND, R-INTEXT)
- [ ] Adversarial review of the evidence complete — the ticket's **seven §5 criteria** answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] C++ touched ⇒ `MAC_RELAY_BETA5.md` round names `simple_handler.cpp`, `SitePermissionType.h`, `SitePermissionMapping.h`
- [ ] `cef-native/include/core/CLAUDE.md` `SitePermissionStore.h` row updated (it still lists five types — invariant 11)
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
