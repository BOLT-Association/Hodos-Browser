# B5-T0-P5-lite — Build, publish and verify the second engine build (ad-block pull + "Hodos" brand) · PHASE CONTRACT

**Track:** B5-T0 Engine — **Build 2** · **Tickets:** `../../tickets/TICKET_engine_behind_its_own_cef_branch_and_upstream_stable.md` (its phases 4–6, for Build 2) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** **1** (Build 2 = P3 + P4 → P5-lite, inside `v0.4.0-beta.7`; not a separate release) · **T0 Q3 → merge** (any fork base move is a merge, never a rebase/force-push; the push is owner-approved at the time) · **T0 Q5 → record the M160 window** (already applied to D2 and the intake ticket at G2 item 8 — carried here only as the reason this build stays on 7871).
**Modelled on:** `../SECURITY_RELEASE_PLAN.md` §"Steps and stops" + §"Verification (P5)" — Build 1's procedure, **"lite"** because Build 2
changes two things on a base Build 1 already proved, so the release-level rows (update path, promote) belong to the
beta.6 release gates (G9/G10), not to this phase.

> **Why this phase also owns the build.** Decision 1 gave Build 2 three phases (P3, P4, P5-lite) and no separate
> "build and publish" phase (Build 1 has P2). Rather than invent a fourth phase, the build, pin and publish steps sit
> here as `L0`–`L4`, ahead of the verification rows. P3's and P4's own evidence tables run on the engine `L1` produces.

---

## 1. Goal

The engine that ships in `v0.4.0-beta.7` carries the ad-block pull and the `"Hodos"` brand **and** everything Build 1
proved — the V8 fix, farbling, codecs, the macOS floor and every shell safeguard — on both platforms.

## 2. Done means

- [ ] One Tier-1 build per platform from **one** fork pin tag carrying Build 1's pin + P3's commits + P4's patch (`L0`, `L1`).
- [ ] Versioned `cef-binaries-{windows,macos}-<cefver>-g<sha>` assets published; `env.CEF_ASSET` bumped in **both** `release.yml` arms in one commit; `CEF_CHECKOUT` bumped in both build scripts; a `workflow_dispatch` validation build green on both platforms (`L3`, `L4`).
- [ ] P3's and P4's evidence tables completed on this build, both platforms (`L9`).
- [ ] Every Build 1 verification row re-passed on Build 2 — none is inherited (`L5`–`L8`, `L10`–`L13`).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| Farbling (`FARBLING_RELEASE_GATE.md`) | farbling rotates per seed, deterministic per seed, on the **shipped** engine | P3 edits the libcef files that carry C2's pull; a stale in-tree copy drops every Hodos patch and builds green |
| The V8 fix (CVE-2026-85046, `085f765`) | Build 1's reason to exist | a fork base mistake (building from the old `9ccef044f` pin, or a stale asset) silently re-opens it |
| Codecs (`NEXT_CHROMIUM_BUILD.md` PART 1 row 1) | video sites work | `GN_DEFINES` drift between builds |
| macOS floor (`CEF_VERSION_UPDATE_TRACKER.md` §"macOS Minimum Deployment Version") | a published minimum that is neither too high (dead auto-update) nor too low (launch crash) | a new framework binary can carry a different `minos` even on the same Chromium |
| `R-GOLD` | gold pill on the paying tab | P3 calls the render-process handler from a new point in context creation — the same handler that relays `payment_success_indicator` |
| `R-INTEXT`, `R-PERIM` | the wallet boundary and the four perimeter gates | run on the new engine; ⛔ never inherited from Build 1 (`../SCOPE.md` §6a) |
| `R-CLOSE`, DPI cells #4/#6/#9 | overlay close guards; layout at 125/150 %, mixed DPI | any engine change re-litigates the shell's runtime (the ticket's "what an engine change actually invalidates") |
| `R-UPDATE` | N−1 → N applies | ⏳ **not this phase** — the Build 2 engine reaches users only inside `v0.4.0-beta.7`, whose G10 runs the real beta.5 → beta.6 update |

