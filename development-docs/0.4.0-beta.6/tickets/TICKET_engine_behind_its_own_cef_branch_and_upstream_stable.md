# 🧱 The engine is behind its own CEF branch and two milestones behind Chromium stable — a full engine rebuild is owed

**Found:** 2026-09-24, 👤 owner at the beta.5 consolidation: *"we should do a new full Chromium build
at the start of beta.5… that needs to be a whole track… that will be what we need to do first
because we'll build everything on top of that."* Version facts gathered the same day (below).
**Status:** ⬜ UNASSIGNED · **Track:** ⭐ proposed as a **beta.5 track — likely Track 0**, decided at
gate `G2` · **Filed by:** Claude, at the owner's request

> ⚠️ **Method note.** Our pin is **read** from `cef-binaries/include/cef_version.h` and
> `.github/workflows/release.yml`. Every upstream version, date and support window below was
> **read on 2026-09-24 by a research agent** from chromiumdash, the Chrome Releases blog, the CEF
> build index (`cef-builds.spotifycdn.com/index.json`), the CEF GitHub branch list and CEF's
> branches page (`chromiumembedded.github.io/cef/branches_and_building`). ⛔ **I did not
> independently re-fetch them** — re-read them at the track's first phase, because CEF support
> windows are policy and move. Nothing here was built or measured.

---

## What happens

| | |
|---|---|
| **We ship** | `CEF_VERSION 150.0.43-7871.3576+g9ccef04+chromium-150.0.7871.187` — CEF branch **7871**, our fork `P4f` |
| **Upstream on our own branch** | CEF 7871 has builds up to **`chromium-150.0.7871.255`** — ⚠️ **we are behind on security fixes inside the branch we already chose** |
| **Chromium stable** | ✅ **154** (`154.0.8037.57/.58`), stable **2026-09-22**. 👤 The owner's belief was correct. 155 is in *early* stable (full stable scheduled 2026-10-06) |
| **CEF for 154** | branch **8037**, CEF's current **Stable** branch; `154.0.26+ge72305f+chromium-154.0.8037.58`. CEF is **not** lagging here |

## ⭐ The target is a real decision, not a formality — CEF's support windows

CEF now tracks **even** milestones only (153 and 155 will never get CEF branches), with three kinds of
branch. As read 2026-09-24:

| CEF branch | Milestone | Role | Last planned refresh |
|---|---|---|---|
| **7871** *(ours)* | 150 | **LTC — long-term** | **~Apr 2027** |
| 7977 | 152 | Extended (~8 weeks) | ~Oct 20, 2026 |
| **8037** | **154** | Stable (~2 weeks) | **~Oct 5, 2026** |
| next LTC | 160 | — | — |

⚠️ **The consequence nobody has said out loud yet:** a build on **154** today lands on a branch that
CEF **stops patching in about two weeks**. After that, a 154-based Hodos gets **no** upstream security
fixes until we bump again (to 156). Our current branch, **7871, is supported until ~Apr 2027** — so
"newer" and "better patched" point in **opposite** directions this month.

| Option | What we get | What it costs |
|---|---|---|
| **A — refresh in-branch** (7871 → `.255`) | Every security fix to date; support into 2027; fork patches re-apply to the **same** branch (smallest rebase) | No new web platform features; Chromium version stays 150 |
| **B — bump to 154** (8037) | Current stable; newest web platform | Full patch re-application across two milestones; **~2 weeks of upstream fixes, then none** until the next bump; Windows SDK moves `10.0.26100.7705` → `10.0.28000.2270`; macOS floor must be re-measured |
| **C — wait for 156** and bump then | Same as B, later | The in-branch security gap stays open until then |

⭐ **What decides it is product intent — owed by 👤 the owner (working rule 1):** *what is the bump
FOR?* Security currency points at **A** (now) with a planned major bump later; a specific web-platform
need or "stay near stable" as a policy points at **B/C**, and makes bumping a **recurring** cost every
~4 weeks. ⛔ This ticket does not choose.

## Why it sequences early — and which argument that is

⭐ **Uncertainty, not dependency.** `RELEASE_CYCLE.md` §3.7 front-loads uncertain work; an engine bump
re-litigates every fork patch and is the highest-uncertainty item in the cycle.

⚠️ **Correction to the serialization argument as first stated.** *"Every other track compiles against
the engine"* is **not** true of this release's tracks: `rust-wallet/` and `frontend/` do not compile
against CEF at all — only `cef-native/` does. What an engine change actually invalidates is **runtime
verification of the shell**: the overlays, farbling, the adblock push/pull, the wallet bridge and the
payment surfaces (`R-GOLD` and the rest of `REGRESSION_SET.md`). ⇒ wallet-layer tracks can be
**built** in parallel; what must follow the engine is their **browser-level evidence**. Landing the
engine early means that evidence is gathered once, on the engine we ship.

