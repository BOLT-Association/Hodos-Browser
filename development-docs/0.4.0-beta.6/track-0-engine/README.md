# B5-T0 — Engine

**Status:** 📌 PROPOSED at G2 (2026-09-25) · **Scope doc:** owed at G2 — goal, integration check, telescope (re-fetch current BRCs/SDKs first), kaleidoscope

**Goal.** Refresh CEF 150 — CEF's long-term branch — in-branch to its newest build, and ship the two queued engine fixes (ad-blocker payload pull; `"Hodos"` in `Sec-CH-UA`).

## Tickets

Tickets stay in `../tickets/` (planning note D11); this list mirrors the register's Track column.

- [`TICKET_engine_behind_its_own_cef_branch_and_upstream_stable.md`](../tickets/TICKET_engine_behind_its_own_cef_branch_and_upstream_stable.md) — 🧱 The engine is behind its own CEF branch and two milestones behind Chromium stable — a full engine rebuild is owed

## Existing material

- Owner decisions D1/D2 in `../README.md`: stay on CEF 150 (branch 7871); build on long-term branches only; next target 160
- `../../DevOps-CICD/NEXT_CHROMIUM_BUILD.md` — ⭐ the entry point (standing list + pending queue)
- `../../DevOps-CICD/CEF_BUILD_RUNBOOK.md`, `../../DevOps-CICD/CEF_VERSION_UPDATE_TRACKER.md`, `../../DevOps-CICD/FARBLING_RELEASE_GATE.md`
