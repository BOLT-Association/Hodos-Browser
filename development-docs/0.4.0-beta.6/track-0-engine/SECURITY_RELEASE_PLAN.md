# Engine security release — `v0.4.0-beta.5` (Build 1 of B5-T0)

**Written:** 2026-09-27, from G2 decision 1. **Status:** 🟡 **STARTED 2026-09-29** (👤 owner go; macOS tasked in relay round W-29a). Pre-flight checks done; step 1 next.

**Pre-flight, 2026-09-29 (measured):** upstream `refs/heads/7871` still `a61e9a5` = `.255`, nothing newer in the CEF index · fork `hodos/7871` = `9ccef04` (the beta.4 engine) · `git diff fb8be17..HEAD` outside docs = the same 6 comment/string lines ⇒ still refresh-only · Windows tree: `chromium/src` at `150.0.7871.187`, 886 GB free; fork checkout's 1,409 modified files are CR-only (`git diff --ignore-cr-at-eol` empty) ⇒ the Q4 CRLF commit.

**Step 1 ✅ 2026-09-29:** fork `hodos/7871` = `7d50c1cab` (merge `fda76cc37` of upstream `a61e9a5`, zero overlap; CRLF-only commit `7d50c1cab`), pin `pin-7d50c1c/7871` pushed (👤 owner OK), `CEF_CHECKOUT` bumped in both build scripts. Drift audit runs after the `.255` checkout (in progress on Windows). *(Correction to the step-1 row: the 1,409 CR-modified files in the Windows `chromium/src/cef` **copy** were a checkout artifact, not the Q4 set, and were not committed.)*

