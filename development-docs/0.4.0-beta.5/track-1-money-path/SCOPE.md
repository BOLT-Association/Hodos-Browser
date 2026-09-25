# B5-T1 — Money path · track scope (gate G2)

**Written:** 2026-09-25 by the G2 research agent for beta.5 Track 1 (Money path). **Status:** 📌 PROPOSED —
for owner review at G2. This is a **track** scope: it proposes *candidate phases*, not phase contracts.
**Reads with:** `README.md` (this folder), `utxo-safety-guard/README.md`, `reqwest-tls-bump/README.md`,
`../README.md` (planning notes, RQ-1/RQ-2), `../HARNESS_DELTA.md`, `../REGRESSION_ADDITIONS.md`, `../TELESCOPE.md` §6.

> **How claims are labelled.** **code** = read in `rust-wallet/src` at `0.4.0` head `4435ba1` today ·
> **git** = from `git log`/`git show` · **measurement** = something run (only `Cargo.lock` reads and greps
> today — nothing was executed against a wallet or the chain) · **doc** = what a repo document says ·
> **web** = fetched 2026-09-25, URL given. Nothing here was run against a wallet, a database or the chain.

---

## ⚠️ Rule 7 check — read first

**No new poisoning defect was found with evidence.** Nothing below was seen corrupting live wallet state.

Two defects that are **already ticketed and still live** meet a rule-7 trip-wire. They lead the phase list:

| Defect | Trip-wire | Evidence |
|---|---|---|
| **A spend the whole network refused comes back as success.** `/signAction` returns `200` plus the BEEF even when every broadcaster rejected it as fatal (`"Missing inputs"`) | 2 — a verdict where an error is owed | **Measurement** 2026-09-02 (ticket §14). **code** today: `handlers.rs :: sign_action`'s `Err(e) => { log::error!("❌ Broadcast failed") … }` still falls through to `HttpResponse::Ok()` |
| **The rescan says "nothing found" when it could not look.** `recover_wallet_from_mnemonic` turns a failed chain fetch into an empty list (`fetch_all_utxos(..).await.unwrap_or_default()`), and a failed history call into "never used" (`if let Ok(true) = address_has_history`) | 2 | **code** only. Not measured |

🟡 **Three suspicions, not evidence. Each has a cheap check, and none is escalated:**

1. **An on-chain restore does not restore the address counter.** The backup carries `wallets.current_index`
   (`backup.rs :: collect_payload`), but `wallet_recover_onchain` never writes it back. The only writer of
   `payload.wallet.current_index` is the **file** import (`handlers.rs :: wallet_import`). The restored
   wallet row is created with a literal `0` (`wallet_repo.rs`, the second `INSERT INTO wallets`). Likely
   consequence: after a restore, **generate address** hands out low indices again, and those can be
   old addresses the backup stripped. That is address reuse, which is a privacy leak and not a loss of
   funds. **Check:** restore a scratch wallet from its on-chain backup, then read `wallets.current_index`.
   ⚠️ The rescan ticket's line *"✅ The backup does carry `current_index`"* is true, but it is only half
   the story.
2. **Every startup re-labels unidentified outputs as spendable money.** A startup fix-up in `main.rs`
   runs `UPDATE outputs SET derivation_prefix='master' … WHERE derivation_prefix IS NULL AND … spendable=1
   AND satoshis != 546`. Anything the wallet stored with *no* derivation becomes signable with the
   master key and selectable. This is a guess at classification that runs on every boot. The
   classification guard (P5) must decide its fate.
   **Check:** `SELECT COUNT(*) … WHERE derivation_prefix='master' AND derivation_suffix='-1'` on a
   long-lived profile.
3. **The addresses table mixes two derivation schemes under one index.** BIP32 and BRC-42 addresses
   share one index space, and `UNIQUE(wallet_id, "index")` forces them to collide. `wallet_rescan`
   inserts a **BIP32** address row wherever no row exists at that index. `TaskSyncPending` then labels
   every later coin at that address as BRC-42 (`upsert_received_utxo_with_confirmed` hard-codes
   `2-receive address`). The wallet would then store a coin under a key that cannot sign for it. The
   same hazard is already named in the code as review finding "D-K1" in the comment block above
   `reconcile.rs :: derive_receive_p2pkh_script`. **Check:** `SELECT` rows whose derivation does not
   re-derive to their address.

---

## 1. Goal

**No path — automatic, user-triggered or recovery — can spend an output the wallet has not positively
classified as money, and the wallet's coin and transaction records stay true to the chain.**
*(Unchanged from the draft. The research supports it.)*

---

## 2. Telescope — the current state of the world (re-fetched 2026-09-25)

Everything in this section is **web**, fetched 2026-09-25. Sources are listed in §8.

### 2.1 What changed versus our repo docs