## Scope — expect several phases

| Candidate phase | Content | Home document |
|---|---|---|
| 1 — Target selection | Option A/B/C above, decided by the owner; the pin must be a real CEF branch. ⭐ Telescope: how Brave and ungoogled-chromium budget their per-bump patch cost (`PRIOR_ART.md`) | this ticket |
| 2 — Fork patches re-applied | Farbling **C1, C3–C6** and the fork's registered set (`HODOS_PATCHES.md` in `Hodos-Browser/cef`; the P4e/P4f pull patches). ⚠️ **The real cost of a bump** — small for A, large for B | `DevOps-CICD/NEXT_CHROMIUM_BUILD.md` PART 1 |
| 3 — The PENDING queue | (i) **adblock payload pull** — the real fix for beta.3 Phase 12 (adblock payload lands in the wrong render process). ⚠️ **Owner decision owed first: shape `b1` (engine exposes a pull call) vs `b2` (engine injects)** — `NEXT_CHROMIUM_BUILD.md` §"Decision still owed". (ii) **`Sec-CH-UA` brand** → `"Hodos"`; already decided (our own brand, not Chrome's) | `NEXT_CHROMIUM_BUILD.md` PART 2 |
| 4 — Build | Hours, on the build host, **both platforms**. Log the patch count (`N patches total`) — the cheapest detector of a source copy that silently dropped every Hodos patch | `DevOps-CICD/CEF_BUILD_RUNBOOK.md` |
| 5 — Pin and publish | ⭐ **Pin a TAG, never a branch** — `refs/tags/pin-<sha7>/<cef-branch>` (`cef_version.py` derives the branch field from the commit's decoration). Upload versioned `cef-binaries-*` assets for **both** platforms; bump `env.CEF_ASSET` in **both arms** of `release.yml` (today: `…-150.0.43-g9ccef04` on lines 145 and 589). 🚨 A stale macOS asset **builds green and ships with no farbling** | `CEF_BUILD_RUNBOOK.md`, `BUILD_AND_RELEASE.md` |
| 6 — Stage and verify | Stage locally on both platforms (⚠️ macOS ignores `CEF_ROOT`; staging is mandatory there). **Farbling release gate re-run** — the promote gate rejects a token whose `engine=` does not match the tag's `CEF_ASSET`. macOS minimum re-measured (`CEF_VERSION_UPDATE_TRACKER.md` §macOS Minimum Deployment Version). Minimal site basket + DPI cells. A `workflow_dispatch` **validation build before the first tag** on the new engine | `FARBLING_RELEASE_GATE.md`, `DPI_RESOLUTION_TEST_MATRIX.md` |

**Deliberately out of scope:** extensions (not unlockable by self-build — `NEXT_CHROMIUM_BUILD.md`);
any new farbling vector; bumping on a schedule as a standing policy — that is a separate decision.

## Test and negative control

| | |
|---|---|
| **GREEN** | Farbling rotation token `verdict=PASS` with `engine=` equal to the new `CEF_VERSION`; codecs play (YouTube); patch count equals the fork's registered count; both `CEF_ASSET` arms name the new build |
| **RED** | The gate's own `--negative-control` goes red; a deliberately stale `CEF_ASSET` on the macOS arm is **rejected** by the engine-binding check rather than building green |
| **SUBJECT** | `CEF_VERSION` read from the **shipped** artifact, never the Chromium version — P4e and P4f both report `150.0.7871.187` |
| **Tier** | release-boundary, per `../../0.4.0-beta.3/HARNESS.md` |

## Links

- `../../DevOps-CICD/NEXT_CHROMIUM_BUILD.md` — ⭐ the entry point (PART 1 standing list, PART 2 queue)
- `../../DevOps-CICD/CEF_BUILD_RUNBOOK.md` · `../../DevOps-CICD/CEF_VERSION_UPDATE_TRACKER.md` · `../../DevOps-CICD/BASELINE_CEF150.md` · `../../DevOps-CICD/FARBLING_RELEASE_GATE.md`
- `../../0.4.0-beta.3/TICKET_engine_pins_are_branches_not_tags.md` — the pin-as-tag convention (closed)
- `../../0.4.0-beta.3/phase-12-adblock-redirect-arrivals/PHASE_CONTRACT_registry_pull.md` — the adblock pull spec
- `../../../cef-native/CLAUDE.md` — bootstrap model; *never merge-copy one distribution over another*
