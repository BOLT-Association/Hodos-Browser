# On-chain Backup & Sync — History Overview (one file instead of a dozen)

**Date:** 2026-09-25. **Written for:** the owner, and any fresh session starting track 4's research.
**Purpose:** the history of the on-chain backup is spread over ~25 documents in five folders. This
file gathers it into one place so that (a) a new session does not have to read all of them, and
(b) nothing important is lost when some of those folders are archived.

> ⛔ **This is a consolidation of history, not a design.** The design authority remains
> **`IMPLEMENTATION_PLAN.md`** (this folder: 8 phases, decisions D1–D15) **until the track's own
> research re-opens it.** Where this file and the plan disagree about *what to build*, the plan wins.
> Where this file and the *code* disagree about *what ships*, the code wins. It does neither.

> 👤 **The owner's bar for the whole system:** **stable, no conflicts, efficient.** If we cannot get
> conflict-free *and* efficient together, the design is not ready. §5 and §6 say plainly where the
> current plan stands against that bar.

> ⚠️ **Disclosure note.** §5–§6 describe the snapshot+delta chain used as a multi-device sync log. The
> patent research (`Marston Enterprises/Patents/RECOMMENDATION.md`, 2026-08-23) calls that exact
> combination the one plausibly-novel element and asks that it not be disclosed before the filing
> decision. This file lives next to the plan it summarises (already on the private `origin`). ⛔ Do not
> push it to the public `release` remote. See §8(c).

### How to read the labels

Every claim carries one of three labels:

| Label | Means |
|---|---|
| **[code]** | I read the current code on branch `0.4.0` today (2026-09-25). References are `file.rs :: symbol`. |
| **[measured]** | Someone measured a real number; the source is named. I did not re-measure it. |
| **[doc]** | A document says so; I did not re-verify it against code or chain. |

Commit shas quoted below were each checked with `git show -s` on 2026-09-25.

---

## 1. Sources, what each contributed, and whether it is still accurate

| Source | What it contributed | Still accurate? |
|---|---|---|
| `Final-MVP-Sprint/backup-double-spend-incident-2026-04-11.md` ⭐ | The worst day: Bugs A/B/C, the timeline, the hand SQL repair of the treasury wallet | Accurate as history. Its "status: bugs unfixed" header is **stale** — A/B/C were fixed next day (§3) |
| `Final-MVP-Sprint/wallet-backup-efficiency-plan.md` ⭐ | The only per-field size measurement (2026-04-11); the strip and consolidation proposals; the "the slope matters more than the size" acceptance rule | Measurements accurate for that day and wallet. Proposed 30-day spent-output window **differs from what shipped (7 days)** |
| `Final-MVP-Sprint/wallet-efficiency-and-bsv-alignment.md` | The eight "do not break" backup invariants; the done/not-done checklist | Checklist accurate. Its answer "multi-device sync is not a current concern" is **superseded** by track 4 |
| `Final-MVP-Sprint/bsv-ecosystem-alignment-plan.md` (backup parts only) | Correction that BEEF compaction does not shrink the backup | Accurate. Rest of the doc not assessed here |
| `Wallet-Hardening/ONCHAIN_BACKUP_REVIEW.md` | The July 2026 field bug (stuck retry loop), the `BS-*` findings register, "full sync breaks things" | Findings mostly still open in code (§2.6). Its §2 ("no graceful shutdown exists") is **wrong and self-corrected** in its own header |
| `0.4.0-beta.5/ONCHAIN_BACKUP_SYSTEM.md` (2026-04-21) | The first description of the design | **A sketch with ≥ 6 verified errors** (listed in §7). Do not cite it for facts |
| `track-4…/README.md` | Work items 0–7, the wallet export/import survey, tests T1–T11, "already decided" block | Annotated in place; plan supersedes where marked |
| `track-4…/IMPLEMENTATION_PLAN.md` | The authority: goals G1–G12, decisions D1–D15, phases 1–8, harness H1–H19 | Current plan. **Nothing in it has been implemented yet** [code: no harness, no header, no intent record] |
| `track-4…/ADVERSARIAL_REVIEW.md` + `research/ADV_R1…R5` | 36 confirmed findings (7 critical) against the plan, each answered by a plan edit; 3 refuted | Accurate against the plan as revised 2026-08-24 |
| `track-4…/TRACK_KICKOFF_PROMPT.md` | The prompt that produced the research and the patent study | Executed; history only |
| `research/A1_code_map.md` | Line-by-line map of what the code does vs the docs; the live-DB size numbers | Accurate at 2026-08-22. **Line numbers have moved** (`handlers.rs` is now 21,619 lines); symbols still hold |
| `research/A2_export_import.md` | File export/import is live in the backend, hidden in the UI | Accurate [code spot-checked] |
| `research/A3_retrospective.md` ⭐ | Episodes E1–E13, seven root-cause mechanisms, sixteen "never again" constraints | Accurate; the best single history source |
| `research/B1`, `B2` | BRC-38/39/40/42/43 digest; table-by-table fit of our payload to BRC-38 | Accurate **as of BRCs commit `c1d12f2` (2026-08-22)** — the specs must be re-fetched (§8a) |
| `research/C1`, `C2`, `C3` | wallet-toolbox (TS, Go) sync machinery; deggen's `go-private-backup-cache` | Accurate as of their pinned commits; upstream may have moved |
| `research/D2_plan_critique.md` | The pre-review completeness critique (11 gaps, all fixed in the plan) | Historical |
| `BSV-Tokens/BSV_TOKEN_PROTOCOLS_COMPARISON.md` §"Case study" | PushDrop is the right container; wrapper is ~0.05% of cost; 546-sat marker is a BTC idiom | Accurate |
| `0.4.0-beta.5/tickets/TICKET_brc140_key_shares_vs_bip39.md` | Split-the-seed recovery option (BRC-140), recommended "research in beta.5, build no earlier than beta.6" | Open, unassigned |
| `0.4.0-beta.5/RELEASE_PLAN.md`, `TELESCOPE.md`, `HARNESS_DELTA.md` (found while searching) | The "ancestry depth, not media" hypothesis; the rule that track 4 must not redesign the plan | Current; see §4.4 and §7 row 16 |
| Commit `ed51099` (2026-09-15) message | A new, measured way the backup's size hurts *other* payments | Accurate [measured] |
| Current code | Everything in §2 | — |

