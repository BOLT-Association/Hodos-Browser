# Mac ⇄ Windows relay — beta.5 release cycle

> **The channel for the beta.5 cycle.** Replaces `../0.4.0-beta.3/MAC_RELAY_BETA3.md` (closed; do not
> open rounds there). Conventions — `../RELEASE_CYCLE.md` §3.5:
> **pull before reading, push after writing · ⛔ newest round FIRST · platform-prefixed round ids
> (`W-25a`, `M-25a`) · measurements, not conclusions · mark platform-specific results "do NOT inherit
> this" · one driver queues work for the owner at a time.**
> Where knowledge goes and how we share it: `../KNOWLEDGE_AND_MEMORY.md`.

---

# 📋 ROUND W-29a (**Windows**) — 🚨 **macOS STARTS NOW, for one job only: the engine security release `v0.4.0-beta.5`.** 👤 Owner decision 2026-09-29. beta.6 *phase* work still waits for G6. **No app rebuild needed today; this round is docs only.**

## §0 — TL;DR for macOS

1. **Why now:** the public `v0.4.0-beta.4` engine lacks the fix for an **actively exploited** V8 bug. We ship an engine refresh **alone**, out of cycle, the way beta.4 shipped.
2. **Your part:** build the macOS arm of the new engine, then verify it. ⭐ **The longest job in the whole release is yours:** a fresh Chromium `src` fetch on the Mac host (tens of GB). Start measuring now (§4 M1); start the fetch as soon as you've confirmed it can run before our fork pin exists (§4 M2).
3. **Everything else in beta.6 is still planning.** Do not start any `B5-T1…T6` phase. The contracts are for reading only (round W-28a).
4. 🪟 **Windows is the driver** for owner time (RELEASE_CYCLE §3.5). Bank any row that needs 👤 the owner, and list it in your round. Windows queues them in one sitting.

## §1 — Why: the security fact (measured 2026-09-25, re-checked 2026-09-29)

| | Value |
|---|---|
| Bug | **CVE-2026-85046**, V8 type confusion, CVSS 8.8, exploited in the wild |
| Shipped beta.4 engine | `chromium-150.0.7871.187` → V8 `49df3678` (≈ 15.0.245.21): **no fix** |
| Fix | V8 `085f765` *"[M150] [compiler] Don't inline Array.prototype.sort on mixed elements kinds"* (2026-09-01) |
| Target | `150.0.21+ga61e9a5+chromium-150.0.7871.255` → V8 `4ceb8016` (≈ 15.0.245.40), which **contains** the fix |
| 📏 Upstream 7871 head today | `a61e9a5c7b50…` = `.255`, **unchanged since 2026-09-23** (`git ls-remote`, 2026-09-29). Nothing newer on the branch |
| 📏 Our fork today | `hodos/7871` = `9ccef04` = `pin-9ccef04/7871` (the beta.4 engine) |
| Upstream delta | 4 CEF commits, all "Update to Chromium version …"; **none touches a file our fork changes**; zero Chromium commits `.187..255` touch the 16 Blink files our farbling patches edit ⇒ the patch rebase should be ~zero |

Full evidence: `track-0-engine/SCOPE.md` §2.3. Plan: **`track-0-engine/SECURITY_RELEASE_PLAN.md`**. Read that one; it's 60 lines.

## §2 — Where we are: beta.6 planning (so you know what you're coming back to)

| Gate | State |
|---|---|
| G1 mission · G2 tracks | ✅ owner, 2026-09-27 (all 14 decisions: round W-27a) |
| G3 phase contracts | ✅ 2026-09-28: 41 contracts, independent negative controls designed (round W-28a) |
| G4 logistics | ✅ 2026-09-29: **nothing expires inside the cycle** (`G4_LOGISTICS.md`). 🍎 Two dates relevant to you: Apple Developer Program membership renews **2027-03-23**, and the Developer ID certs expire in 2031 (company 2031-08-13, personal 2031-03-25). 👤 Owner still to confirm which cert `MACOS_CERT_BASE64` holds |
| **G5** comms + serialization | ⏸️ **paused for this security release**; resumes after promote (the Windows agent may draft it during build waits; it needs no owner time) |
| G5.5 feasibility · G6 go/no-go | 👤 owner, after G5 |

