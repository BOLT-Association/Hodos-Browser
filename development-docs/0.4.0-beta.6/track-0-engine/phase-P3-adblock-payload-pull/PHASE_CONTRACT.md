# B5-T0-P3 — The ad-blocker's early scripts reach the right page on a cross-site arrival · PHASE CONTRACT

**Track:** B5-T0 Engine — **Build 2** · **Tickets:** `../../tickets/TICKET_engine_behind_its_own_cef_branch_and_upstream_stable.md` (its phase 3(i)) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** **1** (two builds; this is Build 2, inside `v0.4.0-beta.6`) · **T0 Q2 → b3** (taken at G3 per `../SCOPE.md` §9 Q2's recommendation, which the owner approved as part of the agent-level set — `../../README.md` "G2 — what is still owed" item 3) · **T0 Q3 → merge** (applies to any fork base move in this build; the push is owner-approved at the time).
**Spec this implements:** `../../../0.4.0-beta.3/phase-12-adblock-redirect-arrivals/PHASE_CONTRACT_registry_pull.md` — its rows `P12-B1…B5` are carried here as `P3-A1…A4` and `P3-A11`. ⛔ Its §3 "b1 recommended" is **superseded by Q2 = b3**.

> **What "b3" means — carried from `../SCOPE.md` §9 Q2, not re-designed here.** libcef files the scriptlet payload
> **browser-side**, exactly as it files the farbling key today; in the renderer, libcef **pulls** it **just before** it
> calls our `OnContextCreated`, and hands it to us through the message callback we **already** handle
> (`preload_cosmetic_script` → `OnProcessMessageReceived`). No new public CEF API. Our injection code is meant to stay
> unchanged. Old app + new engine, and new app + old engine, both keep working — so an engine rollback alone restores
> today's behaviour. **Fallback:** b1 (a public pull API) if the pull cannot be placed before our callback or is too slow
> (SCOPE P3 unknowns 2–3) — ⛔ switching to b1 changes the rollback property (app and engine then roll back together), so
> it **amends this contract and goes to the owner before code** (working rule 1).

---

## 1. Goal

When a user clicks from one site to another (X → YouTube), the ad-blocker's scripts are in the new page **before the
page's own first script runs**, not ~30 ms after it.

## 2. Done means

- [ ] A cross-site arrival injects inside `OnContextCreated` (0 ms late); the P12 mitigation's late-arrival line does **not** fire for that navigation (`P3-A1`).
- [ ] An inline `<script>` at the top of `<head>` on a cross-site arrival sees the **mutated** surface (`P3-A4`) — the row that proves the patch buys anything over what already shipped.
- [ ] Same-process, new-tab, redirect and same-URL-reload arrivals each inject pre-JS exactly once (`P3-A2`, `A3`, `A5`, `A6`).
- [ ] If the pull misses, today's mitigation still fires — the fallback is not lost (`P3-A7`).
- [ ] The browser-side registry stays bounded under navigations that never commit (`P3-A8`).
- [ ] No measurable page-load cost beyond the noise floor of the Build 1 engine (`P3-A9`).
- [ ] `cef-native/` is **unchanged** by this phase — or, if `P3-A6` forces an app change, that change was approved by the owner first (§12 Q1).
- [ ] The fork's P3 commits are LF-only (Build 1's CRLF normalisation is not undone) (`P3-A0`).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-GOLD` | the gold pill fires on the paying tab | `payment_success_indicator` is relayed by `simple_render_process_handler.cpp :: OnProcessMessageReceived` — the **same** callback this phase starts calling from inside libcef, during context creation. A re-entrancy or ordering fault there is a silent loss of the user's primary payment safeguard |
| `R-INTEXT` | internal never prompts, external always gates | the `wallet_call` bridge is a renderer→browser process message from the same render-process handler; the origin re-derivation is browser-side and untouched, but it must be **re-observed** on the new engine, not inherited |
| Invariant 8 (root `CLAUDE.md`) | CEF lifecycle / render-process handler timing not changed without asking | this phase **is** that change: our handler receives a message at a new moment. The ask is Q2 (SCOPE §6a says so); the owner approved the agent-level set. Any change beyond "one message, delivered before `OnContextCreated`" is outside that approval |
| Farbling (C2 pull) | the farbling key is pulled in `CefFrameImpl::OnContextCreated` | P3 edits the same four libcef files as C2/P4e (`frame_host_impl.cc`, `frame_impl.cc`, `cef.mojom`, `render_frame_observer.cc`). A merge slip here silently drops farbling ⇒ `FARBLING_RELEASE_GATE.md` re-run in P5-lite |
| Privacy Shield per-site scriptlet opt-out | `AdblockCache :: isScriptletsEnabled` decides whether a payload is fetched at all | the registry must file **nothing** for a host the user turned scriptlets off for — otherwise the opt-out is bypassed by the new path (`P3-A1`'s RED is exactly this) |

`R-NOSPEND` / `R-CLASSIFY` / `R-RESTORE` / `R-TOKENPERM` / `R-BEEFOUT`: not touched — this phase has no wallet code. Stated, not assumed: the diff is `libcef/` in the fork plus nothing in `rust-wallet/`.

## 4. Evidence table

⛔ Every row runs on the **Build 2 engine produced in `../phase-P5lite-build2-verify/`** (P3 cannot be smoke-tested
before a build — there is no other place to run it). ⛔ **Fresh URL per trial** (`s_scriptCache` is URL-keyed and
one-shot; a leftover entry scored a false 2-of-3 GREEN in beta.3 Phase 12). ⚠️ **Instrument:** the `[RENDER]` lines are
in `cef_debug.log` (truncated per launch), **not** `debug_output-<pid>.log`; `hodos::LogSafeUrl` is origin-only, so the
**renderer PID** is the only thing that tells the source and destination processes apart.
⭐ **The Build 1 engine is this phase's natural feature-off build** — same Chromium, same farbling, no P3. Every
"revert the patch" RED below is run by staging Build 1's `cef-binaries` into a second checkout, not by editing code.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P3-A0` | Every P3 fork commit above Build 1's pin tag is LF-only: `git diff --stat` and `git diff --ignore-cr-at-eol --stat` over `<build1-pin>..<p3-head>` are **identical** | On a scratch branch, save one P3-edited file with CRLF ⇒ the two stats **differ** (the check can see a regression of Build 1's normalisation) | the **standalone** fork checkout `hodos/7871` (`C:\cef\cef150\cef`), not the in-tree copy `<chromium>/src/cef` | T0 | ⬜ |
| `P3-A1` *(= P12-B1)* | Cross-process arrival `github.com` → `youtube.com/watch?v=<fresh>`: the **destination** renderer logs `💉 OnContextCreated: injecting scriptlets` for the YouTube URL, and **no** `💉 P12: late-arrival inject` for it | (a) **Build 1 engine**, same navigation ⇒ the late-arrival line fires, the OnContextCreated inject does not. (b) Scriptlets turned off for `youtube.com` (Privacy Shield) ⇒ **no** injection of either kind, **while** the YouTube `OnContextCreated called` line is still present (proves the context was created and the inject was *withheld*, not missed) | a **tab** browser (`role: tab_<n>` in the shell role log); the inject line's renderer PID **≠** the PID that logged the `github.com` context | T2 | ⬜ |
| `P3-A2` *(= P12-B2)* | Same-process arrival (same-site link on `youtube.com`) injects pre-JS **exactly once** | (a) as `P3-A1`(b). (b) The counter can count two: in a **scratch app build** with `s_injectedUrl`'s guard removed from `OnProcessMessageReceived`, the same run logs **two** injections — shows the "exactly one" instrument is not blind | tab browser; one navigation id; both lines from one renderer PID | T2 | ⬜ |
| `P3-A3` *(= P12-B3)* | New-tab arrival via `OnBeforePopup` → `CreateNewTabWithUrl` injects **inside `OnContextCreated`** (today: only the ~30 ms late inject, measured 2026-09-21) | Build 1 engine ⇒ late-arrival line only | the **new** tab's browser (its own `tab_<n>` role), not the opener | T2 | ⬜ |
| `P3-A4` *(= P12-B4)* ⭐ | An **inline `<script>` at the top of `<head>`** on a cross-site arrival reads the **mutated** value a scriptlet for that host sets | **Build 1 engine** ⇒ the same inline script reads the **un-mutated** value. ⛔ If this RED does not go red, P3 has not been shown to buy anything over `d79a869` | the value the page's **own** inline script read (written to `document.title` / console by the fixture), in a tab, fresh URL. ⚠️ Fixture host + scriptlet **named here before the run** (§12 Q3) — the engine skips `127.0.0.1`, so a local page cannot be the subject | T2 | ⬜ |
| `P3-A5` | Redirect arrival (a 30x chain ending cross-site, and `http`→`https`) injects at `OnContextCreated` under the **final** URL | Build 1 engine ⇒ late-arrival only (or none) | tab browser; the logged URL is the post-redirect origin; renderer PID of the committed document | T2 | ⬜ |
| `P3-A6` | **Same-URL reload** (F5, and back/forward to the same URL) injects at `OnContextCreated` | Build 1 engine ⇒ late-arrival only. ⚠️ **Predicted to FAIL on GREEN too — code reading, not measured:** the payload is delivered *before* our `OnContextCreated`, when `s_contextRanUrl[frameId]` and `s_injectedUrl[frameId]` still hold the **previous** document's URL (they are cleared only inside `OnContextCreated`), so `OnProcessMessageReceived` takes the "already injected — dropped" branch. ⇒ If GREEN fails, **stop** (§12 Q1) | tab browser; same `frame->GetIdentifier()` before and after; renderer PID unchanged (same-process) | T2 | ⬜ |
| `P3-A7` | **Fallback survives:** with the pull forced to miss, the P12 mitigation's late-arrival inject still fires (~16–41 ms), not the ~1.2 s load-complete path. **Paired with `P3-A1`** — pull hit ⇒ no late inject; pull miss ⇒ late inject | the two-sided pair is its own control: a build where libcef **consumes** the `OnLoadStart` re-push instead of forwarding it ⇒ with the pull missing, **nothing** injects until load-complete | tab browser; the forced miss is a scratch libcef switch named in the run, not a network failure | T2 | ⬜ |
| `P3-A8` | The browser-side registry is **bounded**: 200 navigations that never commit (download links, `204`, cancelled) plus every `OnLoadStart` re-push leave the registry at or under its cap, and entries expire | scratch libcef build with eviction disabled ⇒ the logged registry size grows **linearly** with the navigations | the **browser-process** registry size, logged by libcef — not renderer memory, not the app's `s_scriptCache` | T2 | ⬜ |
| `P3-A9` | Context-creation → first page script latency and page-load time on the minimal basket stay within the **noise floor measured on Build 1** (threshold written here **before** the Build 2 run) | scratch libcef build with a 50 ms sleep inside the pull ⇒ the instrument reports the regression | same machine, same sites, same profile; Build 1 vs Build 2; tab browsers only | T2 | ⬜ |
| `P3-A10` | `git diff <build1-release-tag>..HEAD -- cef-native/` is **empty** for this phase (b3's "app unchanged" property) — unless §12 Q1 approved a change, in which case the diff is exactly that change | a scratch one-line edit under `cef-native/` ⇒ the check reports it | the app repo on `origin/0.4.0` at the Build 2 app commit | T0 | ⬜ |
| `P3-A11` *(= P12-B5)* | Minimal basket (youtube, x, github) clean on both platforms: ads absent on **first paint** after a cross-site arrival, pages functional | global ad-block off ⇒ the same viewer sees YouTube ads | 👤 owner at the keyboard, release-shaped dev build, tab browsers | T3 | ⬜ |

**Two-sided rows:** `P3-A1` ↔ `P3-A7` (hit injects early and suppresses the late path / miss keeps the late path).
`P3-A1`(b) is also the Privacy Shield opt-out's control — a registry that files a payload for an opted-out host fails it.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `cef-native/src/handlers/simple_handler.cpp :: SimpleHandler::OnBeforeBrowse` | ✅ | the early `CefProcessMessage::Create("preload_cosmetic_script")` push, guarded by `g_adblockServerRunning && IsGlobalEnabled() && frame->IsMain()`; ~40 lines below it, the `hodos_farble_key` send with the comment naming the intercept — the precedent |
| `cef-native/src/handlers/simple_handler.cpp :: SimpleHandler::OnLoadStart` | ✅ | the P12 mitigation's re-push (tab browsers, main frame). Under b3, whether libcef **files and forwards** or **consumes** this second push decides `P3-A7`/`A8` |
| `cef-native/src/handlers/simple_render_process_handler.cpp :: s_scriptCache`, `s_contextRanUrl`, `s_injectedUrl` | ✅ | URL→JS one-shot cache and the per-frame bookkeeping. `s_contextRanUrl`/`s_injectedUrl` are **cleared only in `OnContextCreated`** — the basis of `P3-A6`'s prediction |
| `cef-native/src/handlers/simple_render_process_handler.cpp :: SimpleRenderProcessHandler::OnContextCreated` | ✅ | injects from `s_scriptCache`, skips `127.0.0.1` |
| `cef-native/src/handlers/simple_render_process_handler.cpp :: SimpleRenderProcessHandler::OnProcessMessageReceived` (`preload_cosmetic_script` arm) | ✅ | three cases: late-arrival inject / duplicate dropped / pre-cache. b3 relies on case 3 (pre-cache) |
| `cef-native/src/handlers/simple_render_process_handler.cpp :: OnProcessMessageReceived` (`payment_success_indicator` arm) | ✅ | the gold-pill relay — same callback (`R-GOLD`) |
| fork `libcef/renderer/render_frame_observer.cc :: CefRenderFrameObserver::DidCreateScriptContext` | ✅ at `9ccef044f` (local checkout, read-only) | calls `handler->OnContextCreated` **before** `framePtr->OnContextCreated` — so the farbling pull (inside the latter) runs **after** our callback; the cosmetic pull must be placed **before** `handler->OnContextCreated` (SCOPE P3 unknown 2, confirmed by reading) |
| fork `libcef/renderer/frame_impl.cc :: CefFrameImpl::OnContextCreated`, `::MaybeApplyHodosFarblingKey`, `::SendMessage` | ✅ at `9ccef044f` | C2's pull and its memo; `SendMessage` drops `hodos_farble_key` belt-and-braces and otherwise calls `handler->OnProcessMessageReceived` — the delivery b3 reuses |
| fork `libcef/browser/frame_host_impl.cc :: CefFrameHostImpl::SendProcessMessage`, `::GetHodosFarblingKey` | ✅ at `9ccef044f` | the intercept of `hodos_farble_key` → `hodos::FarblingRegistry::Set`, consumed (not forwarded); the sync answer side |
| fork `libcef/common/mojom/cef.mojom :: GetHodosFarblingKey` | ✅ at `9ccef044f` | the `[Sync]` pull precedent (`string host => (string key_hex, bool enabled)`) |
| fork `libcef/browser/hodos_farbling_registry.{h,cc} :: hodos::FarblingRegistry` | ✅ exists at `9ccef044f` | copy its **shape**, do not generalise it: farbling is host-keyed + `enabled`-gated + 32 bytes; cosmetic is URL-keyed + one-shot + ~34 KB and growing (SCOPE §3.2) |
| `development-docs/DevOps-CICD/scripts/cef_patch_drift_audit.sh` | ✅ | ⚠️ audits `patch/patches/hodos_*.patch` **only** — P3 is direct `libcef/` commits (like C2/P4e) and is **invisible** to it. P3's presence is proven in P5-lite by the pin SHA + `P3-A1`'s log line, not by the audit |

⚠️ **All fork citations are at `9ccef044f` (the current P4f pin — the local checkout is detached there).** Build 1 moves
the branch (merge of upstream `a61e9a5` + the CRLF commit). ⇒ **Re-verify every fork row above against Build 1's pin
tag at this phase's kickoff** (CEF upstream's 4 commits touch none of these files — SCOPE §4, measured 2026-09-25 — so
drift is not expected, but the CRLF commit rewrites all four as whole-file changes).