**Not read fully:** `Wallet-Hardening/FIX_B_CRASH_SAFETY_SHUTDOWN_PLAN.md`, `FOLLOWUP_RECORD_BEFORE_BROADCAST_TOKENS.md`,
the archived `FIX_A_RECONCILE_PLAN.md`, the Mac forensic docs, and the BRC draft/`DELTA_ANALYSIS.md` in the
Marston tree. Their content reaches this file only through A3, the review and the plan.
**Skimmed (structure + key sections):** B1, B2, C1, C2, C3, ADV_R1–R5 (their conclusions are carried by
`ADVERSARIAL_REVIEW.md` and the plan, which I read in full).

---

## 2. What ships today (code wins)

### 2.1 What it is, in plain terms

Every so often the wallet takes a copy of its own database (minus the recovery phrase and minus
anything it can re-download), compresses it, encrypts it with a key only the recovery phrase can
reproduce, and writes it into a single spendable output on the blockchain. A small "marker" output
next to it sits at an address anyone with the phrase can compute, so a fresh install can find the
backup from the 12 words alone. Each new backup spends the previous one, so only the latest is
unspent on chain.

### 2.2 What is in the payload [code]

- `backup.rs :: BackupPayload` has **21 collections** (wallet row, users, addresses, baskets,
  transactions, outputs, proven_txs, proven_tx_reqs, certificates, certificate fields, output tags +
  map, tx labels + map, commissions, settings, sync_states, domain_permissions,
  cert_field_permissions, parent_transactions, block_headers). The on-chain path empties the last two,
  so **19 carry data** on chain.
- `backup.rs :: collect_payload` excludes: failed transactions (`status != 'failed'`), every backup
  transaction and anything it spent (`reference_number LIKE 'backup-%'`), and the wallet's secret
  columns.
- `backup.rs :: compress_for_onchain` then strips, in order: parent_transactions and block_headers;
  `proven_tx_reqs.raw_tx`/`input_beef`, and history capped to 5 entries; `proven_txs.merkle_path`;
  **spent outputs older than 7 days** (`SPENT_OUTPUT_BACKUP_RETENTION_SECS`) unless they are
  counterparty or non-standard outputs; **dead addresses older than 30 days**; **completed
  transactions older than 60 days** without spendable outputs; dangling foreign keys. Then
  JSON → gzip level 9.
- A confirmed transaction's `raw_tx` is dropped **only when it has a proof** (`proven_tx_id` set).
  An unproven transaction's raw bytes ride along in full — this is what made the August payload huge
  (§4.2).
- The recovery phrase: `compress_for_onchain` blanks it (`payload.mnemonic = String::new()`). ⚠️ The
  password-encrypted **file** export (`backup.rs :: encrypt_backup`, used by the hidden export
  feature) **does include the phrase**.

**Not backed up at all** [code + A1]: `peerpay_received` (395 rows on the production wallet
[measured, A1]), `peerpay_outbox`, `transaction_inputs`/`transaction_outputs`, the three V18
permission child tables, and several columns of `domain_permissions`, `settings`, `transactions`
and `outputs.confirmed`. After a restore these are silently gone or reset to defaults.

### 2.3 Encryption and address [code]

- Key: `backup.rs :: derive_onchain_backup_key` = **SHA-256(master private key ‖
  "hodos-wallet-backup-v1")**. AES-256-GCM, fresh random 12-byte nonce each time
  (`backup.rs :: encrypt_compressed`). The "master private key" is the BIP32 root from the 12 words.
- Address: BRC-42 self-derivation with the literal invoice **`"1-wallet-backup-1"`** in
  `handlers.rs :: do_onchain_backup` and `fetch_onchain_backup`. (That string is security level 1 and
  has a hyphen BRC-43 does not allow — see D5 in §6.) Stored in `addresses` at special index **-3**.

### 2.4 The transaction [code]

`handlers.rs :: do_onchain_backup`:

| Output | Value | What |
|---|---|---|
| vout 0 | **1,000 sats** (`backup_output_sats`) | PushDrop token; the encrypted payload is **one push in the locking script** — no chunking |
| vout 1 | **546 sats** (`marker_sats`) | P2PKH marker at the backup address — the discovery anchor |
| vout 2 | change | back to the wallet |