| # | Change | Consequence for us |
|---|---|---|
| T-1 | 🚨 **The TypeScript `wallet-toolbox` repo is now a redirect.** `github.com/bsv-blockchain/wallet-toolbox` holds only a README (last commit `320a82b`, 2026-06-12, "monorepo migration"). The live code is **`bsv-blockchain/ts-stack` → `packages/wallet/wallet-toolbox`**, with its head at `576cb5e` on 2026-09-25. npm `@bsv/wallet-toolbox` is at **2.14.2**, and `@bsv/sdk` at **2.8.7** | Every repo doc that cites `wallet-toolbox @ master` is citing a frozen tree. That includes `TICKET_reservation_ownership_converge_on_spent_by.md` §3, read on 2026-08-22. **Re-cite ts-stack at G3** |
| T-2 | 🚨 **Rust ports exist.** `b1narydt/rust-wallet-toolbox` ("translating the TypeScript wallet-toolbox into idiomatic Rust", SQLite, "still under development") and the crate `bsv-wallet-toolbox-rs` **0.3.66** | Root `CLAUDE.md` rule 5 says *"No Rust implementation"*. That is **now false** as a statement of fact. The rule's intent (port patterns, never code) still stands. **Reported, not edited** (working rule 3). Neither port was audited |
| T-3 | **go-wallet-toolbox v0.187.1** was published on 2026-09-25 | This is the Go reference for this whole scope |
| T-4 | **ts-stack PR #579** (merged 2026-09-23, released in 2.14.0) hardened baskets. `relinquishOutput` now checks basket membership. Moving an output across baskets on insertion is rejected. `default` can never be an insertion destination | The toolbox treats the **money basket as write-protected from apps**. That matches our `basket_repo.rs` rejection of `default` |
| T-5 | **ts-stack PR #599** (merged 2026-09-24) adds to the README: *"A root key or seed alone is not a complete BRC-100 wallet backup … A wallet can recover its root key and still lose the records needed to use its outputs."* The toolbox has **no gap-limit address scan** (none found) | Scanning from the seed is **not** how the reference wallet recovers. It recovers from storage/backup. This supports the owner's narrow scope for the rescan: generated receive addresses only |
| T-6 | **ts-stack issue #545** (opened 2026-09-17, open). After a reorg, stale proofs stay in `proven_txs`, and any spend that selects the coin fails permanently (`merged Beef failed validation`). The same issue says `reproveProven` cannot replace a stale row | The reference reorg handling is **itself broken right now**. Port the *shape*, and do not assume it is finished |
| T-7 | **go-wallet-toolbox has no reorg / reprove task** (it has CheckForProofs, SendWaiting, FailAbandoned, UnFail). Issue #1035 (2026-09-21, open) proposes reorg-aware invalidation | **The two reference implementations disagree** on whether reorg handling is needed at all. See P7 |
| T-8 | **BRC-100 and BRC-147 contradict each other on basket names.** BRC-100's text says a basket name is "at least 5 characters" and "Must not be `default`". BRC-147 requires `1sat` (4 characters), and the 1Sat SDK funds from `default`. The SDK's own validator accepts 1–300 bytes | **Our validator already follows the SDK (1–300)** (`basket_repo.rs :: validate_and_normalize_basket_name`, **code**), so `1sat` is accepted. ⚠️ We do not reject names starting with `admin`, which BRC-100 does and the toolbox relies on for permission-token baskets. That is noted for T2/T5 |
| T-9 | `reqwest` is now at **0.13.5** (2026-09-08). Our `Cargo.lock` has `reqwest 0.11.27`, `rustls 0.21.12`, `rustls-webpki 0.101.7`, `hyper 0.14.32`, `h2 0.3.27` (**measurement**, a lock-file read). The webpki advisories **RUSTSEC-2026-0098 / 0099 / 0104** and **h2 RUSTSEC-2026-0258** still affect us. The rustls advisory **RUSTSEC-2026-0285** does **not** (its unaffected range is <0.23.13) | The target of the bump is now **0.12 or 0.13**, not "0.12+". Choose at G3. 0.13 is one more API break, for no advisory benefit that we know of |
| T-10 | GitLab lists **CVE-2026-56744**: wallet-toolbox did not verify recipient scripts supplied by storage in `createAction` | **Not investigated.** It is listed so P5's route sweep asks the same question of our code |

### 2.2 The specs, as they read today

| BRC | Last commit | What it **requires** to be persisted | What it leaves to us |
|---|---|---|---|
| **46** Output baskets | `fb14783` 2026-04-24 | Per output: outpoint, satoshis, a retrievable locking script, spendability, **tags**, **customInstructions**. Names are 1–300 bytes, trimmed and lowercased. The `p ` prefix is reserved. `admin …` is allowed as implementation-internal | *How* it is stored — columns and tables |
| **99** P baskets | `f20621b` 2024-12-04 | Reject `p <scheme>` unless the scheme is supported | — |
| **100** Wallet interface | `fb14783` 2026-04-24 | `signAction` result: `txid?`, `tx?: AtomicBEEF`, `sendWithResults?`. With `acceptDelayedBroadcast:false`, *"any errors returned in result"* | The error channel. See the toolbox's answer in §3.4 |
| **147** 1Sat basket profile | `6da7c9e` / `5b5d875` 2026-08-18 | The basket name is exactly **`1sat`**. Eligible outputs have `satoshis === 1` and are intended as a 1Sat tip. **MUST NOT** put change or multi-sat outputs in `1sat`. Unknown tags and the whole `customInstructions` string MUST be preserved. Basket membership alone ≠ a verified inscription | Keeping ordinals out of fee funding is only a **SHOULD** |
| **150** Provenance | `b2cb314` 2026-08-10 | A `provenance` object inside customInstructions. Origin is *unproven* until verified | Storage rides on BRC-147 |
| **165** P1Sat permission scheme | `d7f5913` / `5b5d875` 2026-08-18 | **MUST** file collectables in storage basket `1sat`. Viewing goes through `p 1sat <scope>`; spending goes through the label `p 1sat input id <key>`. **Payment / auto-pay MUST NOT satisfy a spend** | — |

---

## 3. RQ-1 — what does "this output is a token" get saved as? *(research answer; the owner decides)*

### 3.1 What the ecosystem does

| | **ts wallet-toolbox** (ts-stack) | **go-wallet-toolbox** v0.187.1 | **1Sat SDK** (`b-open-io/1sat-sdk`) | **Hodos today** (**code**) |
|---|---|---|---|---|
| Where a token lives | App basket + tags + customInstructions. No token column | Same | Named baskets: `1sat`, `bsv21`, `bsv20`, `lock`, `opns`, … (`ONESAT_ASSET_BASKETS`) | Nothing files a token anywhere. `output_baskets.token_type` / `protocol_id` exist, are carried in backups, and are **never written** |
| What makes an output **money** | **Positive marking.** It must be in the change basket (`default`) **and** have `change=true`, `type='P2PKH'`, `purpose='change'`, `providedBy='storage'`, non-empty derivation fields, and `spentBy IS NULL` (`StorageKnex.ts :: allocateChangeInput`) | **Positive marking, in a separate table.** Only rows with `BasketName NOT NULL AND Change=true AND Satoshis>0` enter the `UserUTXO` index. Funding reads only that index | The `default` basket is the funding basket (and the toolbox beneath it adds the change-flag gate) | **Negative marking.** Money is anything with `spendable=1 AND (basket_id IS NULL OR basket='default') AND derivation_prefix IS NOT NULL` (`output_repo.rs :: get_spendable_by_user`). The `change` column exists and **no selector reads it** |
| Auto-detect a 1-sat / inscription? | **No** | **No** | **No** (none seen) | Value floor `is_token_reserved_value` (≤1 sat) at every selector since beta.3 `383bf4f` |
| Reservation | `spentBy = transactionId` at selection, inside one DB transaction with `forUpdate` | `UserUTXO.ReservedByID` = transaction FK | — | A placeholder string in `spending_description`, with `spent_by` NULL until the txid is known |

### 3.2 Where they agree — the convention