## 4. Evidence table

⛔ **Subject rule for every row:** the engine is identified by **`CEF_VERSION` read out of the artifact** (header and the
loaded `libcef` — md5 on Windows, LC_UUID on macOS, as `farbling_seed_rotation_check.py :: require_engine` does), **never
by the Chromium version** — P4e and P4f both reported `150.0.7871.187`, and Build 1 and Build 2 will both report `.255`.
⭐ **Build 1's engine is the standing control:** it differs from Build 2 only by P3 + P4.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `L0` | **Fork base.** `hodos/7871` = Build 1's pin + P3 commits + P4 patch, **merged** (no rewrite of published SHAs). Upstream `7871` head checked with `git ls-remote`: if it moved past `a61e9a5`, §12 Q1 was answered first and any merge is recorded here. New pin **tag** `refs/tags/pin-<sha7>/7871` | a `git merge-base --is-ancestor <build1-pin> <build2-pin>` check fed a pin that does **not** contain Build 1 (e.g. `9ccef044f`) ⇒ fails | the standalone fork checkout and the remote (`ls-remote`), not the in-tree copy | T4 | ⬜ |
| `L0a` | `cef_patch_drift_audit.sh` against the Build 2 source tree ⇒ **exit 0**; `hodos_* patch.cfg entries` ≥ 8 (7 farbling + brand) | edit one line inside a Hodos patch's target file in a scratch tree ⇒ **exit 1** | the audit's own output on the tree the build uses (`/c/cef/cef150/chromium/src`), with `STANDALONE_CEF` pointing at the pinned checkout | T0 | ⬜ |
| `L1` | Tier-1 build, both platforms, with `--force-cef-update`; the build log's patch list names **every** `hodos_*` patch, the brand patch included | the same list-check run against **Build 1's** log ⇒ reports the brand patch missing. ⛔ Check the **list**, not the count — a stale in-tree copy scores a green run with zero Hodos patches | the build log of the build whose `binary_distrib` is staged (matched by `CEF_VERSION`) | T4 | ⬜ |
| `L2` | **P3's libcef commits are in the binary** (the drift audit cannot see them — they are not `.patch` files): the artifact's `CEF_VERSION` carries the Build 2 `g<sha>`, `L0`'s ancestry check holds for P3's head commit, and `P3-A1`'s OnContextCreated-inject line appears | Build 1's artifact ⇒ wrong `g<sha>`, and `P3-A1` shows only the late-arrival line | artifact header + `P3-A1`'s renderer log on a tab | T4 | ⬜ |
| `L3` | Assets `cef-binaries-windows-<cefver>-g<sha>.zip` and `cef-binaries-macos-<cefver>-g<sha>.tar.bz2` uploaded under **new** names (never `--clobber`); `env.CEF_ASSET` bumped in **both** `release.yml` arms in **one** commit; `CEF_CHECKOUT` bumped in both build scripts | point the macOS arm's `CEF_ASSET` at **Build 1's** asset name on a scratch branch ⇒ the engine-binding step fails (it must not build green) | `release.yml`'s own binding step output in the validation run | T4 | ⬜ |
| `L4` | `workflow_dispatch` validation build green on **both** platforms before anything is tagged on the new engine | — the binding failure in `L3`'s RED is this workflow's control; plus the macOS minos guard's control in `L8` | the CI run's downloaded artifact (`CEF_VERSION`), not the build host's local copy | T4 | ⬜ |
| `L5` | **The V8 fix is still present:** `chrome://version` in a **tab** reports V8 `15.0.245.40` (or later on the same branch) | the shipped `v0.4.0-beta.4` engine reports `15.0.245.21` | a tab browser (`role: tab_<n>`), not an overlay | T2 | ⬜ |
| `L6` | **Farbling rotation token** `verdict=PASS` with `engine=` = the Build 2 `CEF_VERSION`, measured **after** `CEF_ASSET` is bumped (`FARBLING_RELEASE_GATE.md` §6: bump first, then measure) — both platforms | the harness's `--negative-control` exits **red** (inverted exit code) on the same build | `farbling_seed_rotation_check.py --expect-cef +g<sha>` (refuses before launch if the loaded libcef is not the staged one); `--log` points at the logs **directory** so the `role: tab_` cross-check runs | T4 | ⬜ |
| `L7` | Codecs: `canPlayType('video/mp4; codecs="avc1.42E01E, mp4a.40.2"')` ⇒ `"probably"`; a real YouTube video plays | stock prebuilt CEF (no codecs) ⇒ `""` | a tab browser on the staged Build 2 binary | T2 | ⬜ |
| `L8` | 🍎 `vtool -show-build` `minos` of the **new** framework measured; published floor = `max(12.0, measured)` in all three places (`CMakeLists.txt`, `Info.plist`, `helper-Info.plist.in`); the CI minos guard passes | the guard's comparison fed a framework `minos` above the published floor ⇒ fails (run as a script-level check, not a release) | the framework inside the **staged** Build 2 distribution, not Build 1's | T4 | ⬜ |
| `L9` | `../phase-P3-adblock-payload-pull/` and `../phase-P4-hodos-brand-sec-ch-ua/` evidence tables complete on this build, both platforms | carried by those tables (each row has its own RED, Build 1 as feature-off) | those tables' SUBJECT columns | T2/T3 | ⬜ |
| `L10` | Minimal basket (youtube, x, github) + DPI cells **#4/#6/#9** on Windows; basket on macOS | per `DPI_RESOLUTION_TEST_MATRIX.md`'s own failure signatures (clipped toolbar at 150 %/1366) — a cell is RED-capable only if its failure signature is named in the run record | 👤 owner at the keyboard, staged Build 2 dev binary, tab browsers | T3 | ⬜ |
| `L11` | `R-GOLD` on the new engine: an auto-approved payment shows the **gold pill** on the originating tab — **both** the createAction silent-approve path and the BRC-121 paid retry (`firePaymentSuccessIpc()`) | stub `OnWalletCallSuccess`'s emit ⇒ no pill (per `REGRESSION_SET.md` `R-GOLD`) | the correct **tab** — `Tab::id` ≠ `CefBrowser::GetIdentifier()`, translated via `TabManager::GetTabIdForBrowserIdentifier`. Real payment, dev wallet | T2/T3 | ⬜ |
| `L12` | `R-INTEXT` (both halves) and `R-PERIM` (T2 end-to-end) on the new engine | per `REGRESSION_SET.md`: force internal-as-external / external-as-internal; flip each gate's precondition | the **Rust** log (`X-Requesting-Domain` absent/present with the exact host) and the `PermissionDecision` kind — not the modal | T2 | ⬜ |
| `L13` | Diff against `BASELINE_CEF150.md`: `regression_soak.py` crash counts (with `--log`, both detectors) and `farbling_perf_check.py` / `farbling_iframe_perf_check.py` within the saved baselines | each instrument's own documented control (`BASELINE_CEF150.md` §"Crash counting has two detectors"; the saved null-control baselines) | same machine class as the saved baselines (`win-archbold`, `mac`) | T2 | ⬜ |