- **No Hodos service fee** on backups (in-code comment: *"No Hodos service fee for wallet backups —
  this is infrastructure protecting the user"*). Cost per backup = the mining fee; the 1,546 sats in
  vout 0+1 are "parked" and recovered when the next backup spends them.
- Funding (since `ed51099`, 2026-09-15): `select_utxos_smallest_sufficient` — the smallest single coin
  that covers the need, so the change coin stays small (§4.3 explains why).
- **Size control:** only a log warning above 200,000 compressed bytes in `compress_payload`. **No
  hard cap.**
- **Ordering:** build → reserve inputs with a `pending-backup-…` placeholder → sign → build BEEF →
  **broadcast first** → then write the DB records → then re-collect the payload at "now" and store
  its hash as the new baseline.

### 2.5 When it runs [code]

| Trigger | Where | Rule |
|---|---|---|
| Money event | `main.rs :: AppState::request_backup_check_if_significant` | only if the event is **≥ $3.00 USD** at the cached price; if the price cache is empty, nothing happens |
| Debounce | `main.rs :: request_backup_check` + `monitor/mod.rs` | runs 3 min after the last event, hard cap 10 min after the first |
| Periodic | `monitor/mod.rs` schedule `backup: 10800` | every 3 hours |
| Before wallet delete | `handlers.rs` wallet-delete handler calls `do_onchain_backup` | failure blocks the delete |
| Manual | `POST /wallet/backup/onchain` | — |

- `monitor/task_backup.rs :: run` refuses to start below **3,000 sats** balance
  (`MIN_BACKUP_BALANCE_SATS`). Such a wallet never backs up.
- "Nothing changed?" check: SHA-256 of the compressed, unencrypted payload vs `settings.backup_hash`.
  The time-based strips use `last_backup_at` as their clock so ageing alone does not look like a change
  (fix `2f07982`).
- **No backoff.** A failed run leaves the flag set and does not advance `last_backup`, so the next
  monitor tick tries again — the full build-and-broadcast — for as long as it keeps failing.
- The shutdown-time backup was removed (`f768eb3`, 2026-05-07). The shell still gives the wallet
  **5 seconds** to exit after `POST /shutdown` before `TerminateProcess`
  (`cef_browser_shell.cpp :: StopWalletServer`, `WaitForSingleObject(..., 5000)`).

### 2.6 How it restores [code]

`handlers.rs :: wallet_recover_onchain` → `fetch_onchain_backup`:

1. Derive the backup address from the 12 words.
2. Ask WhatsOnChain for **unspent** outputs at that address only (`…/unspent/all`). Spent (older)
   backups are invisible to this query.
3. Pick the "newest" marker with `max_by_key` (unconfirmed counted as newest). Read vout 0 of that
   transaction, decrypt, decompress.
4. If the WhatsOnChain call **errors**, the error is logged and treated as "no backup"; the user is
   told *"No Hodos wallet backup found… use Create New instead."* (finding BS-C2 — still live).
5. Refuses if a wallet already exists; otherwise creates one, deletes its auto-made rows, and imports
   in one DB transaction (`backup.rs :: import_to_db_with_ids`, IDs re-mapped).
6. `refetch_stripped_data` re-downloads raw txs, proofs and missing scripts — **each failure is only a
   warning**; recovery still reports success.
7. `reconcile_backup_tx` adds the backup transaction itself (the payload predates it).
8. Sets `recovery_just_completed`, which now triggers only `TaskCheckForProofs` (the comment in
   `main.rs` still names `TaskValidateUtxos`, removed in `1fa686f`).

There is also a `POST /wallet/backup/onchain/verify` endpoint that compares the on-chain backup to the
live DB by counts.

### 2.7 Tests [code]

`rust-wallet/tests/` contains **no** backup test. `backup.rs` has 12 unit tests (crypto round-trip on a
near-empty payload, path validation). Nothing runs backup → wipe → restore → spend.

### 2.8 Known gaps, today — register [code unless marked]

| Gap (finding id — what it is) | Status in code |
|---|---|
| **BS-C1 — backup takes no spend lock** | Still open. Funding selection and input reservation run without `create_action_lock`/`utxo_selection_lock`; `let _ = output_repo.mark_multiple_spent(...)` still discards the result. The lock is taken only in the stale-pair sweep and the failure-reconcile |
| **BS-H3 / Fix B — crash between broadcast and DB write leaves a "ghost"** | Healed after the fact by Fix A reconcile (`983655a`…`e4f5bb7`, July). Prevented? **No** — no intent record exists (grep `BackupIntent`: nothing). Since `c0894da` (2026-09-08) the case where the DB write *fails* is logged loudly, but the transaction has already gone |
| **R4-1 — ambiguous broadcast (accepted, but reply lost)** | Any broadcast error → `rollback_backup` restores inputs. The plan's "unknown ≠ rejected" rule is not in code |
| **R4-3 — baseline re-collected after broadcast** | Live. A change landing during the broadcast seconds enters the baseline without ever being backed up |
| **BS-M1 — endless retry** | Live (see §2.5) |
| **BS-H2 / BS-M4 / BS-M5 — missing tables and columns** | Live (§2.2) |
| **BS-C2 — indexer error reported as "no backup"** | Live |
| **BS-H1 / BS-H5 — stale or wrong backup chosen on restore** | Live (unspent-only query, `max_by_key`, in-code TODO citing the April incident) |
| **BS-SYNC-1 — full sync makes the backup marker look spendable** | Code reading says still live: `/wallet/sync?full=true` uses `AddressRepository::get_all_by_wallet` (no index filter); `OutputRepository::upsert_received_utxo_with_confirmed` inserts index -3 outputs with NULL derivation and `spendable=1`; `database/helpers.rs :: derive_key_for_output` maps NULL/NULL to the master key — the wrong key for -3. ⚠️ I did not check whether a later selection filter hides these rows |
| **Size cap** | None (warn only) |
| **Round-trip test** | None |

---

## 3. The 2026-04-11 double-spend incident

### 3.1 What happened [doc — the incident file, verified in outline by A3]

On the **treasury** wallet (the real wallet that receives Hodos fees), during testing of the first
size strip, two backups were made about 80 seconds apart across a wallet restart. Three separate
bugs lined up:

- **Bug A — trusting a lagging index to pick inputs.** The new backup asked WhatsOnChain "what is
  unspent at the backup address?" WhatsOnChain's address index runs 30 s to 5 min behind the chain, so
  it still listed a marker the previous backup had *just* spent. The code treated it as a leftover
  ("orphan") and spent it again → guaranteed double-spend.
- **Bug B — reading "HTTP 200" as success.** ARC answered the conflicting transaction with `status 200,
  SEEN_ON_NETWORK` **and the other transaction's txid** — a polite "I already have one". The wallet
  logged success and recorded a transaction that never went on chain.
- **Bug C — a monitor task with destructive power acting on one ambiguous signal.**
  `TaskCheckForProofs` got `DOUBLE_SPEND_ATTEMPTED` for **both** transactions — including the one that
  was actually winning — marked both failed, deleted the winner's outputs and restored its spent
  inputs as spendable.

Result: chain correct, **no coins lost**, local DB wrong (spent coins shown spendable, the live backup's
token and marker missing). Repaired by hand-written SQL against the treasury database the same evening,
after a file backup.

### 3.2 What was fixed [code + git]

| Fix | Commit | In current code? |
|---|---|---|
| Bug A (sweep side): skip a candidate the DB already knows it spent; skip unconfirmed markers younger than 10 min | **`3a6fd2e`** (2026-04-12) — *"fix: three backup pipeline bugs (A, B, C) from 2026-04-11 incident"* | **Yes** — the "Guard A3" and "Guard A2" blocks in `do_onchain_backup` step 5d |
| Bug B: a different txid without a merkle path is a collision, returned as an error | `3a6fd2e` | **Yes** — "TX collision" check in `handlers.rs :: broadcast_transaction` |
| Bug C: cross-check WhatsOnChain before marking failed | `3a6fd2e` | **Yes** — `monitor/task_check_for_proofs.rs` `"DOUBLE_SPEND_ATTEMPTED"` arm: mined → recover; in mempool → wait; 404 → fail; WoC error → skip tick |
| Follow-ups | **`b65c39b`**, **`dce3236`** (2026-04-15, *"three-oracle quorum for failed-tx rollback"*); **`1fa686f`** (2026-04-21, *"chain truth hardening"* — added `TaskVerifyDoubleSpend`, removed `TaskValidateUtxos` and `reconcile_for_derivation`) | Yes |

**Not fixed:**

- **Bug A, recovery side.** The incident doc itself noted the lag works both ways: during a new
  backup's propagation window, a *restore* can find the *old* marker and silently restore stale state.
  Still a TODO comment in `fetch_onchain_backup`.
- **Why ARC said "double spend" about the winner** was never established (three hypotheses, no answer
  — A3 §6, plan §6.3 item 7).

### 3.3 The lesson, as design constraints

From A3 §5 (sixteen constraints), the ones this incident produced:

1. **An indexer answer is a claim with a lag, not the chain.** Never pick inputs from an index younger
   than its propagation window.
2. **Check the txid that comes back**, not the status code.
3. **No monitor task may delete or restore on a single signal.** Destructive changes need independent
   agreement.

These became the plan's G12 (every external call has a written, tested contract), D6 (restore decides
recency from chain linkage, not an index), and D14 (nothing deletes or restores on a timer). The
2026-04-11 day is also why working rule 7 in the root `CLAUDE.md` says *ask the chain first*.