1. **Tokens are classified by basket + tags + customInstructions.** No implementation adds a "token
   type" column on the output.
2. **Money is a positive classification, not a default.** In both toolboxes an output never marked as
   change is never auto-spent, whatever basket it is in. ⭐ **This is exactly the release's fail-closed
   rule, and the reference wallets already have it.** We have the opposite. An output with no basket
   is treated as money.
3. **Nobody guesses from `satoshis == 1`.** Classification comes from how the output arrived: the
   wallet's own change, a "wallet payment" internalize, or a "basket insertion" internalize.
4. The `p ` prefix is reserved.

### 3.3 Where they disagree — the real design questions

| # | Disagreement | Why it matters to us |
|---|---|---|
| D-1 | **Flag on the row (TS) vs. membership in a separate index table (Go)** | Go's shape makes "is this money?" a lookup that fails closed: not in the table ⇒ not money. TS's shape needs every selector to repeat a five-field filter. We have **one** selector family in `output_repo.rs`, but at least **six** callers of it, and `send_max` bypasses coin selection (`utxo-safety-guard/README.md`) |
| D-2 | **BRC-147 says ordinals *SHOULD* stay out of fee funding. BRC-165 says auto-pay *MUST NOT* authorize a spend.** The toolboxes enforce separation structurally, with no token knowledge | Structural separation (positive money marking) protects **every** token standard at once, including ones we do not parse yet (BSV-21, contested). That is kickoff decision 3's "general seam", arrived at independently |
| D-3 | **BRC-100's basket-name rules vs. BRC-147 / the SDK** (T-8) | Follow the SDK. We already do |
| D-4 | **Our unique input — address sync.** Neither toolbox discovers outputs by watching addresses; outputs enter through createAction or internalizeAction, where the *caller says* what they are. We also ingest from indexers (`TaskSyncPending`, rescan, restore), where **nobody** says what the output is | This is the gap no reference solves for us. An address-synced output arrives **unclassified by construction**. That is why RQ-2 (restore) and the classifier's `Unknown` state exist |

### 3.4 Options

