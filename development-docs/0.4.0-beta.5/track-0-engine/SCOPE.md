# B5-T0 — Engine · Track scope (G2)

**Written:** 2026-09-25, G2 research agent (Windows). **Status:** 📌 PROPOSED — owner decisions owed in §9.
**Inputs read:** root `CLAUDE.md`, `../../RELEASE_CYCLE.md` §2/§3.1a/§3.3/§3.7/§4.2, `../README.md`
(planning notes D1/D2, G1, G2), `../../KNOWLEDGE_AND_MEMORY.md`, `./README.md`, the one ticket, and the
DevOps-CICD engine docs named in §3.

> **Claim labels used below.** **code** = read in this repo or in the fork checkout on the build host
> (`C:\cef\cef150\cef`, read-only). **measured** = a command was run and its output is quoted.
> **doc** = a repo document says so and I did not re-check. **web** = fetched 2026-09-25, URL given.
> Nothing was built, staged, committed or run in the browser.

---

## ✅ 0. G2 decisions applied (owner, 2026-09-27)

> The research below is the **evidence**; this block is the **decision**. Where they differ, this block wins.
> Full record: `../README.md` → "✅ Decisions as made".

| Question | Decided |
|---|---|
| §9 **Q1** — does the security refresh wait? | ✅ **(b)+(c), refresh only (decision 1).** **Build 1 = P1 → P2 → P5**, refresh to `.255` only, ships **alone** as **`v0.4.0-beta.5`** — out of cycle, plan in `SECURITY_RELEASE_PLAN.md`. **P4 (brand) moves to Build 2** with P3 — it is irreversible once seen and its header-vs-JS agreement is unmeasured. Promotion is an owner call after P5 |
| Version name | `v0.4.0-beta.4.1` rejected: `release.yml`'s build-number parser scores it **99 = final** (`40099`), outranking every later beta in Sparkle and `UpdateStager::IsNewerBuild`. The planned release becomes `v0.4.0-beta.6` |
| Q2 b3 · Q3 merge · Q4 CRLF fix in P1 · Q5 record M160 window | Taken **per this doc's recommendations** at G3 (owner approved the agent-level set). ⚠️ Q3's fork push is outward-facing — owner approves at the time |

---

## 1. Goal

Move the shipped engine from `chromium-150.0.7871.187` to the newest build on the **same** long-term
CEF branch (7871 → `chromium-150.0.7871.255`), carrying our farbling patches unchanged, and add the two
queued engine fixes — the ad-blocker payload pull and a `"Hodos"` brand in `Sec-CH-UA` — **without
losing farbling, codecs, or any shell safeguard**, on both Windows and macOS.

⭐ **Refinement since the README was written:** the refresh is no longer routine hygiene. The newest
7871 build carries the fix for an **actively exploited** V8 bug (§2.3) that our shipped engine does not
have. That changes the urgency, and possibly the order of the phases (§9, Q1).

---

## 2. Telescope — the state of the world today

### 2.1 What I re-fetched