---

## 4. Efficiency

### 4.1 What was proposed and what was done

The April plan's rule, worth keeping: **the slope matters more than the size** — acceptance was
"≤ 5K sats per backup (10K acceptable), and less than 2× growth from month 1 to month 6."

| Proposal | Status [git + code] |
|---|---|
| Spent-output strip (proposed 30 days) | Done `a525ff3` — **7 days** in code |
| Dead-address strip, 30 days | Done `e10ef76` |
| Cap `proven_tx_reqs.history` to 5 | Done `5649c99` |
| 60-day transaction window + drop block headers | Done `a7f684e` |
| Lazy consolidation in normal sends (≤5,000-sat coins, up to 10 extra) | Done `afd4d0a` |
| Daily dust consolidation (`TaskConsolidateDust`) | Done `1bce17f` — note it pays the 1,000-sat service fee, unlike backups [doc: root `CLAUDE.md` "Wallet Service Fee"] |
| Remove the byte-count instrumentation | Done (`e3abeca` then revert `72ecce1`) |
| Binary encoding (CBOR/MessagePack) | Deferred, not done |
| Same-sender / same-basket / all-coins consolidation | Not done |
| Old-transaction "stub" collapse | Not done |
| Off-chain cold tier, delta backups | Deferred in April; delta backups became track 4 |
| Measure month-1 vs month-6 growth | **Never done** |

