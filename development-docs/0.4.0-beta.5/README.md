# beta.5 release — scope and running order

**Opened:** 2026-08-29, from the beta.4 kickoff prompt (the telescope pass; archived to `../../archived-docs/0.4.0-beta.4-planning/`).
**Status:** ⏸️ **PAUSED 2026-09-25 at the end of G2 research** — owner switched to an urgent `v0.4.0-beta.4`
fix. ⭐ **Resume at the section directly below.**

> **Naming.** This folder produces **`v0.4.0-beta.5`**. It sits beside `0.4.0-beta.3/`, whose cycle
> is closed (AAR written) but whose relay stays the live macOS channel until this release opens its
> own at gate `G5`. ⛔ **Do not edit anything under `0.4.0-beta.3/`.**

> ⚠️ **Why this folder was called beta.4 until 2026-09-24.** It was opened as the beta.4 plan. The
> **beta.4 version number was then spent** by a hotfix: `v0.4.0-beta.4` (promoted 2026-09-24) is the
> beta.3 cycle's code plus two logging fixes, and it superseded the unpublished `v0.4.0-beta.3`.
> ⇒ **The beta.5 release follows the beta.4 release, which was a hotfix. No work in this folder was
> ever in a shipped beta.4.** The folder and its forward-looking references were renamed on
> 2026-09-24 and merged with the two beta.5 files that already existed. Dated provenance lines
> (*"created at the beta.4 telescope pass"*) and mentions of the **shipped** `v0.4.0-beta.4` keep
> the old name on purpose. The two beta.4-era planning files (`RESUME_beta4.md`,
> `SESSION_PROMPT_beta4_kickoff.md`) were folded into this README and archived to
> `../../archived-docs/0.4.0-beta.4-planning/` on 2026-09-25.

## ▶️ RESUME HERE — where beta.5 planning stopped (2026-09-25)

**Ask:** *"Where were we with beta.5?"* → read this section first, then the decisions table.

| Gate (`../RELEASE_CYCLE.md` §2) | State |
|---|---|
| **G0 Oriented** | ✅ done 2026-09-24 (consolidation, cleanup, ticket inventory) |
| **G1 Mission** | ✅ **signed off by the owner 2026-09-27** — see "🎯 G1 — Mission" below; also to be reused as the release notes |
| **G2 Tracks** | 🟡 **research done** — seven `SCOPE.md` files, one per track folder (`track-0-engine/` … `track-6-browser-shell/`); **14 owner decisions owed** (table below). Owner has **not yet read** the scope docs |
| G3–G6 | ⬜ not started. Next after the decisions: G3 phases per track, then G4 logistics, G5 comms + serialization, G5.5 feasibility, G6 go/no-go |
| macOS | 🍎 **not started, by owner decision** — macOS joins after planning is complete. Keep updating `MAC_RELAY_BETA5.md` as planning moves |

### 🟡 2026-09-25 interruption — resolved as NOT an emergency