**Step 2 (Windows) 2026-09-29:** `.255` checkout ✅ (attempts 1–2 died on **Google's LiteRT git-LFS server returning HTTP 503**, reached only via `gclient revert`'s interim `main` state; `.187`/`.255` pin LiteRT `09b4b05`, which has no LFS files. 👤 Owner chose option A: `lfs.fetchexclude=*` repo-local on `third_party/litert/src` only). Pre-build checks: **no LFS placeholder anywhere in the tree** (negative control: the same scan lists 23 placeholders at LiteRT `4dac153`) · drift audit section 3: 120 apply clean + `runhooks.patch` already applied, 0 fail; its section-2 "HARD FAIL" is a false positive on patch-created files (`tickets/TICKET_patch_drift_audit_false_fails_on_new_file_patches.md`) · codec gate **PASS** (run with `HODOS_FARBLING=1`; the repo gate omits it, same ticket) · patcher `120 applied, 1 skipped, 0 failed`; all 7 `hodos_*` reverse-check as applied. Build started via `C:\cef\cef150\run_build_255.ps1` (verdict = the bat's own BUILD SUCCEEDED/FAILED line; its exit code is always 0).

**Step 2 (Windows) build ✅ 2026-09-29:** run 1 (15:03) failed at ~20 min on three Node/TS steps (devtools-frontend `recorder/components` ts_library; `gpu` and `webxr_internals` `lint_ts`); the host was memory-starved and the real error was hidden by eslint `--quiet`, so the cause is **unproven**. Run 2 (15:52 → 20:24, 4 h 32 m), unchanged tree: **BUILD SUCCEEDED**, 0 failed steps (run-1 logs kept as `C:\cef\cef150\build_255_run1*.log`). All 7 `hodos_*` patches listed (`121 patches total`, all skipped = already applied). Artifact `cef_binary_150.0.48-7871.3582+g7d50c1c+chromium-150.0.7871.255_windows64`; its `cef_version.h` reads `CEF_COMMIT_HASH 7d50c1cab…`, `CHROME_VERSION_PATCH 255`.

**Codec gate (Windows) ✅ 2026-09-30**, against the new engine's own `cefclient.exe` (engine-only; the Hodos shell is not rebuilt yet), reusing `codec_check.py`'s rows, controls, assets and page JS unchanged. Build flags: `proprietary_codecs=true`, `ffmpeg_branding="Chrome"`, `USE_PROPRIETARY_CODECS() (1)`, ffmpeg `config/Chrome/win/x64`. Layer A: H.264 ×2 / AAC / MP3 / VP9 `probably`, AV1 `probably`, HEVC `probably`, Dolby Vision `""`; **controls** AC-3 / E-AC-3 / bogus all `""`. Layer B: MP3 +3,031 B, AAC +3,005 B, H.264 +394 B decoded; **AC-3 negative control did not decode**; youtube.com 854×480 video +148,302 B, audio +55,767 B over 3 s. ⚠️ Still owed at P5 on the rebuilt **Hodos** shell: row `L7` (a tab browser, not cefclient).

**Step 3 ✅ 2026-09-30:** VER-5 dist drift audit on the `.255` Windows dist: file set **identical** to the beta.4 dist; the only flags are `bootstrap.exe`/`bootstrapc.exe`, which the beta.4 dist trips identically (CEF 150's launcher, copied in as `HodosBrowser.exe`). Windows asset **`cef-binaries-windows-150.0.48-g7d50c1c.zip`** built in `C:\cef\cef150\asset_7d50c1c\` (dist copied, wrapper rebuilt there: `stdcpp20`, `/MT`, `USE_SANDBOX=ON`, same as beta.4), Python `zipfile`: one top-level `cef-binaries/`, 0 backslashes, **1,689** files, `testzip` clean; 239,479,314 B, md5 `c6cfdd67c8726ac8cab1506906588f21`, sha256 `6696f6f803d21f887ac3b1f26f4e6e2157c87657b7c24b001ef109900bf01228`. `release.yml`'s engine assertion run locally: PASS with the new name, **MISMATCH with `…150.0.43-g9ccef04`** (negative control). 👤 Owner uploaded it; downloaded back: size + md5 identical, `CEF_VERSION` = `150.0.48-7871.3582+g7d50c1c+chromium-150.0.7871.255`. macOS asset uploaded by the Mac side (round M-30b). `CEF_ASSET` bumped in **both** `release.yml` arms in one commit.

**Step 4 ✅ 2026-09-30:** `0a6ffae` fast-forwarded to `origin/staging`, `origin/main` and `release/main` (the 98 commits since `fb8be17` change 6 non-doc lines, all comments or a log string). Validation run `workflow_dispatch` **36716554053** (👤 owner dispatched): `preflight-signing-key` / `build-macos` / `build-windows` **success**, `publish` **skipped**; `gh release list` unchanged (newest `v0.4.0-beta.4`). Engine proof from the run log: Windows downloaded `cef-binaries-windows-150.0.48-g7d50c1c.zip` 239,479,314 B md5 `c6cfdd67…` and macOS `cef-binaries-macos-150.0.48-g7d50c1c.tar.bz2` 127,555,602 B md5 `80c478fc…`, both equal to the uploaded bytes; both arms' assertion read `CEF_VERSION = 150.0.48-7871.3582+g7d50c1c+chromium-150.0.7871.255`.

**Step 5 (Windows, agent rows) ✅ 2026-09-30**, on the dev build rebuilt against the new asset (repo `cef-binaries/` swapped in from `asset_7d50c1c`; the beta.4 dist kept whole at `C:\cef\cef150\cef_binaries_prev_9ccef04`; the deployed `libcef.dll` md5 `2137cba8…` = the new dist's):
- **The fix is present:** `chrome://version` in a **tab** reads `V8 15.0.245.40`, `CEF 150.0.48-7871.3582+g7d50c1c+chromium-150.0.7871.255`. Negative-control value (beta.4 = `15.0.245.21`) cited from SCOPE §2.1, not re-measured, because that would mean driving the owner's installed production browser.
- **Farbling:** `farbling_seed_rotation_check.py --expect-cef +g7d50c1c` all contracts PASS; token `FARBLING-ROTATION-v1 engine=150.0.48-7871.3582+g7d50c1c+chromium-150.0.7871.255 exempt=53225ec8/53225ec8/53225ec8 large=0cdc9b48/0cdc9b48/0cdc9b48 farbled=0e4e6251/5565a0c8/0e4e6251 verdict=PASS`. **`--negative-control` went RED** as required (canvas, WebGL, audio and navigator all collapse to the exempt/native values).
- **Codecs (`L7`)** in a tab (`role=tab_1`): Layer A GATE rows `probably`, AC-3 / E-AC-3 / bogus controls `""`; Layer B MP3 / AAC / H.264 decode, **AC-3 control did not decode**; youtube.com 1280×720 and twitch.tv live decoding; x.com blocked (logged out, no media element: site access, not decode).
- ✅ **Owner rows, 2026-09-30:** minimal basket (youtube, x, github), DPI cells #4/#6/#9 and `R-GOLD` real payment run by 👤 the owner on the same dev build. Owner's words: *"I did all the tests on my end. Everything looks pretty good."* No defects reported; per-row detail not recorded.
**Evidence:** `SCOPE.md` §2.3 (the V8 fix), §4 (patch re-application), §5 P1/P2/P5 (phases), §7 (hours).

## What and why — one paragraph

The public `v0.4.0-beta.4` runs engine `150.0.7871.187`, whose V8 (`49df3678`, 2026-07-17) lacks the fix
for **CVE-2026-85046**, a V8 type confusion **exploited in the wild** (CVSS 8.8). The fix is on our own
long-term CEF branch as `085f765` (merged 2026-09-01) and ships in `150.0.7871.255` (CEF `150.0.21`,
2026-09-23). This release moves the engine to `.255` and **changes nothing else**, so it can ship in days,
not weeks, outside the planned release's cycle — the way `v0.4.0-beta.4` did.

## Scope — exactly this, nothing more

| In | Out (→ Build 2, inside the planned `v0.4.0-beta.6`) |
|---|---|
| Fork `hodos/7871` brought onto upstream `a61e9a5` (= `.255`) by **merge** (SCOPE Q3) | Ad-block payload pull (P3) |
| The fork's CRLF line-ending normalisation, its own commit, `git diff --ignore-cr-at-eol` empty (Q4 — source only, no binary change) | `"Hodos"` in `Sec-CH-UA` (P4) |
| New `cef-binaries-{windows,macos}-<cefver>-g<sha>` assets; `env.CEF_ASSET` bumped in **both** `release.yml` arms in one commit; `CEF_CHECKOUT` bumped in both build scripts | Any app change |

📏 **App code since `v0.4.0-beta.4` (`fb8be17`) is comment/log-string only** — `git diff fb8be17..HEAD` outside
docs touches `handlers.rs`, `reconcile.rs`, `utxo_fetcher.rs` (doc-path comments) and one `promote.yml` echo
string (2026-09-27). ⇒ Tagging from `0.4.0` is a refresh-only release. ⚠️ **Re-run that diff on the day of
tagging**; if any behavioural change has landed, cut the tag from `fb8be17` + the `CEF_ASSET` commit instead.

**Version:** `v0.4.0-beta.5` → build number **40005** > installed 40004 (Sparkle + `UpdateStager::IsNewerBuild`
both compare that integer). ⛔ Not `beta.4.1` — the parser scores it 99 = final (`40099`), a silent dead end.

## Steps and stops

| # | Step | Who / stop |
|---|---|---|
| 1 | **P1** — merge upstream into `hodos/7871`; CRLF commit; new pin tag `pin-<sha7>/7871`; `cef_patch_drift_audit.sh` against a `.255` tree ⇒ exit `0` | agent · ⛔ **fork push + pin tag = outward-facing → owner OK** |
| 2 | **P2** — Tier-1 build, Windows (incremental, ~5 h host time) and macOS (⚠️ **fresh no-history `src` fetch first**; tens of GB). *Corrected 2026-09-29 per round M-29a:* the Mac `src/.git` is not deleted but a **shallow 1-commit repo at `.187`**, and a bare fetch against it wedged in 0.4.0, so the fresh fetch still applies, in a **new** tree `cef150_255` (started 09:12) | agent + 🍎 Mac agent via relay round |
| 3 | Upload the two assets to the org repo's `cef-binaries` release; bump `CEF_ASSET` (both arms) + `CEF_CHECKOUT` | ⛔ **asset upload → owner OK** |
| 4 | `workflow_dispatch` validation build green on **both** platforms before any tag | agent |
| 5 | **P5** — verify (below) | agent + 👤 owner rows |
| 6 | Tag `v0.4.0-beta.5`, build, then **promote** | ⛔ 👤 **owner — irreversible** (G9/G10 equivalent) |
| 7 | Verify live: feed serves 40005, download redirects, **beta.4 → beta.5 self-update** on both platforms | agent + 👤 |

## Verification (P5) — every row with its negative control

| Row | Pass | Negative control |
|---|---|---|
| Patches applied | build log's `N patches total` lists **every** `hodos_*` patch | a stale in-tree `src/cef` (the `--force-cef-update` trap) scores green with zero Hodos patches — check the list, not the count |
| Engine identity | `CEF_VERSION` read out of the **downloaded artifact** = the new build | point one arm's `CEF_ASSET` at the old name ⇒ the binding step fails |
| The fix is present | V8 version on `chrome://version` in a **tab** = `15.0.245.40` (`4ceb8016`, contains `085f765` — SCOPE §2.1) | the shipped beta.4 engine reports `15.0.245.21` (`49df3678`) |
| Farbling | rotation token with `engine=` = new `CEF_VERSION` (`FARBLING_RELEASE_GATE.md`) | `--negative-control` goes red |
| Codecs | `canPlayType` avc1/mp4a ⇒ `probably`; real YouTube playback | stock prebuilt CEF ⇒ `""` |
| macOS floor | `vtool` minos of the new framework; floor stays `max(12.0, measured)` | CI minos guard |
| Shell | minimal basket (youtube, x, github) + DPI cells #4/#6/#9; `R-GOLD` real payment shows the gold pill | — (each row per `REGRESSION_SET.md`) |
| Update path | installed beta.4 → offered and applies beta.5 silently, both platforms | — |

**Owner-hours:** ≈ 4.5 (SCOPE §7). **Rollback:** revert `CEF_ASSET` to `…150.0.43-g9ccef04` (assets are never
clobbered) — re-opens the exploited bug, so it is a last resort.

## After it ships

Record the promote in `../../DevOps-CICD/CEF_VERSION_UPDATE_TRACKER.md` (close the "OWED AT THE NEXT BUILD" block)
and `BUILD_AND_RELEASE.md`; relay round to macOS; the planned release continues as `v0.4.0-beta.6` with Build 2.