### 4.2 Sizes over time

| When | Wallet | Size | Source |
|---|---|---|---|
| 2026-04-11 | "moderate-active" wallet (105 tx, 293 outputs) | 388,814 B JSON → **69,882 B** compressed; ~7,045 sats at 100 sat/KB. Top five fields = 92%: outputs 48.8%, transactions 12.1%, proven_tx_reqs 11.6%, addresses 10.5%, proven_txs 9.2% | [measured] efficiency plan |
| 2026-07-07 | a field wallet (from production logs) | 422 KB JSON → **93 KB** compressed → 284 KB BEEF | [measured] review §3 |
| 2026-08-22 | production wallet | live PushDrop script **431,476 B** — 2.2× the code's own 200 KB warning. Without the stuck raw_tx: ~165 KB. Dev wallet: ~61 KB | [measured] A1 §4 |
| 2026-08-24 | production wallet | cause of the bulk found: 4 `noSend` transactions that were actually **mined**, but whose `proven_tx_reqs` never left `nosend`, so their full raw bytes ride in every backup (~60% of the payload, morning pass) | [measured] plan §3.3 |
| 2026-09-15 | installed wallet | backup ~433 KB; the backup's own change coin had a **436 KB parent**, making a PeerPay BEEF 1.76 MB and undeliverable | [measured] commit `ed51099` |

⚠️ The April and August numbers may not be the same wallet; do not read the ~6× as a growth curve.
What *is* clear: the April acceptance targets were never re-checked, and the payload grew past its own
warning without anything stopping it.

⚠️ **Fee-rate assumption differs by source:** the April plan used 100 sat/KB; A1 used 1 sat/KB; the plan
brackets 1–250 sat/KB and says H8a (read the actual fees of the 114+ historical backup transactions)
must replace the guess. That extraction has not been run.

### 4.3 A second kind of cost — found 2026-09-15

A big backup is not only expensive to write. Its **change output** inherits a huge parent transaction,
and any later payment that spends that coin must carry the parent inside its BEEF. For PeerPay
(MessageBox, 1 MiB body limit) that made a payment impossible to deliver. `ed51099` fixed the symptom
(backups fund from the smallest coin; bundle-sending payments prefer small parents; the payment is
refused *before* broadcast if it cannot fit). The root cause — a 400 KB+ backup — is still there.

### 4.4 The "ancestry depth vs media" hypothesis

**Where it comes from [doc]:** `RELEASE_PLAN.md` and `TELESCOPE.md` (beta.5, belief #3, edge E2):
inscription content is already on chain at its outpoint, so store outpoints and re-fetch; therefore the
size problem for token-heavy wallets is likely **`beefB64` ancestry depth** (the BRC-150 provenance
object that ordinals carry in `customInstructions`), not the images.

**Status: unmeasured.** `HARNESS_DELTA.md` §1.3 obliges track 2 to record real provenance-row sizes
*and ancestry depth* as they land; a repo-wide search on 2026-09-25 found **no recorded numbers**.
`TELESCOPE.md`: *"If media dominates, track 4's phase 3 changes shape."*

Two observations for the research, not conclusions:

- The only measured large payload so far (§4.2) was caused by **neither** media nor provenance
  ancestry — it was stuck raw bytes of mined-but-unproven transactions. A third cause exists.
- The ordinal image bytes live in the output's **locking script**. Today spent outputs' scripts are
  stripped but **spendable** outputs' scripts are carried (A1 measured 615 KB of spendable `master`-
  prefix token scripts on the production wallet, raw). So for held ordinals, media *is* in today's
  payload until plan Phase 3 ("strip rule 6") lands.

---

## 5. Conflict-freedom and multi-device

### 5.1 The history of the question

- **April [doc]:** the parent efficiency plan answered "multi-device sync — **not a current concern**.
  Don't constrain backup design around it."
- **August [doc]:** track 4 made it non-negotiable. The design: every backup token carries its parent
  txid; snapshots plus small "delta" notes form a chain at the one address; every device reads to the
  tip before writing; the chain *is* the sync log.

### 5.2 What the plan decides (D-numbers)

