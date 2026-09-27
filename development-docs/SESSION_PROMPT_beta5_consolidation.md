# SESSION PROMPT — consolidate into `0.4.0-beta.5`, clean `development-docs/`, and orient

**Written:** 2026-09-24, by the Windows session that shipped `v0.4.0-beta.4`, while that cycle was
still in context. **For:** a fresh, memoryless session on **Windows**.

> ⛔ **Read these three first, in this order. They are the contract you are working under.**
> 1. `development-docs/RELEASE_CYCLE.md` — **v3**. The cycle spine and its gates `G0`–`G12`
> 2. `development-docs/0.4.0-beta.3/AAR.md` — the last cycle's review. ⭐ Gate `G0` requires it
> 3. Root `CLAUDE.md` — working rules, especially **1** (ask, don't assume), **7** (poisoning),
>    and **8** (never hand the owner a bare identifier — say what it *is*)

---

## 0. Where things stand

**`v0.4.0-beta.4` shipped 2026-09-24** and is live: feed, download redirects and the N−1 → N
self-update all verified on both platforms. It was the **first 0.4.0 the public ever had** — beta.1
and beta.2 were built and never promoted.

The beta.3 cycle is **closed**: its AAR is written, and the process it produced is
`RELEASE_CYCLE.md` v3. This session does **not** re-open any of that.

⚠️ **The problem you are here to fix.** `development-docs/0.4.0-beta.4/` is the *next* release's
folder — `TELESCOPED`, five tracks, no phase design — but **the beta.4 version number is now spent**,
consumed by the hotfix that shipped. Its target has to become **beta.5**, and a second folder
`0.4.0-beta.6/` already exists with two loose tickets in it. Plus `development-docs/` root has
accumulated finished and misfiled documents.

---

## 1. Your job, in three parts

### Part A — consolidate `0.4.0-beta.4/` + `0.4.0-beta.6/` → **`0.4.0-beta.6/`**

- `git mv` the beta.4 folder's contents into `0.4.0-beta.6/`, merging with the two tickets there
  (`TICKET_chrome_import_bookmarks_history_passwords.md`, `TOOLS_TAB_claim_a_payment.md` — fold the
  latter into `tickets/` if it is one, and say so if it is not).
- Inventory today: **21 tickets**, 5 track folders (`track-0-reqwest-tls-bump` …
  `track-4-onchain-backup-sync`), plus `README.md`, `TELESCOPE.md`, `RELEASE_PLAN.md`,
  `HARNESS_DELTA.md`, `REGRESSION_ADDITIONS.md`, `RESUME_beta4.md`, `WATCH_fungibles.md`,
  `SESSION_PROMPT_beta4_kickoff.md`, `research/`, and one loose ticket.
- ⛔ **Rename every beta.4 → beta.5 reference inside the files too**, not just the folder. The
  README's own line *"This folder produces `v0.4.0-beta.4`"* is the headline case.
- ⚠️ **`RESUME_beta4.md` and `SESSION_PROMPT_beta4_kickoff.md`** are beta.4-era artifacts. Decide:
  renamed and updated, or archived as superseded by this prompt. **Recommend, don't guess silently.**
- ⭐ **Add a note** in the new README that the beta.5 **release** follows the beta.4 **release**,
  which was a hotfix, and that no work in this folder was ever in a shipped beta.4.

### Part B — clean `development-docs/` root

> **The rule:** the root holds only **living, cross-release** documents. Release-specific work lives
> in a release folder. Finished work lives in `archived-docs/`.

👤 **Owner-approved disposition** — execute it:

| File | Action |
|---|---|
| `README.md`, `RELEASE_CYCLE.md`, `SCOPING_PROCESS.md`, `PRIOR_ART.md` | **keep** — living, cross-release |
| `MACOS_CATCHUP_PLAYBOOK.md` | ⭐ **keep** — it is the **macOS agent's boot brief** ("you are a fresh, memoryless agent… this is your complete brief"), actively maintained. ⚠️ **Strip its stale 2026-06-26 priority banner** about update-stability, which is long done |
| `X402_INTEGRATION.md` (64 KB) | → **`0.4.0-beta.6/`**. 👤 Owner: this becomes a **track** this cycle |
| `ONCHAIN_BACKUP_SYSTEM.md` | → **`0.4.0-beta.6/`** as input to the backup track. ⚠️ Dated **2026-04-21** — treat as a starting sketch, **not** a spec |
| `TICKET_brc121_remint_on_retry.md`, `TICKET_debug_log_unfiltered_in_production.md`, `TICKET_knowledge_and_memory_architecture.md`, `TICKET_logged_in_screenshots_in_public_history.md`, `TICKET_profile_lock_misreports_missing_dir.md`, `TICKET_reservation_ownership_converge_on_spent_by.md` | → **`0.4.0-beta.6/tickets/`** |
| `TICKET_farbling_constant_seed_shipped.md` | ⭐ **verify, then archive** — see §2 |
| `QR_SCAN_OVERVIEW.md`, `QR_SCAN_WINDOWS.md`, `QR_SCAN_MACOS.md` | → **archive.** 📏 Verified shipped: `quirc` is vendored and built (`cef-native/CMakeLists.txt`), and two docs say **COMPLETE** in their own titles |
| `MACOS_SPRINT_HANDOVER_20260501.md` | → **archive** — superseded by the playbook |
| `AUDIT_SCOPE.md` | → **archive** |
| `FUTURE_AUTO_APPROVE_ENGINE_ARCHITECTURE.md` | → **archive** — root `CLAUDE.md` already says to read it as history, not a plan |
| `CEF_BINARIES_README.md` | → **`DevOps-CICD/`**. 📏 Verified **zero inbound references**, so no pointers break — but **add it to that folder's index** so it stops being invisible |

⛔ **After every move: grep the whole repo for the old path and fix every pointer.** Root `CLAUDE.md`,
the relays, `DevOps-CICD/README.md`, and ticket indexes all carry paths. A move that leaves a dangling
reference is worse than no move.

### Part B.5 — ⭐ ADD A TRACK: the Chromium/CEF engine bump

👤 **Owner, 2026-09-24, after the disposition was agreed:** *"we should do a new full Chromium
build at the start of beta.5… that needs to be a whole track… that will be what we need to do first
because we'll build everything on top of that."*

⛔ **Create this as a ticket in `0.4.0-beta.6/tickets/` and carry it into your `G2` track proposal as a
likely Track 0.** It is not optional scope and it is not small.

**Why it sequences first — and note WHICH argument this is.** ⭐ It is a **serialization** constraint,
not a priority one: every other track compiles against the engine, so a bump afterwards re-tests
everything. It is also the highest-**uncertainty** work in the cycle, which `RELEASE_CYCLE.md` §3.7
says to front-load — an engine bump re-litigates every fork patch, and you want that in week one.

⚠️ **Version: VERIFY, do not assume.** 👤 The owner believes **Chromium 154 recently went stable**.
📏 We currently ship `CEF_VERSION 150.0.43-7871.3576+g9ccef04+chromium-150.0.7871.187`. ⛔ Confirm the
current stable Chromium **and** which CEF branch tracks it before writing any number into a plan —
CEF branches lag Chromium, and the pin must be a real CEF branch, not a Chromium version we wish for.
⛔ **Read `CEF_VERSION` for identity, never the Chromium version** — three different engines we have
shipped all report `150.0.7871.187`.

**Its existing home documents** — this track should reference rather than re-derive them:

| Doc | What it carries |
|---|---|
| `DevOps-CICD/NEXT_CHROMIUM_BUILD.md` | ⭐ **the entry point.** PART 1 = what goes in **every** build forever (proprietary codecs; farbling patches C1, C3–C6). PART 2 = the **PENDING queue** |
| `DevOps-CICD/CEF_BUILD_RUNBOOK.md` | step by step, ~1,300 lines |
| `DevOps-CICD/CEF_VERSION_UPDATE_TRACKER.md` | pin history, and the macOS **minimum deployment version** — ⚠️ this floor can move on a bump |
| `cef-native/CLAUDE.md` | the bootstrap model, the wrapper build, the *"never merge-copy one distribution over another"* trap |

**📏 Already queued in PART 2, both added 2026-09-21 — this track inherits them:**
1. **Cosmetic-filter payload delivery** — the real fix for **Phase 12** (the adblock push landing in
   the wrong render process). Shipped mitigation only; the engine patch is the cure
2. **`Sec-CH-UA` brand** — say who we are. ⚠️ **A decision is still owed** on this one (§"Decision
   still owed" in that doc), and 👤 the owner already chose **our own brand, not Chrome's**

**Scope the track to include, at minimum** — expect several phases, and say so:
- Target selection and the CEF branch that carries it
- ⛔ **Re-applying the fork patches** (farbling C1, C3–C6 + P4e/P4f) to the new branch. ⚠️ **This is
  the real cost of a bump** and the reason `cef-native/CLAUDE.md` says capability is bounded by
  *patch scale and per-bump maintenance*
- The PART 2 queue above
- The build itself — **hours**, on the build host, both platforms
- ⭐ **Pinning: a TAG, never a branch** — `refs/tags/pin-<sha7>/<cef-branch>`, because
  `cef_version.py` derives the version's branch field from the commit's decoration
- Uploading **versioned `cef-binaries-*` assets for BOTH platforms** to the org repo, and bumping
  `env.CEF_ASSET` in **both arms** of `release.yml`. 🚨 A stale asset is a **silent** failure on
  macOS — it builds green and ships a browser with **no farbling at all**
- Staging locally on both platforms (⚠️ macOS ignores `CEF_ROOT`; staging is mandatory there)
- ⭐ **Re-running the farbling release gate against the new engine** — the promote gate checks the
  token's `engine=` against the tag's `CEF_ASSET`, so an old token will be rejected
- A `workflow_dispatch` validation build **before** the first tag on the new engine

### Part C — gate `G0` (orientation) and propose a mission

Per `RELEASE_CYCLE.md` §1, steps 1–5:

1. **The project** — root `CLAUDE.md`
2. **Higher** — engine pin, signing chain, BRC specs we conform to
3. **Adjacent** — what macOS is doing (its relay), what is live in the field (`v0.4.0-beta.4`), what
   users are running
4. ⭐ **Ourselves** — the beta.3 **AAR**, and a full inventory of every ticket now in `beta.5/tickets/`
   (expect **~28**: 21 + 6 moved + the loose one)
5. **Standing serialization** — restate `RELEASE_CYCLE.md` §3.6

Then **propose** a mission (`G1`) and a first-cut track grouping, and **stop for the owner.**

---

## 2. ⭐ One verification before you archive anything

`TICKET_farbling_constant_seed_shipped.md` reads **OPEN** — *"awaiting owner decision on whether to
fix ahead of the P4 Blink migration."* **That decision was overtaken by events.**

📏 On 2026-09-24 the farbling rotation gate passed on the shipped engine, with its negative control:

```
FARBLING-ROTATION-v1 engine=150.0.43-7871.3576+g9ccef04+chromium-150.0.7871.187
  exempt=53225ec8/53225ec8/53225ec8  large=0cdc9b48/0cdc9b48/0cdc9b48
  farbled=0e4e6251/6eeaa098/0e4e6251  verdict=PASS
```

Seed A ≠ seed B is **exactly** the unlinkability property the constant-seed bug failed. ⇒ Confirm
that reading, mark the ticket **CLOSED with that evidence**, then archive it.

⚠️ **Do not simply delete or silently archive an OPEN ticket.** Close it *with its evidence* — an
undocumented close is how a defect gets re-discovered in six months.

---

## 3. Open questions — bring these back, do not decide them alone

1. ⭐ **The on-chain backup work: one track or two?** 👤 The owner is genuinely undecided and wants it
   settled at `G2` **with scope information in hand**, not now. Context he gave:
   - It is **big** — likely several phases and sub-phases
   - It may produce a **BRC** (a public protocol spec), and ⚠️ **a published spec is far harder to
     change than code** — getting the *format* wrong is expensive in a way that getting the
     *implementation* wrong is not. That asymmetry argues for settling the format early
   - He wants **research and implementation interleaved**, not sequential. ⭐ **This is not bad
     practice** — it is *tracer bullet* / *walking skeleton* / *spike*, all established. ⛔ **But the
     wallet caveat is real: a spike must be thrown away.** The failure mode is the exploratory
     version becoming production by inertia. If interleaved, name the moment it is discarded
   - ⭐ **"Early" is a serialization question, not a priority one** — if other tracks read its
     output, it is a dependency, settled at `G0`/`G5` regardless of importance

2. ⭐ **The engine bump (Part B.5): how many phases, and does it block the other tracks?** Bring a
   recommendation. ⚠️ If other tracks cannot start until the engine lands, say so plainly — that is a
   schedule fact the owner needs at `G1`, not a discovery at week three.

3. **`X402_INTEGRATION.md` → a track.** How does it relate to BRC-121? (Prior finding: x402 is
   *"BRC-121 in a different envelope"*, and `pay_402` is byte-compatible ⇒ an adapter, not a rewrite.)

4. **`RESUME_beta4.md` / `SESSION_PROMPT_beta4_kickoff.md`** — rename or archive?

---

## 4. What NOT to do

- ⛔ **Do not touch `0.4.0-beta.3/`** beyond what §1 requires. That cycle is closed; its AAR is written
- ⛔ **Do not start any track work.** This session ends at `G0`/`G1`. No phase design, no code
- ⛔ **Do not archive `0.4.0-beta.3/` yet** — its relay is still the live channel until beta.5's is
  opened at `G5`. ⭐ *Archive means reviewed, not merely shipped*
- ⛔ **Do not re-litigate** anything in the AAR's §8 ("deliberately not changed")
- ⚠️ **Rebase before you start and rebuild after** — macOS pushes to `0.4.0` several times a day, and
  a clean rebase is not a clean build. Expect append-vs-append conflicts in shared docs; **keep both
  sides**

---

## 5. Carried forward — open, not yours to fix this session

- 🔴 **Regenerate `WEBSITE_DEPLOY_TOKEN`** — never expires, and was pasted into a chat log. 👤 Owner's action
- 🟨 **Phase 12** (adblock push lands in the wrong render process) — mitigation shipped, engine fix
  parked in `DevOps-CICD/NEXT_CHROMIUM_BUILD.md` PART 2
- ⬜ **Phase 13** (bot detection) — parked; reopen only on a reproduction against a current build
- 🔐 **Wallet shared across OS accounts** (macOS-filed, applies to Windows identically) — loopback
  ports are machine-wide, so a browser in any OS account adopts whatever wallet is on `31301`
- ⬜ **`G1` static gate gap** — it is named for a defect it was green through; ticketed, not widened

---

## 6. How to report back

Per working rule 8: **every id carries its subject in plain language.** Not *"moved 21 tickets"* —
name the notable ones. Not *"T2"* — *"Track 2, the 1Sat Ordinals work."*

End with:
1. What moved, and what broke that you fixed *(the pointer sweep)*
2. The **ticket inventory**, grouped as you would propose tracks
3. Your proposed **mission** for beta.5, in one paragraph
4. The **four** open questions from §3, with a **recommendation** on each