**Two-sided:** `L1` ↔ `L2` (`.patch` presence via the build log / libcef-commit presence via the artifact — neither
instrument can see the other's half).

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `.github/workflows/release.yml` — `env.CEF_ASSET` (Windows arm, macOS arm) | ✅ both `…150.0.43-g9ccef04` today | the binding step derives the expected engine from the asset **filename** (`^cef-binaries-windows-(?<ver>…)-(?<sha>g[0-9a-f]+)\.zip$`). Build 1 bumps it first; Build 2 bumps it again |
| `.github/workflows/release.yml` — macOS `minos` guard (step `id: minos`) | ✅ | "every shipped Mach-O's minos ≥ the CEF framework's minos" |
| `.github/workflows/promote.yml` — farbling token check | ✅ (per `FARBLING_RELEASE_GATE.md` §5; not re-read line by line) | refuses a token whose `engine=` lacks the `+g<sha>+` of the tag's `CEF_ASSET` — so Build 1's token cannot promote Build 2 |
| `development-docs/DevOps-CICD/scripts/build_hodos_cef.bat :: CEF_CHECKOUT`, `build_hodos_cef_mac.sh :: CEF_CHECKOUT` | ✅ both `9ccef044f` today | Build 1 bumps; Build 2 bumps again. ⛔ A moving branch tip is not a reproducible build — pin a **tag** |
| `development-docs/DevOps-CICD/scripts/cef_patch_drift_audit.sh :: HODOS_MIN_PATCHES` | ✅ default `1` | raising the floor is optional; ⛔ if raised, **its own commit** (working rule 6) |
| `development-docs/0.4.0/chromium-rebuild/farbling_seed_rotation_check.py :: require_engine`, `--expect-cef`, `--negative-control` | ✅ | the gate harness |
| `development-docs/0.4.0/chromium-rebuild/regression_soak.py`, `farbling_perf_check.py`, `farbling_iframe_perf_check.py` + `p4e_*`/`p4f_*` baselines | ✅ exist | `L13` |
| `cef-binaries/include/cef_version.h :: CEF_VERSION` | ✅ `150.0.43-7871.3576+g9ccef04+chromium-150.0.7871.187` | the staged header; Build 1 and Build 2 each change it |
| `cef-native/CLAUDE.md` — staging rules | ✅ | ⛔ never merge-copy one distribution over another (an old wrapper survives and wins); macOS ignores `CEF_ROOT`, staging mandatory |
| `cef-native/src/handlers/simple_render_process_handler.cpp :: OnProcessMessageReceived` (`payment_success_indicator` arm) | ✅ | `L11`'s subject path |
| `development-docs/DevOps-CICD/CEF_BUILD_RUNBOOK.md` | ⚠️ stale **throughout** (still describes branch 7103) | rewritten inside Build 1's P1 (`../../README.md` G2 item 8). ⛔ If Build 1 has not rewritten it by the time Build 2 starts, this phase follows `../SECURITY_RELEASE_PLAN.md` + the build scripts, not the runbook |

⚠️ This phase **follows Build 1's actual procedure**, which does not exist yet. At kickoff, re-read Build 1's closed
record (the tracker's "OWED AT THE NEXT BUILD" block closed per `../SECURITY_RELEASE_PLAN.md` §"After it ships") and amend
`L0`–`L4` to whatever Build 1 learned.

## 6. Out of scope

- Tagging or promoting anything — Build 2 ships inside `v0.4.0-beta.7` (G9/G10, owner).
- `R-UPDATE` / N−1 → N self-update — the release's G10 row.
- Moving to another CEF branch (M160 is the next long-term branch, LTC 2027-01-13; `0.4.0-beta.8` intake ticket).
- New engine patches beyond P3 + P4. A fix found now that needs the engine goes into `NEXT_CHROMIUM_BUILD.md` PART 2 the day it is found — it does **not** ride this build unless the owner says so.
- Rewriting `CEF_BUILD_RUNBOOK.md` (Build 1 owns it).

## 7. Rollback

Revert the one `CEF_ASSET` + `CEF_CHECKOUT` commit to Build 1's values. Assets are never clobbered, so Build 1's are still
downloadable; the Build 1 engine keeps the V8 fix, so this rollback costs P3 + P4 only, not security.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| Built from the old `9ccef044f` pin (the scripts' current value) — the V8 fix is silently gone | `L0` ancestry, `L5` |
| Stale in-tree `src/cef` copy: green build, zero Hodos patches | `L1` (list, not count), `L6` (farbling absent ⇒ gate red) |
| Stale in-tree copy with the right `.patch` files but **old libcef** (P3's commits are libcef, not patches) | `L2` |
| The macOS arm still names Build 1's asset — builds green, ships no ad-block pull, no brand | `L3` binding step; `L9` on macOS |
| Farbling token measured before `CEF_ASSET` was bumped — promote refuses, or worse, someone re-uses Build 1's token | `L6` ordering + `promote.yml`'s `+g<sha>+` check |
| New framework carries a higher `minos` — update installs on older macOS, then fails to launch | `L8` |
| P3's new call into the render-process handler breaks the gold-pill relay | `L11` |
| Upstream 7871 moved during Build 2 and nobody checked — Build 2 ships behind its own branch again, the exact defect of this track's ticket | `L0` (`ls-remote` check) + §12 Q1 |
| A "lite" verify skips a row because "Build 1 already proved it" | §4 header: every Build 1 row is re-run; none inherited |
| Verification ran on the build host's local binary, not what CI built | `L4` subject (CI artifact). ⚠️ The farbling gate's known weakness #2 (it measures the host build, not the promoted installer) still stands and is stated, not solved |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | `L0`–`L7`, `L9`–`L13` | build host: incremental from the Build 1 tree |
| macOS | `L1`, `L3`, `L4`, `L5`–`L9`, `L10` (basket), `L11`–`L13` | 🍎 via the Mac agent. Build 1 will already have paid the fresh no-history `src` fetch; Build 2 is incremental on top. Farbling gate on Mac: export `HODOS_MAC_DEV_FLAGS=1`, profile `Default` only (CDP 9322 under dev). ⭐ Needs a relay round in `../../MAC_RELAY_BETA5.md` naming this contract (written by the orchestrator, not this agent) |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **≈ 2.25 h** — approvals (fork push + pin tag, asset upload) 0.25 · Windows basket + DPI #4/#6/#9 + codec check 0.75 · macOS basket + codecs + relaunch, owner at the keyboard 0.5 · `R-GOLD` real payment 0.5 · reading the result 0.25. ⚠️ SCOPE §7 put the second build's round at ≈ 2 h **without** `R-GOLD`; it is added because P3 touches the render-process path the pill rides |
| Human-bound rows | `L0` (fork push + pin tag — ⛔ outward-facing, owner OK at the time; any upstream merge is the Q3 push) · `L3` (asset upload to the org repo — ⛔ owner OK) · `L10` (visual, DPI) · `L11` (real payment, gold pill) |
| Unknowns (K) — uncertainty, not difficulty | **0** — hard-but-understood (Build 1 will have exercised every step). Whether upstream has moved is a question to ask, not an uncertainty in the work |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | T0 Build 1 (`../SECURITY_RELEASE_PLAN.md` steps 1–5) | its pin, its assets as the control, its procedure and lessons |
| needs | `../phase-P3-adblock-payload-pull/`, `../phase-P4-hodos-brand-sec-ch-ua/` | the fork changes this phase builds |
| needs | 🍎 Mac agent | build + stage + verify on macOS |
| gives | the beta.6 release boundary (G9) | ⭐ the engine the **release-candidate regression run** must be measured on. Phase-level browser evidence gathered by other tracks on **Build 1** stays valid for their phases; the RC boundary re-runs `REGRESSION_SET.md` + `REGRESSION_ADDITIONS.md` on **Build 2** |
| gives | T6 human sittings (DPI, overlays), T1/T4 real-money sittings | ⭐ scheduling: land Build 2 **before** the release's final human sittings so `R-GOLD` / DPI are sat once on the shipping engine, not twice |

## 12. Open questions for the owner

1. **If upstream `7871` has moved past `.255` by the time Build 2 starts, take it?** ⭐ Recommendation: **yes, by merge**
   (Q3), same checks as Build 1 (`cef_patch_drift_audit.sh`, 4-commit-style compare), because shipping behind our own
   branch is the defect this track's ticket was opened for. The fork push waits for your OK at the time.

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
- [ ] `../../../DevOps-CICD/CEF_VERSION_UPDATE_TRACKER.md` and `NEXT_CHROMIUM_BUILD.md` updated: P3 + P4 rows moved out of PART 2 (invariant 12)
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