| Decision | What it means in plain terms |
|---|---|
| **G3** (goal) | Two devices on one seed: a spend on A is seen by B before B's next spend; simultaneous writes are *detected*, converge in ≤ 2 rounds, no lost row or field. ⚠️ **Scoped down by review:** the backup chain cannot stop two devices spending the *same wallet coin*; for that the promise is detection + bounded healing only |
| **D6** | Restore reads the address's **full history**, decrypts before trusting any header, and picks the tip as "the one no other decryptable token names as its parent" — not the newest by an index's timestamp |
| **D7** | Write a **backup intent record** (every input it reserves, the signed raw tx) *before* broadcast. Broadcast has three outcomes — OK / REJECTED / UNKNOWN — and UNKNOWN never rolls back; it re-sends the identical transaction |
| **D13** | The baseline is what was actually broadcast (never re-collected); a device must apply every other device's deltas **before** writing a snapshot; the producer reads one consistent DB snapshot |
| **D14** | Nothing deletes or restores on a timer. Raw bytes leave the backup only at a declared confirmation depth (≥ 6, provisional). Un-broadcast counterparty transactions go to `abandoned-unbroadcast`, their inputs stay locked |
| **D15** | The chain is the referee (two writes on one parent conflict; the network keeps one). Tie-break named now: **lower `device_id` backs off**. Restore order follows parent links; `seq` is only a sanity check. Two "first ever" backups: the older block wins |

### 5.3 What the adversarial reviews found (2026-08-24)

Five attack lenses, 39 findings, a skeptic pass: **36 confirmed, 3 refuted.** The critical ones, in
plain terms:

| Finding | In one line |
|---|---|
| R4-2 | The plan's own "age out stuck transactions" rule would have re-created Bug C → answered by D14 |
| R1-01 | The shipped "cross-checked" spent test gets *no signal* for backup markers (both oracles return 404 for plain P2PKH) — **live-probed** → recency moved to chain linkage (D6) |
| R1-02 | One missing link in the middle of a delta chain orphans everything behind it → full-history reassembly; "deltas without their snapshot" is a hard error |
| R2-3 | A snapshot built on a stale local copy silently erases another device's change — and looks like a valid chain → D13 merge-before-snapshot |
| R4-1 | "Accepted but reply lost" re-arms the April double-spend → three-valued broadcast (D7) |
| R5-1 | Rate-limiting during restore is swallowed and reported as success → restore must fail loudly and be resumable |
| R3-1 | Any future address migration could silently restore pre-migration state → migrations rejected (D4/D5) |

**Refuted:** R2-1, R2-2, R2-7 — all broken by one structural fact: two backups on the same parent spend
the same outputs, so the network accepts at most one.

**Defended already:** `TaskVerifyDoubleSpend` holds; the txid-collision guard holds; GCM-tag-as-proof-of-
origin survives the crypto review.

### 5.4 Against the owner's bar

- **Conflict-free on the backup chain:** the design argues it convincingly, on paper. **Nothing is
  built or tested** (H4, H19 do not exist).
- **Conflict-free for the user's coins across devices:** **not achievable by this design**, and the
  plan says so (G3 scope, D15 "the backup chain is not a spend serializer"). It offers detection and a
  specified loser state.
- **Efficient:** deltas have **never been produced**; their size (1–2 KB) is an estimate, and so are
  the snapshot-rule constants (0.5× / 20 deltas / 16 KB).

⇒ By the owner's bar the design is **not yet shown ready**: the conflict story has one declared hole
(same-coin races) and the efficiency story has no measurements. That is exactly what the track's
research has to settle.

---

## 6. Settled decisions D1–D15

One line each. "At risk" = some source, or today's owner direction, puts it in question.

| D | Decision | At risk? |
|---|---|---|
| **D1** | Payload = strict BRC-38 document inside an envelope with a `extensions` section; propose a small BRC-38 amendment (`contains` + preserve-unknown) | ⚠️ **Yes** — BRC-38 was read at `c1d12f2` (2026-08-22); owner says the format/export BRCs were recently updated (§8a, 8b). No BRC-38 code exists in our repo [A2] |
| **D2** | Delta = BRC-38 row forms + per-table `deletes` + per-row version counter; merge semantics from BRC-40 | ⚠️ Yes — same spec re-fetch; delta sizes unmeasured |
| **D3** | Borrow two ideas from `go-private-backup-cache` (keep-newest-two, test vectors); carry `prev_payload_sha256` | Minor — upstream pinned at `ca2136e`; repo has no licence (read only) |
| **D4** | Keep the shipped KDF; **migration rejected** absent a cryptographic break (owner, 2026-08-24) | Low — "track-kickoff re-review may re-open with new technical evidence" |
| **D5** | Keep `"1-wallet-backup-1"`; document the conformant `2-wallet backup-1` for new implementations; **address migration rejected** (owner, 2026-08-24) | ⚠️ Interop tension — a foreign wallet following BRC-43 will derive a different address (§8b) |
| **D6** | Restore = full address history, decrypt-before-trust, tip by chain linkage | Depends on an address-*history* endpoint contract that has not been written (G12/D12) |
| **D7** | Persist a backup intent before broadcast; three-valued outcome; reconcile on every trigger | Not implemented; the older "correct as-is" doc still exists (§7) |
| **D8** | Never claim BRC-38/39/40; request a new number | — |
| **D9** | `sync_states` carried but not activated; the chain replaces it | — |
| **D10** | Wording: toolbox sync is BRC-40; 38/39 are file formats | — |
| **D11** | On-chain container is **not** BRC-39 and never claims to be; `.brc39` file interop is Phase 8 | ⚠️ **Yes** — owner's new interoperability goal (§8b) |
| **D12** | Every external lookup contract-first; a second indexer only via shadow mode; it must index plain-P2PKH spent status and history | Second indexer **not chosen** |
| **D13** | Pinned baseline, merge-before-snapshot, consistent reads | Fixes a bug still live in shipped code (R4-3) |
| **D14** | No timer deletes; strip raw bytes only at ≥ 6 confirmations; `abandoned-unbroadcast`; limbo bound | Structure accepted; **constants owner-pending** (age-out horizon, limbo N; 6 confs "provisional") |
| **D15** | Chain arbitrates; lower `device_id` defers; parent-order replay; dual-genesis rule | Validated only by H4, which does not exist yet |

