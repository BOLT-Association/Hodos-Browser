# B5-T3 — On-chain backup & sync: track scope (G2)

**Written:** 2026-09-25, by the G2 research agent for this track. **Status:** 📌 PROPOSED — owner review owed.
**Reads with:** `BACKUP_HISTORY_OVERVIEW.md` (the history), `IMPLEMENTATION_PLAN.md` (the design authority
until the owner re-opens a decision), `../README.md` (planning notes, G1 mission, RQ-2).

> **Labels used on every claim:** **[code]** = I read the current code on branch `0.4.0` (head `399cc4c`,
> 2026-09-25) · **[measured]** = someone measured a number; source named · **[doc]** = a repo document says
> so; not re-verified · **[web: …]** = fetched 2026-09-25, source and version/commit given.
> Nothing was built or run. No wallet database was opened. No code was changed.

> 👤 **The bar (owner):** stable, conflict-free, efficient. If it cannot be both conflict-free and
> efficient, the design is not ready. This document says plainly where each candidate stands against it.

---

## ✅ 0. G2 decisions applied (owner, 2026-09-27)

> The research below is the **evidence**; this block is the **decision**. Where they differ, this block wins.
> Full record: `../README.md` → "✅ Decisions as made".

| Question | Decided |
|---|---|
| §9 **Q4** one track or two | ✅ **Two, both fixed scope (decision 3).** **T3a — Backup you can trust:** P0 → P4. **T3b — Sync & portability:** P5 → P8 |
| 👤 T3a's first step | **Prove the database is correct** (after T1's money-path + schema changes) before any format work; restore must **rebuild** the new money index (derived from `outputs.change`, so omitted from backups) |
| **Q2** format | ✅ **Same wallet, two formats (decision 5):** strict BRC-38/39 export from the full DB; compact on-chain format that never claims to be BRC-38; freeze before the first mainnet broadcast of a new version. ⭐ New required test: **export file ⇄ on-chain restore round-trip → identical wallet** |
| **Q1** root key | ✅ **Provisional (decision 6):** keep BIP-32 `m`; keep the BIP-32 recovery code; import asks where the phrase came from in tiers (Hodos with backup token → other BRC-100 → plain BIP-32 → don't know) and **falls back through the rest** if nothing is found. BRC-157 for new wallets **not** decided |
| **Q3** two devices | ⚠️ **PROVISIONAL R&D direction (decision 4): one active device at a time.** Owner constraints: deltas only, never a full re-broadcast; chain efficiency **tested**, with a cloud/relay fallback on evidence; leave room for a future agents track. P7 opens with R4-1 (how "active" is recorded/switched), R4-2 (chain as sync channel: poll vs push, lag, per-delta cost), R4-3 (agents as writers, BRC-181) |
| **RQ-2** | ✅ **Tiered rule (decision 7)** — see T1's block; restore **never fails outright** on unidentified coins, it reports them |
| 👤 Import/export | "Close to a track of its own" — may move **earlier**; T3's G3 opens with a fresh read of BRC-38/39/40 + newer, TS **and** Go toolbox schemas, SDK, and any **test vectors** |
| New columns to carry | Decision 11 adds `requesting_domain` to `derived_key_cache` — see §6a (cross-track integration re-check) |
| Q5 key shares · Q6 format-first | Per this doc's recommendations (approved) |

---

## 1. Goal

Every Hodos wallet keeps an on-chain backup that a fresh install can restore from the recovery phrase
alone — completely, without inventing or losing coins, without growing out of control, without two
devices fighting over the same coin — and a user can move their wallet into or out of Hodos through the
ecosystem's standard file format.

---

## 2. Telescope — the current state of the world

### 2.1 What I re-fetched

| Source | Version fetched | What changed vs our repo copies |
|---|---|---|
| **BRCs registry** `bitcoin-sv/BRCs` (redirects to `bsv-blockchain/BRCs`) | HEAD `2b959b1`, 2026-09-21; open PRs listed 2026-09-25 | ⚠️ See §2.2 — **BRC-38/39/40 did not change**, but **four merged BRCs our track docs never cite** now speak to backup, recovery, receive-address scanning and stuck `noSend` transactions |
| **`wallet-toolbox` (TypeScript)**, in `bsv-blockchain/ts-stack` | `packages/wallet/wallet-toolbox` **v2.14.2**, ts-stack HEAD `4da08e4`, 2026-09-24 | Our research pinned **v2.10.2 / `8b074a0`**. The BRC-38/39 import/export code (`src/storage/portable/index.ts`) was touched once since, in the 2026-09-22 security merge `b3155fa2` — **formatting and import-path changes only**, no behaviour change (I read the diff). BRC-177 (`noSend` expiry) is now implemented in storage |
| **`go-wallet-toolbox`** | **v0.187.1**, released 2026-09-25 | Still **no BRC-38/39 code** — the repo tree has no portable/export/import files [web: tree listing] |
| **`go-private-backup-cache`** (deggen's off-chain delta rail) | `ca2136e`, 2026-08-21 | **Unchanged** since our research pinned it. Still no licence |
| **1Sat SDK** `b-open-io/1sat-sdk` | HEAD `52cfe69`, 2026-09-20 (`@1sat/wallet` 0.0.112, `@1sat/actions` 0.0.225) | No BRC-38/39. Backup = ZIP of wallet-toolbox sync chunks. Address sync refuses to ingest what it cannot classify (§3.2 / RQ-2) |
| **HandCash Desktop** `HandCash/HANDCASH-DESKTOP` | v1.3.334, HEAD `a62e751`, 2026-09-25 (no licence — read only) | Ships `.brc39` history backups **encrypted with a secret derived from the root key**, BRC-140 key shares, a crash-loop breaker for backups, and — **2026-09-23** — *"Refuse phantom restores: prove a coin unspent before re-enabling it"* (`e8faed0`) |
| **Yours Wallet** `yours-org/yours-wallet` | HEAD `e77a7ed`, 2026-09-16 | Still its own backup format (plus a new USB backup); no BRC-39 [web: tree + code search] |
| ⭐ **A Rust port of wallet-toolbox exists** — `b1narydt/rust-wallet-toolbox` (crate `bsv-wallet-toolbox` 0.11.0) and `b1narydt/bsv-rust-sdk` (crate `bsv-sdk` 0.8.0) | both updated 2026-08-25 / 2026-09-24 | **New to this project.** Community port ("still under development"), crates owned by `Quaakee`, `mitch-burcham`. Has `src/storage/portable/{brc39,export,import,validate}.rs` and a BRC-140 `key_shares.rs`. Licences: toolbox = MIT-style text with a BSVA copyright line; SDK = **Open BSV License v5**. ⛔ Read-only until licence is confirmed (root `CLAUDE.md` rule 5 licence discipline). ⚠️ Root `CLAUDE.md` says *"No Rust implementation — port patterns, never code"*; that sentence is now stale as a fact, though "port patterns, never code" still stands as policy |

### 2.2 The BRCs — what the owner's "recently updated" turns out to be

⚠️ **Honest finding: BRC-38, 39 and 40 have not changed since 2026-04-24** [web: `git log` on
`outpoints/0038.md`, `0039.md`, `0040.md` — last content commit `fb14783` 2026-04-24; 0038 had a link fix
`42ceeb2` 2026-08-10]. The 2026-09-17/18 merge wave touched transactions, overlays, BRC-179 (capability
manifest) and editorial fixes — not the wallet data format. **What did change, and what our track never
read, is this:**

| BRC | Merged | What it says | Why it matters to T3 |
|---|---|---|---|
| **BRC-157** Entropy-Rooted Backup and Recovery (Deggen) | 2026-08-08 (PR #208) | Back up **entropy**, not a key or a phrase; mnemonic and BRC-140 shares become interchangeable. ⭐ Normatively fixes the **root key = BIP-32 `m/0'/0'`** from the BIP-39 seed (empty passphrase) | 🚨 **Hodos's root key is BIP-32 `m`**, not `m/0'/0'` [code: `database/helpers.rs :: get_master_private_key_from_db` → `XPrv::new(&seed)`]. See §2.3 |
| **BRC-154** Pluggable Backup Services for BRC-140 Share Vaults (HandCash) | 2026-08-06 (PR #194) | HTTP profile for vaults that each hold ≤ 1 share; explicitly **out of scope: BRC-38/39/40 wallet data** | Feeds the BRC-140 ticket, not the data backup |
| **BRC-155** Pull-Based Receive Discovery (BSVanon) | 2026-08-06 (PR #197) | A wallet keeps a list of **handed-out** addresses + a counter `nextIndex`; on a fresh device it re-derives the **full range `0..nextIndex-1`** (not a gap scan); the counter is *"the single integer worth syncing"*; multi-device allocation MUST pick single-writer, device-scoped index ranges, or compare-and-set | ⭐ Is the published standard for exactly T1's old-address-scan ticket and for what the backup must carry. Also names the multi-device address-index collision T3 inherits |
| **BRC-177** Wallet-Enforced Expiry for `noSend` Actions (Ty Everett) | 2026-09-01 (PR #239) | A `noSend` action is funded from one dedicated anchor output; past a deadline the wallet reclaims the anchor, which invalidates the withheld transaction. **Expiry is a deadline for broadcast, not confirmation.** "It MUST NOT report successful reclamation merely because the protected txid is unknown" | ⭐ Ecosystem prior art for the stuck-`noSend` class that made our payload 431 KB, and for plan D14's rule that no oracle can prove a counterparty-held transaction dead |

Open, not merged, noted only: BRC-185 wallet substrate discovery (PR #272 — ports, not backup);
BRC-230 index expansion packs (PR #240).

### 2.3 ⭐ The interoperability finding the plan does not have: three root-key conventions

To "move into or out of Hodos cleanly", two wallets must derive the **same identity key** from the same
recovery phrase — the BRC-38 document is keyed to `user.identityKey`, and our on-chain backup address is
derived from the root key. They do not:

| Convention | Root private key from the phrase | Who uses it |
|---|---|---|
| **BIP-32 master** | `m` | **Hodos** [code, above]; 1Sat desktop mnemonic import [web: `wallet-desktop/src/bun/backup-import.ts :: importFromMnemonic`, `52cfe69`]; HandCash's fallback "legacy-hd" scheme |
| **BRC-75** | `SHA-256(seed)` | **HandCash default** for new wallets [web: `src/wallet/vault.ts :: rootKeyFromMnemonicBrc75`, `a62e751`] |
| **BRC-157** | `m/0'/0'` | The newest BRC; the SDK reference implementation |

⇒ A Hodos phrase typed into a BRC-157 wallet produces a **different wallet**. HandCash tries both BRC-75 and
legacy-HD and keeps whichever holds coins — but only to **sweep** coins, not to import data
[web: `src/wallet/phraseSweep.ts`]. **No standard backup format fixes this on its own**; the root
convention is upstream of every format decision. (This also answers plan §6.3 item 8, "how does a
foreign import get its seed at all": through the root convention, or not at all.)

### 2.4 What the reference importer actually enforces [web: `storage/portable/index.ts`, v2.14.2]

- `importBRC38` **rejects** any output whose `transactionId` does not reference an exported transaction,
  and any `spentBy` that does not (`validateOutputs` → `requireRef`).
- It **rejects JSON `null`** (HandCash had to write a repair for it: `src/wallet/brc38HistoryRepair.ts`,
  *"one such note poisons every future backup"*).
- `restore` mode requires an **empty** target; `merge` mode does find-or-insert by identity key.

🚨 **Consequence for plan D1** ("strict BRC-38 inside an envelope, stripped profile"): our shipped
strips **set `output.transaction_id = None`** when the transaction was stripped or is a backup
transaction [code: `backup.rs :: compress_for_onchain`, "Null out orphan FK references"], and the
collection excludes all `backup-%` transactions while keeping their change outputs [code: `backup.rs ::
collect_payload`, transactions query vs outputs query]. **The reference importer would refuse that
document.** A *stripped* document cannot also be a *strict, importable* BRC-38 document.

### 2.5 How the ecosystem handles two devices on one wallet

| Who | Answer | Source |
|---|---|---|
| **wallet-toolbox** | ⭐ **Single writer.** Exactly one *active* storage; the rest are backups that receive pushes. If two stores disagree about which is active, **the active is disabled** (`isActiveEnabled` false) until the user runs `setActive`, which merges the conflicting stores. Multi-device in practice = several devices talking to **one** remote storage server | [web: `storage/WalletStorageManager.ts :: isActiveEnabled`, `setActive`, v2.14.2] |
| **BRC-155** | For address allocation: single writer, **device-scoped index ranges**, or compare-and-set | [web: `wallet/0155.md` §Concurrency] |
| **HandCash** | Coins are hidden, never deleted; an unknown spender → **quarantine**; release a coin only on **affirmative** evidence (a node rejecting a spend as already spent); *"prove a coin unspent before re-enabling it"* | [web: `src/wallet/staleOutputRelease.ts`, `utxoLifecycle.ts`, `a62e751`] |
| **go-private-backup-cache** | Per-device append logs — **partitions around** the multi-writer problem; merge left unspecified | [doc: C3; unchanged at `ca2136e`] |

⇒ **Nobody in the ecosystem has shown two independent writers spending one coin set conflict-free.**
The working answers are *one writer at a time* or *partition so the writers never share*. Our plan's
D15 (detect the race, heal afterwards) is the third option, and by its own words it is not
conflict-free (`IMPLEMENTATION_PLAN.md` G3, D15 "the backup chain is not a spend serializer").

---

## 3. Kaleidoscope — do we already have this?

### 3.1 Reuse — code that exists and should be extended, not duplicated [code]

| Need | Existing code | Note |
|---|---|---|
| Payload collection, compression, encryption | `backup.rs :: collect_payload`, `compress_for_onchain`, `compress_payload`, `encrypt_compressed`, `derive_onchain_backup_key`, `serialize_for_onchain`, `deserialize_from_onchain` | Every format change lands here; G11 legacy-decode path must be kept |
| Import into a DB | `backup.rs :: import_to_db_with_ids`, `import_entities` | Already one DB transaction; the pre-import cleanup is outside it (BS-H4) |
| Write path | `handlers.rs :: do_onchain_backup`, `rollback_backup`, `adopt_onchain_backup` | Intent record (D7) extends this, not a parallel builder |
| Restore path | `handlers.rs :: wallet_recover_onchain`, `fetch_onchain_backup`, `refetch_stripped_data`, `reconcile_backup_tx` | RQ-2's surface and the restored-wallet marker hang off these |
| Verify endpoint | `POST /wallet/backup/onchain/verify` (`handlers.rs`) | Counts only; extend rather than add a second verifier |
| Spend lock | `AppState.create_action_lock`, `utxo_selection_lock` (`main.rs`) | BS-C1: `do_onchain_backup` takes `utxo_selection_lock` only in the stale-pair sweep and the failure reconcile, not around funding selection/reservation; `let _ = output_repo.mark_multiple_spent(...)` still discards its result [code, confirmed] |
| Spent-status oracle | `reconcile.rs :: check_outpoint_spent` | Plan R1-01: inert for plain P2PKH markers. Reuse as corroboration only |
| Double-spend verification | `TaskVerifyDoubleSpend` (`monitor/mod.rs`, `arc_status.rs`) | The plan's loser-state defence |
| Seed scans | `recovery.rs :: recover_wallet_from_mnemonic` (BIP-32 only), `handlers.rs :: wallet_rescan` | T1 owns the BIP-32 + BRC-42 self scan; T3 calls it after restore |
| File export/import | `backup.rs :: encrypt_backup` / `decrypt_backup` / `export_to_json`; export/import handlers in `handlers.rs` | Private `.hodos-wallet` format; **includes the mnemonic** (A3 §7.4). The `.brc39` exporter should be a sibling that reuses `collect_payload`, not a second collector |
| Settings for baseline | `settings.backup_hash`, `last_backup_at` | Per-device; must never travel (plan D1) |
| `sync_states` table | exists, 0 rows, no writer [doc: A1] | Plan D9: carried, not activated |

### 3.2 Shapes we would otherwise duplicate

- **A second "unknown output" state.** T1's classification seam will create a held-not-spendable state
  (`track-2-1sat-ordinals/SCOPE.md` §7: *"T1's `Unknown` needs a representation the ecosystem does not
  have"*). ⚠️ Correction to that line: HandCash does have one — **quarantine** (`spendable:false,
  spentBy:null` + a diagnostic) [web]. RQ-2's answer must **reuse T1's state**, never invent a
  restore-only one.
- **A second address-index counter.** `wallets.current_index` and `MAX(addresses.index)` already disagree
  (`TICKET_two_next_address_index_sources_can_reuse_addresses.md`). BRC-155's `nextIndex` is the same
  concept. The backup must carry **one** high-water mark, whichever T1 settles on.
- **A second intent/write-ahead record.** T1's reservation-ownership ticket (`spent_by` owned by a
  transaction row) and plan D7's intent record both describe "who holds these inputs". One mechanism.
- **A crash-loop breaker.** Plan G5/BS-M1 backoff = HandCash `backupWatchdog.ts` ("a durable 'attempt
  open' marker written before the heavy work") [web]. Same shape as the D7 intent record; one table can
  carry both.
- **`noSend` lifecycle.** Plan D14 `abandoned-unbroadcast` ≈ BRC-177's reclaim model. Adopt BRC-177's
  vocabulary and rules rather than coining our own.

---

## 4. Ticket review

The track register holds **one** ticket; the folder also carries three research documents that act as
open finding registers, plus one cross-track ticket.

| Item | Still true? | Disposition | Why |
|---|---|---|---|
| **`TICKET_brc140_key_shares_vs_bip39.md`** — split the recovery secret into shares, any N of M recover | ✅ No defect, still a choice. **Its research questions are now mostly answered:** Q1 (a Rust implementation?) — yes, community `bsv-rust-sdk :: primitives/key_shares.rs`, Open BSV v5 licence, unconfirmed maturity [web]; Q2 (seed or key?) — BRC-140 splits a key; **BRC-157 splits entropy**, which makes shares and the phrase interchangeable [web]; Q5 (does any wallet ship it?) — **HandCash**, shares of its root key, plus BRC-154 share vaults [web] | **Item in candidate phase P5 (decision only), not a build.** Recommend keeping the ticket's option **B** (decide in beta.5, build no earlier than beta.6) | ⚠️ New blocker: BRC-157's root is `m/0'/0'`; ours is `m`. Shares that round-trip our phrase work for Hodos, but another BRC-157 wallet given them derives a **different** wallet. Adopting shares is therefore downstream of the root-convention question (§9 Q1) |
| `research/ONCHAIN_BACKUP_REVIEW.md` — the `BS-*` finding register (July 2026) | ✅ Mostly live [doc: overview §2.8; spot-checked BS-C1 myself, code] | **Items in P2** (BS-C1, C2, H2, H4, M1, M4, M5, M8, M9, L1–L4, SYNC-1/2/4) and **P4** (BS-H1, H3, H5, H6, M2, M7). BS-M3 closed as benign [doc] | Each is a small, known, code-level fix; none needs the new format |
| `research/FIX_B_CRASH_SAFETY_SHUTDOWN_PLAN.md` — crash between broadcast and DB write | ✅ Live [doc: overview §2.8, grep `BackupIntent` = nothing] | **Item in P4** (plan D7) | Resolved by design, not built |
| `research/FOLLOWUP_RECORD_BEFORE_BROADCAST_TOKENS.md` — "backup is correct as-is" | ⚠️ **Its own trigger has fired**: it says record-before-broadcast becomes mandatory *"when we add non-self-derivable token outputs (1Sat ordinals…)"* — T2 adds them this release | **Merges into D7 / P4.** Keep its "under-approximation" principle (never delete on absence) — HandCash independently arrived at the same rule [web] | — |
| Cross-edge: `TICKET_rescan_cannot_find_payments_to_generated_addresses.md` (owned by **T1**) | ✅ [code: see §6 Q3] | **T1 owns the scan. T3 owes it four guarantees (§7.3)**, delivered in P2 | — |

---

## 5. Candidate phases

⭐ **Ordering rule used:** front-load the **uncertain**, not the hard (`RELEASE_CYCLE.md` §3.7). Two
questions invalidate everything downstream if answered late — *what does the data actually cost* and
*which conflict model and root convention* — so they come first, as measurement and decision, before
any format is frozen.

### B5-T3-P0 — Measure, and bring the three decisions to the owner *(no shipped code)*

**Objective.** Replace every estimate the design rests on with a number, and put the three
format-shaping decisions (§9 Q1–Q3) in front of the owner with evidence.

- **Efficiency measurements** (design in §5.1).
- **Re-verify two root causes the plan relies on**, because code moved since 2026-08-24: (a) the stuck
  `noSend` requests — `task_check_for_proofs.rs` now selects `transactions.status = 'nosend'` [code], but
  whether `proven_tx_reqs` rows still sit at `nosend` forever is **not re-verified**; (b) with T1, the
  BRC-42 phantom-coin cause (§7.3).
- **Endpoint contracts** (plan G12/D12) for the one call D6 depends on and nobody has specified:
  WhatsOnChain address **history** (depth, ordering, pagination, unconfirmed visibility) — recorded
  fixtures, no code depending on them yet.
- **Interop probe:** export a real HandCash `.brc39` and read it with the toolbox reference; list every
  field and root-key consequence. No import into Hodos yet.

**Tickets:** none closed; feeds the BRC-140 decision. **Unknowns:** all of it — that is the point.
**Negative control:** the measurement tool is run on a DB copy with a **known injected** 100 KB raw
transaction; it must attribute ≥ 95 KB to that row. A tool that cannot see a planted cause cannot be
trusted to find a real one.
**Spike rule (answers overview §8(d)):** a throw-away delta producer and envelope prototype are allowed,
**on a branch named `spike/b5-t3-p0` that is never merged**, run only against DB **copies** and the mock
chain. ⛔ **It is deleted the day P0's GitHub issue closes** — i.e. when the owner signs the three §9
decisions. Its only surviving output is numbers in the evidence table. It never broadcasts on mainnet.

#### 5.1 The efficiency measurement, designed

| # | Measurement | How (read-only) | Answers |
|---|---|---|---|
| E1 | **The size and fee of every backup ever broadcast** | The chain already holds it: walk the backup address history (114+ `backup-%` transactions [measured, A1]); record each transaction's byte size and fee (inputs − outputs, parents fetched by txid) | Plan H8a; the real fee rate; ⭐ the **month-1 vs month-6 slope** the April plan asked for and nobody measured — free, from history |
| E2 | **What fills the payload** | On a DB copy: collect the payload, then **leave-one-out** gzip — remove one table (then one column, then one row class such as unproven raw_tx, held-token scripts, `custom_instructions`) and record the compressed-size drop | Overview open Q1 (stuck bytes vs media vs ancestry) with a number per cause |
| E3 | **How big a delta is** | Two DB copies taken hours/days apart (existing file backups in the backups dir, or two copies made during an owner sitting) → spike producer diff → compressed size; repeat for "one payment", "one receive", "3-hour window", "one day" | Overview open Q2; the 0.5× / 20 / 16 KB constants |
| E4 | **Cost of a big backup to *other* payments** | For each backup's change coin: parent size, and the BEEF size of the next spend that used it | The 2026-09-15 PeerPay failure class (`ed51099`) — a size cost that is not the fee |
| E5 | **Restore time and fetch count** | Mock chain first; one live read-only restore of the dev wallet's backup to a scratch data dir | Plan G9; whether a pre-spend poll is affordable (plan H11) |
| E6 | **Token workload** | T2 fixtures: held-ordinal script sizes and `custom_instructions` sizes | ⚠️ T2 found **nobody produces BRC-150 `beefB64`** [T2 `SCOPE.md` §2.2, Q3] — the "ancestry depth" hypothesis has no data source; measure media in held scripts instead |

⭐ Acceptance for "efficient" is then set **from** E1–E3 by the owner, in two numbers: bytes per backup
at steady state, and slope over six months. (The April rule — *"the slope matters more than the size"* —
is the right frame [doc].)

### B5-T3-P1 — Round-trip harness on the shipped format *(plan Phase 1 — survives)*

**Objective.** Seed-only restore → spend is tested in CI before anything changes.
H1 (restore and spend) and H2 (schema drift gate) on today's format; mock chain generated from the P0
contracts. **Unknowns:** none new. **Negative control:** remove one column from the H2 manifest → H2 red
naming that column; strip `derivation_prefix` from the payload → H1 red *at the spend*, not at import.

### B5-T3-P2 — Make today's backup trustworthy *(plan Phase 2 — survives, extended)*

**Objective.** The shipped single-snapshot backup stops losing data, stops looping, fails closed, and
takes the spend lock.
- `BS-C1` lock around selection + reservation; stop discarding `mark_multiple_spent`.
- `BS-C2` indexer error ≠ "no backup"; `BS-H4` atomic import incl. cleanup; `BS-M8` bounds-checked parsing.
- `BS-M1` backoff to a quiet, visible error state (HandCash-style durable attempt marker).
- Missing tables/columns per H2 (`peerpay_received`, V18 child tables, `domain_permissions`,
  `settings`, `outputs.confirmed`).
- `BS-SYNC-1` full sync must not make the backup marker spendable.
- Baseline pinning (plan D13 / R4-3 — a live silent-loss bug).
- Stuck-`noSend` lifecycle, **aligned with BRC-177's rules** (no timer restores inputs; reclaim only on
  chain evidence).
- Hard pre-broadcast **size cap** (fail closed).
- ⭐ **Restore report + restored-wallet marker + one high-water mark** — the RQ-2 surface and T1's
  guarantees (§7.3).
**Tickets:** the P2 list in §4. **Unknowns:** the RQ-2 behaviour (owner decision). **Negative control:**
per item; e.g. kill the WoC mock mid-restore → user must see "indexer unavailable", **never** "no backup";
re-run with the fix reverted → the old wording returns.

### B5-T3-P3 — Keep re-fetchable bytes off chain *(plan Phase 3 — survives, reason corrected)*

Strip held-token locking scripts (media) and rehydrate by outpoint (H13 byte-match). ⚠️ The justification
changes: it is **media in held scripts** [doc: A1 measured 615 KB raw of spendable token scripts], not
BRC-150 ancestry (T2: nobody produces it). Also restore **referential integrity** in the payload
(no null `transaction_id` on carried outputs) — needed for §9 Q2 either way.
**Negative control:** corrupt one rehydrated byte in the mock → H13 red on that outpoint.
**Depends on:** T1's fabricated-locking-script fix (a rehydrate that trusts a fabricated script is no
test), T2 P1 classification.

### B5-T3-P4 — Freeze the format, then chain + crash safety *(plan Phase 4 — survives, with a gate)*

Token header, intent record (Fix B), three-valued broadcast outcome, history-based discovery with
decrypt-before-trust (plan D6/D7). ⛔ **Format-freeze gate:** the envelope and header are frozen and
signed **before the first mainnet broadcast of a new token version** — from that moment every such token
must decrypt forever (plan G11), with or without a published BRC. **Unknowns:** the address-history
contract (from P0). **Negative control:** H5 crash matrix with the intent record disabled → the
2026-04-11 double-spend shape reappears in the mock.

### B5-T3-P5 — Portability: standard files in and out *(plan Phase 8 item 7 — moved earlier, changed)*

**Objective.** A user can export a strict `.brc38.json` / `.brc39` file and import one, and is told
before import what will and will not carry over.
⭐ **Changed:** the interop file is generated from the **full local DB** (strict BRC-38, importable by the
reference), **not** from the stripped on-chain payload (§2.4). *Interop lives in the file; efficiency
lives on chain.* Root-key convention per §9 Q1. BRC-140 decision recorded here (no build). T8–T11 from
the README kept. **Unknowns:** HandCash's importer against our identity; whether "merge" import is safe
with our permission tables. **Negative control:** import a file whose identity key our phrase does not
control → must refuse **before** writing a row.

### B5-T3-P6 — Deltas *(plan Phase 5 — survives only if P0's numbers say so)*

Row-level changes as the sync log (plan D2, D13, D14). ⚠️ The README records deltas as already decided
by the owner (*"no-brainer"*); what P0 changes is only the **constants** and whether padding (plan
Phase 6) is worth its bytes this release. **Negative control:** H3 property test with delete-records
disabled → replay must diverge from the source DB.

### B5-T3-P7 — Two devices, one wallet *(plan Phase 7 — changes: the mechanism is an owner decision)*

**Objective.** Two devices on one phrase never spend the same coin, per the owner's bar. The mechanism
is §9 Q3. **Unknowns:** the largest in the track. **Negative control:** the two-device mock with the
chosen mechanism switched off must produce a same-coin double-spend within N rounds (proves the test can
see the race it claims to prevent).

### B5-T3-P8 — Publish the BRC *(plan Appendix A — survives)*

Written from frozen, tested code; renumbered (never claims 38/39/40, plan D8).

### 5.9 What survives, changes or is invalidated in `IMPLEMENTATION_PLAN.md`

| Plan item | Verdict | Why |
|---|---|---|
| Phases 1, 2, 3, 4, 6 (padding), 8 (BRC) | **Survive** (P1, P2, P3, P4, P6, P8) | Nothing fetched contradicts them |
| Phase 5 deltas | **Survives, constants provisional** | Delta sizes still unmeasured → P0 E3 |
| Phase 7 multi-device | ⚠️ **Changes** | D15 is detection + heal; the ecosystem's working answers are single-writer or partition (§2.5) — owner chooses (§9 Q3) |
| Phase 8 cross-wallet | ⚠️ **Changes and moves earlier** | Root-key divergence (§2.3) and importer strictness (§2.4) |
| **D1** strict BRC-38 in an envelope, stripped profile | 🔴 **At risk — likely invalidated as written** | The reference importer rejects nulled FKs and unexported transactions; a stripped document is not a strict one (§2.4) |
| D2 delta = BRC-38 row forms + deletes | Survives | BRC-40 still has no deletes (unchanged since April) |
| D3 borrow from backup-cache | Survives | Upstream unchanged at `ca2136e` |
| D4 KDF, D5 address (owner: migration rejected) | Stand | ⚠️ But D5's promise of interop via a conformant derivation for *new* implementations is hollow while root keys differ (§2.3) |
| D6 history discovery | Survives, **blocked on the history contract** (P0) | — |
| D7 intent record | Survives, strengthened | FOLLOWUP's own trigger has fired (§4) |
| D8, D9, D10, D12, D13 | Survive | — |
| **D11** on-chain container is not BRC-39 | ⚠️ **Worth re-opening** | HandCash already uses BRC-39 with a root-key-derived secret (`SHA-256("handcash.brc39.v2" ‖ rootKeyHex)` as the "password") [web: `historyCryptoSecret.ts`]. A standard container is available; the vendor label is not standardised. Owner decision (§9 Q2) |
| D14 retention/no timer deletes | Survives; **adopt BRC-177's vocabulary** | Same rule, now a published standard |
| D15 chain arbitrates, lower `device_id` defers | ⚠️ **Insufficient against the owner's bar** | Its own text: not a spend serializer |
| `README.md` "Why this order" #4 — backup last because BRC-150 rows are *the* token workload | 🔴 **Invalidated** | T2: nobody produces BRC-150 `beefB64`. Measurement does not need to wait for T2's build |

---

## 6. Integration check (`RELEASE_CYCLE.md` §3.3)

**1. What existing invariant could this violate?**

| Invariant | Risk | Guard |
|---|---|---|
| Invariant 2 — **no schema change without asking** | Intent table, device table, restored-wallet marker, high-water mark | Each is an explicit owner ask in its phase contract |
| Invariant 3 — **no crypto/derivation change without asking** | KDF, backup address, root convention (§9 Q1), and possibly moving backup change off the user's receive-address namespace (§7.3) | Owner decisions D4/D5 stand; anything new is asked |
| Working rule 7, trip-wire 1 — **money row in an impossible state** | Restore inventing coins (the BRC-42 phantom), or a marker made spendable (BS-SYNC-1) | RQ-2 matrix; R-RESTORE; T1's phantom negative control |
| Trip-wire 2 — **a verdict where an error is owed** | "No backup found" on an indexer error (BS-C2, live) | P2 |
| Trip-wire 3 — **durable state that outlives the process** | A bad restore writes a DB the next session reads as fact; **a mainnet token is permanent** | Restore report before first spend; format-freeze gate |
| Fail-closed classification (T1, `HARNESS_DELTA.md` §1.1) | Restore is the moment the wallet knows least | RQ-2; R-RESTORE's two opposed checks |
| Service fee | Backups carry none [code]; a refactor that routes backups through `create_action_internal` would add 1,000 sats per backup | State it in P2/P4 contracts |
| Gold pill | Untouched — backups are not dApp payments | — |

**2. What does it touch that we did not write?** ⭐
WhatsOnChain's address **history** and unspent semantics (30 s–5 min lag; plain-P2PKH spent status gives
no signal — plan R1-01); ARC broadcast verdicts (a 200 carrying another txid — the April Bug B); the
mempool (eviction, first-seen) and reorgs; BRC-38/39/40/140/155/157/177 text; **the wallet-toolbox
importer's strictness** (§2.4); HandCash's and 1Sat's root-key conventions and HandCash's vendor BRC-39
label; the `bip39`/`bip32` crates; SQLite transaction semantics. Nothing in CEF, Chromium or Sparkle.

**3. What would we have to un-ship if wrong?**
- ⛔ **Any token broadcast to mainnet** — permanent, and must stay decryptable forever (plan G11).
- ⛔ **A published BRC** — others build on it.
- ⛔ **`.brc39` files users have exported** — they keep them; a wrong root convention or field meaning
  ships inside every file.
- A restored wallet's DB after a bad restore — reversible only by a fresh restore, and only if the user
  notices.
- Code-only changes (locks, backoff, strips, lifecycles) are ordinary reverts.

---

## 6a. Cross-track integration re-check — the two G2 schema changes *(2026-09-27, code reading)*

| Change | Carried by backup / export today? | Consequence for T3 |
|---|---|---|
| **Decision 2a — money-index table** (T1-P3), built from `outputs.change` | The index is **derived**, so neither the on-chain payload nor a BRC-38 file carries it (the TS toolbox has no such table). `BackupOutput` **does** carry `change` (`backup.rs :: BackupOutput`) | Restore **rebuilds** the index from `change`. ⚠️ **Old backups predate the stamp's meaning:** today received payments are written `change=0` (`upsert_received_utxo*`, T1 SCOPE §3.4), so restoring an existing backup and rebuilding from `change` alone would leave real money **unselectable — the balance would appear to drop**. ⇒ T3a's restore path must run every restored output through T1's classifier (decision 7) **before** the rebuild, and the upgrade test set must include **restore of a pre-beta.6 backup**. Negative control: rebuild without the classifier pass ⇒ a restored plain payment is not selectable |
| **Decision 11 — `requesting_domain` on `derived_key_cache`** (T5-P3 part 2) | **Not carried at all** — `derived_key_cache` is absent from `BackupPayload` (`backup.rs`) | A restored wallet starts the cross-site detector with **no history** — acceptable for a warn-and-log detector (it re-learns); stated in the restore report wording, not hidden. If the detector ever **refuses**, this must be revisited |
| ⚠️ **Found in the check:** `derived_key_cache.derived_pubkey` is `UNIQUE` and written with `INSERT OR REPLACE` (`migrations.rs`, `handlers.rs` ~`:459`) | — | Two sites given the same `(invoice, counterparty)` produce the **same** `derived_pubkey`, so the second write **replaces** the first — adding a `requesting_domain` column to that row would **overwrite the very evidence part 2 needs**. ⇒ T5-P3's contract: record requesters in a **child table** (FK + CASCADE, the `cert_field_permissions` pattern root `CLAUDE.md` names) or change the uniqueness — and keep the PushDrop-signing lookup (`AppState.derived_key_cache`, pubkey → params) working. Owner approved *a* schema change; the exact shape is decided in the contract |

## 7. Feasibility inputs

### 7.1 Owner-hours (N) — human-bound rows only

| Row | Why human | Estimate |
|---|---|---|
| P0 decision session: §9 Q1–Q3 + RQ-2 matrix + efficiency acceptance numbers | owner decisions, not verification | ~2.0 h |
| P0 interop probe: export a real HandCash `.brc39` | a second wallet, owner's account | ~0.5 h |
| P2 restore sitting: seed-only restore of a funded scratch wallet on a clean profile, read the restore report, spend one coin (plan H18) | real money, visual judgement of the report | ~1.0 h |
| P5 interop sitting: Hodos → HandCash and HandCash → Hodos, round trip with the capability diff shown | two wallets, real data, UI wording | ~1.5 h |
| P7 two-device sitting: two machines (or profiles) on one phrase, race a spend, confirm the chosen mechanism holds | real money, two machines | ~2.0 h |
| Per-release live restore smoke (H18) at release | real money | ~0.5 h |

≈ **7.5 owner-hours** for the full track. Without P7: ~5.5 h.
⚠️ The P2 restore sitting **cannot be batched to the end**: later phases read restored state
(`RELEASE_CYCLE.md` §3.8 — same test as the poisoning rule).

### 7.2 Unknowns (K)

**K = 5 phases with a genuine unknown:** P0 (what the numbers are), P2 (RQ-2 behaviour), P4 (address-
history contract), P5 (root convention + importer strictness in practice), P7 (conflict mechanism).
P1, P3, P6, P8 are hard-but-understood.

### 7.3 Cross-track dependencies

| Edge | What T3 needs / owes | With |
|---|---|---|
| **T1 classification seam (RQ-1)** | Every restored or scanned output goes through **T1's** classifier; RQ-2's "held" state **is** T1's `Unknown` | T1 |
| **T1 old-address scan** (`TICKET_rescan_cannot_find_payments_to_generated_addresses.md`) | ⭐ **T3 owes four guarantees:** (1) **one high-water mark** travels in the backup — `max(wallets.current_index, MAX(addresses.index))` until T1 unifies them (BRC-155: *"the single integer worth syncing"*); (2) restore writes a **restored-wallet marker** with that index, so indexes ≤ it get a derivation scan once; (3) every carried output keeps its **outpoint and derivation** and its transaction link — today the strip nulls `transaction_id` for outputs whose transaction was stripped or is a backup [code], which weakens outpoint-level de-duplication; (4) **backup change must be identifiable as already-known by outpoint.** [code] Backup change is paid to `2-receive address-{MAX(addresses.index)}` — **reusing the most recent address handed to the user** — and recorded with prefix `2-receive address`, suffix `{i}` (`handlers.rs :: do_onchain_backup`). A BRC-42 scan of that index therefore finds backup change; that is the likely seat of the phantom-coin cause (**hypothesis, unverified** — T1 reproduces it). Moving backup change to its own derivation would also end the address reuse, but it is a derivation change ⇒ owner ask (invariant 3) | T1 |
| T1 fabricated locking script ticket | Backup carries spendable outputs' scripts; P3's rehydrate must read real scripts | T1 (blocking P3) |
| T1 reservation ownership (`spent_by`) | D7's intent record should use the same ownership model | T1 |
| T1 20-per-address sync cap | Restore's refetch must not stop at 20 | T1 |
| T1 auto-unlock accepts another wallet's phrase | Restore must check the phrase controls the wallet it restores | T1 |
| **T2 token rows** | Baskets `1sat`/`bsv21`; `custom_instructions` ≤ 1,000 B (reference client cap); **no BRC-150 `beefB64` produced** — T3's token workload is held-script media + CI | T2 `SCOPE.md` |
| Multi-device address allocation | Two devices handing out addresses collide on the index (BRC-155) | T1 + P7 |

### 7.4 One track or two? — with numbers

| | T3a — **Backup you can trust** (single device) | T3b — **Sync and portability** |
|---|---|---|
| Phases | P0, P1, P2, P3, P4 | P5, P6, P7, P8 |
| `BS-*` findings closed | ~20 of ~24 open | ~4 (multi-writer) |
| Harness tests (plan H-ids) | H1, H2, H5, H6, H7, H10, H13–H18 (12) | H3, H4, H9, H11, H12, H19 + T8–T11 (10) |
| Unknowns (K) | 3 | 2 — but the two largest |
| Owner-hours | ~3.5 h | ~4.0 h |
| Irreversible outputs | first new-format mainnet token (P4) | `.brc39` files, the BRC, the conflict protocol |

**Recommendation: two tracks, both in beta.5's fixed scope.** They have different risk shapes: T3a is
mostly known fixes to a live system that users depend on today, and it must not wait behind T3b's
research. T3b holds the two decisions that can invalidate designs (conflict model, root convention). One
track hides that; two tracks let G5.5 feasibility see exactly where the uncertainty sits. ⚠️ This is
a presentation choice, not a cut: the owner fixed backup **and sync** as in scope (planning note D3).

---

## 8. Prior-art rows for `development-docs/PRIOR_ART.md`

| Date | Question | Source(s) read | What we learned | Verdict | Landed in |
|---|---|---|---|---|---|
| 2026-09-25 | Did the wallet data-format BRCs change around 2026-09-18? | `BRCs` HEAD `2b959b1`: `git log` on `outpoints/0038/0039/0040.md`; PR list | No — last content change 2026-04-24. The relevant new texts are **BRC-154, 155, 157, 177**, never cited by the track | 🟢 paid off — found four missed standards | §2.2 |
| 2026-09-25 | Can a phrase move between wallets? | BRC-75, BRC-157; HandCash `vault.ts`, `phraseSweep.ts` (`a62e751`); 1Sat `backup-import.ts` (`52cfe69`); our `database/helpers.rs` | Three root conventions: `m` (Hodos, 1Sat), `SHA-256(seed)` (HandCash, BRC-75), `m/0'/0'` (BRC-157) | 🟢 paid off — interop blocker found before design | §2.3, §9 Q1 |
| 2026-09-25 | Would the reference importer accept our stripped payload? | `wallet-toolbox` v2.14.2 `storage/portable/index.ts` (`validateOutputs`, `requireRef`, restore/merge) | No — it rejects nulled FKs and unexported transactions, and JSON null | 🟢 paid off — D1 at risk | §2.4 |
| 2026-09-25 | How does the ecosystem keep two devices from spending one coin? | `WalletStorageManager.ts :: isActiveEnabled/setActive`; BRC-155 §Concurrency; HandCash `staleOutputRelease.ts` | Single active writer (conflicting actives disable spending) or partition; no one does two free writers | 🟢 paid off — reframes plan Phase 7 | §2.5, §9 Q3 |
| 2026-09-25 | What do others do with an output they cannot identify? (RQ-2) | HandCash `utxoLifecycle.ts` (quarantine), `staleOutputRelease.ts` (`e8faed0`); 1Sat `actions/src/sync/index.ts` (holds the cursor rather than ingest without origin); toolbox (DB is the authority) | Three distinct answers exist in shipping code | 🟢 paid off — fills the matrix | §9 RQ-2 |
| 2026-09-25 | Stuck `noSend` lifecycle — is there a standard? | BRC-177; toolbox `WalletStorageManager` BRC-177 guards | Yes; matches plan D14's rule that absence is not proof | 🟢 paid off — adopt vocabulary | §3.2, P2 |
| 2026-09-25 | Is there a Rust wallet-toolbox / BRC-140? | crates.io `bsv-wallet-toolbox` 0.11.0, `bsv-sdk` 0.8.0; `b1narydt` repos | Yes, community, licence to confirm (MIT-style / Open BSV v5) | 🟡 read-only until licence confirmed | §2.1, §4 |
| 2026-09-25 | Crash-looping backups — prior art? | HandCash `backupWatchdog.ts` | Durable "attempt open" marker before heavy work, growing backoff, cleared by upgrade | 🟢 same shape as plan G5/D7 | §3.2 |

---

## 9. Open questions for the owner — each with a recommendation

### Q1 — Which root key does a Hodos phrase produce, for portability?
Today: BIP-32 `m`. Options: (a) keep `m` and document it; (b) move new wallets to BRC-157 `m/0'/0'`; (c)
keep `m` and, on import of a foreign phrase, try all three conventions and ask the user (HandCash's
approach, but for data, not only sweeping).
**Recommendation: (a) + (c).** Moving existing wallets is a derivation change that moves every address
(the same R3-1 money-loss surface the owner already rejected in D5). Recognising foreign conventions on
import costs little and makes "move in" work.

### Q2 — Should the portable file and the on-chain payload be the same format?
**Recommendation: no.** The file is strict BRC-38/39 from the full DB (importable by the reference);
the chain carries our stripped, efficient profile and never claims to be BRC-38. Re-open D11 only for
the *container*: using BRC-39 bytes with a phrase-derived secret on chain (HandCash's pattern) would
be more recognisable, at the cost of Argon2id work on every restore — measure in P0 before deciding.

### Q3 — What does "conflict-free" mean for two devices? *(the largest open question)*
| Option | Good | Bad | Ugly |
|---|---|---|---|
| **A. Detect and heal** (plan D15) | Both devices spend freely | A same-coin race is possible; one payment fails after the fact | A user sees a payment "sent" on one device fail minutes later |
| **B. One active device at a time** (toolbox model) — the other is read-only until the user switches | Conflict-free by construction when used as intended; matches the ecosystem | Switching is a deliberate act; a second device cannot pay until it takes over | Both devices open at the moment of a switch, within the indexer's 30 s–5 min lag, can still race — the window shrinks, it does not vanish |
| **C. Per-device coin partitions** (BRC-155 device-scoped ranges, applied to coins) | Conflict-free by construction; both devices spend | Balance fragments across devices; a device can be "out of money" while the wallet is not | A lost device's coins are only reachable by a restore — which works, but the user must know to do it |
| **D. The spend carries the backup chain tip** (every payment spends the tip, so two racing payments conflict on it) | Truly conflict-free, no extra transaction | Every payment carries ~1.5 KB and the 1,546-sat float; changes BRC-100 transactions apps receive | 🚨 **Publicly links every payment the user ever makes into one chain** — a privacy disaster |
**Recommendation: B for beta.5**, with the window measured in P7 and shown honestly; C as the next step
if two simultaneously-spending devices is a real user need. ⛔ Not D.

### RQ-2 — What does restore do with an output it cannot identify? *(owned by this track — for the owner to decide, not a recommendation)*

**Where such outputs come from in Hodos [code]:** a seed scan that finds coins at derived addresses
(`recovery.rs`, and T1's new BRC-42 self scan); `refetch_stripped_data` bringing back a script that is
not plain P2PKH; full sync inserting outputs at wallet addresses (BS-SYNC-1: the backup marker lands
`spendable=1`); a foreign BRC-38 import with baskets or `customInstructions` we do not understand.

| Candidate behaviour | Good case | Bad case | 🚨 Ugly case (user loses something and does not find out for months) | Who does this |
|---|---|---|---|---|
| **1. Import as spendable** (today's effective default for scanned coins) | Plain coins "just work" | An unknown 1-sat output is treated as money | The dust consolidator (daily, automatic) or the next send **burns an ordinal**; nothing tells the user it existed | Hodos today (scan paths) |
| **2. Drop it** (don't import) | Nothing unknown can be spent | The balance is lower than the chain says | The user's ordinal or token is **invisible forever** in Hodos; they assume it was never received | — |
| **3. Hold, visible, not spendable** ("quarantine") | Nothing burned, nothing hidden; the user sees "N items we could not identify" | Plain coins that failed classification (e.g. indexer down) are frozen until reclassified | The user never looks at the held list; real money sits frozen until they do | **HandCash** (`quarantine`) [web]; R-RESTORE's GREEN a + b |
| **4. Block the restore** until everything is identified | No partial state | One unreachable indexer stops the whole restore | The user, told "restore failed", creates a new wallet over the old one (the BS-C2 shape) | **1Sat address sync** holds its cursor rather than ingest an ordinal without its origin — *"a stalled sync is visible where a silent skip is not"* [web] |
| **5. Classify later** — hold (as 3), then promote automatically when the classifier confirms plain money | Plain coins return without user action | Promotion runs on an indexer's word | A wrong "plain money" verdict from a lying or lagging indexer silently re-arms case 1 | Partly HandCash (re-enable only after proving unspent) [web] |
| **6. Ask the user per item** | The user decides | Most users cannot judge a script | Users click "spendable" to make the number go up | — |
| **7. Only restore what the backup declares; the chain is not consulted** | No guessing | Anything received after the last backup is missing until a scan | A late payment is never found without the scan T1 is building | **wallet-toolbox** (the DB is the authority; outside outputs arrive only via `internalizeAction`) [web] |

Notes for the conversation: behaviours 3, 4 and 5 compose (hold, then promote on evidence, with a
visible stall); R-RESTORE's two checks are each other's control whichever is chosen; and every row's
user-facing wording should be settled at the P2 restore sitting, not in code.

### Q4 — One track or two?
**Recommendation: two** (§7.4), both in scope.

### Q5 — BRC-140 / BRC-157 key shares
**Recommendation: decide in P5, build no earlier than beta.6** (the ticket's option B), because shares
are downstream of Q1.

### Q6 — Format-first or code-first?
**Recommendation: format-first for what is irreversible, code-first for everything else.** The
irreversible moment is **the first mainnet broadcast of a new token version** (P4's freeze gate) and the
first `.brc39` file a user exports — not the BRC's publication date. Portable files adopt BRC-38/39 as
published (no invention). The BRC text follows tested code (P8). The only spike is P0's, deleted when P0
closes.