| Option | Shape | Schema change? | Pros | Cons |
|---|---|---|---|---|
| **A — Basket only** (today's plan) | Classifier files tokens into `1sat` etc. Money stays "NULL or `default`" | No | Smallest change | ⛔ **Still fail-open.** Any ingest route that forgets to classify leaves the output as money. R-CLASSIFY's RED ("prove the default is refusal") can never pass |
| **B — Toolbox TS semantics** ⭐ | Money = `change=1` **and** the existing basket/derivation filters. Tokens go into BRC-147 baskets with tags and customInstructions. **Unknown** = `change=0`, no token basket: *held, visible, not auto-spendable* | **No** — `outputs.change`, `type`, `purpose`, `provided_by` all exist today (`migrations.rs`) | Fail-closed by construction. Matches the reference wallet column for column. `Unknown` is visible, which satisfies R-RESTORE (b), "not lost". Existing backups already carry `change` | A data migration: existing rows that pass the classifier get `change=1`. The six `output_repo` selectors and `send_max` each gain one predicate. Received payments are written with `change=0` today (`upsert_received_utxo*`), so **every** ingest route must classify, which is the point |
| **C — Go semantics** | A separate "spendable money" index table, reserved by transaction FK | **Yes** (invariant 2 — owner) | The strongest fail-closed shape. Also answers the reservation ticket (`ReservedByID`) | A new table on the money path, plus backup-format impact (T3) |
| **D — A classification column** (`spend_class`) | New enum column | **Yes** | Explicit | ⛔ **Diverges from every reference.** No one else does it, so it is a Hodos-only format in the backup that T3 wants to make interoperable |

**Recommendation: B.** It is the reference wallet's own semantics, it needs no schema change, it is
fail-closed, and it gives "Unknown" a visible home. Keep option **C** as the upgrade path if P3's
reservation work wants an index table anyway. **Take no decision here — owner, at G3.**
⚠️ Option B changes **what counts as balance**. A row with `change=0` must still be *shown*, or the
balance silently drops. That is R-RESTORE's "silently lost" failure, in everyday use.

---

## 4. Kaleidoscope — do we already have this?

**Reuse — existing shapes (code):**

| Need | Already exists | Note |
|---|---|---|
| "Is this an index we can sign for?" | `reconcile.rs :: derive_receive_p2pkh_script`, `verify_receive_index` (re-derive `2-receive address-{N}` and byte-compare) | ⭐ The right primitive for the BRC-42 scan **and** for fixing `reconcile_backup_tx`'s change lookup. Already unit-tested |
| BRC-42 receive derivation for a scan | `recovery.rs :: derive_brc42_address` | **Dead since `1094a8e`** (no callers, by grep). Revive it rather than write a third copy |
| BIP32 scan | `recovery.rs :: recover_wallet_from_mnemonic` | Keep. It gains error honesty and stops writing BIP32 rows into the BRC-42 index space |
| Mark the backup tx's inputs spent after a restore | `handlers.rs :: reconcile_backup_tx` (`a1bdfc2`, 2026-03-27) | ⭐ **This is the root fix for the historical phantom** (§5). It is incomplete (§5.3) |
| Authoritative "is this outpoint spent?" | `reconcile.rs :: check_outpoint_spent` (WoC + GorillaPool, fail-closed `Unknown`) | Use it for any scan verdict. Never a bare 404 |
| Insert that never resurrects a known row | `output_repo.rs :: upsert_received_utxo_with_derivation` (`INSERT OR IGNORE` on `UNIQUE(txid,vout)`) | ⭐ The property the scan needs. It must get a negative control, because nothing asserts it today |
| Token-filing exclusion | `output_repo.rs` selectors (`basket_id IS NULL OR b.name='default'`) | Works. See RQ-1 for why it is the wrong polarity |
| Reservation release by transaction | `monitor/task_fail_abandoned.rs`, `task_send_waiting.rs :: cleanup_failed_sending` | The P3 refactor extends these; it does not add a third mechanism |
| Classification slots on baskets | `output_baskets.token_type`, `protocol_id` | Exist and are never written. Under option B they stay unused. Do **not** start using them as a parallel classifier |

**Shapes we would otherwise duplicate:**

- A **third** BRC-42 receive-derivation helper (there are already two, in `recovery.rs` and `reconcile.rs`).
- A **second** "next index" source for the scan. The scan must read the **same** high-water mark that
  `generate_address` and change derivation use. Today those are two different sources
  (`TICKET_two_next_address_index_sources…`), and the on-chain restore sets neither (suspicion 1).
- **76** `reqwest::Client::builder()/new()` construction sites across **35** files (**measurement**, grep).
  P1 must not multiply them. Keep the per-site timeouts (`services::CallClass`) exactly as they are.

**Belief 1 (`TELESCOPE.md` §6) — "the seam is cheap because exclusion already works." Partly false,
by code reading.** These spend or select paths do **not** go through the `output_repo` basket filter
as a classifier:

| # | Path | What it does |
|---|---|---|
| 1 | `create_action` **`send_max`** | `select_all_spendable` — the value floor only. Already known |
| 2 | **dApp-named inputs** (`createAction.inputs` + `inputBEEF`, `handlers.rs` user-provided-input loop) | Validated against the BEEF only, never against the wallet's own classification of that outpoint. The spend authority is `createSignature` ⇒ **T2's token-spend permission class** (BRC-147/165). This goes on the route list |
| 3 | `main.rs` startup `derivation_prefix='master'` fix-up | **Promotes** unidentified rows into money (suspicion 2) |
| 4 | `reconcile_backup_tx`'s "restore falsely marked external-spend" `UPDATE` | Flips `spendable 0→1` for non-default-basket rows by a string heuristic, not by the chain. It is excluded from payment selection by basket, so this is not a money spend. It **is** a "no code expects this" write |
| 5 | `do_onchain_backup` funding | Uses `get_spendable_*` (inherits the filter) ✅ |
| 6 | `task_consolidate_dust` | Uses `get_spendable_confirmed_by_user` ✅ + the value floor ✅ |

⇒ The guard is **somewhat bigger than scoped**: two extra routes (2, 3), plus polarity (RQ-1). It is
not a different track.

---

## 5. ⭐ The rescan ticket — the phantom-coin root cause (owner priority)

`TICKET_rescan_cannot_find_payments_to_generated_addresses.md`. The owner's instruction: *fix the
phantom cause at its root, not bypass it.*

### 5.1 What actually happened in March — reconstructed from git and code

1. **The backup captures the wallet *before* the backup transaction exists** (**code**). In
   `handlers.rs :: do_onchain_backup`, Step 2 compresses the payload. Step 7 reserves the funding coins
   and Step 8 builds the transaction afterwards. So the payload written into backup transaction **T(n)**
   shows T(n)'s own funding coin as **spendable**, and does **not** contain T(n)'s change coin.
2. **The funding coin is usually the previous backup's change** (**code**). Backup change goes to a
   fixed, reused address, `2-receive address-{MAX(addresses.index)}` (`1094a8e`: *"Backup change
   reuses existing address"*). Since beta.3 P10d-A3, backups are funded from the **smallest coin that
   covers the need**, which is typically that small previous change.
3. **So a restore from T(n) held a stale row.** Coin X, the previous change, is marked spendable, but
   on chain T(n) spent it.
4. **The BRC-42 scan then found the real coin.** Y, T(n)'s change, sits at the **same address**. The
   wallet held X *and* Y: two coins where one exists, both "at the backup change address". That
   matches the commit message word for word: *"creates phantom/duplicate UTXOs at backup change
   addresses"* (`1094a8e`, 2026-03-26).
5. ⛔ **Disabling the scan removed the real coin and kept the phantom.** Y disappeared from the
   restored wallet, and X (already spent) stayed "spendable". The double balance went away because the
   wrong half was deleted. The defect was hidden, not fixed.
6. **The root fix landed the next day and the scan was never switched back on.** `a1bdfc2`
   (2026-03-27, *"Complete on-chain backup/recovery with backup tx reconciliation"*) added
   `reconcile_backup_tx`. After import, it parses T(n), marks its inputs spent (with `spent_by` set, so
   `TaskReviewStatus` cannot undo it), and inserts Y. **That is the root cause, fixed.** A second
   phantom mechanism was closed the same day by `58cf9a3`: outputs spent by *earlier* backup
   transactions had `spent_by` nulled by the backup's FK clean-up, and `TaskReviewStatus` revived them.

⚠️ **This is a reconstruction from git and code (**git**, **code**), not a reproduction.** The owner
asked for a reproduction. §5.4 gives one.

⚠️ **A duplicate *row* was never possible.** `outputs` has `UNIQUE(txid, vout)` and the scan inserts with
`INSERT OR IGNORE` (**code**, `migrations.rs`, `output_repo.rs`). The "duplicate" was always one coin
counted twice under **two different outpoints**, one of them already spent.

### 5.2 Therefore — the correct fix, not a bypass

The scan was never the cause. The cause was that **the restored database made a claim the chain
contradicts**. The rule the fix enforces:

> **A scan may only ever ADD a coin the chain says is unspent and that we can prove we can sign for. It
> may never mark a known coin spendable again, and it may never report "none" when it could not ask.**

### 5.3 What is still open today (code), in order of risk

| # | Gap | Effect |
|---|---|---|
| G1 | **`reconcile_backup_tx` failure is logged and ignored** (*"sync will fix later"*). Nothing does fix it: `TaskValidateUtxos`, which the comments still name, was disabled in `9ba106b` (2026-04-19) and no longer exists | The March phantom **returns** whenever that parse fails. The only later healer is the send path's `reconcile_spent_inputs`, which runs *after* a user's send has already failed |
| G2 | **Its change lookup reads the address table instead of re-deriving.** If the backup stripped the change address row (used, no spendable coin in the payload, not pending, older than 30 days), then `change_addr_index = -1` and Y is **silently not inserted** | A real coin missing after restore. It is not a phantom; it is the "silently lost" twin. ⇒ Use `verify_receive_index` |
| G3 | **Index-space collision** (suspicion 3). BIP32 rows written into BRC-42 index slots | Coins recorded under a key that cannot sign |
| G4 | **Fetch errors become "nothing here"** (Rule 7 table) | False "not found". The gap counter also stops early |
| G5 | **The 20-per-address bulk cap** (`TICKET_bulk_utxo_sync_truncates…`, measured 2026-09-16) | A scan that finds a busy address silently stops at 20 coins |
| G6 | **No classification on scan ingest.** Scan-found outputs go straight in as `spendable=1`, basket NULL | A 1-sat inscription at a receive address becomes money (the floor catches only ≤1 sat) |
| G7 | **The high-water mark is not restored** (suspicion 1), and the two index sources disagree | A scan bounded by `max(current_index, MAX(addresses.index))` is bounded by **the wrong number after a restore**. ⇒ The restore marker must record the **payload's** `current_index` at restore time. That value is in hand in `wallet_recover_onchain` and is thrown away |
| G8 | **Backup change is paid to the user's newest receive address** (`get_max_index`) | ⚠️ Privacy (T5): the backup is linkable to an address the user handed out. It is also why every backup's change lands at a scannable address. Flag for T5; no change proposed here |

### 5.4 The design the owner chose, confirmed and sharpened

- **Two separately tested functions**, both *pure* (derive → ask chain → return findings, **no DB writes**):
  `scan_bip32(seed, 0..=hw+margin)` and `scan_brc42_self(master, 0..=hw+margin)`, where the invoice is
  `2-receive address-{i}`. ⛔ **BRC-42 is scannable only for counterparty = self.** PeerPay / BRC-29 keys
  use the sender's key and a random prefix and suffix, so no scan can find them. The Tools-tab card
  must say so (T6).
- **One writer**, shared by both scans. It re-verifies every hit (`verify_receive_index`, or the BIP32
  equivalent) against the chain's own locking script (P4). It inserts **only** unknown outpoints, sends
  each through the classifier (P5), and records `derivation_prefix` from what was **verified**, never
  from the index.
- **High-water mark** = `max(wallets.current_index, MAX(addresses.index ≥ 0), restore_marker.index)`,
  plus a margin of 20. **No gap limit is needed below the mark.** The gap limit is only for the margin.
- **Restore marker.** Record that the wallet was restored and the payload's `current_index`. Run the
  derivation scan **once** after a restore, and on the Tools-tab button. ⚠️ Where the marker lives is a
  schema or settings question. The `settings` repo already exists (`SettingsRepository`). Prefer it over
  a new column (invariant 2 — owner).
- ⚠️ **The index-space collision (G3) needs a decision.** Either stop writing BIP32 rows into
  `addresses` (keep them out of TaskSyncPending), or add a derivation-method column to `addresses`
  (schema — owner). Recommendation: **the first**, unless the owner wants BIP32 addresses watched for
  90 days.

### 5.5 Negative controls — what must go red

Each row is **T1** (unit, seeded DB, fake chain) unless marked **T2** (real wallet, scratch profile, real
money). ⭐ Per `RELEASE_CYCLE.md` §4.2, the controls for the phantom rows should be **designed by someone
other than whoever writes the fix**.

| # | Assertion (GREEN) | RED — remove this and it must fail | Subject |
|---|---|---|---|
| **NC-1** ⭐ *the March bug* | Import payload P (X spendable at addr i) → `reconcile_backup_tx(T)` → BRC-42 scan (fake chain lists Y at addr i). **Exactly one** spendable row at addr i (Y); balance = Y; X has `spendable=0`, `spent_by` = T's row | Disable the input-marking loop in `reconcile_backup_tx` ⇒ X + Y both spendable, **the 2026-03-26 symptom reproduced** | Rows X and Y by outpoint, and the balance figure |
| **NC-2** *never resurrect* | A DB row Z is `spendable=0` with `spent_by` set. The fake indexer (stale) lists Z unspent. After the scan, Z is still spent | Change the insert to `INSERT OR REPLACE` / an upsert that sets `spendable=1` ⇒ Z revived | Row Z |
| **NC-3** *stripped change address* | The payload lacks the address row for Y's index. After restore, Y is present and spendable | Revert to the address-table lookup ⇒ Y absent. That is **G2, the silently lost coin** | Row Y |
| **NC-4** *signable or not at all* | For every row a scan inserts: re-derive the key from `(prefix, suffix)` → P2PKH == **the chain's** locking script | Hard-code `2-receive address` (today's `upsert_received_utxo`) for a BIP32 hit ⇒ mismatch | ⚠️ Compare against the **chain's** script, never the stored one. The stored one is fabricated from the address (`TICKET_synced_outputs…`), so the check would pass vacuously. This is the `feedback_check_whether_a_stored_value_is_an_observation` trap |
| **NC-5** *an error is not an answer* | The fake fetcher returns `Err` for batch k. The scan returns *incomplete* with the failed range, and does not advance the gap counter | Restore `.unwrap_or_default()` ⇒ the scan reports "0 found" | The scan's return value |
| **NC-6** *high-water, not current_index* | `current_index=5`, `MAX(addresses.index)=12`, restore marker = 30, coin at 28 ⇒ found | Bound by `current_index` alone ⇒ not found | The coin at 28 |
| **NC-7** *classify on scan* | A 1-sat inscribed output at addr i lands as Token/Unknown, not money | Bypass the classifier call in the writer | That row's classification (and P5's R-CLASSIFY route list gains "rescan" and "restore scan") |
| **NC-8** *>20 coins* | An address with 22 coins ⇒ 22 rows | Remove the truncation re-fetch (P4) ⇒ 20 | Count |
| **NC-9** *BRC-42 scan is self-only* | A BRC-29 payment (random prefix, foreign sender) at a PeerPay key is **not** found, and the UI text says so | — (a scope statement, not a feature). Assert the doc and UI string exist | Tools-tab copy |
| **NC-10** **T2** ⭐ *the owner's acceptance* | Pay an old generated address that the backup stripped. Restore onto a scratch profile. The scan credits it **once** | Today's `/wallet/rescan` finds **nothing** (run it first — this is the ticket's own RED) | `/wallet/balance` + the `outputs` row + WhatsOnChain for the outpoint |
| **NC-11** **T2** *reproduce March* | Scratch wallet: backup T1 → backup T2 (funded by T1's change) → restore with a **dev-only switch** that skips `reconcile_backup_tx` and the scan enabled ⇒ X + Y (the phantom). Flip the switch back ⇒ Y only | This *is* the red half | ⛔ **Ask the chain first:** WhatsOnChain outspend for X must show it spent by T2 before calling X a phantom (rule 7) |

⚠️ **Residue declaration (rule 7).** NC-11 deliberately creates a row in trip-wire-1 shape (X spendable
while spent on chain). It must run on a **scratch profile** that is deleted afterwards, and its row in
the evidence table must say so.

---

## 6. Ticket review — each one checked against today's code

| Ticket | Still true? (how verified) | Becomes | Why |
|---|---|---|---|
| `rescan_cannot_find_payments_to_generated_addresses` | ✅ Gap 1 (**code**: BIP32 only). ✅ Gap 2 (**code**: address strip). ➕ **Extended:** `current_index` is not restored (suspicion 1), and the index collision (G3) | **Phase P6** (owner priority) | §5 |
| `bulk_utxo_sync_truncates_at_20_per_address` | ✅ (**measured** 2026-09-16; **code**: `BULK_BATCH_SIZE` is addresses per request, and nothing re-fetches at 20) | **Item in P4** | Completeness of ingest is a precondition for classification and for the scan |
| `two_next_address_index_sources_can_reuse_addresses` | ✅ (**code**: `generate_address` uses `current_index+1`; change uses `get_max_index()+1`, with self-heal between). ➕ The on-chain restore resets `current_index` to 0 | **Merge into P6** (proposed track move from "identity & privacy" to T1) | The scan's high-water mark *is* this number. One source, decided once |
| `synced_outputs_store_a_fabricated_locking_script` | ✅ (**code**: `utxo_fetcher.rs` generates the script at all three fetch sites) | **Item in P4** (first) | The classifier and NC-4 both read the script. A fabricated input makes both vacuous |
| `signaction_response_not_brc100_shape` — leftovers | ✅ Fatal broadcast ⇒ `200` (**code**). ✅ `sendWith` is parsed and never read in `sign_action` (**code**, grep) | **Items in P2** | Rule-7 trip-wire 2, already measured |
| `auto_unlock_accepts_another_wallets_mnemonic` | ✅ (**code**: `try_dpapi_unlock` → `validated_mnemonic` checks the shape only; no identity-key comparison) | **Item in P2** | Small, independent, and touches key handling (invariant 3 — ask) |
| `dead_cert_tx_builder_marks_coins_spent_on_404` | ✅ (**code**, grep: `create_certificate_transaction` has a definition and zero callers; a compiler dead-code check was **not** run) | **Item in P2** — delete it | Hygiene. Removes a fail-open pattern before anyone revives it |
| `reservation_ownership_converge_on_spent_by` | ✅ (**code**: `new_reservation_placeholder` → `mark_multiple_spent` writes the placeholder into `spending_description`, `spent_by` NULL) | **Phase P3** | Now re-cite from ts-stack (T-1). Go's `ReservedByID` is a second reference |
| `transaction_row_can_sit_at_created_while_its_coin_is_on_chain` | ✅ (**code**: `TaskFailAbandoned` checks `unprocessed/unsigned`, `TaskSendWaiting` checks `sending`, and `create_action_internal` never sets `sending`) | **Merge into P3** | The same lifecycle: status, not strings, owns a transaction's fate |
| `confirmed_tx_never_rechecked_after_reorg` | ✅ (**code**: no reorg/reprove code in `rust-wallet/src`, by grep) | **Phase P7**, ⚠️ with an owner question | The references disagree (T-6, T-7) |
| `wallet_cannot_shed_large_parents` | ✅ (**code**: the consolidator filters on value only) | **Item in P7** | An *automatic spend path*, so it must come after the guard (P5) |
| `loopback_host_form_wallet_routing` — rows W4/W6/W7/W8 | ✅ W6 (**code**: `permission_service/request_gate.rs` — no `X-Requesting-Domain` ⇒ `GateOutcome::Proceed`) | **Propose a move to T5** (Identity & privacy) | These are C++ trust-boundary work, not money-path logic. ⚠️ **W6 is money-relevant** (an unmarked request is fully trusted, including to spend). If T5 is cut, W6 comes back here |

---

## 7. Candidate phases — ordered

⭐ Sequencing rule (`RELEASE_CYCLE.md` §3.7): **front-load the uncertain.** P1–P2 are certain and
small. P4 carries the first real unknowns. The **pure** scan functions of P6 (§5.4) can be built and
unit-tested in parallel from P4 onward, **but nothing may write scan results to the database until P5
lands.**

### B5-T1-P1 — TLS validator bump (`reqwest` 0.11 → 0.12 or 0.13)
- **Objective:** clear RUSTSEC-2026-0098/0099/0104 (webpki) and 0258 (h2). Transport only: nothing the
  wallet sends or signs changes.
- **Tickets:** `reqwest-tls-bump/README.md`.
- **Unknowns:** the size of the API break at `authfetch.rs` (custom headers, response-header reads);
  whether 0.13 is worth the second break.
- **Negative control:** a `cargo audit` run against the **old** `Cargo.lock` must list the four
  advisories. The same run on the new lock must list none. Plus a local TLS test with a certificate
  that violates a name constraint: rejected after the bump. (Whether that reproduces on 0.101.7 is
  unknown. If it does not, say so, and rely on the audit RED.)

### B5-T1-P2 — A failure must say it failed
- **Objective:** four silent successes on the money path stop being silent.
- **Items:**
  1. `signAction` fatal broadcast ⇒ an error (see Q2).
  2. `sendWith` either honoured or rejected, never ignored.
  3. Auto-unlock refuses a phrase whose master key ≠ `users.identity_key`.
  4. Delete the dead certificate builder.
- **Unknowns:** the error *shape* for item 1. The toolbox throws `WERR_REVIEW_ACTIONS`, carrying
  `sendWithResults`, `txid` and `tx` (**web**; the `ReviewActionResult` status values were seen in search
  snippets only).
- **Negative control:**
  - Item 1: fake a fatal broadcast (`"Missing inputs"`) ⇒ non-success. Revert ⇒ `200` reappears.
    ⚠️ The SUBJECT is **the SDK's view**: drive `@bsv/sdk`'s `HTTPWalletJSON`, not our own handler (the
    ticket's own §4 lesson).
  - Item 3 is the ticket's existing GREEN/RED.

### B5-T1-P3 — Reservations owned by the transaction row
- **Objective:** selection writes `spent_by = <transaction row id>` from the first moment, in one DB
  transaction. Release follows transaction status. `created` rows are reconciled.
- **Tickets:** the reservation-ownership ticket; the `created`-row ticket.
- **Unknowns:**
  - Six-plus placeholder sites (`do_onchain_backup`'s `pending-backup-…` among them) — how many
    must change *together*.
  - Whether option C of RQ-1 (an index table) is taken, which would absorb this phase.
- **Negative control:** start two concurrent sends for the same coin. The second must see a **local**
  conflict (spent_by is set) without touching the network. Revert to placeholders ⇒ the second send
  proceeds to broadcast.

### B5-T1-P4 — Ingest truth
- **Objective:** what we store about a coin is an **observation**, and a short answer is not mistaken
  for a complete one.
- **Items:**
  1. Store the chain's own locking script, bounded (the ticket's three options; ⚠️ not a 2.6 MB BLOB
     in every backup).
  2. Re-fetch any address the bulk call returned exactly 20 coins for.
  3. Fetch errors propagate out of `recover_wallet_from_mnemonic` and `address_has_history`.
- **Unknowns:**
  - The bounded-script option.
  - Whether WhatsOnChain bulk supports paging (**unverified**).
  - The owner's second explorer (the ticket could not identify "banana blocks").
- **Negative control:**
  - The ticket's 22-coin test: 22 ⇒ 22, not 20.
  - An inscription's stored script length > 25. Reinstate `generate_p2pkh_script_from_address` ⇒ 25.

### B5-T1-P5 — The classification guard (`utxo-safety-guard/` 1.1–1.5)
- **Objective:** as the README says, with RQ-1 decided.
- **Route list additions from this pass:**
  - dApp-named inputs
  - the `main.rs` master fix-up
  - rescan / restore-scan writers
  - `reconcile_backup_tx`
- **Unknowns:** RQ-1 (owner); the migration of existing rows; whether the beta.3 1-sat floor stays as
  defence in depth.
- **Negative control:** R-NOSPEND (three separate reds) and R-CLASSIFY ("prove the default is refusal").
  ⭐ Under option B the RED is concrete: insert a row by a route that skips the classifier ⇒ `change=0` ⇒
  not selectable. Under option A it cannot be observed.

### B5-T1-P6 — Find late payments to generated addresses (owner priority)
- **Objective:** §5 — two scans, one writer, a restore marker, one index source, and the phantom rules
  enforced.
- **Tickets:** rescan; two-index-sources.
- **Depends on:** P4 (items 2 and 3), P5 (classifier), and **T3** (the restore flow and backup strip are
  T3's code; see §8.1).
- **Unknowns:**
  - Where the restore marker lives.
  - The BIP32 address-row decision (G3).
  - Whether T3's redesign moves backup change off the receive address (G8).
- **Negative control:** NC-1 … NC-11 (§5.5).

### B5-T1-P7 — Records that stay true over time
- **Objective:** a coin's state is re-checked when the chain changes under it, and a coin is never
  permanently undeliverable.
- **Items:**
  1. Reorg re-proving, bounded window (see Q4).
  2. Large-parent shedding: spend back to self once the parent is proven, through the guard.
- **Unknowns:**
  - The reorg signal source (we have `block_header_repo`; no header-follower task).
  - Whether shedding pays the 1,000-sat service fee (the consolidator does — **doc**, root `CLAUDE.md`).
  - Timing against proof arrival.
- **Negative control:**
  - Reorg: replace a stored proof's block hash with a non-main-chain hash ⇒ the task must flag it.
    Disable the task ⇒ the stale proof stands. (A real reorg cannot be produced. Say so.)
  - Shedding: a coin with a 400 KB parent becomes deliverable over MessageBox. Disable ⇒ still refused.

---

## 8. Integration check (`RELEASE_CYCLE.md` §3.3)

### 8.1 What existing invariant could this break?

| Invariant | Phases | How |
|---|---|---|
| **Gold pill** (`payment_success_indicator`) | P2, P3 | P2 changes an HTTP result on a spend path, and P3 reorders `create_action_internal`, the pill's silent-approve origin. ⚠️ Whether `signAction` reaches `OnWalletCallSuccess` at all is **not verified**; read it at G3. **R-GOLD must run** after both |
| **Per-session counters** (`PermissionService.session_counters`) | P3 | Earlier transaction-row creation plus status-driven release must not double-count or un-count a spend. **R-COUNT** |
| **Privacy perimeter** | P5 (with T2) | The token-spend permission class (BRC-165: auto-pay MUST NOT authorize) lives in T2. P5 must not let dApp-named inputs bypass it. **R-PERIM** |
| **Balance shown to the user** | P5 (option B), P6 | A reclassification that hides money looks like theft. "Unknown" must be **shown** (R-RESTORE b) |
| **Backup format** (T3) | P5, P6 | Option B writes `change`, which is already in the payload. P6 edits `reconcile_backup_tx` and depends on the address strip. ⛔ **T3's plan is "not to be redesigned"** (`../README.md`). P6 touches its restore path, so this edge needs an explicit owner decision point, like `TELESCOPE.md` E3 |
| **Service fee** | P7 | Automatic shedding would pay the treasury fee on a schedule the user did not trigger. The consolidator already does, and it is flagged in root `CLAUDE.md` |

### 8.2 What do we touch that we did not write?
WhatsOnChain's bulk endpoint (20-cap, confirmed-only, no scripts) · GorillaPool · ARC · `reqwest`,
`rustls`, `rustls-webpki`, `hyper`, `h2` · the `@bsv/sdk` `SignActionResult` shape and the toolbox's
error convention · BRC-46/99/100/147/150/165 (two of which contradict each other, T-8) · SQLite
semantics we lean on (`UNIQUE(txid,vout)` treats NULL txids as distinct; `INSERT OR IGNORE`) · the OS
credential stores (DPAPI / Keychain) for P2 item 3, **so macOS parity is owed**.

### 8.3 What would we have to un-ship if we are wrong?
- **P5 / P6 write durable rows** (trip-wire 3), and the on-chain backup then **publishes** them. A wrong
  classification or a scan phantom survives a restore. ⇒ Every row a scan or classifier writes must be
  **findable afterwards**, for example with `purpose` set to a scan-specific value. That column already
  exists, so no schema change is needed. A rollback is then a query, not an archaeology dig.
- **P1** rolls back cleanly: revert `Cargo.toml` and `Cargo.lock`.
- **P2 item 1** changes what dApps see. Un-shipping it after a dApp adapts is a compatibility break, so
  get the shape right once (Q2).
- **P3** reorders the money path. It needs an un-ship plan in its contract (a revert commit that is
  known to build), per the reservation ticket's own warning.

---

## 9. Feasibility inputs (G5.5)

**N — owner-hours (rough; the first estimate, to be scored in the AAR):**

| Phase | Human-bound rows | Est. |
|---|---|---|
| P1 | Send, PeerPay receive, one BRC-121 paid page, price shown | 1 h |
| P2 | A zanaadu-style two-phase mint with a forced failure (may be simulable); auto-unlock on **Windows and macOS** | 1–1.5 h (+ Mac side) |
| P3 | Sends, a cancelled send, concurrent sends, R-GOLD, R-COUNT | 1.5 h |
| P4 | A 22-output transaction; receive an inscription | 1 h |
| P5 | Receive 1-sat and inscribed outputs; forced consolidator timer; send-max | 1.5 h |
| P6 | Backup → restore on scratch; pay an old address; the March reproduction (NC-11) | 2.5–3 h |
| P7 | Large-parent shedding (a reorg cannot be produced by hand) | 1 h |
| **Total** | | **≈ 10–11 owner-hours**, plus macOS sittings for P1/P2 |

**K — phases with a genuine unknown (uncertainty, not difficulty): 5 of 7.**
- **P2:** error shape
- **P4:** bounded script, paging
- **P5:** RQ-1
- **P6:** restore marker, G3, T3 edge
- **P7:** reorg signal, fee

P1 and P3 are hard, but they are understood.

**Cross-track dependencies:**
- **T2** — RQ-1; BRC-165 normalization; the token-spend permission class (dApp-named inputs)
- **T3** — the restore path, backup strip, `reconcile_backup_tx`, R-RESTORE, RQ-2, G8
- **T4** — the BEEF budget ⇄ large parents
- **T5** — the loopback W rows; the G8 privacy leak
- **T6** — the Tools-tab "scan my old addresses" button (card 2)

---

## 10. Prior-art rows — ready for `development-docs/PRIOR_ART.md`

| Date | Question | Source(s) read | What we learned | Verdict | Landed in |
|---|---|---|---|---|---|
| 2026-09-25 | RQ-1: how do conforming wallets persist "token, not money"? | BRC-46/99/100/147/150/165 (BRCs master); ts-stack `wallet-toolbox` `TableOutput*.ts`, `StorageKnex.ts :: allocateChangeInput`, `managedChange.ts`, `createAction.ts`, `internalizeAction.ts`; go-wallet-toolbox v0.187.1 `models/output.go`, `user_utxo.go`, `repo/outputs.go`, `funder/sql.go`; 1sat-sdk `constants.ts` | Tokens = basket + tags + customInstructions everywhere. **Money is a positive mark** (`change=true` + managed fields in TS; a `UserUTXO` index in Go). Nobody auto-detects by `satoshis==1`. BRC-100 vs BRC-147 contradict on basket names | 🟢 | `SCOPE.md` §3 |
| 2026-09-25 | Where does wallet-toolbox live now? | github.com/bsv-blockchain/wallet-toolbox; ts-stack | **Moved to `bsv-blockchain/ts-stack`** (the old repo is a redirect since 2026-06-12). Two Rust ports exist (unaudited) | 🟢 | `SCOPE.md` T-1, T-2 |
| 2026-09-25 | Reorg prior art | ts-stack `TaskReorg.ts`, `WalletStorageManager.reproveHeader`; issue #545; go-wallet-toolbox monitor tasks, issue #1035 | TS re-proves on deactivated headers but has an open bug (#545). **Go has no reorg task.** The references disagree | 🟡 | `SCOPE.md` P7, Q4 |
| 2026-09-25 | Error on a failed undelayed signAction | BRC-100 text; ts-stack `WERR_errors.ts :: WERR_REVIEW_ACTIONS` | The toolbox **throws** a review-actions error carrying `sendWithResults`, `txid` and `tx`. It does not return 200 | 🟢 | `SCOPE.md` P2, Q2 |
| 2026-09-25 | Does any reference wallet scan seed-derived addresses on restore? | ts-stack README (PR #599), bsv-wallet PR #22 | **No.** *"A root key or seed alone is not a complete BRC-100 wallet backup."* Recovery is from storage | 🟢 | `SCOPE.md` §5.4 |
| 2026-09-25 | Reservation ownership | ts-stack `createAction.ts`, `StorageProvider.updateTransactionStatus`; go `user_utxo.go` `ReservedByID` | Both: the reservation is a transaction FK, and release follows status. TS releases **without** a chain check, except BRC-177 no-send expiry | 🟢 | `SCOPE.md` P3 |
| 2026-09-25 | reqwest / TLS advisory state | crates.io reqwest; rustsec.org (reqwest, rustls, rustls-webpki, h2) | reqwest 0.13.5 is latest. webpki 0098/0099/0104 and h2 0258 affect us. rustls 0285 does not | 🟢 | `SCOPE.md` T-9, P1 |

---

## 11. Open questions for the owner — each with a recommendation

| # | Question | Recommendation |
|---|---|---|
| **Q1** | **RQ-1 — how is "money vs token" stored?** (options A–D, §3.4) | **B**: toolbox semantics. Money = positively marked (`change=1`), tokens go into BRC-147 baskets with tags and customInstructions, and Unknown is *held and shown, not spent*. No schema change |
| **Q2** | **A signAction whose broadcast is fatally refused: what does the dApp see?** | Follow the toolbox: **an error response** (non-2xx) carrying `txid`, `tx` and a review-actions result. Do **not** return 200 |
| **Q3** | **Move the loopback leftovers (W4/W6/W7/W8) to T5?** | **Yes.** They are trust-boundary work. Keep W6 in view here: an unmarked request is fully trusted |
| **Q4** | **Reorg handling (P7): build it, given the Go reference has none and the TS one has an open bug?** | **Build a narrow version**: detect (a stored proof's block no longer on the main chain) and **flag and re-fetch**. Do not auto-revert statuses. Cheap, and it is the part both references would agree on |
| **Q5** | **Rescan: stop writing BIP32 addresses into the `addresses` table, or add a derivation column?** | **Stop writing them.** No schema change. A BIP32 hit gets its coin row, labelled `bip32`, but no watched address |
| **Q6** | **P6 edits T3's restore path (`reconcile_backup_tx`, the address strip). Who owns the edit?** | **T1 writes it, T3 reviews it**, recorded as an explicit edge decision like `TELESCOPE.md` E3. The alternative, T3 absorbing P6, delays the owner's priority behind the backup redesign |
| **Q7** | **Two-index-sources ticket: move it from "identity & privacy" into T1 (P6)?** | **Yes.** The scan's high-water mark is that number |
| **Q8** | **Root `CLAUDE.md` rule 5 says there is no Rust wallet-toolbox. Two now exist.** | Correct the sentence in a docs commit of its own. Keep "port patterns, never code". Do not adopt either port without an audit |
| **Q9** | **reqwest target: 0.12 or 0.13?** | **0.12**, unless P1's scoping finds 0.12 is itself close to end-of-life. One API break, same advisory benefit (as far as we know) |