| Source | What it says (2026-09-25) | Label |
|---|---|---|
| CEF build index `https://cef-builds.spotifycdn.com/index.json` (10.4 MB, fetched) | Newest on our branch: **`150.0.21+ga61e9a5+chromium-150.0.7871.255`**, published **2026-09-23** (Windows, macOS arm64, macOS x64). Between ours and it: `150.0.18` (`.213`, 08-14), `150.0.19` (`.252`, 08-27), `150.0.20` (`.253`, 09-11). ⇒ **four upstream refreshes behind**, not three | web + measured |
| same index, other branches | 8037/M154 `154.0.28` (09-25) · 7977/M152 `152.0.10` (09-24) · 7559/M144 `144.0.35` (09-13) | web + measured |
| `git ls-remote https://github.com/chromiumembedded/cef.git refs/heads/7871` | upstream 7871 head = **`a61e9a5c7b50…`** = the newest published build (no unreleased commits waiting) | measured |
| GitHub compare API `chromiumembedded/cef/compare/94c1726...a61e9a5` | **4 upstream commits**, all "Update to Chromium version …" (`db11278`, `cf60f42`, `a832838`, `a61e9a5`). Files touched: `CHROMIUM_BUILD_COMPATIBILITY.txt`, `libcef/browser/osr/render_widget_host_view_osr.{cc,h}`, and 11 of **upstream's own** `patch/patches/*.patch`. **None** of them is a file our fork changes | web + measured |
| Chromium gitiles `+log/150.0.7871.187..150.0.7871.255?name-status=1` (579 commits paged) | **106 `[M150]` merge commits** (cherry-picked fixes), 19 of them in Blink. **Zero** commits touch any of the 16 Blink files our 7 farbling patches edit (list in §4) | web + measured |
| Chromium `DEPS` at both tags | V8 `49df3678…` (≈ V8 15.0.245.21) → `4ceb8016…` (15.0.245.40): **18 V8 security merges** in between | web + measured |
| `https://chromiumembedded.github.io/cef/branches_and_building` | 8037/154 Stable → Oct 5 2026 · 7977/152 Extended → Oct 20 2026 · **7871/150 LTC → Apr 13 2027** · 7559/144 LTS → Oct 6 2026. LTC/LTS = "platform-agnostic security fixes for ~9 months"; after M150 the next long-term branch is **M160**, then every 12th milestone | web |
| chromiumdash `fetch_milestone_schedule?mstone=160` / `=150` | **M160: branch 2026-11-30, stable 2027-01-05, LTC 2027-01-13.** M150: stable 2026-06-30, LTC 2026-07-21 | web |
| Chromium `components/embedder_support/user_agent_utils.cc` @ `150.0.7871.255` | `GetUserAgentBrandList` sets `brand = version_info::GetProductName()` **only when not `CHROMIUM_BRANDING`**; our build is Chromium-branded, so only `"Chromium"` + GREASE are emitted. The function already takes an `additional_brand_version` parameter | web (code reading of upstream) |

### 2.2 What changed versus our repo docs

| Repo doc says | Today | Consequence |
|---|---|---|
| Ticket + README D1: upstream is at `.255` | ✅ still true — `.255` is also the branch **head** | none |
| `CEF_VERSION_UPDATE_TRACKER.md` §"OWED AT THE NEXT BUILD" (2026-09-16): **"CVE-2026-85046 is NOT on our branch at all … assume our branch stays unpatched"** | ⛔ **No longer true.** See §2.3 — the fix was merged to M150 on 2026-09-01 and is in `.255` | Premise of the owner's 2026-09-16 "do not rebuild now" decision has changed ⇒ Q1 |
| Tracker: "three refreshes behind" (`.253` newest) | **four** (`.255` published 2026-09-23) | update tracker at G3 |
| README D2: next long-term branch "M160, expected around spring 2027 *(estimate)*" | chromiumdash: **M160 stable 2027-01-05, LTC 2027-01-13** | The 150→160 move window is **Jan 13 → Apr 13 2027** (~3 months), not "spring". Plan it into beta.6/beta.7, not later |
| Ticket's option table (A/B/C) | Decided (D1 = A) | The ticket's candidate "phase 1 — target selection" closes |
| `CEF_BUILD_RUNBOOK.md` "Current known-good configuration" table still says branch `7103` | 7871 since 2026-08-04 | stale doc row — report only (rule 3), fix in the phase that edits the runbook |

### 2.3 ⭐ The security fact that matters most

- **CVE-2026-85046** — V8 type confusion, CVSS 8.8, **exploited in the wild**, fixed upstream in Chrome
  152.0.7977.82 on 2026-09-03 (web: `https://thehackernews.com/2026/09/google-releases-chrome-update-to-patch.html`,
  fetched 2026-09-25). Our tracker names the V8 fix as `e0562d87ad9c…` (doc).
- `e0562d87` ("[compiler] Don't inline Array.prototype.sort on mixed elements kinds", **Fixed: 542403045**)
  was cherry-picked to M150 as **`085f76513d` "[M150] [compiler] Don't inline Array.prototype.sort on
  mixed elements kinds"** (Bug 546215236, 542403045), committed **2026-09-01** — and `085f765` is an
  ancestor of the V8 revision `4ceb8016` that Chromium **`150.0.7871.255`** pins (web + measured, V8
  gitiles log from `4ceb8016`).
