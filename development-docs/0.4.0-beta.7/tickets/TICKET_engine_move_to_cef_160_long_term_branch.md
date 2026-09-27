# 🧭 Move the engine to CEF 160, the next long-term branch, before branch 7871 loses support

**Found:** 2026-09-25, beta.5 T0 research (`../../0.4.0-beta.6/track-0-engine/SCOPE.md` §2.1, §2.2, Q5).
**Status:** ⬜ UNASSIGNED · **Track:** intake — planned engine move, not a defect · **Filed by:** Claude, from G2 decision (T0 Q5 approved 2026-09-27)

> ⚠️ **Method note.** Dates are from CEF's `branches_and_building` page and chromiumdash's milestone
> schedule, fetched 2026-09-25 (**web**). Nothing was built.

## The window

| | Date |
|---|---|
| M160 stable | 2027-01-05 |
| **M160 long-term (LTC) branch starts** | **2027-01-13** |
| **Our branch 7871 (M150 LTC) support ends** | **2027-04-13** |

⇒ About **three months** to move. Planning note **D2** (build on long-term branches only) makes M160 the
next target; after M160, a long-term branch comes every 12th milestone.

## Known before starting

- **CRLF trap:** six libcef files in our fork were committed as whole-file CRLF rewrites (`116b7fd8b`,
  2026-08-07). The beta.5 security release plans a line-ending-only fix (T0 Q4); if it did not land, any
  upstream edit to those files at the M160 rebase becomes a whole-file conflict.
- A milestone bump re-litigates every fork patch (farbling C1–C6, P4e/P4f, registry pull) — run
  `cef_patch_drift_audit.sh` against the M160 tree first.
- macOS `minos` floor may change with the new framework.

## First step

At the planning of whichever release falls in Jan–Apr 2027: confirm the dates are still true (re-fetch),
then scope it with `../../DevOps-CICD/NEXT_CHROMIUM_BUILD.md` and `CEF_BUILD_RUNBOOK.md`.