**Prior art (working rule 5).** Our own `hodos::FarblingRegistry` (same defect, same shape — followed). Brave
`cosmetic_filters_js_render_frame_observer.cc` resolves in `ReadyToCommitNavigation` in the committing frame, keeps a
**fallback key** (empty / `about:blank` URL ⇒ security origin) and **waits** on a missing payload
(`../../../PRIOR_ART.md`, 2026-09-19 row) — we follow the first two (fallback key is required by `P3-A5`/`A6`'s keying);
the wait is **not** ported, because the browser fills the registry synchronously in `OnBeforeBrowse`, before commit, so a
miss means "none filed", not "not yet". Brave keeps its sync load behind a feature flag — hence `P3-A9`.
**Docs to read and cite before the diff (working rule 4):** Mojo `[Sync]` attribute semantics
(`mojo/public/tools/bindings/README.md`) and CEF `CefRenderProcessHandler::OnContextCreated` /
`OnProcessMessageReceived` (`include/cef_render_process_handler.h`) — both already used in this fork/app, but the
re-entrancy of calling the latter from inside the former's caller is new.

## 6. Out of scope

- **The CSS half** (`inject_cosmetic_css`, still post-load only). The beta.3 spec §5 says a registry carrying scriptlets
  *should* carry selectors; SCOPE P3's objective names scriptlets only, and carrying CSS needs an app-side receiver at
  context creation (no `<head>` yet) — which breaks b3's "app unchanged" property. ⇒ §12 Q2; ticket it if the owner wants it.
- b1 (a public CEF API) unless the fallback trigger fires — and then only after the owner sees the amendment.
- Brave's "wait on the payload" guard (reason in §5).
- Subframe scriptlets (today: main frame only; unchanged).
- Phase 13 (bot detection). Whether this closes it is **unknown** — beta.3 spec §7; not assumed.
- Any change to the filter lists, `AdblockCache`, or the adblock engine.

## 7. Rollback

Revert `env.CEF_ASSET` (both `release.yml` arms) to **Build 1's** assets — the app is unchanged, so the engine alone
restores today's behaviour (b3's designed property). The fork commits stay on `hodos/7871` history; nothing is force-pushed.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| The pull is placed after our callback (the farbling precedent's spot) — everything "works" via the late-arrival path and looks like a pass | `P3-A1` (the late-arrival line must be **absent**) and `P3-A4` |
| Same-URL reload / back-forward is dropped as a "duplicate" because bookkeeping from the previous document survives until our `OnContextCreated` | `P3-A6` (predicted to fail — see §12 Q1) |
| libcef consumes the `OnLoadStart` re-push too; when the pull misses (key mismatch, `about:blank`), nothing injects until load-complete — a silent regression vs today | `P3-A7` |
| Every non-committing navigation and every re-push files ~34 KB that is never pulled — the browser process grows all session | `P3-A8` |
| Sync IPC of ~34 KB per document on the critical path slows every page | `P3-A9` |
| Registry keyed on the request URL, context reports the post-redirect / fragment-stripped URL — silent misses on exactly the arrivals Phase 12 was named for | `P3-A5`, `P3-A7` |
| Stale in-tree `src/cef` copy builds green with **none** of P3's libcef edits (the `--force-cef-update` trap; drift audit cannot see libcef commits) | P5-lite `L3` (pin SHA in the artifact) + `P3-A1`'s log line only exists in new code |
| P3's merge conflicts with C2 in `frame_impl.cc` and a hand resolution drops the farbling pull | P5-lite farbling gate (`L6`) |
| Re-entrancy: calling `OnProcessMessageReceived` from `DidCreateScriptContext` breaks the gold-pill relay or a wallet call on that frame | `R-GOLD`, `R-INTEXT` in P5-lite |
| The Privacy Shield scriptlet opt-out is bypassed because the registry files regardless | `P3-A1`(b) |
| The fixture for `P3-A4` has no working scriptlet, so both builds read the same value and the row is void | `P3-A4`'s RED — a void fixture shows GREEN == RED and is reported INCOMPLETE |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | build host; incremental Tier-1 from the Build 1 tree |
| macOS | `P3-A1`–`A7`, `A9`, `A11` | 🍎 via the Mac agent on the staged Build 2 framework (macOS ignores `CEF_ROOT`; staging mandatory). `cef_debug.log` location differs — the Mac agent names it in its relay. `P3-A0`, `A8`, `A10` are platform-neutral (fork/app diffs, browser-process log) — run once |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **≈ 0.4 h** — `P3-A11` visual spot-check both platforms (0.25) + §12 Q1–Q3 (0.15). Scored in the AAR |
| Human-bound rows | `P3-A11` (visual judgement). ⛔ The fork push of P3's commits is outward-facing — batched with P5-lite's push approval, not separate |
| Unknowns (K) — uncertainty, not difficulty | **1 phase, four unknowns inside it:** placement before our callback; sync-IPC cost; same-URL reload bookkeeping (`P3-A6`); how the `OnLoadStart` re-push behaves once libcef intercepts the name (`P3-A7`/`A8`) |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | T0 Build 1 (`../SECURITY_RELEASE_PLAN.md`) | its merged + CRLF-normalised pin is P3's base; its shipped engine is every RED's feature-off build |
| gives | `../phase-P5lite-build2-verify/` | the P3 fork commits; P5-lite builds, pins and publishes them and runs this table |
| batched with | `../phase-P4-hodos-brand-sec-ch-ua/` | one Tier-1 build carries both (`NEXT_CHROMIUM_BUILD.md` "batch it") |
| informs | beta.3 Phase 13 (bot detection, parked) | whether an early inject changes challenge verdicts is **not** measured here |

## 12. Open questions for the owner

1. **If `P3-A6` fails as predicted, may the fix touch our app?** The minimum change is in
   `simple_render_process_handler.cpp` (let a payload delivered during context creation bypass the "already injected"
   check). That breaks b3's "app unchanged" property only slightly — new app + old engine still works (the old engine never
   delivers at that moment) — but it is invariant 8 territory and not what Q2 approved. ⭐ Recommendation: **yes, that one
   change**, measured by `P3-A6`; the alternative (libcef sends a distinct message name) is a larger fork surface.
   ⚠️ Found while reading: the same bookkeeping means **today's** early push on a same-URL reload is also dropped (the
   mitigation's late inject covers it). Code reading only — not measured.
2. **CSS half:** leave post-load (as scoped), or ticket it for the release after? Recommendation: ticket it.
3. **`P3-A4` fixture:** OK to add a scratch filter rule (dev profile only) targeting a test page on a host we control, so an
   inline head script can read a known scriptlet effect? The alternative — a live third-party site with a known scriptlet —
   is not controllable and can change under us.

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