- Our `.187` pins V8 `49df3678` (July), which predates the merge (measured). Public write-ups reference a
  working exploit against `150.0.7871.181` (web search summary, 2026-09-25 — not independently read).
- ⇒ **The in-branch refresh closes an actively exploited renderer bug.** The tracker's route B
  ("backport the V8 commit by hand") is unnecessary.

---

## 3. Kaleidoscope — do we already have this?

### 3.1 Reuse — existing shapes this track should extend, not duplicate

| Need | Existing thing | Where |
|---|---|---|
| Browser-side store the renderer **pulls** from at context creation | `hodos::FarblingRegistry` + the intercept of `hodos_farble_key` in `CefFrameHostImpl::SendProcessMessage` + mojom `GetHodosFarblingKey(string host) => (string key_hex, bool enabled)` + `CefFrameImpl::MaybeApplyHodosFarblingKey` | fork: `libcef/browser/hodos_farbling_registry.{h,cc}`, `libcef/browser/frame_host_impl.cc`, `libcef/common/mojom/cef.mojom`, `libcef/renderer/frame_impl.cc` (code) |
| Our renderer-side scriptlet injection, escaping, logging | `s_scriptCache` (URL → JS, one-shot) consumed in `OnContextCreated`; `preload_cosmetic_script` handler with the P12 late-arrival inject | `cef-native/src/handlers/simple_render_process_handler.cpp` (code) |
| The browser-side push sites | `OnBeforeBrowse` early push + the P12 `OnLoadStart` re-push | `cef-native/src/handlers/simple_handler.cpp` — both `CefProcessMessage::Create("preload_cosmetic_script")` sites (code) |
| Patch apply health before a build | `cef_patch_drift_audit.sh` (exit 0/2/1; presence gate `HODOS_MIN_PATCHES`), `cef_dist_drift_audit.sh`, `cef_gn_args_gate.sh` | `development-docs/DevOps-CICD/scripts/` (code) |
| Build + pin | `build_hodos_cef.bat` / `build_hodos_cef_mac.sh` (`CEF_CHECKOUT`, `--force-cef-update` unconditional); pin-as-tag convention `refs/tags/pin-<sha7>/7871` | scripts (code); runbook §"The source pin is a tag" (doc) |
| Engine identity in CI | `release.yml` derives the expected engine from `env.CEF_ASSET` (lines 145 and 589, both still `…-150.0.43-g9ccef04`) and asserts it out of the artifact | code |
| Farbling proof | `farbling_seed_rotation_check.py` (+ `--negative-control`), token `engine=` bound to the fork SHA by `promote.yml` | `development-docs/0.4.0/chromium-rebuild/` (code), `FARBLING_RELEASE_GATE.md` (doc) |
| Baseline to diff the new build against | `BASELINE_CEF150.md`, `regression_soak.py`, `farbling_perf_check.py`, `farbling_iframe_perf_check.py` + saved p4e/p4f baselines | doc + code |
| UA-CH brand hook | Chromium's own `additional_brand_version` parameter on `GetUserAgentBrandList` | web |

### 3.2 Shapes we would otherwise duplicate

- ⚠️ **A new public CEF API (option b1) duplicates a channel we already have.** Our code already talks
  to the renderer through `OnProcessMessageReceived`; a new `CefFrame::GetCosmeticPayload()`-style API
  would be a second channel for the same payload. The fork's CEF also has **API versioning**
  (`cef_api_versions.json`, `tools/version_manager.py`, `translator.py`, `cef_api_hash.py` — code), so a
  public addition is not just a header: it is translator-generated C API + a version entry, re-litigated
  at every bump. See §5 P3 and Q2 for the alternative that avoids this.
- ⚠️ **A second registry class** is fine (the payload is ~34 KB vs a 32-byte key, so lifetime/eviction
  differ — P12 contract §5), but it should copy `FarblingRegistry`'s **shape**, not generalise it: the
  farbling one is `enabled`-gated and host-keyed, the cosmetic one is URL-keyed and one-shot.
- ✅ No duplicate needed for the brand: one call site in Chromium feeds both the `Sec-CH-UA` header and
  `navigator.userAgentData.brands` (inference from upstream code — **to be measured**, §5 P4).

