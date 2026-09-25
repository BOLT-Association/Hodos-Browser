# 🔧 Nobody has reviewed what our CI/CD pipelines actually test, and nothing builds on push

**Found:** 2026-09-25, 👤 owner during beta.5 planning: *"I never even look at those. I should look at
them and see what the tests are."*
**Status:** ⬜ UNASSIGNED · **Track:** unassigned (beta.6) · **Filed by:** Claude, at the owner's request

> ⚠️ **Method note.** Nothing below was measured for this ticket; it records the question and the
> known context.

---

## What is known

- The workflows live in `.github/workflows/` (e.g. `release.yml`, `promote.yml`). They run the
  **release** build on the public `release` repo; `promote.yml` gates promotion.
- ⛔ **Nothing compiles the C++ shell on push, by decision** (root `CLAUDE.md`, owner 2026-09-12):
  `origin` is private on the free GitHub plan. Windows and macOS each build locally; the release build
  is the backstop.
- `preflight -Full` is a **local** code gate, and the beta.3 AAR records that it is often quoted as if
  it were a release verdict.

## What the review should answer

1. What does each workflow run, and which tests actually execute in CI? What would pass with the
   feature absent (the negative-control question)?
2. What would it cost to build/test on push — free-plan minutes, a self-hosted runner on the build
   host, or public-repo CI?
3. How do GitHub issues (beta.5 trial, `RELEASE_CYCLE.md` §4.1a) and CI connect — status on pull
   requests, `Closes #N`?

## Links

- `../../0.4.0-beta.3/AAR.md` §4 (instruments that did not reach where users live)
- `../../DevOps-CICD/BUILD_AND_RELEASE.md`, `../../DevOps-CICD/TESTING.md`