A public beta.4 report ("permission prompts don't appear") turned out to be a **two-window Z-order bug**: the
prompt opens **behind** the other window. Single window works. 👤 Next: the owner walks through what he sees with
the modal in general → **new tickets into `tickets/`** (plus Lead B: deleting a site in the advanced wallet does
not reach the browser's approval cache) → then resume the decisions below. Detail:
`../HOTFIX_2026-09-25_prompts_not_showing/README.md`.

### 🚨 Carried out of planning — urgent

**The public build lacks the fix for an actively exploited Chromium/V8 hole** (verified 2026-09-25 against
Chromium's and V8's own source): shipped `150.0.7871.187` pins V8 `49df3678` (2026-07-17); the fix
`085f765` *"[M150] [compiler] Don't inline Array.prototype.sort on mixed elements kinds"* landed on our
branch 2026-09-01; the newest branch build `150.0.7871.255` pins V8 `4ceb8016`, which contains it. The CVE
mapping (CVE-2026-85046) is from our tracker, not re-checked. ⇒ **Decision 1** below. Detail:
`track-0-engine/SCOPE.md` §2.3.

### Owner-hours estimated by the research (agent estimates, not measurements)

T0 ~4.5 · T1 ~10–11 · T2 ~3.5 · T3 ~7.5 · T4 ~4–5 · T5 ~9 · T6 ~4 → **~43 owner-hours** before G5.5 cuts.
Research was run on Fable for T2 and T4, the current model for the rest — comparable quality on a sample of two.

### ⭐ The 14 decisions owed — each with a recommendation

| # | Decision | Recommendation | Detail |
|---|---|---|---|
| ✅ 1 | 🚨 Ship the engine refresh **now, alone, as a security release?** | **Yes.** Pick its version name deliberately so "beta.5" is not spent twice | `track-0-engine/SCOPE.md` |
| ✅ 2 | **What counts as money (RQ-1).** Both reference wallets treat a coin as money only when **positively marked**; ours is money by default | Adopt their rule using existing columns, **no schema change** | `track-1-money-path/SCOPE.md` §3 |
| ✅ 3 | **Split backup into two tracks**: *backup you can trust* and *sync & portability* | Yes; both stay in beta.5 | `track-3-backup-sync/SCOPE.md` |
| ✅ 4 | **Multi-device:** nobody runs two devices spending the same coins without conflict | **One active device at a time** for beta.5 — "detect and heal" fails the *no conflicts* bar | same |
| ✅ 5 | **Backup format:** the reference importer would reject our stripped on-chain backup | A standard export file (`.brc39`) built from the full database; the compact on-chain copy never claims to be the standard; settle the format before the first mainnet broadcast of a new version | same |
| ✅ 6 | **Recovery phrases are not portable:** HandCash, Hodos and BRC-157 derive different wallets from the same 12 words | Research phase: have import try other wallets' derivations | same |
| ✅ 7 | **What restore does with a coin it cannot identify (RQ-2)** — good/bad/ugly table | HandCash's *visible but unspendable* fits our fail-closed rule; owner reads the table | same |
| ✅ 8 | **Ordinals:** add BSV-21 *recognise, hold, show*; transfers → beta.6; per-action prompt for token spends, and build that permission first | Yes | `track-2-1sat-ordinals/SCOPE.md` |
| ✅ 9 | **x402 adapter:** nothing upstream moved | To beta.6; do "check the chain before freeing or re-paying" first | `track-4-402-payments/SCOPE.md` |
| ✅ 10 | **Run the read-only chain check on the owner's wallet** — has a "freed but actually spent" coin ever happened? | Yes — needs owner OK and which wallet (production or dev) | same, §5 P1 |
| ✅ 11 | **Derived keys:** prompt at levels 1 and 2 (the reference wallet's norm), or also scope keys to the asking site (no other wallet does) | Prompt first; site-scoping is a separate deliberate choice (breaks compatibility) | `track-5-identity-privacy/SCOPE.md` |
| ✅ 12 | **`createSignature` with no counterparty:** SDK default is `anyone`, our code would fill `self` | Match the SDK — **signing code, invariant 3: owner approval** | same |
| ✅ 13 | **User count:** adopt Brave's usage ping | Yes; state plainly that the server sees IP addresses | same |
| ✅ 14 | **Browser-shell cut line:** split view and Chrome password import → beta.6 | Yes | `track-6-browser-shell/SCOPE.md` |

### ✅ Decisions as made (G2 sitting, 2026-09-27)

| # | Owner decision | Consequence recorded |
|---|---|---|
| 1 | **Ship the engine refresh now, alone, as a security release — refresh only.** Engine → `chromium-150.0.7871.255` (closes the exploited V8 bug). The fork's CRLF line-ending cleanup (T0 Q4) may ride along: source-only, no binary change. **Not** in it: the `"Hodos"` `Sec-CH-UA` brand (irreversible once seen, and its header-vs-JS agreement is unmeasured — T0 SCOPE §5 P4, §6c) — it moves to the **second** engine build with the ad-blocker pull. Promotion stays an owner call after P5 verification. ⭐ **Version: `v0.4.0-beta.5`.** `v0.4.0-beta.4.1` was rejected on evidence: `release.yml`'s build-number parser (both the mac build step and the appcast step) only reads a trailing `-beta.<digits>`, so `beta.4.1` scores **99 = "final 0.4.0"** → build number `40099`, which outranks every later beta (`beta.5` = `40005`) in Sparkle and in `UpdateStager::IsNewerBuild` — a silent auto-update dead end. `0.4.1-beta.1` would outrank the unshipped 0.4.0 final. ⇒ **the release planned in this folder becomes `v0.4.0-beta.6`**; this folder renames to `0.4.0-beta.6/` and the intake folder to `0.4.0-beta.7/` (owner, 2026-09-27), done as one rename commit after the 14 decisions | T0 becomes two builds: **Build 1** = P1 → P2 → P5 (security release); **Build 2** = P3 ad-block pull + P4 brand → P5-lite, inside the planned release. T0 Q2/Q3/Q5 still owed at G3 |
| 2 | **RQ-1 → option B, the TS wallet-toolbox rule.** A coin is money only when **positively marked**: `change=1` on top of today's basket/derivation filters. Tokens are filed into BRC-147 baskets with tags + customInstructions. A coin nobody classified is **Unknown: held, shown, never auto-spent**. **No schema change** — `outputs.change/type/purpose/provided_by` already exist. Option C (Go's separate money-index table) stays the upgrade path if T1-P3 reservations want a table anyway | T1-P5 (classification guard) proceeds on B. Its contract must carry: (a) a one-time data migration setting `change=1` on rows that pass the classifier; (b) the `change` predicate in all six `output_repo` selectors **and** `send_max`; (c) every ingest route classifies (the `main.rs` master fix-up's fate decided there); (d) ⚠️ **the balance display includes Unknown coins** — otherwise balance silently drops (R-RESTORE's "silently lost", in everyday use). RED for R-CLASSIFY: a row inserted by a route that skips the classifier is not selectable |
| 2a | **Amended same sitting: B now, *plus* option C in T1-P3.** Go's separate spendable-money index (a table built from the `change=1` stamp; coin selection reads **only** that table; it also records which transaction reserved each coin) lands in **T1-P3 (reservations)**. 👤 **Owner approves the schema change (invariant 2)**, on the condition that it is tested exhaustively: *"test and test and test every little negative control possible to make sure… we're not breaking any wallets."* Rationale: one place to get right instead of seven selectors (six in `output_repo` + `send_max`) | T1-P3 contract must carry: migration + rollback plan for existing wallets; a negative control per selector route (disable the table read ⇒ an unstamped coin becomes selectable); upgrade from a beta.4 DB and from a restored backup. ⭐ **Edge to T3:** the index is **derived** from `outputs.change`, so backups/exports omit it and restore **rebuilds** it — T3 must test the rebuild. **Order:** T1 (money path, incl. this change) lands before T3's backup format work |
| 3 | **Backup splits into two tracks, both fixed scope (D3 unchanged).** **T3a — Backup you can trust** (single device): P0 measure → P1 round-trip harness → P2 make today's backup trustworthy → P3 re-fetchable bytes off chain → P4 freeze format + chain/crash safety. **T3b — Sync & portability**: P5 standard files in/out → P6 deltas → P7 two devices → P8 publish the BRC | A presentation choice, not a cut (T3 SCOPE §7.4). Track ids at G3: `B5-T3a-*`, `B5-T3b-*` (whether the `B5-` prefix follows the folder rename is settled in that commit). G5.5 sees T3b's two design-invalidating unknowns separately |
| 4 | ⚠️ **PROVISIONAL — an early R&D direction, not a final design.** Direction: **one active device at a time** (wallet-toolbox model, T3 SCOPE §9 Q3 option B). 👤 Owner: *"this decision is very early in the research and development phase… as soon as we kick off that ticket, it's gonna be a lot of work to design it."* Constraints stated by the owner: (a) sync must **never re-broadcast the whole backup** — deltas or something equally small; (b) the chain approach is **tested for efficiency, not assumed** — *"if this whole method is just too inefficient, then we're gonna have to just start over with using a cloud or something"*; (c) agents running on a device are a **separate future track**, but this design must not block them | T3b-P7 opens with three research questions: **R4-1** how the active device is recorded and switched (sketch: the active-device marker rides in the backup chain tip; a device that is not named is read-only) · **R4-2** the chain as the sync channel — poll vs push subscription, indexer lag (30 s–5 min today), per-delta fee and bytes, round-trip time between devices; measured from P0's E1/E3 numbers; **exit: if it cannot meet the bar, the cloud/relay question returns to the owner with evidence** (middle ground to evaluate first: off-chain relay + chain as authority, `go-private-backup-cache`) · **R4-3** agents as writers — an agent on the active device runs under its authority; an agent elsewhere likely needs its own coin allowance (option C) and BRC-181 (agent spend policy, merged 2026-09-25) ⇒ a future agents track, recorded here so P7's design leaves room for it |
| 5 | **Same wallet, two formats.** **Export/import file** = strict BRC-38/39, generated from the **full** local DB (importable by the reference `importBRC38`, which rejects dangling `transactionId`/`spentBy` refs and JSON `null` — T3 SCOPE §2.4). **On-chain backup** = our compact, stripped format; it **never claims** to be BRC-38. Format-freeze gate: the on-chain envelope/header is frozen before the **first mainnet broadcast of a new token version**. 👤 The owner's *"has to match with our on-chain backup"* means **same content**, not same bytes | ⭐ New required test for T3: **export file ⇄ on-chain restore round-trip** — both, from one wallet, must rebuild an identical wallet (incl. the decision-2a money index). Fresh read of BRCs, schemas and test vectors at T3's G3 stands (follow-ups §) |
| 6 | ⚠️ **PROVISIONAL — owner direction; detailed design when the phase opens.** (1) Hodos wallets keep BIP-32 `m`; **the BIP-32 recovery code stays in the wallet** even if nothing calls it — 👤 *"we want to keep all of the functions so we can recover wallets for people when… wallets go out of business"*. The Centbee-style PIN-protected BIP-32 path (owner said "SentBee" — confirm the name) is likely removed from the UI at that phase, code kept. (2) **Today's advice to users moving from a BIP-32 wallet:** create a fresh Hodos wallet and send the funds over; key-bound tokens are the exception (the user may want to keep their keypair). Longer term, export/import (decision 5) becomes the easy path; Project Babbage's BIP-32 → BRC-100 migration work is prior art to read. (3) **Import asks the user where the phrase came from, in tiers:** ① *a Hodos wallet* → check for a backup token (the only source that restores full data today — must work first) · ② *another BRC-100 wallet* → try BRC-157 / BRC-75 (key only; that wallet's data needs its export file or storage) · ③ *a plain BIP-32 wallet* · ④ *I don't know*. ⭐ **Triage:** if the chosen tier finds nothing, **fall back through the other tiers anyway**. Wording is user-friendly, not convention names | T3b-P5 (portability) owns it. Research at that phase: the three root conventions (T3 SCOPE §2.3), HandCash's try-both sweep (`phraseSweep.ts`), Babbage's BIP-32 migration path, a real HandCash `.brc39` (T3-P0 probe). Moving new wallets to BRC-157 is **not** decided — a separate future call (invariant 3) |
| 7 | **RQ-2 → the tiered rule** (composes T3 SCOPE behaviours 3 + 5, rejects 1/2/4/6; 7 stays true for backup-declared rows). 👤 Principle: *nothing stays unclassified for good.* ① Fetch the coin's **real** script (T1-P4 "ingest truth" — today's synced scripts are fabricated from the address) and classify it. ② **Multi-sat, plain P2PKH, no inscription envelope ⇒ money** (stamped `change=1`, enters the money index) — released on **evidence**, never on an indexer's word alone. ③ **1-sat and still unreadable ⇒ held** (spending gains 1 sat and risks an NFT). ④ Anything unresolved is **shown, never hidden**: a line under the balance ("N items being identified"), automatic retry, and a manual **"treat as money"** action. ⑤ **Restore never fails outright** because of unidentified coins — it completes and reports them | Owners: T1-P4 (real scripts, first) → T1-P5 (classifier + the Unknown display, incl. decision 2's "balance must show Unknown") → T3a-P2 (restore report). User-facing wording is settled at the **P2 restore sitting** with real coins. Negative controls: fabricated-script input must NOT classify (the vacuous-pass trap); a 1-sat inscription restored with the indexer down ⇒ held, not spendable; restore with the indexer down ⇒ completes with a report, not an error |
| 8 | **Ordinals track takes all three recommendations** (T2 SCOPE §9 Q1/Q2/Q4/Q6). (1) **BSV-21 recognise, hold, show** is in this release — P1 classify-and-file (`bsv21` basket, never `1sat`; BRC-163 MUST) and P4 balances. (2) **BSV-21 sending (P5) is deferred** to the release after this one (written "beta.6" in the scope docs; **beta.7** after the folder rename). (3) **Every token spend prompts per action, no standing grant** (BRC-165 MUST — the MUST is in 165, not 147 as our docs said), and the **permission class P2 is built first** (no T1 dependency; closes a live hole) | ⚠️ Watch item, not scope: BRC PR **#273** (2026-09-27) revives BSV-21 binary encoding (BRC-162), contradicting T2 SCOPE §2.1 item 3. Safe under decisions 2 + 7 (an unreadable binary BSV-21 output is a held 1-sat coin); the classifier may need a second encoding if it merges. Fix the 147-vs-165 attribution in `README.md` "Two rules" / `R-TOKENPERM` at G3 |
| 9 | **x402 adapter (T4-P3) deferred** to the release after this one (beta.7 after the rename), with a re-check condition: PR #2890 merges, **or** a live BRC-29 `exact` server exists, **or** the owner wants bsv.cx specifically. **T4-P1 goes first — a 402 payment never loses track of whether it was paid:** before freeing a payment's coins or minting a second payment, ask the chain; *exists* ⇒ keep and say "paid"; *not found* ⇒ free as today; *can't tell* ⇒ do nothing and tell the user "you may have paid — txid — we won't pay again until this settles" (T4 SCOPE §9 Q1). Also settled: **Q2** a payment the site refused but the chain shows is **recorded, never broadcast by us**; **Q4** always pay exactly `amount`, no tolerance knob | T4 SCOPE Q5 (reuse TTL → 20 s with A3), Q6 (body transport: defer), Q7 (fix `X402_INTEGRATION.md` §4a at G3), Q8 (read #2890's unread comment before the deferral is final) — agent-level, taken at G3 per the scope's recommendations unless the owner objects. P1 Step 0 = decision 10 |
| 10 | 👤 **Approved: run T4-P1 Step 0, the read-only "freed but actually spent" chain check, on BOTH wallets — right after decision 14, before G2 closes.** Production = the verdict; dev = read against its **known test residue** (rigs deliberately fail payments — rule 7's residue warning). Method: SQLite opened read-only (`mode=ro`), BRC-121 rows with `status IN ('failed','nosend')`, one WhatsOnChain `tx/hash` lookup per txid; **positive control first** (known-broadcast `73eab753…` ⇒ 200, known-failed `1f8a2e4e…` ⇒ 404) or the run means nothing | Any production hit ⇒ **rule 7 stop**: tell the owner, audit that row's restored inputs with `check_outpoint_spent`, re-run anything measured on that wallet since 2026-09-17. Zero ⇒ T4-P1 stays a hardening phase. Result recorded here |
| 11 | **Derived keys: parts 1 + 2 in this release** (ticket `TICKET_derived_public_keys_have_no_prompt_and_can_match_across_sites.md`). **Part 1:** derived `getPublicKey` at levels 1–2 goes through `dispatch_scoped_grant` — the same per-site grant `createSignature` has (owner agreed 2026-09-21; moves the login prompt one call earlier, adds none). **Part 2 (owner's 2026-09-21 idea — this overrides T5 SCOPE §9 Q5's "defer"):** record the requesting site on `derived_key_cache` (`requesting_domain`) and, when two different sites are given the same `(invoice, counterparty)`, **warn and log**; refusing waits until real traffic has been seen. ⭐ Why: part 1's prompt cannot show the user that two sites agreed on a `counterparty: self/anyone` key — only part 2 **sees** it. **Schema change approved** (invariant 2), same testing bar as decision 2a. **Part 3** (wallet adds the site to the keyID) → **not this release**; carried in the BRC draft `originator-scoped-authentication-keys`. **Level 0 stays open** (every BRC-100 wallet), stated plainly in the docs; part 2's log shows whether it is abused | T5-P3 contract. Negative controls: the ticket's RED (two origins, `[2,'login']`, keyID `1`, `counterparty:'self'` ⇒ same key, no prompt) must go GREEN after part 1; part 2's detector disabled ⇒ the second site's request produces **no** warning (proves the detector is what fires). Owner's level-0 idea (silence level 0 only when the site holds a higher grant) stays open, decided after part 2's data |
| 12 | 👤 **Approved (invariant 3): a missing `counterparty` is accepted with the `@bsv/sdk` 2.8.7 per-call defaults.** `createSignature` ⇒ **`anyone`**; `createHmac` and `verifySignature` ⇒ **`self`** (encrypt/decrypt already default `self`, correct). ⛔ Do **not** reuse `handlers.rs :: resolve_counterparty_pubkey`'s `None → self` for signing. The signing math is unchanged — only the default filling a blank field. Also: validate the request **before** the permission prompt (today a user can approve a call that was always going to fail) | T5-P3 item. Negative control pins the **default**, not "no error": a signature made without a counterparty verifies under `anyone` and **fails** under `self`. Decision 11's part-2 detector must watch `anyone` as well as `self` (both give every site the same key for a given protocol + keyID) |
| 13 | **User count: cut-down Brave usage ping, OPT-OUT, with three conditions** (T5 SCOPE §5 P5, §9 Q8/Q9). Once a day, one cookieless HTTPS GET: `daily`/`weekly`/`monthly`/`first` flags + OS, channel, version. **No** install date/week (`dtoi`/`woi` — near-identifying at our size), no referral, no ID/account/key/balance/URL. Conditions: ① first-run notice; ② the public **"every byte we send"** page is live **before** the ping ships; ③ a **wire-level** negative control proves the off switch sends **zero** requests (Brave shipped exactly that bug, `brave-browser#45271`). Stated plainly to users: the server **sees the IP** at request time — protection is **policy** (country at the edge, small countries suppressed, IP dropped before write, no access logs), never called "anonymous" | T5-P5. Endpoint location/readers (K11) and page wording (K12) settled at G3. ⛔ **Switch-on is outward-facing and irreversible once data exists — it returns to the owner** (unconditional stop 4) |
| 14 | **Browser-shell cut line: split view (#5) and Chrome password import (#7 slice C) → the release after this one** (beta.7 after the rename). Split view gets its own scoping run **after** session restore (T6-P2) lands. Password import comes back as **CSV only** — never touching Chrome's keys (App-Bound Encryption; Brave and Firefox both retreated to CSV). **Stays:** Chrome bookmarks + history import (slices A + B, T6-P6) | T6 Groups 1–2 stand as the keep set; Group 3 is first to cut at G5.5. If the save-password bubble is ever wanted branded, that is a `NEXT_CHROMIUM_BUILD.md` PART 2 row |

### Research follow-ups raised during the G2 sitting (2026-09-27)

- **BRC movement since the 2026-09-25 research** (`gh api repos/bsv-blockchain/BRCs/pulls`, sorted by
  update, **titles only — texts not read**): PR **#273** *"BRC-162: smaller BSV-21 binary outputs"*
  (open, 2026-09-27) re-proposes the binary encoding T2 SCOPE found **being withdrawn** (PR #267) ⇒ T2's
  "one encoding: JSON (BRC-161)" conclusion is back in question — feeds decision 8. **BRC-181** agent
  spend policy **merged 2026-09-25** (T2 SCOPE lists it as open). New, unread: #277 BRC-186 (UTXO-driven
  apps, private overlays), #278 BRC-188 (User Management Protocol). **No new BRC on money-vs-token
  classification** — BRC-46/147 still govern it.
- 👤 **"Unknown" must not be a permanent resting state** (owner, on decision 2): *"We need to classify it
  as something one way or another."* What an Unknown coin resolves to — and how it is shown — is taken
  up in decision 7 (RQ-2).
- 👤 **Backup track direction (owner, 2026-09-27, while amending decision 2):**
  1. **T3 starts by proving the database is correct** — after T1's money-path and schema changes — before
     any backup-format work; restore must rebuild the new money index correctly.
  2. **Import/export (T3b-P5) is close to a track of its own** and may deserve to go **earlier**: "can we
     export our wallet?" is worth testing first.
  3. **T3's own planning (G3) opens with a fresh read** of the current BRCs on wallet data export/import
     (BRC-38/39/40 and anything newer), the current database schemas in wallet-toolbox (TS **and** Go)
     and the SDK, and **any published test vectors** — then checks whether our export/import can match
     the standards being built now, and how that relates to the on-chain backup (decision 5).

### Confirmed by the orchestrating session (not only by the research agents)

- **Address counter after a restore:** on-chain restore never writes `current_index` back (stays 0 ⇒ address reuse);
  the file import's `UPDATE wallets … WHERE id = payload.wallet.id` targets the backup's old id and discards the
  result (`handlers.rs :: wallet_import`). Privacy, not lost money; **not yet measured**. In T1 scope.
- **The phantom coins were never the scan's fault** — the backup recorded its own transaction too late; fixed in
  March (`a1bdfc2`, `reconcile_backup_tx`) but its failures are silently ignored. 11 negative controls designed in
  `track-1-money-path/SCOPE.md` §5.


## 📝 Planning notes — owner conversation, 2026-09-24 → 25 *(pre-cycle; read before G1)*

⚠️ These are **decisions and directions from the owner, recorded before the cycle's planning gates
run**. Where they conflict with older text further down this file (the track table, "why this
order"), **these win** until the cleanup below rewrites that text.

### Decided

| # | Decision | Detail |
|---|---|---|
| D1 | **Engine: stay on CEF 150** (branch 7871) | Refresh in-branch to the newest 7871 build (upstream is at `chromium-150.0.7871.255`; we ship `.187`) and add the two queued engine fixes (ad-blocker payload pull, `"Hodos"` in `Sec-CH-UA`). Reason: 7871 is CEF's **long-term** branch, patched until ~Apr 2027. The 154 branch (8037) is a normal branch that CEF stops patching ~Oct 5, 2026, so a 154 build would need another bump within weeks to stay patched. See `tickets/TICKET_engine_behind_its_own_cef_branch_and_upstream_stable.md` (option A) |
| D2 | **Engine policy: build on CEF long-term branches only.** | We are too small a team to bump every four weeks. An off-cycle bump needs a reason (a critical security issue, or a platform feature we actually need). Next long-term branch named by CEF: **160** — its long-term branch **starts 2027-01-13** (T0 research, 2026-09-25; corrects an earlier "spring 2027" estimate). Our branch ends ~2027-04-13 ⇒ about a three-month window to move |
| D3 | **On-chain backup & sync is fixed scope — not cuttable** | The money path goes first, but backup ships in this release |
| D4 | **OpNS is out of this release** | Its track folder moves to `Future-Features/Decentralized-Naming/`. A different naming system may be researched later; it is R&D too large for this cycle |
| D5 | **Every ticket lands in a track** | Each becomes a phase, sub-phase or item inside a track, or is closed (fixed / duplicate / stale) or deferred with a re-check condition. The groupings proposed at G0 become tracks: **identity & privacy** is a real track (incl. BRC-103/104 server identity verification and `/.well-known/auth`); **browser shell** is a candidate track; **instruments & hygiene** are background items, not a track |

### Decided 2026-09-25 (second round)

| # | Decision | Detail |
|---|---|---|
| D6 | **GitHub issues: one per phase** | Opened at G6, closed at G7 by the pushed commit; markdown stays the truth. `../RELEASE_CYCLE.md` v5 §4.1a. Trial, reviewed in the AAR (`AAR_NOTES.md`) |
| D7 | **Release notes / social posts reuse the G1 mission** | 👤 The mission paragraph is the release announcement, plus bullets per track at promote time |
| D8 | **BSV-21 is T2's FIRST research question, not a late review** | Most 1Sat activity today is BSV-21 fungible tokens. Re-check BRC-163 vs BRC-175 (`track-2-1sat-ordinals/WATCH_fungibles.md`) and how the **current** 1Sat SDK handles BSV-21. Protection is already covered: a BSV-21 transfer is a 1-sat inscribed output, so T1's guard classifies it as a token |
| D9 | **Active-user count is in beta.5** | A phase in T5 — count users **without identifying them** (👤 needed for marketing and fundraising) |
| D10 | **Next-release intake folder exists** | `../0.4.0-beta.6/` — first ticket: the CI/CD pipeline review |
| D11 | **Tickets stay in `tickets/`** | The register's Track column is the assignment; each track's scope doc links its tickets. A ticket becomes a phase or item at G3 |

### ⭐ Research step one, for every track — the ecosystem is current, our copies are not

- **Re-fetch the current SDKs, libraries and specs before any design. Never trust our copies** — every
  doc in this repo predates recent ecosystem changes.
- **1Sat Ordinals:** the 1Sat TypeScript SDK was updated days before 2026-09-25. `../BSV-Tokens/` is
  kept as background with a pointer from the ordinals track, but **nothing is built on its research
  without re-checking** it against current sources.
- **Backup & sync:** review the BRCs on **wallet database format and export/import** (updated around
  2026-09-18). Goal: **interoperability** — our storage follows the standard format so users can move
  into or out of Hodos cleanly, and sync rests on a format others recognise.
- 👤 **Compatibility is a top product priority.** We build inside a set of standard protocols (BRCs)
  and alongside other apps and wallets we must work with.

### Backup track — its history is consolidated ✅

⭐ **Read `track-3-backup-sync/BACKUP_HISTORY_OVERVIEW.md` first** (2026-09-25). One agent
mapped every backup doc — the 2026-04-11 double-spend incident, the efficiency plan, the July backup
review, `track-3-backup-sync/ONCHAIN_BACKUP_SYSTEM.md`, the track's plan and research — against today's code, and wrote a
single overview: what ships, what was fixed, what contradicts what, and what the research must answer.
👤 **The bar the whole system must clear: stable, no conflicts, efficient.** If we cannot do both
conflict-free and efficient, the design is not ready. ⚠️ The overview's own verdict: **not shown
ready yet** — conflict-free for the backup chain on paper, not for two devices spending the same coin,
and delta sizes have never been measured.

### Cleanup — ✅ done 2026-09-25

| Item | Result |
|---|---|
| `RESUME_beta4.md`, `SESSION_PROMPT_beta4_kickoff.md` | Live content folded into this README (**Research questions owed**); both archived to `../../archived-docs/0.4.0-beta.4-planning/` |
| Open tickets in `../0.4.0-beta.3/` | File-by-file review of all 51: **16 open → moved to `tickets/`**; **17 that read "open" but were fixed → stamped closed with evidence**, left in beta.3. See `tickets/README.md` |
| `Final-MVP-Sprint/` | Backup inputs captured in the overview; five unscheduled leftovers → one ticket; folder archived to `../../archived-docs/Final-MVP-Sprint/`; `CLAUDE.md` pointers updated |
| `Dolphin Milk + Edwin Integration/` | Moved to `../Future-Features/` |
| `Wallet-Hardening/` | Reviewed: 7 open items → 7 tickets; the 3 docs the backup plan still cites → `track-3-backup-sync/research/`; the rest archived to `../../archived-docs/Wallet-Hardening/` |
| `../BSV-Tokens/` | Kept; the ordinals track now opens with *re-fetch first, BSV-Tokens is background* |
| `track-3-opns-naming/` | Moved to `../Future-Features/Decentralized-Naming/OPNS_TRACK_SCOPE_beta5_deferred.md` (D4) |
| Every move | Pointer sweep across the repo, including every `CLAUDE.md`, code comments and CI files |

**Next:** triage every ticket in `tickets/` into tracks (`../RELEASE_CYCLE.md` §1 step 9, gate G2), after the mission is signed off at G1.

## 🎯 G1 — Mission *(✅ SIGNED OFF by the owner 2026-09-27)*

> **beta.5 makes Hodos a wallet people can trust with more than coins — and can take with them.**
> Nothing the wallet cannot positively identify as money can be spent, by any path. Ordinals can be
> held, seen and moved on purpose. The wallet's state is backed up on chain and restored across
> devices in a standard format — without conflicts and without bloat. Every site learns only what the
> user chose to show it. All of it runs on a supported, fully patched engine and is built to the
> current BRCs, so it works with the rest of the ecosystem.

**Fixed scope (owner):** the money path and on-chain backup cannot be cut. **Out:** OpNS (D4).
**If the release runs long, cut from the back:** browser-shell items first, then the x402 adapter.

## 🧭 G2 — Tracks and ticket triage *(PROPOSED 2026-09-25)*

Every ticket in `tickets/` now names a track in the register's **Track** column (`tickets/README.md`).

| Track | Goal, one line | Tickets | Existing material |
|---|---|---|---|
| **B5-T0 Engine** | Refresh CEF 150 (long-term branch) in-branch and ship the two queued engine fixes | 1 | `tickets/TICKET_engine_behind…`, `../DevOps-CICD/NEXT_CHROMIUM_BUILD.md` |
| **B5-T1 Money path** | No path can spend what the wallet has not classified as money, and the coin/transaction record stays true to the chain | 11 | `track-1-money-path/utxo-safety-guard/`, `track-1-money-path/reqwest-tls-bump/` *(becomes a phase here)*, the reservation-ownership ticket |
| **B5-T2 1Sat Ordinals** | Hold, show, receive and deliberately transfer ordinals (BRC-147/150/165) | 2 | `track-2-1sat-ordinals/` |
| **B5-T3 Backup & sync** | On-chain backup and multi-device restore in a standard, interoperable format — stable, conflict-free, efficient | 1 | `track-3-backup-sync/` ⭐ start at `BACKUP_HISTORY_OVERVIEW.md` |
| **B5-T4 402 payments** | The x402 adapter over our BRC-121 client, plus the open 402 defects | 4 | `track-4-402-payments/X402_INTEGRATION.md` |
| **B5-T5 Identity & privacy** | A site gets only the keys, identity and wallet surface the user chose to give it; servers prove who they are (BRC-103/104); count users without identifying them | 11 | — |
| **B5-T6 Browser shell** | Multi-window, tabs, import, update visibility and consent-UI defects | 16 | `track-6-browser-shell/TOOLS_TAB_claim_a_payment.md` (outline) |
| *Background* | Agent-run instruments and hygiene — **not a track** | 7 | — |
| *Closed / owner decision / defer* | Chromium `debug.log` ✅ closed · Big Sur feed ✅ closed · screenshots in public history ✅ closed (accepted risk) · knowledge & memory architecture ✅ adopted as `../KNOWLEDGE_AND_MEMORY.md` | 4 |

⚠️ **7 tracks is above the 4–5 guideline.** That is a **feasibility (G5.5)** question, answered in
owner-hours, not now. Recommendation already on record: cut from the back — `T6`, then `T4`.

✅ **Folders match the tracks** (reorganised 2026-09-25): `track-0-engine/` … `track-6-browser-shell/`, each
with an index `README.md` listing its tickets; tickets themselves stay in `tickets/` (D11).

**Still owed to close G2** (per `../RELEASE_CYCLE.md` §2): for each track, a **scope doc** with its
**integration check** (what invariant could it break; what does it touch that we did not write; what
would we un-ship), a **telescope** pass that **re-fetches the current BRCs and SDKs first** (§3.1a),
and a **kaleidoscope** pass (*do we already have this shape?*). Then RQ-1 and RQ-2 above.

### ✅ Process change — approved 2026-09-25

**Ecosystem currency** is now part of orientation for every cycle: `../RELEASE_CYCLE.md` **v4**, §1
step 2 and §3.1a.

## The naming convention — settled 2026-09-18, applies from beta.4 forward

👤 Owner decision. Four rungs, all named from the field, and **"sprint" is retired as a scope word**
— in Scrum, SAFe and Shape Up it names a *timebox*, never a chunk of work, which is what made
"the beta.4 sprint" and "sprint 2" collide.

| Rung | Name | What it is | Artifact | Example here |
|---|---|---|---|---|
| 1 | **Release** | the version we ship, 1–3 months | `README.md`, `RELEASE_PLAN.md` | `v0.4.0-beta.5` |
| 2 | **Track** | one of the 4–5 big parallel efforts — a feature, or a bug bundle | folder + scope doc | Track 2 — 1Sat Ordinals |
| 3 | **Phase** | a slice of a track finished and **verified on its own** | **phase contract** + evidence table | *(none yet — microscope produces them)* |
| 3.5 | **Sub-phase** | ⭐ **optional, used only when needed.** A phase that grew too big to verify in one contract splits into sub-phases, each with its own | its own phase contract | — |
| 4 | **Item** | one numbered defect or row inside a phase | a row in the evidence table | `A1` |

⭐ **Sub-phase is a pressure valve, not a required rung.** Most phases go straight to items. Reach
for it when a phase has too many large items to carry one honest evidence table — and when you do,
say so in the phase contract rather than letting it happen silently.

⛔ **Never write a bare rung number.** Every reference starts at the release: *"beta.5, Track 2
(1Sat Ordinals), Phase 3"* in prose, **`B5-T2-P3`** as a short id — `B5-T2-P3.1` for a sub-phase. A
sentence that opens with "track 2" does not say which release, and beta.3 is live beside this one.

## Statuses, which are not rungs

**Candidate phase** — a proposal, during a pass, for something that *may* become a phase. It means
exactly what the words mean: we are looking into making it one. It becomes a **Phase** when the
microscope pass confirms it, and the heading over it changes from *Candidate phases* to *Phases*.
⛔ It is not a name for a rung, and nothing is a candidate once it has a contract.

⭐ **"Phase" means rung 3 and nothing else.** The planning stages are **passes** — the *telescope
pass* sets the tracks, the *microscope pass* turns one track into phases. ⛔ Do not write "scoping
phase"; write "telescope pass" or "scoping".

⚠️ **beta.3 is not renamed and will not be.** It ships in days and its vocabulary is internally
consistent; it simply calls rung 3 a phase and has no name for rung 2. Read `0.4.0-beta.3/` as
written. This convention starts here.

---

## What beta.5 is

beta.3 is browser-shell work — overlays, DPI, window identity, logging, the trust boundary.
**beta.5 is the wallet's asset layer.** Four tracks, in a fixed order, that take the wallet from
"cannot tell a token from a coin" to "holds, spends and recovers tokens deliberately".

| # | Track | Folder | One-line goal |
|---|---|---|---|
| **0** | **`reqwest` 0.11 → 0.12+** — the wallet's TLS certificate validator | `track-1-money-path/reqwest-tls-bump/` | Every outbound HTTPS call validates the server with a library that has no open advisories, without changing what the wallet sends or signs. 👤 **Added 2026-09-15 by owner decision** from the beta.3 Phase 9 dependency review; sits *ahead of* the settled 1–4 order because it is a money-path dependency change, not an asset-layer feature |
| **0.5** | **UTXO reservation ownership** — `tickets/TICKET_reservation_ownership_converge_on_spent_by.md` | *(folder at its microscope pass)* | Reservations are owned by a transaction row (`spent_by`), not a placeholder string, converging on wallet-toolbox. 👤 Placed here 2026-09-15: the ticket forbids doing it in beta.3 (it reorders `create_action_internal`), and track 1's classification seam touches the same `output_repo` exclusion logic, so it should land **before** the guard, not under it. Owner to confirm at the beta.4 kickoff |
| 1 | **UTXO safety guard** | `track-1-money-path/utxo-safety-guard/` | No path — automatic or manual — can spend an output the wallet has not classified as spendable |
| 2 | **1Sat Ordinals** | `track-2-1sat-ordinals/` | Hold, display, receive and deliberately transfer 1Sat ordinals to BRC-147 + BRC-150 |
| 3 | **OpNS unique names** | `../Future-Features/Decentralized-Naming/OPNS_TRACK_SCOPE_beta5_deferred.md` (⛔ **out of this release** — planning note D4) | Resolve and register OpNS names against BRC-174, with a live overlay proof-of-concept |
| 4 | **On-chain backup & sync** | `track-3-backup-sync/` | Delta-chain backup measured against real token workloads, multi-device sync |
| — | Tickets | `tickets/` | Reviewed and assigned into tracks by the owner, not worked ad hoc |

## Why this order — settled, do not relitigate

**Guard → 1Sat → OpNS → Backup.** Each one is a prerequisite for the next, not a preference:

1. **Guard is first because the hazard is already live.** Users can receive a 1-sat output at a Hodos
   address today, and three code paths treat it as spendable value — one of them
   (`monitor/task_consolidate_dust.rs`) runs automatically every 24 hours with no user action. Every
   later track *creates* the assets those paths destroy. Shipping track 2 before track 1 means
   shipping a feature and its own destroyer in the same release. **This track is needed even if we
   never ship ordinals.**
2. **1Sat before OpNS** because an OpNS name is carried by a 1-sat output. Names inherit ordinal
   handling; building names first means building ordinal handling badly, twice.
3. **OpNS before Backup** because names are the second real token workload, and the backup track's
   central open question is measured against real rows.
4. **Backup last** because BRC-150 provenance rows (`beefB64`) *are* the token-heavy workload that
   deltas exist to solve. Sequencing it last means measuring a workload that exists rather than
   designing for one that does not.

## Decisions taken at kickoff (owner, 2026-08-29)

| # | Question | Decision |
|---|---|---|
| 1 | Minimal defensive floor in beta.3? | ✅ **Yes — ship now.** `satoshis > 1` floor in the dust consolidator, recovery sweep and the coin-selection dust pass, landing in beta.3 under its existing ticket. Do **not** wait on the exposure question; it changes urgency, not correctness. The full classification guard stays track 1. |
| 2 | Harness: inherit or fork? | ✅ **Reference beta.3, extend by delta.** `../0.4.0-beta.3/HARNESS.md` and `REGRESSION_SET.md` are the standard. beta.5's additions live in `HARNESS_DELTA.md` and `REGRESSION_ADDITIONS.md` only. Fold both into a version-neutral `development-docs/HARNESS.md` **after** beta.3 closes. |
| 3 | Guard reach in track 1? | ✅ **General seam, one classifier.** Build the classification point with a general shape (`Spendable` / `Token` / `Unknown`, fail closed on `Unknown`); implement only the 1-sat/inscription classifier. BSV-20/21 slots in later as a classifier, not as a rewrite of the call sites. |

## The harness — inherited, not copied

⛔ **The standard is `../0.4.0-beta.3/HARNESS.md`.** Read it before writing a phase contract. It is
not restated here and it has not been forked. The reason for referencing rather than copying: its §9
gate baseline registry is *measured state*, and beta.3 is still lowering baselines. Two copies of a
measurement diverge silently.

| File | Where it lives |
|---|---|
| The standard — phase contracts, four-column evidence table, tiers, ratchets, adversarial posture | `../0.4.0-beta.3/HARNESS.md` |
| The standing regression set — R-INTEXT, R-GOLD, R-CLOSE, R-PERIM, R-COUNT, R-UPDATE | `../0.4.0-beta.3/REGRESSION_SET.md` |
| Phase contract template | `../0.4.0-beta.3/PHASE_CONTRACT_TEMPLATE.md` |
| **What beta.5 adds** — new tiers, new gates, the token-specific evidence rules | **`HARNESS_DELTA.md`** |
| **What beta.5 adds to the standing set** — new invariants that must survive every later phase | **`REGRESSION_ADDITIONS.md`** |

## The one rule carried forward from beta.3

> ⛔ **The claim follows the matrix, never the other way round.** A green result is reported with its
> red half or not at all. A row with an empty RED or SUBJECT cell is not done.

And its beta.5 restatement, because this release handles assets that cannot be recovered:

> ⛔ **Fail closed.** An output the wallet cannot classify is not spendable. "We could not tell, so we
> spent it" is the failure mode that destroys a user's asset permanently, and it has no undo.

## Folder mechanics — done 2026-08-29

Both prior standalone track folders were **moved into this folder**, not copied — originals no
longer exist, and BSV-21 is out of the ordinals folder name per §2 of the kickoff:

| Was | Now |
|---|---|
| `development-docs/1SatOrdinals-BSV21/` | `0.4.0-beta.5/track-2-1sat-ordinals/` |
| `development-docs/Onchain-Backup-and-Sync/` | `0.4.0-beta.5/track-3-backup-sync/` |

Cross-references rewritten in `development-docs/README.md`, the moved
`TRACK_KICKOFF_PROMPT.md`, and four `research/*.md` files. Verified: **zero stale references remain
outside `0.4.0-beta.3/`.**

⚠️ **Six files under `0.4.0-beta.3/` still carry the old paths** — five `SESSION_PROMPT_beta3_*.md`
and `TICKET_token_outputs_destroyed_by_dust_paths.md`. They were **deliberately not edited**, because
another session owns that folder. Five of the six are historical session prompts and are archaeology.
The ticket is live and its "Links" section points at the old ordinals path — **that one is owed**, and
belongs to whoever next touches beta.3.

## What each track folder contains right now

| Folder | State |
|---|---|
| `track-1-money-path/utxo-safety-guard/` | ⬜ **New. Scope only** (`README.md`). No prior research existed; track 1 was created at this kickoff. |
| `track-2-1sat-ordinals/` | 🟡 Carried in: `README.md` (scope + the 2026-08-05 BRC-147/150 decision) and `RESEARCH_FINDINGS.md` (protocol mechanics, provider APIs, indexer infrastructure). Both predate the guard work — read against `TELESCOPE.md` before trusting scope claims. |
| `../Future-Features/Decentralized-Naming/OPNS_TRACK_SCOPE_beta5_deferred.md` (⛔ **out of this release** — planning note D4) | ⬜ **New. Scope only.** ⛔ A new naming track doc is **required** — see below. |
| `track-3-backup-sync/` | 🟢 Carried in and **authoritative**: `IMPLEMENTATION_PLAN.md` (8 phases, D1–D15 decisions, two adversarial reviews, owner sign-offs), `README.md`, `ADVERSARIAL_REVIEW.md`, `research/`. ⛔ **Not to be redesigned.** |

⛔ **`Future-Features/Decentralized-Naming/`** (README, `OPNS_REVIEW.md`, `OPNS_RESOLVER_SCOPE.md`,
Xanaverse review) **predates BRC-174 and will be archived.** It stays where it is, is read **once**
during the track-3 outline pass, and is **not referenced after that**. Track 3 gets a new doc built
on the merged BRC, not on the old research.

## Ecosystem position — as of 2026-08-29

| | |
|---|---|
| **BRC-174** (ours, OpNS) | **MERGED** 2026-08-28 as `tokens/0174.md`, zero review comments. The development base for track 3. ⚠️ Merging is publication, **not endorsement** — §4 and §10.1 are unimplemented by anyone. |
| **Collectables** — BRC-147, 150, 159, 160, 165 | All merged and mutually coherent. **Safe to build on.** This is track 2's foundation. |
| **Fungibles** — BRC-163 vs BRC-175 | 🔴 **Contested. Do not build on.** See `track-2-1sat-ordinals/WATCH_fungibles.md` for the state and the explicit re-check gate. |
| **BSV-21 encodings** | Two exist: BRC-161 (JSON) and BRC-162 (binary/CBOR). Relevant only if the fungibles gate ever opens. |

## Verified code findings carried in — do not re-derive

Read by direct inspection of `rust-wallet` on 2026-08-29. Full detail in
`0.4.0-beta.3/TICKET_token_outputs_destroyed_by_dust_paths.md` (read-only) and in
`track-1-money-path/utxo-safety-guard/README.md`.

**Nothing files a 1-sat output into a protective basket, and three paths treat it as spendable:**

| # | Path | Trigger | Effect |
|---|---|---|---|
| 1 | `monitor/task_consolidate_dust.rs` | **Automatic, every 86,400 s** (`monitor/mod.rs:79`) | Selects `satoshis <= 1000` from the default basket, fires at 20 accumulated, packs into one output. **Destroys ordinals with no user action and no prompt.** |
| 2 | `recovery.rs` | User-triggered — but it is the **restore** flow | External scan sums every UTXO with no value filter; `build_sweep_transactions` batches into one P2PKH output. Only guard is a dust check on the *output*. |
| 3 | `handlers.rs:7257` `select_utxos_with_preference` | Ordinary coin selection | `dust_threshold_sats: 5000`, and line 7330 *deliberately includes* UTXOs at or under it. |

⭐ **The exclusion logic already exists and works.** `database/output_repo.rs:98,148` filter payment
selection to `basket_id IS NULL OR b.name = 'default'`, so a correctly-basketed output is already
safe. **The gap is classification on ingest, not exclusion.** That is why decision 3 above builds a
classification seam rather than adding value checks at five sites.

⚠️ **Unverified and owed:** whether an ordinary incoming 1-sat payment becomes a tracked
default-basket row without a recovery scan. If yes, path 1 is live for any user who has ever received
one. This changes **urgency, not the fix** — which is why the beta.3 floor ships without waiting.

### Basket and permission mechanics — verified against BRC-46, 99, 147, 165

- `p ` is **request-time routing only**. BRC-165 normalization rewrites the storage basket to `1sat`;
  the DB never stores a `p ` name.
- `database/basket_repo.rs:62-65` already rejects `p ` names per BRC-99 — the correct fail-closed
  default. It becomes route-if-supported when we implement a scheme.
- Our `domain_basket_permissions` (V18, `domain_permission_repo.rs:407`) grants **one domain, one
  basket, binary**. BRC-99/165 scopes let a grant name an axis (`all` / `collection` / `app` /
  `creator` / `id`) with the value carried in tags. Ours is narrower than the spec.
- ⛔ **Spend is a `createAction` label** (`p 1sat input id <key>`), not a basket. **BRC-147 says pay
  and auto-pay grants MUST NOT authorize ordinal spends.** So the auto-approve engine needs a
  separate token-spend permission class, and the approval modals must show which baskets and
  sub-categories a grant covers. This lands in track 2, and it touches the privacy-perimeter
  surface — treat it with the seriousness of `R-PERIM`.

## What "done" looks like for beta.5

To be filled in at the end of the microscope pass, not now. Two structural rules that hold regardless:

1. **Every track doc sets specific goals with well-defined outcomes** for functions and tests,
   against the testing methods identified by research (c) — see `research/`.
2. **A track that ships a token capability without the guard in front of it is not done**, however
   green its own evidence table is.

## Decisions owed before track 1 implementation starts

1. **Exposure question** (above) — answer by experiment, not by reading. Sizes the guard's urgency
   and tells us whether existing users are already affected.
2. **Where the classification seam sits** — ingest-only, or ingest plus a verification pass on
   reconcile. Microscope decision; do not settle it here.
3. **The BSV-20/21 review phase's own gate question** (owner's): *do wallets need BSV20/21 code at
   all, or is it only the apps that talk to wallets?* Carried into track 2 as a **review** phase,
   not a build phase. Record the testing problem with it: most 1Sat/BSV21 apps ship their own
   wallets, so we may have nothing to test against.
4. ~~**Whether the OpNS overlay PoC is in scope**~~ — ✅ **moot: OpNS is out of this release** (planning
   note D4, 2026-09-25).
5. ⭐ **RQ-1 and RQ-2** — the two research questions below. Both are **research tasks, not decisions**.

## Research questions owed before track work *(carried from `RESUME_beta4.md` §3, archived 2026-09-25)*

⭐ Two questions sit **between** tracks. If each track's session answered them on its own, you would
get incompatible answers — so they are researched once, up front, and then the owner decides.
⛔ **Owner correction, 2026-08-30: both are RESEARCH TASKS.** Research them and present them for
decision; do not settle them on the spot or from first principles (`CLAUDE.md` working rule 5).
⭐ Per planning note "Research step one": **re-fetch the BRCs and SDKs first**; our copies are stale.

### RQ-1 — What does "this output is a token" get saved as?

When the wallet decides an output is a token rather than spendable money, **where is that fact
stored, and in what form?** The guard track has to build it; the ordinals track then implements a
spec with its own rules for how tokens are filed. If the guard guesses and the spec disagrees, the
ordinals track rewrites the one thing the release's safety rests on.

| # | Read | Looking for |
|---|---|---|
| 1 | **BRC documentation** — 46, 99, 147, 150, 165 in particular | What the spec *requires* to be persisted, versus what it leaves to the implementer |
| 2 | **BSV Association `wallet-toolbox`** — **TypeScript** and **Go** | How a conforming wallet actually stores basket/token classification — the closest thing to a reference answer |
| 3 | **The other BSV SDKs**, across languages | Where they agree, that is the convention. ⭐ **Where they disagree, that is the real design question** — report it as such |
| 4 | Our own code — `output_repo.rs`, `basket_repo.rs`, `domain_permission_repo.rs` | What we already have, and how far it is from the above |

⚠️ There is no Rust `wallet-toolbox`: we port **patterns and semantics, never code**.
**Output:** a short document — what the ecosystem does, where implementations disagree, what we
should do **and why**, with the trade-offs visible. Then the owner decides.

### RQ-2 — What does restore do with an output it cannot identify?

Restoring from a recovery phrase is the moment the wallet knows least. ⛔ **Owner correction,
2026-08-30: this needs a full conversation and a good / bad / ugly outcome matrix, not a two-option
recommendation.**

| Required | Meaning |
|---|---|
| ⭐ **A good / bad / ugly outcome matrix** | For **each** candidate behaviour: the good case, the bad case, and **the ugly case** — the one where the user loses something and does not find out for months |
| The candidate behaviours | At least: show as held-but-unidentified · block the restore · classify-later-on-reconcile · anything the ecosystem does that we have not thought of |
| ⭐ **How other wallets handle it** | `wallet-toolbox`'s recovery path, the BSV SDKs, any BRC that speaks to recovery. Somebody has hit this already |
| The user-facing consequence of each | In plain language, not database state |

**Why it matters this much:** the two failure modes are opposites, and a naive fix for one causes the
other. An output that is **silently spendable** breaks the release's core rule at the worst moment.
One that is **silently dropped** is safe from spending and effectively lost. That is why `R-RESTORE`
in `REGRESSION_ADDITIONS.md` is two checks that are each other's control. It is also the sharpest
edge between the guard track and the **backup** track, which owns restore.

### Also still owed *(from `RESUME_beta4.md` §5)*

- **The exposure question** (decision 1 above) — ⚠️ it has been *read* twice already; **answer it by
  running something**.
- **`../SCOPING_PROCESS.md` §7a / §7b / §7b-ii** — ~19 proposed adoptions, one owner decision each,
  none in force. Not urgent.
- `tickets/TICKET_e2e_specs_wrong_subject_and_never_run.md` is second-hand and unverified; its first
  step is verification.
- ⭐ **The belief to test first** (`TELESCOPE.md` §6): *the classification seam is cheap because the
  exclusion logic already works.* The guard track's day-one call-site sweep tests it. If there are
  UTXO-selecting paths that bypass `output_repo`, the guard is bigger than scoped and the plan changes.

## Reading order for a fresh session

⛔ **Resuming after a gap?** Start at the **Planning notes** section near the top of this file — it
carries the owner's latest decisions and the cleanup state. (`RESUME_beta4.md` did this job until
2026-09-25; its live content now lives in this README and it is archived.)

```
0. this README — Planning notes, then Research questions owed
1. TELESCOPE.md            ← how the four tracks interact, and how to run the microscope pass
2. RELEASE_PLAN.md          ← the track-level breakdown
3. ../0.4.0-beta.3/HARNESS.md + REGRESSION_SET.md   ← the standard
4. HARNESS_DELTA.md + REGRESSION_ADDITIONS.md       ← what beta.5 adds to it
5. the one track folder you are working in — and no others
```

⛔ **Do not read all four track folders in one context.** That is the specific mistake
`TELESCOPE.md` §"Context strategy" exists to prevent.