### 3.3 Found while looking (report, not fix — working rule 3)

🔴 **Six libcef files in the fork were committed with CRLF line endings — whole-file rewrites.**
Measured on raw blobs (Python byte count): at upstream base `94c17267e` all six are LF (e.g.
`frame_host_impl.cc` 0 CRLF / 769 lines); at our pin `9ccef044f` every line is CRLF (815/815).
Affected: `libcef/browser/frame_host_impl.{cc,h}`, `libcef/browser/browser_frame.{cc,h}`,
`libcef/common/mojom/cef.mojom`, `libcef/renderer/blink_glue.h`. Introduced by
**`116b7fd8b` — "C2: replace the farbling-key PUSH with a renderer-side [Sync] PULL"** (2026-08-07).
`git diff --stat` shows +3934/−1407 lines; with `--ignore-cr-at-eol` the real fork delta is **+848/−1**.
**Harmless for this refresh** (upstream touched none of these six files since our base), but **any**
future upstream edit to one of them — very likely at the M160 bump — becomes a whole-file conflict. ⇒ Q4.

---

## 4. Ticket review

| Ticket | Still true today? | Becomes | Why |
|---|---|---|---|
| `TICKET_engine_behind_its_own_cef_branch_and_upstream_stable.md` — **the engine is behind its own CEF branch and two milestones behind Chromium stable** | ✅ **Yes, and understated.** Verified: shipped pin `150.0.43-7871.3576+g9ccef04+chromium-150.0.7871.187` (`cef-binaries/include/cef_version.h`, code); `CEF_ASSET` still `…150.0.43-g9ccef04` in both `release.yml` arms (lines 145, 589, code); upstream 7871 head is `.255` (measured). Stable is now **154.0.8037.58** (index, measured). New: `.255` contains the exploited-V8 fix (§2.3) | **The whole track.** Its candidate phases map to §5: its phase 1 (target) **closes** by D1; phase 2 → P1; phase 3 → P3 + P4; phases 4–6 → P5 + P6 | Option A chosen; the ticket's phase table was written before the CVE fact and before the CRLF finding |

**Fork patch re-application — how much applies cleanly within the branch?** (the question the prompt asked)

| Patch / change | Kind | Files | Changed upstream `.187 → .255`? | Expected |
|---|---|---|---|---|
| C1 `hodos_farble_session_cache` | `.patch` (Blink) | `core/execution_context/build.gni` + 2 new files | no | ✅ clean |
| C3 `hodos_farble_canvas2d` | `.patch` | `base_rendering_context_2d.cc`, `html_canvas_element.cc` | no | ✅ clean |
| C4 `hodos_farble_webgl` | `.patch` | `webgl_rendering_context_base.cc` | no (a sibling WebGL file changed: "Validate framebufferTextureMultiviewOVR") | ✅ clean |
| C5 `hodos_farble_webaudio` | `.patch` | `audio_buffer.{idl,h,cc}`, `analyser_node.cc` | no (a sibling file changed: "Fix duplicate 'ended' event in AudioBufferSourceNode") | ✅ clean |
| C6 `hodos_farble_navigator` | `.patch` | `navigator_base.{h,cc}`, `navigator_device_memory.h` | no | ✅ clean |
| P4f `hodos_farble_offscreen_canvas` | `.patch` | `offscreen_canvas.cc` | no | ✅ clean |
| P4f `hodos_farble_worker_key` | `.patch` | `global_scope_creation_params.h`, `dedicated_worker.cc`, `dedicated_worker_global_scope.{h,cc}` | no | ✅ clean |
| C2 + P4e registry pull | **direct fork commits** to `libcef/` (not `.patch` files) | 8 libcef files + `BUILD.gn` + `patch/patch.cfg` | **no** — upstream's 4 commits touch none of them | ✅ clean merge/rebase |

(measured — gitiles name-status aggregation and GitHub compare, both 2026-09-25.)
⚠️ Exact-context `git apply -p0` can still fail on a changed **line in a file we touch** that my path
check would catch — it did not — but the **authoritative** answer is `cef_patch_drift_audit.sh` run
against the `.255` tree **before** the build, which is P1's exit condition. Expectation: exit `0`.

