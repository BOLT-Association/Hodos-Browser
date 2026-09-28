# B5-T0-P4 — The browser names itself "Hodos" in its Client Hints, and the header and the JavaScript agree · PHASE CONTRACT

**Track:** B5-T0 Engine — **Build 2** · **Tickets:** `../../tickets/TICKET_engine_behind_its_own_cef_branch_and_upstream_stable.md` (its phase 3(ii)) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** **1** — the brand moved **out** of the security release into Build 2 *because it is irreversible
once seen and its header-vs-JS agreement is unmeasured* (`../SCOPE.md` §0, §5 P4, §6c). ⇒ **This contract's central job
is to measure that agreement on every surface before anyone outside our machines sees the brand.**
👤 Owner, 2026-09-21: **our own brand, not Chrome's** (`../../../DevOps-CICD/NEXT_CHROMIUM_BUILD.md` PART 2) — carried.
**Spec:** `../../../0.4.0-beta.3/phase-13-bot-detection/STEP0_AND_SIGNAL_SHEET.md` §"Addendum" (signal `B1`).

---

## 1. Goal

A site that asks which browser this is gets **one consistent answer** — "Hodos, which is Chromium 150" — from the
`Sec-CH-UA` header and from `navigator.userAgentData`, instead of today's "bare Chromium" that contradicts our UA string.

## 2. Done means

