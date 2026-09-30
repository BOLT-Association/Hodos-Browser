# B5-T3a-P0 — Prove the database is correct, and replace every estimate with a number · PHASE CONTRACT

**Track:** B5-T3a Backup you can trust (single device) · **Tickets:** none closed here; feeds `../../tickets/TICKET_brc140_key_shares_vs_bip39.md` (decided in T3b-P5) and `../../tickets/TICKET_final_mvp_efficiency_leftovers_never_scheduled.md` (the "measure month-1 vs month-6" item) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Fable 5.1), G3 agent for T3a · **Platforms:** both (Rust-only; read-only DB copies + chain reads) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** 2, 2a (the money index is derived from `outputs.change`; T1 lands first), 3 (T3a = P0→P4), 5 (two formats; freeze before first mainnet broadcast), 7 (tiered rule; restore reports unidentified coins), 10 (the method for a read-only chain check on the owner's wallets: `mode=ro`, positive control first), `SCOPE.md` §0 and §6a. Decisions 4 and 6 are **provisional** and are consumed here only as "what to measure for".

> 👤 Owner direction this phase carries (`../../README.md` "Research follow-ups", 2026-09-27): *T3 starts by proving the database is correct — after T1's money-path and schema changes — before any backup-format work; restore must rebuild the new money index correctly.* And: *T3's planning opens with a fresh read of the current BRCs, the toolbox schemas (TS and Go) and any test vectors.* The fresh read is §0 below; it is the reason this contract, not a research doc, carries it.

---

## 0. Fresh read — 2026-09-28 (what T3a depends on; T3b reads this too)

Every claim below was fetched today; commit shas are what the GitHub API returned.

| Source | Fetched | What it says that T3a depends on |
|---|---|---|
| `bsv-blockchain/BRCs` `outpoints/0038.md` / `0039.md` / `0040.md` | HEAD `8f36bdf` (2026-09-28). Last **content** commit on all three: `fb14783` 2026-04-24 (`0038` had a link fix `42ceeb2` 2026-08-10) | **Unchanged since T3 SCOPE's 2026-09-25 read.** BRC-38: `brc:38`, `title`, `formatVersion:1`, `exportedAt` (UTC, ms), `sourceStorage`, exactly one `user`, `tables` with 12 arrays (empty arrays present); optional fields **omitted, never `null`**; binaries base64 with padding; JSON-in-string columns (`history`, `notify`, `syncMap`) exported **decoded**; arrays sorted by PK; importers MAY remap ids, MUST preserve `status`/`isDeleted`. BRC-39: magic `WDAT`, version `0x01`, password protector, inner format `0x26` (=38), Argon2id (`7` iterations, `131072` KiB, `p=1`, 32-byte hash, 32-byte salt), AES-256-GCM. BRC-40: sync of 12 entities by watermark; **no deletes, no versioning**. ⛔ **None of the three carries test vectors** |
| `bsv-blockchain/ts-stack` `main`, `packages/wallet/wallet-toolbox` **v2.14.3**; `src/storage/portable/index.ts` (1,078 lines, last touched `b3155fa` 2026-09-22) | today | The reference importer: `formatVersion !== 1` ⇒ throw; `validate` throws `BRC-38 <path> must omit null values` on any `null`; `requireRef` on `output.transactionId`, `output.spentBy`, `commission.transactionId`, `txLabelMap.transactionId`, `provenTxReq.provenTxId`; `mode:'restore'` throws `BRC-38 restore requires an empty target storage except settings`; `mode:'merge'` remaps ids and sync maps. **Confirms SCOPE §2.4**: our stripped on-chain payload (nulled `transaction_id`, excluded `backup-%` transactions, `backup.rs :: compress_for_onchain` "Null out orphan FK references") cannot be a strict BRC-38 document — decision 5's *two formats* stands |
| same repo, `test/storage/portable.test.ts` (509 lines, 11 tests) | today | ⭐ **The closest thing to a published conformance corpus.** Fixtures are built in code (`validates canonical BRC-38 shape, nulls, base64, JSON fields, and relationships`; `exportBRC39 + importBRC39 round-trip preserves user state`; `merges BRC-38 into non-empty SQLite storage with ID and sync-map remapping`). **No vector files exist anywhere we found** (repo tree, npm, BRC text, web search 2026-09-28). ⇒ T3b-P5 must **generate** its vectors by running the reference exporter, not download them |
| `src/storage/schema/tables/TableOutput.ts` | today | `spendable`, `change`, `basketId`, `spentBy` on the output row; **no separate money-index table** in the TS toolbox — so a BRC-38 file cannot carry decision 2a's index, which confirms §6a: the index is derived and restore rebuilds it |
| `bsv-blockchain/go-wallet-toolbox` `main`, **v0.187.1** (2026-09-25); `pkg/internal/storage/database/models/user_utxo.go`, `output.go` | today | `UserUTXO { UserID, OutputID, UTXOStatus, BasketName, Satoshis, ReservedByID }` — the shape decision 2a adopts in T1-P3; `Output` has `Spendable`, `Change`, `BasketName`. **Still no portable / BRC-38 code in Go** (tree grep: none) |
| BRC-157 `key-derivation/0157.md` (`6f24652`, 2026-08-08) | today | Root key = `m/0'/0'` from the BIP-39 seed with an **empty passphrase**; the entropy key MUST NOT be the root. Unchanged. Hodos's root is `m` (`database/helpers.rs :: get_master_private_key_from_db` → `XPrv::new(&seed)`, re-read today) — decision 6's tiered import stands, T3b-P5 |
| BRC-155 `wallet/0155.md` (`161c365`, 2026-08-05) | today | `nextIndex` is *"the single integer worth syncing"*; `recover` rescans the **full** range `0..N−1`; multi-writer allocation MUST use single writer, device-scoped ranges, or compare-and-set. ⇒ the one high-water mark T3a owes T1 (P2.3) |
| BRC-177 `wallet/0177.md` (`558603b`, 2026-08-31) | today | Anchor/reclaim model; the wallet MUST persist label, anchor outpoint, deadline and lifecycle state **before returning**; expiry is a broadcast deadline, not a confirmation one. ⇒ P2.2 adopts its vocabulary for stuck `noSend` rows and never restores inputs on a timer |
| Open BRC PRs updated since 2026-09-27 (titles only) | today | #283 BRC-191 *Identity, Privacy and Recovery on the Metanet* (2026-09-28, **unread** — T3b-P5/P8 should read it before publishing), #278 BRC-188 UMP (a beta.7 intake ticket exists), #272 BRC-185. **None touches 38/39/40** |
| Ecosystem movement on BRC-38/39 since the 2026-09-25 read | today | `p2ppsr/peacock-wallet` PR #30 **merged 2026-09-24**: *"imports retain immutable originals, validate all 13 record categories, restore an isolated copy and require identity/network compatibility before merge or activation"*; `bsv-blockchain/bsv-browser` PR #152 (open): *"import, preview, isolated restore, compatible merge, explicit activation"*; `ts-stack` PR #569 (open, 2026-09-24): bounded resumable sync + canonical proof recovery. ⭐ **Preview → isolated restore → compatibility check → explicit activation** is now the reference importer's shape in two shipping wallets — prior art for T3b-P5 and for decision 6's tiered ask; recorded here so T3b does not re-derive it |

**Consequence for T3a:** nothing in the fresh read changes P0–P4. Two things it settles: (a) there are **no test vectors to import** — the harness generates its own (P1) and T3b-P5 generates its BRC-38 vectors with the reference exporter; (b) the reference importer's strictness is exactly as SCOPE §2.4 recorded, so the on-chain payload must never claim BRC-38 (decision 5) and P3's "restore referential integrity" is for **our** importer's benefit, not for interop.

---

## 1. Goal

Before any backup-format code changes, the owner can see — as numbers, not estimates — what today's backup costs and what fills it, and has proof that the wallet database T3a will back up agrees with the chain and with T1's new money index.

## 2. Done means

- [ ] **E1–E6 measured** (§4) and written into this folder as `MEASUREMENTS.md`, each row a number with its method; the SCOPE's constants (0.5× / 20 / 16 KB; 1–250 sat/KB; "~60% stuck raw bytes") are each replaced or confirmed by a measurement
- [ ] **The owner has set the two acceptance numbers** from E1–E3: bytes per backup at steady state, and slope over six months (SCOPE §5.1)
- [ ] **On both the production and dev wallets (read-only)**: every row the DB calls money agrees with the chain; the derived money index equals its defining query; zero rows in a trip-wire-1 state that no code expects — or each one is written down with its outpoint and a rule-7 escalation
- [ ] The two root causes the plan rests on are **re-verified or refuted** against today's code: (a) stuck `noSend` `proven_tx_reqs`; (b) the BRC-42 phantom mechanism (T1 §5 — T1 reproduces it; P0 only confirms whether the production DB carries one today)
- [ ] The WhatsOnChain **address-history** contract (depth, ordering, pagination, unconfirmed visibility, rate-limit shape) is recorded as fixtures in this folder — no code depends on it yet
- [ ] A real HandCash `.brc39` has been exported by the owner and read with the reference toolbox; its field list and root-key consequence are recorded for T3b-P5
- [ ] ⛔ The spike branch `spike/b5-t3a-p0` (if used) is **deleted** the day this phase's issue closes; its only surviving output is numbers

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| Working rule 7 | A money row in an impossible state is escalated on evidence | This phase **looks for** such rows on the production wallet. The trap is escalating on a DB reading alone — decision 10's method (positive control, then one chain call per row) is mandatory |
| Invariant 2 | No schema change | None here. The "prove the DB" queries read; the spike writes only DB **copies** |
| `R-DUST` / `R-NOSPEND` | no incidental spend | Untouched — nothing here selects or spends. A spike must never broadcast (§2 last item) |
| Production isolation (root `CLAUDE.md` Dev Runbook) | The installed wallet is never contended | Production DB opened `mode=ro` on a **copy**, never the live file; no dev process started against the production port |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. A green result is reported with its red half or not at all.
⛔ Money, schema and crypto rows: the RED is **designed by someone other than the assertion's author** — a second agent (`../../../RELEASE_CYCLE.md` §4.2). Rows P0-A7…A10 are money-state rows and carry the placeholder.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P0-A1` (E1) | Every `backup-%` transaction the production wallet ever broadcast (A1 measured 114→115 rows, 2026-08-22) is walked **on chain**; for each: byte size and fee (inputs − outputs, parents by txid). A per-month table and the month-1→month-6 slope are produced; parents unavailable are **counted**, and the fee numbers marked partial if any | Feed the tool a txid list with one **fabricated** txid ⇒ it must report that row as *unfetchable*, not silently drop it and print a total. Second RED: remove the parent fetch ⇒ every fee reads as `−(outputs)` and the tool must **refuse** to print a fee column, not print negatives | The chain (`tx/{txid}/hex` per backup and per parent), never `transactions.satoshis` (which stores the *estimated* fee — `handlers.rs :: do_onchain_backup` Step 12 writes `estimated_fee` into `satoshis`) | T2 (read-only, network) | ⬜ |
| `P0-A2` (E2) | Leave-one-out gzip on a DB copy: remove one table, one column class, one row class (unproven `raw_tx`; held-token `locking_script`; `custom_instructions`; `proven_tx_reqs.history`) and record the compressed-size drop for each. Answers "stuck bytes vs media vs ancestry" with a number each | ⭐ SCOPE's control: plant one **known 100 KB raw transaction** in a DB copy; the tool must attribute ≥ 95 KB of the drop to that row class. A tool that cannot see a planted cause cannot be trusted with a real one | `backup.rs :: compress_for_onchain` called with the copy's connection and a fixed `reference_timestamp` — the **production** strip rules, not a re-implementation. State the `reference_timestamp` used | T1 (DB copy, no network) | ⬜ |
| `P0-A3` (E3) | Delta size for "one payment", "one receive", "a 3-hour window", "one day", from two DB copies taken apart in time (the dev wallet's file backups in its `backups/` dir, or two copies during a sitting) → the spike producer's diff → compressed bytes. Repeated for ≥ 3 pairs | Diff two **identical** copies ⇒ the producer must emit **0 rows / a fixed-size empty envelope**; a producer that emits volatile columns (`updated_at`, `balance`) shows a non-zero diff on identical state and is not measuring change | The two copies' `schema_version` and row counts recorded beside every number; volatile columns named explicitly in `MEASUREMENTS.md` | T1 | ⬜ — feeds **T3b-P6/P7 (R4-2)** |
| `P0-A4` (E4) | For each backup's change coin: parent size, and the BEEF size of the next spend that used it (the `ed51099` cost class) | On a wallet whose backup change was never spent, the tool reports **"never spent"** for that row, not 0 bytes | `parent_transactions.raw_hex` length ⋈ `outputs.spent_by` → the spending tx's stored BEEF or a rebuilt one; say which | T1 | ⬜ |
| `P0-A5` (E5) | Restore wall-time and fetch count: mock chain first (from P1's fixtures if P1 has landed, else a recorded fixture), then **one** live read-only restore of the dev wallet's backup into a scratch data dir | Point the live restore at a scratch dir that already holds a wallet ⇒ `wallet_recover_onchain` returns `409 Wallet already exists` and **no fetch happens** (its Step 2 check runs before any network) — proves the count is of a real restore, not a refused one | `handlers.rs :: wallet_recover_onchain` + `refetch_stripped_data`'s own `raw_tx_fetched / proofs_fetched / errors` counters in its response JSON; wall time from the log timestamps | T2 (scratch profile, `HODOS_DEV=1`) | ⬜ |
| `P0-A6` (E6) | Token workload: held-ordinal `locking_script` sizes and `custom_instructions` sizes from T2's fixtures (or, before T2-P1 lands, from the production wallet's `master`-prefix token rows — A1 measured 615 KB raw) | Run against a copy with the token rows deleted ⇒ the row reports **0 rows / no data**, not a stale number from a cache | The query and the copy's row count printed with the result. ⚠️ T2 SCOPE: nobody produces BRC-150 `beefB64`, so "ancestry depth" has **no** data source — record that as *not measurable*, not as 0 | T1 | ⬜ |
| `P0-A7` ⭐ *money agrees with the chain* | On a `mode=ro` copy of **each** wallet (production = the verdict; dev = read against its declared test residue, decision 10): every output with `spendable=1` (and, after T1-P5, `change=1`) is **unspent on chain**; every output with `spent_by` set names a transaction that **exists on chain** (`tx/hash/{txid}` 200). Positive control first: a known-spent outpoint and a known-broadcast txid must read as such before the run means anything | **Switches (all on a scratch COPY, never the read-only source copy):** (1) plant a `spendable=1, spent_by NULL` row whose outpoint the chain shows SPENT (take an input of a real `backup-%` tx from the same copy and flip it back) ⇒ the tool must list that outpoint as DISAGREE · (2) plant a `spent_by` pointing at a `transactions` row with a fabricated txid ⇒ `tx/hash` 404 ⇒ DISAGREE · (3) point the tool at a dead host (or inject `IndexerError`) ⇒ every row must read UNVERIFIED and the run INCOMPLETE — a tool printing *N rows agree* is reading `check_tx_exists_on_chain`'s `NotFound/Unknown ⇒ Ok(false)` shape, or `reconcile.rs :: SpentStatus::Unknown`, as a verdict · (4) ⭐ the NoSignal trap: `check_outpoint_spent` returns `Unknown` for every plain P2PKH (both providers `NoSignal`), so a tool built on it can never prove *unspent* — plant a `spendable=1` row whose address the chain shows with an EMPTY `unspent/all` list ⇒ DISAGREE, which only address-unspent membership can produce · **Red for the right reason:** the planted outpoints are named in the DISAGREE list while the same run's positive control (a real spent outpoint, a real txid) still reads correctly — red on the positive control is a harness fault, not a finding · **Residue:** none on the source copies; planted rows live only in the scratch copy and die with it — designed by controls-C (Fable), 2026-09-28 | The chain, one call per row, with the positive control recorded first (decision 10's method). Never `TaskReviewStatus`'s opinion, never the balance figure | T2 (read-only, network) | ⬜ |
| `P0-A8` ⭐ *the index is what it claims* | After T1-P3 lands: the money-index table on each wallet copy equals `SELECT … FROM outputs WHERE spendable=1 AND change=1 AND (basket_id IS NULL OR basket = 'default')` row for row; balance shown = sum of index rows, and the Unknown line = rows held outside it (decision 2's "balance must show Unknown") | **Switches (scratch copy):** (1) insert one extra `money_utxos` row for an outpoint with `change=0` ⇒ the diff must name it as index-only · (2) delete one index row for a real money outpoint ⇒ diff names it as query-only · (3) run the comparison with the shipped predicate's basket clause dropped (a paraphrase) ⇒ held-token rows appear on the query side ⇒ a diff on a CORRECT index — proves a paraphrased predicate is not the instrument · (4) ⭐ vacuity: the identity holds trivially when both sides are empty — assert the copy has ≥ 1 index row AND ≥ 1 row held outside it (a token or `1-wallet-backup` row) before comparing; balance leg: swap `satoshis` between two index rows (sum unchanged) ⇒ the row-for-row diff must still fire — a tool comparing sums passes · **Red for the right reason:** the diff names the planted outpoint; an untouched copy in the same run diffs clean · **Residue:** scratch copy only — designed by controls-C (Fable), 2026-09-28 | The table itself vs the defining query, both on the same copy in one transaction; the exact predicate T1-P3 ships is pasted into `MEASUREMENTS.md` on the day, not paraphrased | T1 | ⬜ — **gated on T1-P3 + T1-P5** |
| `P0-A9` ⭐ *no impossible rows* | Zero rows in each copy matching any trip-wire-1 shape: `spendable=0 AND spent_by IS NULL AND spending_description IS NULL`; `status='nosend'` with a spendable output of its own; `derivation_prefix='1-wallet-backup'` with `spendable=1` **and** `basket_id IS NULL`; a `pending-%` reservation older than 15 min with no live transaction. Each hit is listed by outpoint, checked against the chain, and either explained (declared residue) or escalated (rule 7) | **Switches (scratch copy):** (1) plant exactly one row of each of the four shapes (`spendable=0, spent_by NULL, spending_description NULL`; a `nosend` transaction owning a `spendable=1` output; a `1-wallet-backup` row with `spendable=1 AND basket_id NULL`; a `pending-%` reservation aged 20 min with no transactions row) ⇒ each query returns exactly its planted outpoint — 4 hits, named · (2) plant a `pending-%` reservation aged 10 min ⇒ must NOT hit (the 15-min threshold has teeth — `task_sweep_reservations.rs :: MAX_AGE_SECS`) · (3) ⭐ subject: on the DEV copy the declared M4 residue must be found AND matched by outpoint to `PAYMENT_TEST_BATCH.md` — a dev run reporting 0 hits without citing M4 opened the wrong file; a run listing M4 as a live defect did not read the declaration · **Red for the right reason:** the 4 planted outpoints are in the hit list and every hit carries its chain answer (a hit without a chain answer is INCOMPLETE, not a finding) · **Residue:** scratch copy only; the M4 rows are pre-existing declared residue, not this row's — designed by controls-C (Fable), 2026-09-28 | The rows by outpoint; the chain answer per row. ⚠️ M4 residue on the dev wallet is **expected** and must be cited from `PAYMENT_TEST_BATCH.md`, not rediscovered | T1 + T2 | ⬜ |
| `P0-A10` *root causes re-verified* | (a) `proven_tx_reqs` rows at `nosend` whose txid is **mined**: count on each copy today (A1 found 4 in August; `monitor/task_check_for_proofs.rs` now selects `status IN ('sending','unproven','nosend')` — re-read 2026-09-28 — so the count may have dropped). (b) The phantom: does the production copy hold a spendable row whose outpoint the chain shows spent by a `backup-%` transaction? | **Switches (scratch copy):** (a) plant a `proven_tx_reqs` row at `status='nosend'` whose txid is a real MINED backup txid from the same copy ⇒ count ≥ 1 naming it; plant a second at `nosend` with a txid the chain 404s ⇒ reported *unverifiable*, never counted as stuck-but-mined · (b) take a real input of a mined `backup-%` transaction and flip its row to `spendable=1, spent_by NULL` ⇒ the tool must classify it PHANTOM (spent by a backup); remove the `spent_by ⋈ reference LIKE 'backup-%'` join and the same row degrades to a generic DISAGREE — the wrong label, which must be seen · (c) dead host ⇒ both counts print INCOMPLETE, never 0 · **Red for the right reason:** the planted txid/outpoint appears under the expected label while A7's positive controls read correctly in the same run · **Residue:** scratch copy only. ⚠️ The dev copy's M4 residue may match (b)'s shape — cite it, do not count it — designed by controls-C (Fable), 2026-09-28 | (a) `proven_tx_reqs.status` ⋈ chain `tx/hash`; (b) A7's per-row output filtered to `spent_by`'s reference `LIKE 'backup-%'` | T2 | ⬜ |
| `P0-A11` *history contract* | Recorded fixtures for WoC `address/{addr}/history` (and `/unspent/all`, `/tx/{txid}/hex`, `/tx/hash/{txid}`) on the dev backup address: max depth returned, ordering, pagination behaviour, whether unconfirmed appear, the 429 shape, and **whether spent markers appear in history** (BS-M7's premise) | Replay the fixture through a conformance test with **one field renamed** (`tx_hash` → `txid`) ⇒ the test must go red naming the field — proves the contract test reads the real shape, not `unwrap_or_default()` (which is how `handlers.rs :: do_onchain_backup` Step 5c reads the unspent list today) | The recorded HTTP bodies, dated, with the address; not a paraphrase | T2 (read-only) | ⬜ — **P4 (D6) depends on this** |
| `P0-A12` *interop probe* 👤 | A real HandCash `.brc39` exported by the owner; opened with the reference toolbox `decryptBRC39`/`importBRC38` in `restore` mode into an empty SQLite; the field list, `user.identityKey`, and which of our tables have no BRC-38 home are recorded | Open the same file with a **wrong** password ⇒ `BRC-39 authentication failed` (the toolbox's own string). A file that opens with any password was not the file | The reference code's own error string; the decrypted document's `brc`, `formatVersion`, table row counts | T2 (owner sitting) | ⬜ — consumed by **T3b-P5** |

**Two-sided rows:** `P0-A7` (the DB claims nothing the chain denies) and `P0-A9` (the DB is not silent about what it holds) are each other's control at the wallet level: a DB that "agrees" only because it dropped rows passes A7 and fails A9.

### 4a. Independent control notes (2026-09-28)

*controls-C (Fable), second agent per `../../../RELEASE_CYCLE.md` §4.2. Findings on GREEN/SUBJECT cells I did not rewrite. One line each: row — problem — suggested fix.*

- P0-A7 — SUBJECT says *one call per row* but not which call. `reconcile.rs :: check_outpoint_spent` returns `Unknown` for every plain P2PKH (NoSignal rule), so a tool built on it cannot prove *unspent*, and a tool that reads `Unknown`/`Ok(false)` as *agree* passes with the chain unreachable — name the endpoint per leg (`address/{addr}/unspent/all` membership for `spendable=1`; `tx/hash/{txid}` for `spent_by`) and make UNVERIFIED a third state in the output.
- P0-A9 — the four shapes are SQL; a query with a typo returns 0 and reads as clean. Only the planted-row control gives 0 a meaning — record the planted-copy hit count beside every real-copy result.
- P0-A10(a) — the row counts `proven_tx_reqs.status='nosend'` but cites `task_check_for_proofs.rs :: run`, which selects `transactions.status IN ('sending','unproven','nosend')` — two tables, two status columns, they can disagree. Say which table each number comes from, and count both.

## 5. Blast radius

Nothing shipped changes. The spike (if any) lives on `spike/b5-t3a-p0`, runs against copies and the mock, and is deleted at close.

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/backup.rs :: collect_payload`, `compress_for_onchain`, `compress_payload` | ✅ | E2 calls these on a copy. `compress_payload` only **warns** above `200_000` bytes — no cap (unchanged) |
| `rust-wallet/src/handlers.rs :: do_onchain_backup` Step 12 | ✅ | Writes `estimated_fee` into `transactions.satoshis` — why A1 reads fees from the chain |
| `rust-wallet/src/handlers.rs :: do_onchain_backup` Step 5c | ✅ | Reads WoC `unspent/all` with `unwrap_or_default()`; on network error *"trusting DB"* — the shape A11's RED targets |
| `rust-wallet/src/handlers.rs :: wallet_recover_onchain`, `refetch_stripped_data` | ✅ | Step 2 refuses when a wallet exists (E5's RED); the refetch returns counters incl. `errors` |
| `rust-wallet/src/monitor/task_check_for_proofs.rs` | ✅ | Selects `status IN ('sending','unproven','nosend')` — A10(a) re-verifies whether reqs still stick |
| `rust-wallet/src/database/helpers.rs :: get_master_private_key_from_db` | ✅ | `XPrv::new(&seed)` — root `m` (§0, BRC-157 row) |
| `rust-wallet/src/database/connection.rs :: WalletDatabase::migrate` | ✅ | Gates through `current_version < 25` — **the layer docs say V23/V24; the code is V25** (`migrations.rs :: migrate_v24_to_v25`). Copies must be opened by a build at the same version or read with plain SQLite, never migrated |
| `../research/A1_code_map.md` (numbers) | ✅ read | 114→115 backup rows, 431,476 B live PushDrop script, 615 KB `master`-prefix token scripts — the baselines E1/E2/E6 re-measure |

## 6. Out of scope

Any change to the backup, restore, strip, trigger or monitor code. Any schema change. Deciding the format (P4) or the container (D11). The BRC-140 decision (T3b-P5, per SCOPE Q5). Producing deltas for real (T3b-P6). Reading BRC-191 (#283) — noted for T3b. Fixing anything A7–A10 finds: a finding becomes a **ticket or a rule-7 stop**, not a P0 edit.

## 7. Rollback

Nothing to roll back: no product code, no schema. If the spike branch exists at close, `git branch -D spike/b5-t3a-p0` is the rollback and the deliverable.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| The numbers were measured on the dev wallet only and the production wallet's shape is different (it always has been — 431 KB vs 61 KB) | A1/A2/A7 name **both** wallets; a row run on one only is INCOMPLETE |
| E2's leave-one-out was run with a fresh `reference_timestamp` and the time-based strips moved the answer | A2's SUBJECT pins the timestamp; run twice with the same value ⇒ identical bytes, or the tool is nondeterministic |
| "The DB is correct" was concluded from `TaskReviewStatus` having nothing to say | A7's SUBJECT is the chain; the positive control is run first |
| The phantom (A10 b) exists and the sitting continues "to finish the numbers first" | Rule 7: A10(b) positive ⇒ stop, tell the owner, re-run anything measured since |
| The history endpoint fixture was recorded once, on a quiet day, and P4 builds on a shape that changes under load (429s) | A11 records the 429 shape explicitly; P4's H16 conformance replays it |
| The spike was "too useful to delete" and became the delta producer | §2 last item is a done-criterion; the G6 issue closes only when the branch is gone |
| The M4 residue on the dev wallet was reported as a live defect | A9's SUBJECT cites the declared residue by name |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | The production wallet copy lives here; `mode=ro` copies of both DBs |
| macOS | A2, A5 (mock half) | The tooling is Rust and reads copies, so it runs unchanged; macOS **does not** need to repeat the production reads. 🍎 If the macOS side holds a distinct field wallet, A7/A9 on its copy is a bonus row, not owed |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1.25 h**: approve opening production (read-only copy) + name which wallet is which, 0.25 h · export a HandCash `.brc39` (A12), 0.5 h · set the two acceptance numbers from E1–E3, 0.5 h |
| Human-bound rows | A12 (a second wallet, the owner's account); the acceptance numbers (a decision, not verification). ⚠️ A7–A10 need the owner's **permission** and the wallet mapping, not his hands |
| Unknowns (K) — uncertainty, not difficulty | **Yes, 1 (this phase counts toward K):** what the numbers are, and whether WoC address **history** is usable at all for P4's D6 (depth/pagination unknown until A11) |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | **T1-P3** (money index) + **T1-P5** (classifier, `change=1` migration) | A8 cannot run until both land. ⭐ **A1–A6, A11, A12 have no T1 dependency** and should run first — G5 may schedule P0's measurement half before T1 closes |
| gives | **T3b-P7** (R4-2 chain-as-channel decision) | E1 (real fee rate + slope), E3 (delta bytes), E5 (fetch count/latency). Without these, P7's efficiency question has no evidence |
| gives | **T3b-P5** | A12's HandCash field list and root-key consequence; §0's "no vectors exist — generate them" finding |
| gives | **P4** (D6 discovery) | A11's address-history contract fixtures |
| gives | **P2.2** (size cap, baseline) | E1/E2 numbers the cap and the acceptance bytes are set from |
| gives | **T1** (Q6 review edge) | A10(b): whether the production DB carries a phantom today — evidence for T1-P6's reproduction |
| watch | **T4-P1** | A10(a)'s `nosend` rows include BRC-121 payments; P0 counts them, **never** changes their status (T4 owns it) |

## 12. Open questions for the owner

1. **Which wallets are "production" and "dev" for A7–A10?** Decision 10 used the installed Windows wallet as production; confirm the same two, and whether the macOS field wallet is a third.
2. **The acceptance numbers** (bytes per backup at steady state; six-month slope) are yours to set once E1–E3 exist — recorded here so they are not set by the agent.
3. No evidence in this contract that any G2 decision is wrong.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed (rows A7–A10: RED designed by the second agent, name recorded)
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] `spike/b5-t3a-p0` deleted (or never created) — recorded
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