---

## 5. Candidate phases

Ordered by uncertainty first (§3.7). Short ids per the naming convention: **B5-T0-P1 …**

### B5-T0-P1 — Bring the fork onto upstream 7871 head
- **Objective:** `hodos/7871` contains upstream `a61e9a5` plus all our commits; a new pin tag; drift
  audit clean against a `.255` source tree; `CEF_CHECKOUT` bumped in both build scripts.
- **Tickets:** the engine ticket (its phase 2).
- **Unknowns:** (1) **merge vs rebase** — a rebase rewrites published SHAs and needs a force-push of
  `hodos/7871` (outward-facing); a merge does not. Old pins are safe either way (they are tags,
  measured via `ls-remote`). (2) Whether a merge commit changes `cef_version.py`'s `PATCH` count /
  decoration so the version string still matches `release.yml`'s asset-name regex — small, checkable.
  (3) 🍎 **Mac build-host state:** the runbook says the Mac host's `chromium/src/.git` was **deleted**
  to reclaim space (doc). Moving from `.187` to `.255` means the tree is **not** at the target version,
  so the runbook's cheap recovery (`--no-chromium-history` reuse) does **not** apply — Mac needs a
  fresh no-history fetch of `src` (tens of GB). Windows is fine: `src/.git` is a full-history 83 GB
  repo at `150.0.7871.187`, not shallow, 879 GB free (measured) ⇒ an **incremental** update.
- **Negative control:** run `cef_patch_drift_audit.sh` against a tree with one Hodos patch's target
  deliberately edited ⇒ must exit `1`. And the build log's `N patches total` must list **every**
  `hodos_*` patch as applied — the stale in-tree `src/cef` copy (`--force-cef-update` trap) scores a
  green run with zero Hodos patches.

### B5-T0-P2 — Security refresh build and publish *(could ship alone — Q1)*
- **Objective:** Tier-1 build of the P1 pin on **both** platforms; new versioned assets
  `cef-binaries-{windows,macos}-<cefver>-g<sha>.{zip,tar.bz2}`; `env.CEF_ASSET` bumped in both arms in
  one commit; `workflow_dispatch` validation build green before any tag.
- **Unknowns:** none of substance — hours, not uncertainty (Windows ~5 h per past farbling builds, doc).
- **Negative control:** point the macOS arm's `CEF_ASSET` at the **old** asset name ⇒ the engine-binding
  step must fail (not build green). Subject: `CEF_VERSION` read out of the **downloaded** artifact —
  never the Chromium version (P4e and P4f both said `.187`).

### B5-T0-P3 — Ad-blocker payload pull (beta.3 Phase 12 candidate (b))
- **Objective:** a cross-site arrival injects the scriptlets **inside** context creation (0 ms late),
  so an inline `<script>` at the top of `<head>` sees the mutated surface.
- **Tickets:** engine ticket phase 3(i); spec `../../0.4.0-beta.3/phase-12-adblock-redirect-arrivals/PHASE_CONTRACT_registry_pull.md`.
- **Unknowns (genuine):** (1) **the shape — Q2** (b1 / b2 / the b3 hybrid below). (2) **Ordering:** in
  `libcef/renderer/render_frame_observer.cc :: DidCreateScriptContext`, the client's
  `handler->OnContextCreated` runs **before** libcef's `framePtr->OnContextCreated` (where the farbling
  pull lives) — code. The cosmetic pull must run **before** the client callback, i.e. at a different
  point from the farbling precedent. (3) **Cost of a ~34 KB synchronous IPC** per document on the
  critical path (Brave keeps its sync load behind a feature flag — `PRIOR_ART.md` 2026-09-19 row).
  (4) How the P12 `OnLoadStart` re-push interacts once the browser side intercepts the message name.
- **Negative control:** the contract's rows `P12-B1…B5`; ⛔ **`P12-B4` (inline head script) is the
  row that proves the patch buys anything**, and its RED is "revert the patch ⇒ un-mutated surface".
  Fresh URL per trial (the one-shot cache scores false greens).