- [ ] Every surface that reports the brand list reports **the same three brands** — `Hodos`, `Chromium`, one GREASE brand — with the same versions and the **same order** (`P4-A1`…`A5`).
- [ ] The header-vs-JS comparison has been **seen to report a mismatch** when fed one (`P4-A3`'s RED) — i.e. the agreement claim is a measurement, not an inference from reading `user_agent_utils.cc`.
- [ ] The User-Agent **string** is byte-identical to Build 1's (`P4-A6`).
- [ ] The brand patch is present **by name** in the build's applied-patch list and is gated on its own `patch.cfg` condition (`P4-A7`).
- [ ] No third-party site saw the brand before `P4-A1`–`A6` passed on our own echo server (`P4-A0`) — §12 Q1.

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| Farbling (C6 navigator patch) | `navigator` values farbled natively | C6 edits `navigator_base.{h,cc}`; the brand lives in `components/embedder_support` and reaches the renderer as UA metadata. No file overlap (checked by path), but a patch-order slip in `patch.cfg` could still break C6's apply ⇒ farbling gate in P5-lite |
| "Do not rewrite the header in C++" (`STEP0_AND_SIGNAL_SHEET.md` §Addendum) | the header and `navigator.userAgentData` are produced by **one** engine source | the tempting shortcut (rewrite `sec-ch-ua` alongside DNT/GPC) makes them **disagree** — strictly worse than nothing. `P4-A3` is the tripwire; ⛔ no `cef-native/` change in this phase |
| UA string unchanged | sites that parse the UA keep working | the patch touches the function next to the UA-string builder (`GetUserAgentInternal`) |
| `R-GOLD`, `R-INTEXT`, `R-PERIM` | wallet surfaces | untouched by this phase's diff (engine UA metadata only); run on the Build 2 engine in P5-lite, not here |

## 4. Evidence table

⛔ Every row runs on the **Build 2 engine produced in `../phase-P5lite-build2-verify/`**. ⭐ **Build 1's engine is the
feature-off build** (same Chromium `.255`, no brand patch) — every RED below that says "Build 1" is a real run, not a claim.
⛔ **Subject rule (the three farbling-harness failures):** measure from a **tab** browser (`role: tab_<n>` in the shell role
log), never from the header or overlay browsers CDP also lists as `type:"page"`; never open the page with `PUT /json/new`.
Reuse `../../../0.4.0-beta.3/phase-13-bot-detection/p13_signals.py`'s tab-subject assertion rather than writing a new one.
⭐ **Instrument:** a **local HTTPS echo server** we run (logs raw request headers; can send `Accept-CH` / `Critical-CH`)
— `127.0.0.1` is a potentially-trustworthy origin, so low-entropy Client Hints are sent to it. The same page reads
`navigator.userAgentData` in the window, a dedicated worker and a service worker, and posts all of it back to the server,
so **header and JS are captured from one document, one load**.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P4-A0` | The first run of `P4-A1`–`A6` on Build 2 is against **our own echo server only**; no Build 2 dev build has loaded a third-party page before it passed | the run log is read for any non-`127.0.0.1` navigation preceding the echo run — shown to catch one by grepping a Build 1 basket run's log, which has them | the dev profile's navigation log for the Build 2 binary (`CEF_VERSION` carries the Build 2 `g<sha>`) | T2 | ⬜ |
| `P4-A1` | The **navigation** request's `Sec-CH-UA` has exactly three entries: `"Hodos";v="150"`, `"Chromium";v="150"`, and one GREASE brand | **Build 1 engine** ⇒ two entries (`"Not;A=Brand";v="8", "Chromium";v="150"` shape — measured 2026-09-21 on `.187`) | the raw header **as the echo server received it**, from a tab browser | T2 | ⬜ |
| `P4-A2` | Subresource (`fetch`, `<img>`) and same-origin `<iframe>` requests carry a `Sec-CH-UA` **byte-identical** to `P4-A1`'s | Build 1 ⇒ Hodos absent on every one | server-side raw headers, grouped by request type; all from the one tab | T2 | ⬜ |
| `P4-A3` ⭐ | `navigator.userAgentData.brands` in the window equals the header list — **same brands, versions and order** | ⭐ **The comparator's own control:** feed it Build 2's header with Build 1's `brands` (captured in the Build 1 run) ⇒ it must report **MISMATCH**. A comparator that has never said "mismatch" has not measured agreement | header and JS values from **one** page load in one tab (same document) | T2 | ⬜ |
| `P4-A4` | `brands` read inside a **dedicated worker** and a **service worker** equals the window's | Build 1 ⇒ Hodos absent in both | values posted from the worker contexts themselves (not the window reading a cached copy) | T2 | ⬜ |
| `P4-A5` | High-entropy agrees: `getHighEntropyValues(['fullVersionList'])` and the `Sec-CH-UA-Full-Version-List` header (sent after the echo server's `Accept-CH`) both carry `Hodos` with the **same full version** as `Chromium` and the same order | Build 1 ⇒ Hodos absent from both. Comparator control as `P4-A3` | echo server's second request after `Accept-CH` + the JS value, one tab | T2 | ⬜ |
| `P4-A6` | `navigator.userAgent` and the `User-Agent` header are **byte-identical** to Build 1's | launch Build 2 with `--user-agent-product=Chrome/150.0.0.0` ⇒ the comparator reports a difference (shows it can see one) | same echo run, UA captured both ways | T2 | ⬜ |
| `P4-A7` | The brand patch appears **by name** in the Build 2 build log's applied list, is registered in the fork's `patch/patch.cfg` under **its own condition** (not `HODOS_FARBLING`) and in `HODOS_PATCHES.md` | the list-check run against **Build 1's** build log ⇒ reports the brand patch **missing** | the build log of the build whose artifact P5-lite publishes (matched by `CEF_VERSION`) | T4 | ⬜ |
| `P4-A8` | Brand list is **stable across restarts** (GREASE value and order are seeded by the major version, so they must not move between launches) | two launches with different profiles on Build 2 must agree; a deliberately altered captured value fed to the comparator ⇒ MISMATCH | two separate launches, tab browsers | T2 | ⬜ |
| `P4-A9` | 🍎 `P4-A1`–`A6`, `A8` on macOS: identical brand list to Windows | Build 1 macOS engine ⇒ two brands | tab browser on the staged Mac framework; macOS CDP binds only for profile `Default` (9322 under dev) | T2 | ⬜ |

**Two-sided:** `P4-A1`–`A5` (Hodos **must** appear, everywhere) ↔ `P4-A6` (the UA string **must not** change).

⚠️ **Do not assert the literal order `Hodos, GREASE, Chromium`.** `GenerateBrandVersionList` shuffles the list with a
seed derived from the major version (read at `.187`), so the order is deterministic but not ours to choose. Assert the
**set**, and **agreement of order across surfaces** — that is the property sites score.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| Chromium `components/embedder_support/user_agent_utils.cc :: GetUserAgentBrandList` | ✅ read in the build-host tree at **`150.0.7871.187`** (`C:\cef\cef150\chromium\src`); SCOPE read it at `.255` via gitiles | `brand = version_info::GetProductName()` only under `!BUILDFLAG(CHROMIUM_BRANDING)` — our build is Chromium-branded, so only `Chromium` + GREASE. Already takes `additional_brand_version` |
| same file `:: GetUserAgentMetadata(bool only_low_entropy_ch)` | ✅ at `.187` | ⭐ both `brand_version_list` (→ `Sec-CH-UA`, `brands`) and `brand_full_version_list` (→ full-version-list) come from `GetUserAgentBrand{Major,Full}VersionListInternal(std::nullopt)` — **one** function feeds both, so a single patch point is plausible. **Code reading — `P4-A1`…`A5` are what make it a measurement** |
| same function, custom-UA branch | ✅ at `.187` | if a UA override is given on the **command line**, only low-entropy hints (or blank, under `kUACHOverrideBlank`) are returned. Hodos sets **no** UA override today (grep of `cef-native/src`, `cef_browser_shell.cpp`, `cef_browser_shell_mac.mm` for `user_agent`: no hits) — a future override would silently change this phase's surfaces |
| Chromium `chrome/browser/chrome_content_browser_client.cc :: ChromeContentBrowserClient::GetUserAgentMetadata` | ✅ at `.187` | returns `embedder_support::GetUserAgentMetadata()`; the fork's `libcef/` has **no** caller of its own (grep: none) — CEF uses the Chrome runtime's client |
| fork `patch/patch.cfg` (Hodos block) + `HODOS_PATCHES.md` | ✅ at `9ccef044f` | 7 `hodos_*` entries today, all under `HODOS_FARBLING` ("one gate for the whole farbling set, never per-patch"). The brand is **not** farbling ⇒ its own condition, exported by both build scripts |
| `development-docs/DevOps-CICD/scripts/build_hodos_cef.bat`, `build_hodos_cef_mac.sh` | ✅ | export `HODOS_FARBLING=1`; the brand's condition is added next to it |
| `development-docs/DevOps-CICD/scripts/cef_patch_drift_audit.sh :: HODOS_MIN_PATCHES` | ✅ default `1` | the presence floor. Raising it to 8 is optional (the file says so); ⛔ if raised, **its own commit** (working rule 6) |
| `development-docs/0.4.0-beta.3/phase-13-bot-detection/p13_signals.py` | ✅ exists | the tab-subject probe to reuse |

⚠️ **Re-verify the two Chromium rows at the `.255` tree** once Build 1 has synced it — SCOPE's `.255` reading was via
gitiles, and the local tree is still `.187`.

**Prior art (working rule 5).** **Brave** — measured on this machine 2026-09-21: UA string byte-shaped like Chrome's (no
"Brave" token), Client Hints name Brave: `Brave;v=153, Not_A Brand;v=8, Chromium;v=153`. ⭐ Followed: own brand in the
hints, UA string unchanged, brand version = the Chromium version. Pattern only (MPL-2.0). **Chrome** — `Google Chrome`
brand; ⛔ not followed, by the owner's 2026-09-21 decision. **Tor / Mullvad** answer fingerprinting with uniformity, and
a unique brand is the opposite of uniform — that disagreement is the design question, and the owner already decided it
(`STEP0_AND_SIGNAL_SHEET.md` §Addendum: "makes us nameable"). **Docs to cite before the diff (rule 4):** the WICG UA
Client Hints spec (brand list, GREASE) and the Chromium `additional_brand_version` call sites.

## 6. Out of scope

- The UA **string** (unchanged by design).
- Any C++ header rewrite in `cef-native/` (⛔ banned — §3).
- Signal `B2` (`window.hodosBrowser` / `CWI` / `__hodos_*` globals) — that is our own code, T6/T5 territory.
- Bot-vendor reaction to a new brand — **not measurable before shipping**; Brave's precedent is the only evidence (SCOPE P4 unknown 2).
- Re-running beta.3 Phase 13's challenge matrix — its negative control never went red (`STEP0_AND_SIGNAL_SHEET.md` §`P13-C`).

## 7. Rollback

Before promotion: revert `env.CEF_ASSET` (both arms) to Build 1's assets, or rebuild with the brand's condition unset.
⚠️ **After** `v0.4.0-beta.6` ships the brand has been **seen**; reverting is technically one commit and socially sticky
(SCOPE §6c). That is why `P4-A0` and §12 Q1 exist.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| Header patched, `brands` in JS not (or the reverse) — the new inconsistency the addendum warns is worse than nothing | `P4-A3` + its comparator control |
| Low-entropy agrees, `fullVersionList` does not (two call sites, one patched) | `P4-A5` |
| Workers read UA metadata through a different path and still say two brands | `P4-A4` |
| The GREASE brand disappears or duplicates because the patch appends where the shuffle expects a fixed count | `P4-A1` (exactly three entries) |
| The patch leaks into the UA string | `P4-A6` |
| The test measured an overlay or the header browser, which report the same metadata but are not what sites see | SUBJECT column (tab role); reused `p13_signals.py` assertion |
| Stale in-tree `src/cef` copy builds green without the brand patch | `P4-A7` (by name) + P5-lite `L2` |
| The brand is seen by third parties during testing before anyone checked it is consistent | `P4-A0` |
| A future UA override (e.g. a user-agent setting) silently drops high-entropy hints | recorded in §5; not a row — no override exists today |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | `P4-A0`–`A8` | build host |
| macOS | `P4-A9` (repeats `A1`–`A6`, `A8`) | 🍎 Mac agent on the staged Build 2 framework. `P4-A7` is platform-specific per build log — run on each platform's build log |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **≈ 0.25 h** — read the agreement table and answer §12 Q1 before any third-party exposure |
| Human-bound rows | none of the rows needs a person (all instrument-insensitive — the argument in `STEP0_AND_SIGNAL_SHEET.md` "Why CDP is legitimate"). ⛔ Shipping the brand is the owner's call at G10 with the rest of the release |
| Unknowns (K) — uncertainty, not difficulty | **1:** whether one patch point feeds **every** surface (navigation + subresource header, `brands` in window and workers, `fullVersionList`). Code reading says yes; unmeasured |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | T0 Build 1 (`../SECURITY_RELEASE_PLAN.md`) | the `.255` tree to author against; Build 1's engine as every RED's feature-off build |
| gives | `../phase-P5lite-build2-verify/` | the brand patch; built and published there |
| batched with | `../phase-P3-adblock-payload-pull/` | one Tier-1 build |
| related | T6 Group 3 "window globals" (signal `B2`) and T5 "dApp-reachable surface" | same bot-signal family, opposite direction (P4 makes us *nameable*; B2 makes us *identifiable*). Not a dependency — ⭐ the echo page built for `P4-A1` can record `B2`'s globals in the same load if those phases want it |

## 12. Open questions for the owner

1. **Third-party exposure during testing.** P5-lite's basket (youtube, x, github) and any Build 2 dev browsing send the
   brand to real sites before release. ⭐ Recommendation: accept it **after** `P4-A0`–`A6` pass on our own echo server —
   a handful of sites seeing a dev build is not "shipping", and the rows that make it safe will have run first.
2. None other — the brand name and shape are decided (2026-09-21).

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
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