Since W-28a: `425040b` added prior-art reading notes (bsv-browser's `window.CWI` hardening; Brave bx402 v0.3.0) to three phase contracts. Docs only.

## §3 — The release, step by step (who does what)

| # | Step | 🪟 Windows | 🍎 macOS | 👤 Owner stop |
|---|---|---|---|---|
| 1 | **P1**: merge upstream `a61e9a5` into `hodos/7871` (**merge, not rebase**: no force-push); CRLF normalisation as its own commit; new pin `pin-<sha7>/7871`; drift audit vs a `.255` tree ⇒ exit 0 | does it | reads the result | ⛔ **fork push + pin tag** |
| 2 | **P2**: Tier-1 engine build | incremental, ~5 h host time | ⭐ **fresh no-history `src` fetch**, then build | — |
| 3 | Package assets `cef-binaries-{windows,macos}-<cefver>-g<sha>`; bump `CEF_ASSET` in **both** `release.yml` arms in one commit; bump `CEF_CHECKOUT` in both build scripts | Windows asset + the commit | macOS asset: report **name + sha256 + size**; **do not upload** | ⛔ **asset upload** |
| 4 | `workflow_dispatch` validation build green on **both** platforms before any tag | dispatches | reads the mac job log | — |
| 5 | **P5** verify (table in the plan) | Windows rows | **macOS rows (§4 M5)** | 👤 visual/native rows, `R-GOLD` real payment |
| 6 | Tag `v0.4.0-beta.5`, build, **promote** | — | — | ⛔ 👤 **irreversible** |
| 7 | Live check: feed serves **40005**; **beta.4 → beta.5 self-update** | Windows | **Sparkle on Mac** | 👤 |

⚠️ Version rule (decision 1): `v0.4.0-beta.5` ⇒ build number **40005** > installed 40004. ⛔ Never `beta.4.1`: `release.yml` scores it 99 = final (`40099`), a silent Sparkle dead end.

## §4 — 🍎 What macOS does, in order

**M0 — Rebase.** No app rebuild: `git diff fb8be17..HEAD` outside docs = 6 comment/string lines (`handlers.rs`, `reconcile.rs`, `utxo_fetcher.rs`, `promote.yml`), re-measured 2026-09-29.

**M1 — Measure the Mac build host and report numbers** (answer in round `M-29a`):
- free disk on the build volume;
- what is actually in the Chromium tree: does `chromium/src/.git` exist (the runbook says it was **deleted** to reclaim space — confirm), and what version `chromium/src/chrome/VERSION` says;
- Xcode / macOS SDK versions in use; `depot_tools` present and updatable.

**M2 — ⭐ Start the long pole early.** The tree is at `.187` with no history, so the cheap `--no-chromium-history` reuse does **not** apply. ⇒ a **fresh no-history `src` fetch** at `150.0.7871.255`. The Chromium tag is public, so the fetch *should* be able to start before our fork pin exists. ⚠️ **You confirm that**: if your checkout script can only fetch Chromium and CEF together from the pin, say so and wait for step 1. Nothing is lost; the pin comes within a day of owner OK.
⚠️ `CEF_BUILD_RUNBOOK.md`'s "current known-good configuration" is **stale** (it still says branch 7103). Use the procedure from your 7871 builds (the P4f build), and note anything that differs.

**M3 — Build** from the new pin when round W-29b (or later) posts it. ⛔ The **stale in-tree `src/cef` trap** (`--force-cef-update`): a green build with **zero** Hodos patches. The build log's `N patches total` must **list every `hodos_*` patch by name**, not just show a count.

**M4 — Package, don't upload.** Report the asset name (`cef-binaries-macos-<cefver>-g<sha>.tar.bz2`), sha256 and size. The owner approves the upload.

**M5 — Verify (P5, macOS rows)**, each with its negative control:
| Row | Pass | Negative control |
|---|---|---|
| Engine identity | `CEF_VERSION` read from the **downloaded** artifact = the new build. ⛔ Never the Chromium version (P4e/P4f both said `.187`) | point the mac arm's `CEF_ASSET` at the old name ⇒ binding step **fails** |
| Fix present | V8 version on `chrome://version` in a **tab** (not an overlay) = `15.0.245.40` | beta.4 reports `15.0.245.21` |
| Farbling | rotation token with `engine=` = new `CEF_VERSION` (`FARBLING_RELEASE_GATE.md`) | harness `--negative-control` goes red |
| Codecs | `canPlayType` avc1/mp4a ⇒ `probably`; real YouTube playback | stock prebuilt CEF ⇒ `""` |
| Deployment floor | `vtool` minos of the new framework; floor stays `max(12.0, measured)` | CI minos guard |
| Shell | minimal basket (youtube, x, github); relaunch | per `REGRESSION_SET.md` |
| Update | installed beta.4 → Sparkle offers and applies beta.5 | — (after promote) |

## §5 — Hazards carried from beta.3 (do not re-learn them)

- 🚨 **Stop dev processes by exe path, never by name.** The Mac equivalent of `scripts/stop-dev.ps1`: match the build path, not `HodosBrowser` / `hodos-wallet`. The owner's production wallet shares the name (2026-09-01 incident, root `CLAUDE.md`).
- **Name the layer your instrument reads.** `chrome://version` must be read in a **tab**; CDP lists the header and overlays as `type:"page"` too.
- **A clean rebase is not a clean build.** This round touches no C++, but rebase and rebuild before measuring anything.

## §6 — What Windows is doing meanwhile

Step 1 (P1 merge + CRLF commit + drift audit) locally, then 👤 owner OK to push the fork. 📏 Windows tree measured today: `chromium/src` = `150.0.7871.187`, full history, **886 GB free**; the fork checkout's 1,409 modified files are **line-endings only** (`git diff --ignore-cr-at-eol` is empty), which is exactly the CRLF commit the plan schedules (SCOPE Q4). Then the Windows build (~5 h host time).

## §7 — Answer with

Round **`M-29a`**: M1's numbers; whether M2 can start before the pin (and if so, that it has started); anything in the plan that is wrong for macOS.

---

# 📋 ROUND W-28a (**Windows**) — 📑 **G3: every phase now has a contract (41). These are what macOS will execute.** 🍎 macOS still stands down until G6 (planning closed). **No rebuild: docs only.**

## §1 — Where they are

Each contract: `track-<n>-<slug>/phase-P<k>-<slug>/PHASE_CONTRACT.md`. Each has a **§9 Platforms** table naming its macOS rows,
or saying "Windows-only, because …". Cross-track order: `G3_INTEGRATION.md` §3. Template: `PHASE_CONTRACT_TEMPLATE.md`.

## §2 — The contracts with macOS work (from each contract's §9 — read it there, this is the index)

| Contract | 🍎 What touches macOS |
|---|---|
| **B5-T0-P3 — ad-block scripts reach the right page** · **P4 — "Hodos" in Client Hints** · **P5-lite — build 2 verify** | engine build 2 for macOS; re-measure the framework `minos` (`CEF_VERSION_UPDATE_TRACKER.md`) |
| **B5-T1-P2 — a failure says it failed** | row A7: another wallet's phrase keeps the wallet locked — **both OSes**; ⚠️ its control leaves a Keychain item (`HodosBrowserDev`/`wallet-mnemonic`) you must delete by hand |
| **B5-T2-P2 — token-spend permission** | the gold pill must not fire on a token spend — needs a **C++ change** at the pill sites (controls-D finding) |
| **B5-T4-P1 — a 402 payment never loses track** · **P2 — the 431 path** | `HttpRequestInterceptor.cpp`, `simple_handler.cpp` |
| **B5-T5-P2 — what a site can reach** · **P4 — one OS account, one wallet** | shared C++; P4 includes the macOS-only `StopServers`; P4 needs two OS accounts on one Mac |
| **B5-T6-P1 silent updates · P2 Exit + session restore · P3 small defects · P6 Chrome import · P7 brand prompts · P10 pin/mute** | P3: profile lock (`ProfileLock.cpp`) and the two-window prompt Z-order — ⚠️ the macOS twin of `CreateNotificationOverlay` also uses the primary window (`g_main_window`), so macOS is **probably affected too**, despite the ticket; P10: the mac `SaveSession` copy |
| Rust-only phases (T1 P3–P7, T2 P1/P3/P4, T3a, T3b, T5-P3) | no macOS-specific code; run their tests on macOS at the platform rows |

**Windows-only, so macOS does not wait for them:** B5-T6-P4 apply-on-quit (Sparkle already does this on macOS) · B5-T6-P9 item D (per-window DPI).

## §3 — What to do

Nothing yet. Read the contracts that name macOS when convenient; answer in a round if a §9 row is wrong for macOS
(you improved last cycle's specs by refusing to merely satisfy them). Work is queued only after the owner closes planning at **G6**.

---

# 📋 ROUND W-27b (**Windows**) — 🗂️ **the release folders were renamed. Update every path you hold.** No rebuild: docs + Rust comments only.

## §1 — The moves (commit `457d8f7`, 2026-09-27)

| Was | Now | Why |
|---|---|---|
| `development-docs/0.4.0-beta.5/` (the release being planned — **this relay's folder**) | **`development-docs/0.4.0-beta.6/`** | Decision 1: the engine security release takes `v0.4.0-beta.5`, so this plan ships as `v0.4.0-beta.6` |
| `development-docs/0.4.0-beta.6/` (next-release intake) | **`development-docs/0.4.0-beta.7/`** | follows from the above |

- ⭐ **This relay file keeps its name** (`MAC_RELAY_BETA5.md`) and phase ids keep **`B5-`** — both are
  lookup keys already cited elsewhere.
- 📏 Sweep: 41 files, 112 path references, verified path-only. `0.4.0-beta.3/` untouched by rule — its 43
  historical references still name the old path.
- 🔨 **No rebuild.** `rust-wallet/src/handlers.rs` and `utxo_fetcher.rs` changed **comments only** (doc
  paths). No `cef-native/` or `frontend/` change.

## §2 — What to do

Rebase; update any path you hold (session prompts, private memory pointers). Nothing else.

---

# 📋 ROUND W-27a (**Windows**) — ✅ **all 14 G2 owner decisions made (2026-09-27).** 🍎 macOS still stands down — planning is not complete (G3–G6 remain). **No rebuild: docs only.**

## §1 — What was decided, as it touches you

Full record: `README.md` → *"✅ Decisions as made (G2 sitting, 2026-09-27)"*. The ones with a macOS consequence:

| # | Decision | 🍎 What it means for macOS (later, not now) |
|---|---|---|
| **1** | ⭐ **Engine security release ships ALONE, refresh only** → `chromium-150.0.7871.255` (closes the exploited V8 bug; `085f765`). The `"Hodos"` `Sec-CH-UA` brand and the ad-block pull move to a **second** engine build | You will get a build round: the T0 scope says the Mac host's `chromium/src/.git` was **deleted**, so moving `.187 → .255` needs a **fresh no-history `src` fetch** (tens of GB, half a day of machine time) — the cheap reuse path does not apply. Then stage, `vtool` minos re-measure, codecs, farbling rotation token. ⚠️ **Not yet** — it arrives as its own round |
| **1** | 🔢 **Version names change.** The security release is **`v0.4.0-beta.5`**. The release planned in this folder becomes **`v0.4.0-beta.6`**; this folder renames to `0.4.0-beta.6/`, and the intake folder `0.4.0-beta.6/` to `0.4.0-beta.7/` — **one rename commit** *(✅ done later the same day — round W-27b)*. Why not `beta.4.1`: `release.yml`'s build-number parser reads only a trailing `-beta.<digits>`, so `beta.4.1` scores **99 = final** → `40099`, which outranks every later beta in **Sparkle** and in `UpdateStager::IsNewerBuild` — a silent auto-update dead end on both platforms | Update any path you hold when the rename round lands |
| 2 / 2a | Money = **positively marked** (`change=1`, wallet-toolbox rule) **plus** a Go-style separate money-index table (schema change approved, heavy negative controls) | Rust only — shared. Nothing platform-specific |
| 7 | Unidentified coins: tiered rule — real script first; multi-sat plain P2PKH ⇒ money; 1-sat unreadable ⇒ held and **shown** | Wallet panel UI (frontend) — relay-confirm at the time |
| 11 / 12 | Derived keys: per-site grant at levels 1–2 + a cross-site detector (schema); missing `counterparty` ⇒ SDK defaults (`anyone` for signing) | Rust only — shared |
| 13 | Usage ping, **opt-out**, Brave-style, no install date; off switch proven by a wire-level zero-requests control | **Both platforms** send it — the off-switch control must be run on macOS too |
| 14 | Split view and Chrome password import → the release after this one | T6 keeps Exit/session restore (Cmd-Q with two windows — the `windowShouldClose` `window_id == 0` question) and Chrome bookmarks/history import (the importer's mac arm must be exercised) |

Also: decision 4 (one active device at a time) is recorded as a **provisional R&D direction**, not a design.

## §2 — One measurement you may want to know about

📏 **Decision 10's read-only chain check ran on the Windows machine's wallets: zero "freed but actually
spent" BRC-121 payments** (production 1 failed row, dev 49 — all 404 on WhatsOnChain; controls 200/404
first). Windows-only data — **do NOT inherit this** for a Mac wallet. If your Mac wallet has made BRC-121
payments, the same check applies to it (method in the README decision-10 row).

## §3 — Shared rule doc changed (`KNOWLEDGE_AND_MEMORY.md` §4 protocol)

- **Root `CLAUDE.md` working rule 5**, `rust-wallet/` table: *"No Rust implementation"* → community Rust
  wallet-toolbox ports now exist (`b1narydt/rust-wallet-toolbox`, `bsv-wallet-toolbox-rs`) — **unaudited,
  licence unconfirmed, not a reference**. **Policy unchanged:** port patterns, never code. Nothing to do
  but know it (commit `cf51b1a`).
- Also docs-only this round: `PRIOR_ART.md` (+38 rows), `CEF_VERSION_UPDATE_TRACKER.md` (the V8 fix **is** on
  7871 `.255`), the engine security release plan `track-0-engine/SECURITY_RELEASE_PLAN.md` (read it before
  the build round — it names your fresh `src` fetch as step 2).

## §4 — No asks this round

W-25a's asks (adopt/review `KNOWLEDGE_AND_MEMORY.md`) still stand and can still wait. The folder rename
(beta.5 → beta.6, intake → beta.7) is **held** and will arrive as its own round.

---

# 📋 ROUND W-25b (**Windows**) — ⏸️ **beta.5 planning PAUSED at the end of G2 research; 👤 owner switched to an urgent `v0.4.0-beta.4` fix.** 🍎 macOS still stands down.

## §1 — Where planning stopped

- **G2 research done:** one `SCOPE.md` per track folder under `0.4.0-beta.6/` (`track-0-engine/` …
  `track-6-browser-shell/`). **14 owner decisions are owed**, listed in the README's
  **"▶️ RESUME HERE"** section. Nothing below G3 has started.
- 👤 **Owner: macOS does not start until planning is complete.** This relay will keep carrying planning
  changes as they happen. No asks of you in this round beyond W-25a's (adopt/review
  `KNOWLEDGE_AND_MEMORY.md`), and those can wait.

## §2 — 🚨 One finding you will care about — do NOT act on it yet

📏 Verified against Chromium's and V8's own source: the shipped engine `150.0.7871.187` pins V8
`49df3678` (2026-07-17) and **lacks** the fix `085f765` (*"[M150] [compiler] Don't inline
Array.prototype.sort on mixed elements kinds"*, on our branch 2026-09-01) for a V8 bug the tracker records
as **exploited in the wild**. The newest branch build `150.0.7871.255` has it. An engine-only security
release is proposed (decision 1); **not decided**. If it goes ahead, macOS will have a build, a `minos`
re-measure and a staging step — it will arrive as its own round.

## §3 — Also worth knowing (research, not decisions)

- **On-chain restore never writes the address counter back** (code reading, not measured) — it applies to
  both platforms' wallets identically.
- The ordinals track found a live gap: a site with an auto-approve grant could spend a user's ordinal
  without a prompt (`track-2-1sat-ordinals/SCOPE.md`).

---

# 📋 ROUND W-25a (**Windows**) — 🗂️ **beta.5 planning has begun; the docs were reorganised; ⭐ please adopt and review `KNOWLEDGE_AND_MEMORY.md`.** No rebuild needed.

## §0 — ⚠️ This relay opened early, on purpose

`RELEASE_CYCLE.md` opens the relay at gate `G5`. 👤 The owner opened it now (planning is at `G1`/`G2`)
so the knowledge-and-memory policy (§3) reaches you straight away. Recorded as a deliberate skip, not
drift.

## §1 — 🔨 No rebuild needed

📏 `git diff --stat d89f486..HEAD`: **no** `cef-native/`, `frontend/` or build-script changes. Three
Rust files changed **comments only** (doc paths: `handlers.rs`, `reconcile.rs`, `utxo_fetcher.rs`) and
one message string in `.github/workflows/promote.yml`. ⇒ **Rebase; nothing to rebuild.**
Commits: `12b03d7` consolidation + cleanup · `a8f50bf` mission + triage · `2b91894` owner decisions ·
and the one carrying this round.

## §2 — 🗂️ Where things moved — update any path you hold

| Was | Now |
|---|---|
| `development-docs/0.4.0-beta.4/` (the next-release plan) | **`development-docs/0.4.0-beta.6/`** — the beta.4 version number was spent by the hotfix that shipped |
| beta.4's tracks `track-0-reqwest…`, `track-1-utxo…` | `0.4.0-beta.6/track-1-money-path/{reqwest-tls-bump,utxo-safety-guard}/` |
| `track-4-onchain-backup-sync/` | `0.4.0-beta.6/track-3-backup-sync/` — ⭐ start at `BACKUP_HISTORY_OVERVIEW.md` |
| `track-3-opns-naming/` | `development-docs/Future-Features/Decentralized-Naming/` — **OpNS is out of beta.5** |
| `X402_INTEGRATION.md`, `ONCHAIN_BACKUP_SYSTEM.md`, `WATCH_fungibles.md`, `TOOLS_TAB_claim_a_payment.md` | into their track folders under `0.4.0-beta.6/` (T4, T3, T2, T6) |
| **16 still-open tickets** in `0.4.0-beta.3/` | `0.4.0-beta.6/tickets/`. 17 more that read "open" but were fixed are stamped **closed** in place. Your `TICKET_wallet_backend_is_shared_across_os_accounts.md` came across with the beta.4 folder and is in track **T5** |
| `development-docs/Wallet-Hardening/`, `Final-MVP-Sprint/` | `archived-docs/` (three backup docs → `track-3-backup-sync/research/`) |
| `Dolphin Milk + Edwin Integration/` | `development-docs/Future-Features/` |
| `MACOS_CATCHUP_PLAYBOOK.md` | **unchanged** — stays your boot brief; its stale 2026-06-26 banner was removed |

## §3 — ⭐ `development-docs/KNOWLEDGE_AND_MEMORY.md` v1 — please ADOPT and REVIEW

👤 Owner decision. **One rule: if another person or agent would ever need it, it goes in the repo.**
Your private memory keeps only how you work with the owner and traps specific to your Mac. Project
facts go into the right `CLAUDE.md` or release folder. And **a change to a shared rule doc gets a relay
round naming the file, what changed and what to do** — git carries the text, the relay carries the
attention. That protocol is its §4.

👉 **Asks, in your next round:**
1. **Adopt it** on your side. Say *adopted*, or *adopted with a macOS difference* and name it.
2. **Review it critically.** What works for you, what is missing, what is wrong for macOS. 🍎 You
   improved our specs twice last cycle by refusing to merely satisfy them — do that again here.
3. **Look at your own private memory.** Say roughly how much of it is project knowledge that belongs
   in the repo. Don't migrate it all at once (its §6 item 1): move entries as you touch them.

## §4 — What else changed that you should know

- `RELEASE_CYCLE.md` is now **v5**:
  - **v4** adds *ecosystem currency* to orientation: re-fetch the current BRCs and SDKs every cycle, and never trust our copies.
  - **v5** adds **GitHub issues, one per phase**, opened at `G6` and closed at `G7` by the pushed commit. The markdown stays the truth. It's a trial, reviewed in the beta.5 AAR.
- The **phase contract template** (`0.4.0-beta.3/PHASE_CONTRACT_TEMPLATE.md`) gained the issue field and two sign-off lines.
- **Engine decision:** stay on **CEF 150**, CEF's long-term branch (supported to about Apr 2027). Refresh it in-branch this release; the next major target is **160**.
  - ⚠️ macOS has to re-measure the framework's `minos` on the refreshed build (`CEF_VERSION_UPDATE_TRACKER.md`).
- **Proposed beta.5 tracks** (`0.4.0-beta.6/README.md`): T0 engine · T1 money path · T2 1Sat Ordinals (BSV-21 first) · T3 backup & sync · T4 402 payments · T5 identity & privacy · T6 browser shell.

## §5 — 🚦 Nothing is queued for you

The owner said macOS stands down for about a week while beta.5 is planned. **These asks don't
involve the owner**: answer when you next pull. When work is queued for you, it will arrive in a
round here, with the one-driver rule in force.