### B5-T0-P4 — `"Hodos"` in `Sec-CH-UA`
- **Objective:** header and `navigator.userAgentData.brands` both read
  `"Hodos";v="150", "<GREASE>", "Chromium";v="150"`; UA string unchanged.
- **Tickets:** engine ticket phase 3(ii); spec `../../0.4.0-beta.3/phase-13-bot-detection/STEP0_AND_SIGNAL_SHEET.md` §Addendum. Owner already chose "our own brand" (2026-09-21).
- **Unknowns:** (1) whether the one upstream function feeds **every** surface — header on navigation,
  header on subresources, `brands`, and `getHighEntropyValues().fullVersionList` (inference, must be
  measured; the NEXT_CHROMIUM_BUILD row warns a header/JS mismatch is **worse than doing nothing**).
  (2) Bot-vendor reaction to an unknown brand — not measurable before shipping; Brave's precedent is
  the only evidence.
- **Negative control:** build with the brand patch's condition unset ⇒ header shows only two brands.
  Subject check: measured from a **tab** browser (tls.peet.ws echo + `brands` in the same page), never
  from the header/overlay browsers CDP also lists as `page`.

### B5-T0-P5 — Stage and verify, both platforms
- **Objective:** the release-boundary evidence for the new engine.
- **Rows:** staging per `cef-native/CLAUDE.md` (Windows bootstrap model; ⛔ never merge-copy a
  distribution; macOS ignores `CEF_ROOT`, staging mandatory) · **farbling rotation token** with
  `engine=` = new `CEF_VERSION`, plus `--negative-control` red · codecs (`canPlayType` avc1/mp4a →
  `probably`, real YouTube playback) · macOS `vtool` minos of the new framework, floor stays
  `max(12.0, measured)` · minimal basket + DPI cells #4/#6/#9 · diff against `BASELINE_CEF150.md`
  (soak, perf) · `REGRESSION_SET.md` rows that exercise the shell on a new engine, **`R-GOLD` included**
  (the gold pill rides the render-process IPC relay).
- **Unknowns:** none beyond platform time.
- **Negative control:** each row carries its own (farbling gate's built-in inversion; a codec check
  against the stock prebuilt CEF, which must say `""`).

### Recommended build plan (feeds Q1)

| Plan | Builds | Security fix ships | Owner sittings |
|---|---|---|---|
| **One build** (P1 → P3 → P4 → P2 → P5) — `NEXT_CHROMIUM_BUILD.md` "batch it" rule | 1 per platform | when P3's design + authoring finish (weeks) | 1 verify round |
| ⭐ **Two builds** — Build 1 = P1 + P4 (both near-zero unknowns) → P2 → P5 ; Build 2 = P3 → P5-lite | 2 per platform | **now** (days) | 2 verify rounds (second is smaller) |

---

## 6. Integration check (RELEASE_CYCLE §3.3)

**(a) What invariant or safeguard could this violate?**
- **Farbling** (native, fork patches) — silently absent if the in-tree CEF copy is stale, or if a
  stale macOS asset is pulled (**"builds green and ships with no farbling"**, doc). Guarded by the
  presence gate, the `release.yml` engine binding, and the rotation token.
- **Gold pill** (`payment_success_indicator`, relayed by `simple_render_process_handler.cpp`) — any
  engine change touches the render-process IPC it rides. P3 edits the **same** renderer callback path.
- **Invariant 8** (CEF lifecycle/threading — "do not change render-process handlers without asking"):
  P3 changes **when** our render-process handler receives a message. That is exactly the ask; Q2 is it.
- **Privacy perimeter / wallet bridge** — the `wallet_call` process-message bridge and the
  `X-User-Approved` re-issue path run through CEF; a refresh inside one branch should not change them,
  but R-PERIM / R-INTEXT must be re-run on the new engine, not inherited.
- **macOS floor** — a new framework could change `minos`; the CI minos guard catches it (doc).
- **Invariant 9** — every C++ change in `cef-native/` gets a macOS relay note (Branch rule, root `CLAUDE.md`).