⚠️ **Standing tension:** `RELEASE_PLAN.md` says the plan *"stands and is NOT to be redesigned"*; the
owner's requirements in §8 ask the research to re-fetch the specs and weigh format-first. Both can hold
only if the research's job is to *surface* where a D-decision breaks, and the owner re-opens it.

---

## 7. Contradictions between sources

| # | Claim | Source A says | Source B says | What the code says |
|---|---|---|---|---|
| 1 | Service fee on backups | `ONCHAIN_BACKUP_SYSTEM.md`: 1,000 sats to treasury | Review, A1, plan: none | **None** (`do_onchain_backup` comment) |
| 2 | Spent-output retention | Efficiency plan: 30 days | `ONCHAIN_BACKUP_SYSTEM.md`: 7 days | **7 days** |
| 3 | Multi-device sync | Efficiency parent plan (Apr): "not a current concern" | Track 4 (Aug): non-negotiable (G3) | Not implemented; `sync_states` has no writer |
| 4 | Graceful shutdown | Review §2: does not exist | Review's own correction / FIX_B: exists, 5 s then kill | `StopWalletServer`: `POST /shutdown`, 5,000 ms wait, then `TerminateProcess` |
| 5 | Record-before-broadcast | `FIX_B…PLAN.md`: needed | `FOLLOWUP_RECORD_BEFORE_BROADCAST_TOKENS.md`: "correct as-is" | Broadcast-first; no intent record. `b9afc0b` tried record-first and was reverted the same day (`d324ab8`) with no reason given |
| 6 | How many tables | README item 0: 18 | A1/B2: 19 tables, 21 members | 21 members, 19 carry data |
| 7 | Wallet row carries PIN salt / DPAPI blob | `ONCHAIN_BACKUP_SYSTEM.md`: yes | A1/B2: no | **No** |
| 8 | Restore triggers a full UTXO sync | `ONCHAIN_BACKUP_SYSTEM.md`: yes | A1: no | Sets a flag → `TaskCheckForProofs` only |
| 9 | "The mnemonic is never in the backup" | `ONCHAIN_BACKUP_SYSTEM.md`, BRC-140 ticket, `backup.rs` header | A3 §7.4 | True on chain; **false for the file export** |
| 10 | Encryption key | BRC draft: BRC-42 level 2 | A1, plan D4 | SHA-256(root ‖ "hodos-wallet-backup-v1") |
| 11 | Address invoice | BRC draft: level 2 | B1: conformant form is `… wallet backup …` with spaces | `"1-wallet-backup-1"` |
| 12 | Chunking of large payloads | BRC draft §5 describes it | A1: does not exist | **None**; single push, no cap |
| 13 | Where strips run | `ONCHAIN_BACKUP_SYSTEM.md`: `prepare_backup_payload()` | A1 | `compress_for_onchain`; the other name does not exist |
| 14 | Recovery "needs only the 12 words" | `ONCHAIN_BACKUP_SYSTEM.md` | A3: "the 12 words **and a working indexer**" | An indexer error reads as "no backup" |
| 15 | Backup takes the spend lock | — | Review BS-C1: no lock | Still no lock on selection/reservation |
| 16 | Track 4 may redesign | `RELEASE_PLAN.md`: must not | Owner, 2026-09-25 (§8): re-fetch specs, weigh format-first | — |
| 17 | BRC-38/39/40 status | BRC draft: "reserved" | B1/C1/C2: merged, implemented | — |
| 18 | Fee rate for cost figures | Efficiency plan: 100 sat/KB | A1: 1 sat/KB | Not checked; H8a would settle it |
| 19 | Incident bugs A/B/C | Incident doc header: "UNFIXED" | A3, review: fixed | Fixed (`3a6fd2e`), except recovery-side A |
| 20 | `domain_permissions` columns carried | A1: 5 | B2: 7 of 12 | Not re-counted; both agree the 3 that matter are dropped |

---

## 8. Open questions for the track's research

### Owner requirements (verbatim intent, 2026-09-25)

**(a) Re-fetch current SDKs and specs first; never trust our copies.** Every spec and SDK claim in the
research folder is pinned to a commit from 2026-08-22 (BRCs `c1d12f2`, ts-stack `8b074a0`,
go-wallet-toolbox `9d188d5`, go-private-backup-cache `ca2136e`). Re-fetch before relying on any of it,
and record the new commit next to every claim.

