# Mac ⇄ Windows relay — beta.5 release cycle

> **The channel for the beta.5 cycle.** Replaces `../0.4.0-beta.3/MAC_RELAY_BETA3.md` (closed; do not
> open rounds there). Conventions — `../RELEASE_CYCLE.md` §3.5:
> **pull before reading, push after writing · ⛔ newest round FIRST · platform-prefixed round ids
> (`W-25a`, `M-25a`) · measurements, not conclusions · mark platform-specific results "do NOT inherit
> this" · one driver queues work for the owner at a time.**
> Where knowledge goes and how we share it: `../KNOWLEDGE_AND_MEMORY.md`.

---

# 📋 ROUND W-27a (**Windows**) — ✅ **all 14 G2 owner decisions made (2026-09-27).** 🍎 macOS still stands down — planning is not complete (G3–G6 remain). **No rebuild: docs only.**

## §1 — What was decided, as it touches you

Full record: `README.md` → *"✅ Decisions as made (G2 sitting, 2026-09-27)"*. The ones with a macOS consequence:

| # | Decision | 🍎 What it means for macOS (later, not now) |
|---|---|---|
| **1** | ⭐ **Engine security release ships ALONE, refresh only** → `chromium-150.0.7871.255` (closes the exploited V8 bug; `085f765`). The `"Hodos"` `Sec-CH-UA` brand and the ad-block pull move to a **second** engine build | You will get a build round: the T0 scope says the Mac host's `chromium/src/.git` was **deleted**, so moving `.187 → .255` needs a **fresh no-history `src` fetch** (tens of GB, half a day of machine time) — the cheap reuse path does not apply. Then stage, `vtool` minos re-measure, codecs, farbling rotation token. ⚠️ **Not yet** — it arrives as its own round |
| **1** | 🔢 **Version names change.** The security release is **`v0.4.0-beta.5`**. The release planned in this folder becomes **`v0.4.0-beta.6`**; this folder renames to `0.4.0-beta.6/`, and the intake folder `0.4.0-beta.6/` to `0.4.0-beta.7/` — **one rename commit, not yet done**. Why not `beta.4.1`: `release.yml`'s build-number parser reads only a trailing `-beta.<digits>`, so `beta.4.1` scores **99 = final** → `40099`, which outranks every later beta in **Sparkle** and in `UpdateStager::IsNewerBuild` — a silent auto-update dead end on both platforms | Update any path you hold when the rename round lands |
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

- **G2 research done:** one `SCOPE.md` per track folder under `0.4.0-beta.5/` (`track-0-engine/` …
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
| `development-docs/0.4.0-beta.4/` (the next-release plan) | **`development-docs/0.4.0-beta.5/`** — the beta.4 version number was spent by the hotfix that shipped |
| beta.4's tracks `track-0-reqwest…`, `track-1-utxo…` | `0.4.0-beta.5/track-1-money-path/{reqwest-tls-bump,utxo-safety-guard}/` |
| `track-4-onchain-backup-sync/` | `0.4.0-beta.5/track-3-backup-sync/` — ⭐ start at `BACKUP_HISTORY_OVERVIEW.md` |
| `track-3-opns-naming/` | `development-docs/Future-Features/Decentralized-Naming/` — **OpNS is out of beta.5** |
| `X402_INTEGRATION.md`, `ONCHAIN_BACKUP_SYSTEM.md`, `WATCH_fungibles.md`, `TOOLS_TAB_claim_a_payment.md` | into their track folders under `0.4.0-beta.5/` (T4, T3, T2, T6) |
| **16 still-open tickets** in `0.4.0-beta.3/` | `0.4.0-beta.5/tickets/`. 17 more that read "open" but were fixed are stamped **closed** in place. Your `TICKET_wallet_backend_is_shared_across_os_accounts.md` came across with the beta.4 folder and is in track **T5** |
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
- **Proposed beta.5 tracks** (`0.4.0-beta.5/README.md`): T0 engine · T1 money path · T2 1Sat Ordinals (BSV-21 first) · T3 backup & sync · T4 402 payments · T5 identity & privacy · T6 browser shell.

## §5 — 🚦 Nothing is queued for you

The owner said macOS stands down for about a week while beta.5 is planned. **These asks don't
involve the owner**: answer when you next pull. When work is queued for you, it will arrive in a
round here, with the one-driver rule in force.