**(b) What does it touch that we did not write?**
CEF (libcef source edits in our fork, CEF's API-versioning/translator tooling if b1), Chromium/Blink,
V8, `components/embedder_support` (UA-CH), depot_tools/gclient/`automate-git.py`, the Windows SDK / VS
and Xcode 26.5 toolchains, the org repo's `cef-binaries` GitHub release, the UA Client Hints spec
(WICG), and third-party bot-detection vendors' allow-lists (they key on brand names). ⭐ The
`.255` upstream CEF delta itself is tiny (4 version-bump commits) — the unknowns are in **our** patches
and toolchain, not in CEF.

**(c) What would we have to un-ship if wrong?**
- **Refresh (P1/P2):** cheap to roll back — assets are never clobbered, so revert `CEF_ASSET` in both
  arms to `…150.0.43-g9ccef04`. Cost: re-opens the exploited V8 bug.
- **Brand (P4):** a brand seen in the wild cannot be un-seen; sites/vendors may start keying on
  `"Hodos"`. Reverting is technically trivial, socially sticky.
- **Ad-block pull (P3):** with **b1** the app depends on a new engine API ⇒ app and engine must roll
  back **together**. With **b2** the injection logic lives only in the engine. With the **b3** hybrid
  the app is unchanged, so an engine rollback alone restores today's behaviour (the mitigation).

---

## 7. Feasibility inputs (for G5.5)

**Owner-hours (N) — estimate, to be scored in the AAR**

| Human-bound row | Hours |
|---|---|
| Decisions Q1–Q5 (one sitting) | 0.5 |
| Approve outward-facing steps: fork push + pin tags, org-repo asset upload, `CEF_ASSET` bump, validation dispatch | 0.25 per build |
| Windows verify: install/upgrade smoke of the CI build, YouTube codec check, basket, DPI cells #4/#6/#9 | 1.5 |
| macOS verify: same basket + codecs on the staged build, minos sanity, relaunch | 1.5 (via the Mac agent; owner at keyboard for native input/visual) |
| `R-GOLD` real-money payment on the new engine | 0.5 |
| P3 acceptance visual spot-check (ads gone on first paint, cross-site) | 0.25 |
| **Total — one build** | **≈ 4.5 h** |
| **Total — two builds** (second round is smaller: basket + P3 rows + codecs only) | **≈ 6.5 h** |

Build-host time (not owner time): Windows incremental from an intact tree — hours (≈5 h per past
farbling build, doc); macOS likely a **fresh** `src` fetch first (§5 P1 unknown 3) — half a day or more.

**Unknowns (K): 3 phases** — P1 (Mac host tree; merge-vs-rebase version string), P3 (shape, ordering,
sync-IPC cost), P4 (one call site covers every surface?). P2 and P5 are hard-but-understood.

**Cross-track dependencies**
- **Gives to every track with browser-level evidence** — T1 (money path, `R-GOLD`), T4 (402 payments
  run through `HttpRequestInterceptor`'s resource handlers), T5 (identity & privacy; the brand is a
  privacy signal), T6 (browser shell, overlays, DPI). ⭐ Land the engine **before** those tracks gather
  their browser-level evidence, so it is gathered once (ticket §"Why it sequences early").
- **Needs from:** the macOS agent (build + stage + verify on Mac — relay round), and owner decisions.
- **No compile dependency** for `rust-wallet/` or `frontend/` (ticket's own correction, confirmed).

---

## 8. Prior-art rows (ready to paste into `development-docs/PRIOR_ART.md`)

| Date | Question | What we looked at | Finding | Worth it? | Used by |
|---|---|---|---|---|---|
| 2026-09-25 | Is our in-branch engine refresh routine, or security-urgent? | CEF `index.json`; GitHub compare `94c1726...a61e9a5`; Chromium gitiles log `150.0.7871.187..255` (name-status); Chromium `DEPS` at both tags; V8 gitiles log from `4ceb8016` | ⭐ The exploited V8 bug CVE-2026-85046 (`e0562d87`) **was** merged to M150 as `085f76513d` on 2026-09-01 and is in `.255` — our tracker's "not on our branch" is outdated. Upstream CEF changed 4 version-bump commits, none touching our fork; zero Chromium commits touched our 16 Blink patch targets | 🟢 paid off — turned a hygiene task into a security fix and sized the patch rebase at ~zero | B5-T0 SCOPE |
| 2026-09-25 | When does the next CEF long-term branch arrive? | `chromiumembedded.github.io/cef/branches_and_building`; chromiumdash milestone schedule M150/M160 | LTC/LTS = every 12th milestone from M160; **M160 LTC 2027-01-13**, 7871 ends 2027-04-13 ⇒ ~3-month overlap, earlier than our "spring 2027" estimate | 🟢 | B5-T0 SCOPE, D2 |
| 2026-09-25 | Where does Chromium decide the `Sec-CH-UA` brand list? | Chromium `components/embedder_support/user_agent_utils.cc` @ `150.0.7871.255` | `GetUserAgentBrandList` omits the product brand under `CHROMIUM_BRANDING`, and already has an `additional_brand_version` hook — a one-function patch; Brave does the same job (pattern only, MPL-2.0) | 🟢 | B5-T0-P4 |
| 2026-09-25 | Can the ad-block pull reuse our farbling pull without a new public CEF API? | Our fork: `render_frame_observer.cc :: DidCreateScriptContext`, `frame_impl.cc :: SendMessage`, CEF `tools/version_manager.py` / `cef_api_versions.json` | Yes in principle: libcef can pull and hand the payload to the client's **existing** `OnProcessMessageReceived` before calling `OnContextCreated` (b3). A public API (b1) would carry CEF's API-versioning cost at every bump | 🟡 design lead, unverified until built | B5-T0-P3, Q2 |

---

## 9. Open questions for the owner

**Q1 — Does the security refresh wait for the ad-blocker patch?** *(the biggest one)*
On 2026-09-16 you chose "don't rebuild now, fold into the next build" (tracker). The reason given then
— the exploited V8 bug was not on our branch — **is no longer true**: `.255` has the fix (§2.3).
Options: (a) one build carrying all three changes (fewest sittings; fix ships in weeks);
(b) **two builds** — refresh + brand now, ad-block pull second (≈ +2 owner-hours; fix ships in days);
(c) (b) and also ship build 1 as an out-of-cycle hotfix, like beta.4 was.
⭐ **Recommendation: (b)**, and decide (c) once build 1 has passed P5 — an exploited-in-the-wild
renderer bug in a browser that moves money is the case working rule 7's spirit covers, but promotion is
outward-facing and yours.

**Q2 — Ad-blocker pull shape: b1, b2, or a third option (b3)?**
- **b1** (contract's recommendation): new public CEF API our code calls. Keeps logic testable, but adds
  API surface that CEF's versioning tooling makes us re-generate every bump, and couples app+engine
  rollback.
- **b2:** libcef injects itself. No API, but logic moves where our tests can't reach.
- **b3 (new, found in this pass):** libcef files the payload browser-side (exactly like farbling) and,
  in the renderer, pulls it **just before** calling our `OnContextCreated`, delivering it through the
  message callback we already handle. Our injection code stays unchanged and testable; no new API;
  old app + new engine and new app + old engine both keep working.
⭐ **Recommendation: b3**, with b1 as the fallback if the sync pull cannot be placed before our callback
or proves too slow (P3 unknowns 2–3). This supersedes the "b1 vs b2" decision owed in
`NEXT_CHROMIUM_BUILD.md`.

**Q3 — Merge or rebase upstream into `hodos/7871`?** Rebase = linear history but a force-push to a
published branch. Merge = no rewrite. ⭐ **Recommendation: merge** — the four upstream commits touch
none of our files, so it is conflict-free, and nothing outward-facing gets rewritten.

**Q4 — Fix the fork's CRLF line endings now?** (§3.3) Not needed for `.255`; will bite at the M160 bump.
⭐ **Recommendation: yes, as its own fork commit in P1** (line endings only, verified by
`git diff --ignore-cr-at-eol` being empty), so the M160 rebase starts clean.

**Q5 — Record the M160 window now?** 7871 support ends 2027-04-13; M160's long-term branch starts
2027-01-13. ⭐ **Recommendation:** amend D2's "spring 2027" to **"M160, LTC 2027-01-13; move between
Jan and Apr 2027"** and open a beta.6 intake ticket for it.