**(b) Review the BRCs on wallet database format and export/import — recently updated — with the goal of
interoperability: users can move into and out of Hodos cleanly.** Today: no BRC-38/39 code exists in
Hodos [A2]; our file export is a private `.hodos-wallet` format [code]; import only works into an empty
wallet and hard-checks our own identity-key convention [A2]; our backup address uses a non-conformant
invoice string (D5). The README's survey (Aug 2026) found HandCash shipping `.brc39` and Yours shipping
a different ZIP-of-chunks format. Questions: what do the updated specs now require; does D1/D11/D5 still
stand; how does a foreign import get its seed at all (plan §6.3 item 8).

**(c) Format-first vs code-first.** The old plan says **the BRC follows the code**, and
`RELEASE_PLAN.md` says the BRC draft *"waits on the provisional-patent decision"*. The owner now notes
that **a published spec is far harder to change than code**. The research must weigh this explicitly.
Input to that: the patent recommendation (research, not legal advice) says any filing must precede the
BRC's publication, and sets planning dates of **engage counsel by 2026-09-15** and **decide by
~2026-10-31** [doc — `Marston Enterprises/Patents/RECOMMENDATION.md`]. Whether those dates were met is
not recorded anywhere I read.

**(d) Research and implementation may interleave, but any spike must be thrown away at a named
moment.** Name the moment when the spike is started, not after.

### Carried open questions

1. **What really makes backups big?** Stuck raw bytes (measured), media in held-token scripts (code
   says it is carried today), or provenance ancestry (`beefB64`, hypothesis)? No numbers exist for the
   last two (§4.4).
2. **How big is a delta?** Never measured; the snapshot-rule constants hang on it.
3. **Two devices, one coin.** Is detection + bounded heal acceptable against "no conflicts", or does
   the owner's bar need a different mechanism (§5.4)?
4. **Why did ARC say "double spend" about the winning tx on 2026-04-11?** Unknown.
5. **Second indexer:** which one indexes plain-P2PKH spent status and address history (D12)?
6. **D14 constants:** age-out horizon, limbo bound N, confirmation depth — owner-pending.
7. **How many field wallets are diverged today?** Never measured (review §8, A3 §6).
8. **Has any real user restore ever succeeded in the field?** No record (A3 §6); H18 exists to make it
   routine.
9. **BRC-140 key shares** (split the key into n-of-m shares): the ticket recommends research in
   beta.5, build no earlier than beta.6 — owner decision owed.
10. **The file export includes the recovery phrase** while the module header says it never does —
    classify it in Phase 2's manifest (plan §6.3 item 9).

---

## 9. What can be archived after this

"Captured" means everything a later session needs from that file is in this overview or in the plan;
archiving moves it, it does not delete it.

### Fully captured — can be archived

| File | Note |
|---|---|
| `Final-MVP-Sprint/backup-double-spend-incident-2026-04-11.md` | §3 carries the story, fixes and lessons. The hand-SQL detail stays readable in the archive |
| `Final-MVP-Sprint/wallet-backup-efficiency-plan.md` | §4 carries the measurement table, what shipped and what did not |
| `Final-MVP-Sprint/wallet-efficiency-and-bsv-alignment.md` | Backup half captured. ⚠️ Non-backup half (ecosystem items 1a–3c) **not assessed here** — archive only if that work is also done/owned elsewhere |
| `Final-MVP-Sprint/bsv-ecosystem-alignment-plan.md` | Backup-relevant part (one correction) captured. Same caveat as above |
| `0.4.0-beta.5/ONCHAIN_BACKUP_SYSTEM.md` | Superseded; its errors are listed in §7. The plan says to rewrite it after Phase 2 — archive the old one then, or now with a pointer here |
| `track-4…/TRACK_KICKOFF_PROMPT.md` | Executed; history only |
| `track-4…/research/D2_plan_critique.md` | All 11 gaps fixed in the plan |

⚠️ The rest of `Final-MVP-Sprint/` (`CLAUDE.md`, `SECURITY_MINDSET.md`, `TESTING_GUIDE.md`, `macos-port/`)
was **not** in scope; do not archive that folder wholesale on the strength of this file.

### Must stay live

| File | Why |
|---|---|
| `track-4…/IMPLEMENTATION_PLAN.md` | The design authority |
| `track-4…/ADVERSARIAL_REVIEW.md` | Finding → plan-edit index; stops the same attacks being re-run |
| `track-4…/README.md` | Keeps T8–T11 verbatim, the wallet export/import survey, and the "already decided" block |
| `track-4…/research/A1`, `A2`, `A3` | The evidence with exact code citations; A3's sixteen constraints are the test of any new design |
| `track-4…/research/B1`, `B2`, `C1`–`C3`, `ADV_R1`–`R5` | Detailed evidence behind D1–D15; keep beside the plan until the re-fetch in §8a replaces them |
| `Wallet-Hardening/ONCHAIN_BACKUP_REVIEW.md` | Its `BS-*` finding ids are cited throughout the plan and are still open in code (§2.8). Archive after plan Phase 2 closes them |
| `Wallet-Hardening/FIX_B_…`, `FOLLOWUP_RECORD_BEFORE_BROADCAST_TOKENS.md` | The live disagreement D7 resolves; keep until the intent record ships |
| `BSV-Tokens/` | Owned by other tracks; only its backup case study is captured here |
| `0.4.0-beta.5/tickets/TICKET_brc140_key_shares_vs_bip39.md` | Open decision |
| This file | Until the research replaces it |
