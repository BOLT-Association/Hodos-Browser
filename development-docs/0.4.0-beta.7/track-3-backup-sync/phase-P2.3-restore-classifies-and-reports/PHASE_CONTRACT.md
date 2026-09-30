# B5-T3a-P2.3 — Restore never says "no backup" when it could not look, and tells the user what it found · PHASE CONTRACT

**Track:** B5-T3a Backup you can trust (single device) · **Tickets:** closes `../research/ONCHAIN_BACKUP_REVIEW.md` **BS-C2, BS-H4, BS-M8, BS-M9**; owns `R-RESTORE` (`../../REGRESSION_ADDITIONS.md`) in full; delivers decision 7 ⑤ (the restore report) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 agent for T3a (resumed) · **Platforms:** both (Rust restore path + `WalletPanelPage.tsx` recovery result) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` (§1.1 fail closed — the SUBJECT names the output) + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** **2 / 2a** (money only when `change=1`; the index is **derived** ⇒ restore **classifies first, then rebuilds**), **5** (the on-chain half of the export ⇄ restore round-trip must be able to go green here — T3b-P5-A4 runs it), **6** tier ① (*a Hodos wallet → check for a backup token* "must work first" — that is BS-C2), **7** in full (restore never fails outright on unidentified coins; shown, never hidden; indexer down ⇒ completes with a report; 1-sat unreadable ⇒ held), **11** via T5-P3 (the cross-site detector restarts empty — the report says so), `SCOPE.md` §0, §6a, §7.3 (the four guarantees owed to T1).

> Part of **T3a-P2** (P2.1 carry · P2.2 write path · **P2.3 restore**) — split justified in P2.1's header.
> ⭐ This is the P2 part **T3b-P5 waits on**: its tier ① and its `P5-A4` on-chain half both close here.

---

## 0. What the code does today, re-read 2026-09-28 (the chain of harm BS-C2 starts)

1. `handlers.rs :: fetch_onchain_backup` returns `Err` on a network error or non-2xx (correct), but returns **`Ok(None)` — "no backup"** when the body is neither an array nor `{result:[…]}` (shape drift, or an error object with a 200).
2. `handlers.rs :: wallet_recover_onchain` Step 3 maps **every** `Err` to `None` (*"will try chain scanning"* — no such scan exists in this path) and answers `{"backup_found": false, "error": "No Hodos wallet backup found for this mnemonic. If this is a new wallet, use Create New instead."}`.
3. `frontend/src/pages/WalletPanelPage.tsx :: doRecoverWallet`, on `backup_found === false`, **silently falls back** to `POST /wallet/recover` — a mnemonic-only scan that creates a wallet **without** the backup's tokens, certificates, BRC-42 counterparty outputs, labels or permissions.
4. **Hypothesis from code reading, not reproduced:** the degraded wallet's next `do_onchain_backup` runs Step 5c / `adopt_onchain_backup`, adopts the **good** on-chain marker and token as "previous backup" inputs and **spends them** into a new token carrying the degraded state — the good backup is superseded. Row `P2.3-A2` reproduces or refutes it on the mock before the fix, and the result is recorded either way.

⇒ Not a rule-7 stop: no durable state is wrong today and nothing shows it has fired (it needs an indexer failure *during a restore*); it is a known **Critical** register finding (BS-C2) that this phase fixes first.

---

## 1. Goal

A user restoring from twelve words either gets their backup — with every coin it declared, classified, the money index rebuilt, and a plain report of anything held or missing — or is told the backup **could not be checked** and nothing is created; they are never told "no backup" because an indexer was down, and a partial restore can be resumed instead of trapping them.

## 2. Done means

- [ ] **BS-C2 — three answers, not two.** `fetch_onchain_backup` returns *Found* / *NotFound* (the indexer answered 2xx with a well-formed, empty list for the backup address) / *Unavailable(reason)* (any error, non-2xx, 429, unexpected shape). `wallet_recover_onchain` answers `backup_found:false` **only** for *NotFound*; *Unavailable* is a distinct response (`backup_check: "unavailable"`, reason, retry advice) that creates **nothing**. `WalletPanelPage.tsx` falls back to the scan-only restore **only** on *NotFound*, and on *Unavailable* shows *"We couldn't reach the network to look for your backup. Nothing was changed — try again."* (wording confirmed at the sitting)
- [ ] **BS-H4 — atomic and resumable.** Wallet creation, the pre-import clean-up `DELETE`s and `import_to_db_with_ids` either all commit or none do; a restore killed or failed at any step boundary leaves **no** wallet or a resumable one — never the 409 *"Wallet already exists"* trap
- [ ] **BS-M8 — no panics on bad bytes.** `reconcile_backup_tx`, `extract_output_script`, `extract_output_value_and_script` bounds-check every slice and `try_into`; truncated or corrupt transaction bytes return a typed error (T1-P6 owns `reconcile_backup_tx`'s *semantics*; this phase owns only its parsing — §11)
- [ ] **Classify, then rebuild.** Every restored output passes T1-P5's seam (`classify_output`, route I3) **before** T1-P3's `money_utxos` rebuild runs; a **pre-beta.6 backup** (received payments carried as `change=0`) restores with every plain payment **selectable** and every held token **not**; with the indexer **down**, classification uses the backup's carried script (T1-P4's `script_length`/`locking_script`), so a multi-sat P2PKH is money on the carried evidence and a 1-sat output whose script is unreadable is **held**
- [ ] **Decision 7 ⑤ — the restore report.** Restore **completes** whenever the backup decrypted, and returns + shows: backup txid and age; counts restored (P2.1's tables included); money balance; **Unknown: N items being identified** with each outpoint and value; refetch results (`raw_tx_fetched`, `proofs_fetched`, `errors`) and what an error means for spending; whether T1-P6's derivation scan ran; *"site-linking detector starts fresh after a restore"* (T5-P3). **BS-M9:** a refetch failure is reported as *"N coins need their history re-downloaded before they can be spent — retrying"*, never as silent success
- [ ] **The four guarantees owed to T1 (SCOPE §7.3):** (1) the payload's `wallet.current_index` is written as `max(wallets.current_index, MAX(addresses.index ≥ 0))` — the one high-water mark (BRC-155 `nextIndex`); (2) restore hands that mark to T1-P6's counter write-back and scan; (3) every carried output keeps outpoint + derivation (the nulled `transaction_id` is P3's); (4) backup change is identified by outpoint as already-known (`reconcile_backup_tx` — T1-P6 writes, **T3 reviews**)
- [ ] Post-restore baseline (Step 6b) uses P2.2's three-valued write (no `let _`)
- [ ] **H18 — live seed-only restore sitting** (owner, funded scratch wallet, clean profile): restore, read the report, spend one coin; report wording settled with real coins (decision 7)

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-RESTORE` a + b | an unclassifiable output is not spendable **and** not lost | This phase owns it end-to-end; a restore that drops what it cannot classify passes (a) and fails the user |
| `R-CLASSIFY` | every ingest route classifies | Restore is route I3; ordering classify → rebuild is this phase's |
| Working rule 7, trip-wires 1–3 | no impossible rows; no verdict where an error is owed; durable state | Restore writes the DB every later session reads as fact — the phase exists to stop an error becoming "no backup" and a partial import becoming a trap |
| Load-bearing safeguards | permission rows restored intact | P2.1 carries them; this phase must not re-default them on the resume path |
| Invariant 2 | no schema change | None: the report is a response + UI; "resumable" is decided from existing rows (a wallet row with no user / an import marker in `monitor_events`), not a new column — if kickoff proves a column is needed, it returns as an owner ask |
| Invariant 3 | no crypto/derivation change | None: decryption, KDF and backup address untouched |

## 4. Evidence table

⛔ Money, schema and crypto rows: RED designed by a second agent (`../../../RELEASE_CYCLE.md` §4.2). Per `HARNESS_DELTA.md` §1.1 every SUBJECT names the output (txid:vout, sats, basket).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P2.3-A1` ⭐ *indexer error ≠ no backup* (BS-C2; T3b-P5 tier ① depends on it) | Mock with a valid backup; the unspent query armed with each fault (timeout, 429, 500, 200-with-`{"error":…}`, 200-with-unexpected-shape): the response is `backup_check: "unavailable"` with the reason; **no** wallet row exists afterwards; the UI shows the *couldn't check* message and **does not** call `/wallet/recover` | **Switches:** (1) revert the tri-state to today's shape (`wallet_recover_onchain` Step 3 `Err ⇒ None`; `fetch_onchain_backup` unrecognised body ⇒ `Ok(None)`) ⇒ each armed fault returns `backup_found:false` and the renderer's network log shows `POST /wallet/recover` ⇒ red, per fault · (2) ⭐ prove the fault was served: P1-A6's probe must show the fault observed in the same run — a green against an unarmed mock is void · (3) two-sided: mock 200 `[]` (genuinely empty) ⇒ `backup_found:false` with the fallback offered — a tri-state that maps everything to Unavailable hides the real new-wallet path · (4) UI half after a HARD reload (Vite HMR fakes controls): revert only the React branch ⇒ Rust says unavailable, the renderer still calls `/wallet/recover` ⇒ red — the two halves are separate reverts · **Red for the right reason:** the response JSON, `SELECT COUNT(*) FROM wallets` = 0, and the renderer network log — the wallet log line is not evidence · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | Per fault: the HTTP response JSON, the DB's `wallets` row count, and the network log of the renderer (no request to `/wallet/recover`) — not the log line | T1 + T3 | ⬜ |
| `P2.3-A2` *the good backup is never superseded* | After an *Unavailable* attempt and a later successful restore, the original on-chain marker + PushDrop are **unspent** on the mock until the restored wallet's own first backup spends them as its parent | **Switches:** (1) ⭐ today's code is the red and the mechanism is readable: `do_onchain_backup` Step 5c *No backup in DB — discovery mode* adopts the newest on-chain marker (`adopt_onchain_backup`) and Steps 7–8 spend marker + PushDrop as inputs — a coins-only fallback wallet (created by `/wallet/recover`, no `1-wallet-backup` rows) holding ≥ 3,000 sats will, on its first cycle with the mock indexer UP, spend the good backup; run this first and record the spend map · (2) after the fix: re-enable the fallback (revert A1's React branch) and run one backup cycle on the fallback wallet ⇒ the original outpoints show SPENT ⇒ red · (3) two-sided: the legitimately restored wallet's OWN first backup must spend them as its parent — assert the spending txid is that wallet's, or *unspent* may only mean backups are broken · (4) timing: the harm needs the indexer DOWN during the restore and UP during the backup cycle — script exactly that · **Red for the right reason:** the mock's spend map names the spending txid and whose backup it was · **Residue:** the pre-fix run leaves a superseding token on the scratch mock chain — declared, mock only — designed by controls-C (Fable), 2026-09-28. ⚠️ Record first, on today's code, whether §0 step 4 reproduces (the fallback wallet's backup consumes the good token) | The mock chain's spend map for the original marker/token outpoints | T1 | ⬜ |
| `P2.3-A3` ⭐ *classify, then rebuild — old backup* | A backup produced by **today's (pre-beta.6) build** (H15 fixture; received payments `change=0`), restored on the new binary: every plain multi-sat P2PKH is in `money_utxos` and selectable by the production selector; every token row is in its basket and **not** in the index; balance shown = sum of index rows; Unknown line = the rest | **Switches:** (1) reorder: run the `money_utxos` rebuild BEFORE `classify_output` ⇒ the H15 fixture's `change=0` payments are absent from the index straight after restore ⇒ red · (2) two-sided: force the classifier to money-for-all ⇒ the fixture's token rows enter the index ⇒ red · (3) ⭐ subject trap: `wallet_recover_onchain` Step 7 calls `Monitor::start`, whose first tick may rebuild the index — read `money_utxos` inside the restore's response window with the Monitor disabled in the test build, then apply switch (1) again: if the row still passes, the Monitor did the work, not the restore · (4) selector leg: call the production selector (`select_utxos_with_preference`) on the restored DB and assert candidate-set EQUALITY with the fixture's declared money outpoints — none of the token outpoints — not a count · **Red for the right reason:** `money_utxos` rows by outpoint vs the fixture's declared money set, read in the fresh process before any Monitor tick · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | The **restored** DB (fresh process): `money_utxos` rows by outpoint vs the fixture's declared money outpoints; the production selector's candidate set | T1 | ⬜ |
| `P2.3-A4` ⭐ *indexer down ⇒ completes with a report* (decision 7 control) | Mock serves the backup, then every other indexer call fails: restore **completes**; carried-script multi-sat P2PKH coins are money; the report lists refetch errors and says which coins cannot be spent until history is re-downloaded | **Switches:** (1) revert the carried-script classification so the classifier consults the indexer ⇒ with every post-fetch call failing the restore either overruns (bound it: wall time < 3 × `CallClass::IndexerBulk` timeout) or returns `success:false` ⇒ red · (2) ⭐ vacuity: with the indexer down `refetch_stripped_data` must report `errors > 0` — a report with `errors = 0` means the refetch did not run against the armed mock (wrong subject) · (3) prove the backup was really fetched: assert the decrypted payload's output count in the same run · (4) two-sided with A5 in this run: the multi-sat P2PKH coins are money AND at least one un-refetched coin is listed as *cannot be spent until history is re-downloaded* · **Red for the right reason:** `success:true` + report fields + restored rows by outpoint, with the mock's per-call log showing every post-fetch call refused · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | The response `success:true` + report fields; the restored DB rows by outpoint | T1 | ⬜ |
| `P2.3-A5` ⭐ *1-sat inscription, indexer down ⇒ held* (decision 7 named control; R-RESTORE a) | A 1-sat output carrying an inscription envelope, script over T1-P4's inline cap (so not carried), restored with the indexer down: **not** in `money_utxos`, not selectable, **shown** in the Unknown list with outpoint and value | **Switches:** (1) revert the fail-closed default (unknown script ⇒ money) ⇒ the 1-sat row lands in `money_utxos` ⇒ red · (2) ⭐ the value-floor trap: beta.3's 1-sat floor holds a 1-sat output regardless of any classifier, so add a 2-sat and a 546-sat inscription variant in the same test, over cap, script not carried ⇒ each must also be held — if either becomes money, the 1-sat green came from the floor, not the classifier · (3) fixture hygiene: assert `locking_script IS NULL` and `script_length > cap` BEFORE restore — a fixture carrying a fabricated 25-byte P2PKH script is T1-P5-A3's test, not this one · (4) shown-not-hidden: the report's Unknown entry carries the outpoint AND `1` sat — a missing or zero value is red · **Red for the right reason:** that outpoint in the restored DB, absent from the selector's candidate set, present in the report · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | That output (txid:vout, 1 sat, basket) in the restored DB, the selector candidate set, and the report's Unknown list | T1 | ⬜ |
| `P2.3-A6` *nothing lost* (R-RESTORE b) | Force every output to `Unknown`: all still appear (Unknown list count = restored output count − index count); none dropped | **Switches:** (1) build the report from `money_utxos` only (revert the Unknown enumeration) ⇒ Unknown count 0 while `outputs` holds N ⇒ red · (2) ⭐ vacuity: `Unknown = outputs − index` is true at 0 = 0 − 0 — assert N ≥ 3 and that the restored `outputs` count EQUALS the payload's first (rows dropped at import — an FK failure on the nulled `transaction_id` — shrink both sides and pass) · (3) drop one payload output before serving ⇒ the identity must go red on the payload-vs-restored leg, not pass on the DB-vs-DB leg · (4) with everything Unknown the balance line is 0 and the Unknown list sums to the payload total · **Red for the right reason:** three counts printed: payload outputs, restored outputs, Unknown list length · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | Restored `outputs` count vs payload `outputs` count; the report's list | T1 | ⬜ |
| `P2.3-A7` *atomic + resumable* (BS-H4) | Kill the restore at each step boundary (after wallet create, after clean-up, mid-import, after import, mid-refetch); retry with the same phrase ⇒ completes; never 409; the final DB equals an uninterrupted restore (`canonical()`) | **Switches:** (1) revert atomicity (today's shape: Step 4 creates the wallet, `DELETE`s run outside the import transaction, `import_to_db_with_ids` inside its own) and kill after Step 4 ⇒ the retry returns 409 ⇒ red · (2) kill mid-`import_entities` (`cfg(test)` abort point after the `outputs` loop) ⇒ today the transaction rolls back but the wallet row stays ⇒ 409; after the fix a retry that re-runs the import on a half-imported DB doubles rows and `canonical()` must show it · (3) ⭐ the DPAPI/Keychain write inside `create_wallet_from_existing_mnemonic` — on macOS the Keychain item outlives the DB: a retry must not fail on *item exists* and must not read the OLD item; assert on both platforms · (4) kill after commit but before Step 6b (`set_backup_hash`) ⇒ a complete wallet with no baseline — the next TaskBackup must NOT broadcast a full backup of the just-restored state (needs P2.2-A3's three-valued read); assert mock broadcast count 0 over the next tick · **Red for the right reason:** HTTP status per kill point, and the comparator diff naming duplicated outpoints · **Residue:** kill runs leave scratch data dirs; case (4) leaves a restored wallet with no baseline — declared — designed by controls-C (Fable), 2026-09-28 | `canonical(retried)` vs `canonical(clean)`; the retry's HTTP status per kill point | T1 | ⬜ |
| `P2.3-A8` *no panic on bad bytes* (BS-M8) | Truncated, over-long-varint and wrong-vout raw transactions into `reconcile_backup_tx` / `extract_output_script` / `extract_output_value_and_script` return typed errors; restore reports them | Revert one bounds check ⇒ the fuzz case panics and the test harness catches the panic (`catch_unwind`) ⇒ red | The function return values on the fixed corpus + a 10k-case fuzz run | T1 | ⬜ |
| `P2.3-A9` *the report is shown and true* | The recovery screen renders each §2 report field from the response, with the Unknown list expandable; the numbers equal the restored DB's | Remove one field from the render (e.g. the Unknown count) ⇒ the rendered-text assertion goes red; feed a response whose Unknown count disagrees with the DB ⇒ the cross-check goes red | Rendered text after a hard reload (Vite HMR trap) vs the restored DB | T2 | ⬜ |
| `P2.3-A10` *high-water mark travels* (guarantee 1) | A wallet with `current_index`=7 and `MAX(addresses.index)`=12 produces a payload with `wallet.current_index`=12; after restore + T1-P6's write-back the next receive index is 13 | **Switches:** (1) revert to writing `wallets.current_index` alone ⇒ the payload says 7 ⇒ after restore + T1-P6 write-back, `generate_address` (`wallet.current_index + 1`) yields 8, which COLLIDES with a fixture address (8..12) that already received money ⇒ red — assert the next generated address is not in the fixture's used set, not merely ≥ 8 · (2) ⭐ vacuity: a fixture where `current_index == MAX(addresses.index)` passes both ways — the fixture must carry the gap (7 vs 12) and the test asserts exactly 12 in the payload and exactly 13 next · (3) `get_max_index` filters `"index" >= 0` — add a `-3` backup address to the fixture and assert it does not raise the mark · (4) T1-P6's write-back absent ⇒ the next index is 8 even with a correct payload — the row records which side failed · **Red for the right reason:** decoded `wallet.current_index`, then the `addresses` row generated after restore · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | The decoded payload's field; the restored wallet's next generated address index | T1 | ⬜ |
| `P2.3-A11` *T1-P6 review* | A written review, in this folder, of T1-P6's `reconcile_backup_tx`, counter write-back and scan-after-restore hook against this contract's §2 — each finding filed or answered | The review names the March phantom fixture (T1 NC-1/NC-11) and states whether P2.3-A3/A5 still pass on top of T1-P6's change; a review that runs no row is incomplete | The review file + the two row re-runs | — | ⬜ |
| `P2.3-A12` 👤 *H18 live restore sitting* | Owner: funded scratch wallet on mainnet, real backup, wait for indexer visibility, clean profile, seed-only restore, read the report, spend one coin; wording signed | Before the sitting, point the restore at a scratch dir that already holds a wallet ⇒ 409 and no fetch (P0-A5's control) — proves the sitting restores for real | The restored scratch wallet's DB + the spend's txid on chain | T4 (human) | ⬜ |

**Two-sided rows:** A5 (held, not spendable) ↔ A6 (not lost) — `R-RESTORE`'s own pairing. A1 (error ⇒ nothing created) ↔ a *NotFound* case in the same test (genuinely empty ⇒ `backup_found:false` and the fallback still offered).

### 4a. Independent control notes (2026-09-28)

*controls-C (Fable), second agent per `../../../RELEASE_CYCLE.md` §4.2. Findings on GREEN/SUBJECT cells I did not rewrite. One line each: row — problem — suggested fix.*

- P2.3-A2 — the §0 step-4 hypothesis has a named mechanism in today's code: `do_onchain_backup` Step 5c discovery branch (no backup in DB + markers on chain ⇒ `adopt_onchain_backup` ⇒ Steps 7–8 spend marker and PushDrop as inputs). Not observed firing (needs the indexer down at restore, up at the next 3-hour cycle, and ≥ 3,000 sats in the fallback wallet). Raise A2's pre-fix expectation from *reproduce or refute* to *expected to reproduce* and keep it first in the phase.
- P2.3-A3 — `wallet_recover_onchain` Step 7 starts the Monitor; a `money_utxos` read after its first tick may be the Monitor's rebuild, not the restore's. The SUBJECT's *fresh process* is necessary, not sufficient — disable the Monitor in the test build for this row.
- P2.3-A5 — the beta.3 1-sat value floor is still live (T1-P5 Q2), so a 1-sat GREEN passes with no classifier at all; the row is only meaningful with the 2-sat / 546-sat variants in its RED.
- P2.3-A6 — the count identity is vacuous at zero and after import-time row drops; assert payload count = restored count before the identity.
- P2.3-A7 — a kill after commit but before Step 6b leaves a complete wallet with no baseline; with today's `get_backup_hash` shape the next tick broadcasts a full backup of the restored state (the §0 harm by a second route). Sequence P2.2-A3 before or with this row.
- P2.3-A4 / A9 (report) — `refetch_stripped_data` rewrites `outputs.locking_script` from the raw tx for rows with a NULL script (`scripts_restored`); whether classification runs before or after it changes the Unknown count the report shows. State the order in §2 (classify → refetch → re-classify the rows whose script arrived), or the report over- or under-counts by `scripts_restored`.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/handlers.rs :: fetch_onchain_backup` | ✅ | `Err` on transport/non-2xx; `Ok(None)` on empty **or unrecognised** body; `max_by_key` tip pick (BS-H5 — P4) |
| `rust-wallet/src/handlers.rs :: wallet_recover_onchain` | ✅ | Step 3 `Err ⇒ None`; Step 4 creates the wallet **before** import; clean-up `DELETE`s with `let _` outside the import transaction; `reconcile_backup_tx` failure logged *"sync will fix later"*; Step 6b `let _ = set_backup_hash`; comment names `TaskValidateUtxos` (removed — T1-P6) |
| `rust-wallet/src/backup.rs :: import_to_db_with_ids`, `import_entities` | ✅ | `BEGIN`/`COMMIT` around `import_entities` only |
| `rust-wallet/src/handlers.rs :: reconcile_backup_tx`, `extract_output_script`, `extract_output_value_and_script` | ✅ | Unchecked `raw_tx[pos..pos+32]` and `try_into().unwrap()` (BS-M8) |
| `rust-wallet/src/handlers.rs :: refetch_stripped_data` | ✅ | Counts `errors`; restore still returns `success:true` (BS-M9); fetches raw tx only for `payload.transactions` + `proven_txs` txids |
| `rust-wallet/src/backup.rs :: BackupWallet.current_index`, `collect_payload` | ✅ | The field exists — the high-water mark needs no new field |
| `frontend/src/pages/WalletPanelPage.tsx :: doRecoverWallet` | ✅ | `backup_found === false` ⇒ `POST /wallet/recover` fallback; recovery result shows counts + balance only |
| T1-P5 `classify_output` (route I3), T1-P3 `money_utxos` rebuild, T1-P4 carried-script shape | ⬜ not yet built | Re-verify at kickoff |

## 6. Out of scope

Discovery by address history, recency and the stale-backup window (BS-H1/H5/M7 — P4). Intent record and crash matrix of the **write** path (P4). Stripping/rehydrating token scripts (P3). The derivation scan itself and `reconcile_backup_tx`'s semantics (T1-P6 — reviewed here). Portable file import (T3b-P5). Any "treat as money" logic (T1-P5 owns the button; the report links to it).

## 7. Rollback

Revert the phase's commits: Rust restore flow (tri-state + atomic import + bounds checks) in one, UI in another. Restored wallets written meanwhile are ordinary wallets the old code reads. ⚠️ Reverting reinstates BS-C2 — say so in the revert message.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| The Rust side returned *Unavailable* but the UI treated any `success:false` as "not found" and still fell back | A1's SUBJECT is the renderer's network log |
| Classification ran, but after the index rebuild — so the old backup's payments were missing from the index until the next restart | A3 checks `money_utxos` straight after restore, in a fresh process |
| With the indexer down, the classifier waited on the indexer and the restore hung instead of completing | A4 (completes) with a bounded timeout per call |
| A 1-sat inscription was classified money because its **fabricated** script (pre-T1-P4 rows) looked like P2PKH | A5 uses a script over the cap; T1-P5-A3 is the fabricated-script control — both must be green |
| "Resumable" re-ran the import on a half-imported DB and duplicated outputs | A7's `canonical()` diff against a clean restore |
| The report said "0 Unknown" because it counted before classification | A9's DB cross-check |
| T1-P6 rewrote `reconcile_backup_tx` after this phase and reintroduced an unchecked slice | A8 runs in `cargo test`; A11 re-runs it on T1-P6's change |
| The live sitting restored into a profile that already had a wallet and read the old DB | A12's pre-check |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | all `cargo test` rows; A9 on the macOS wallet overlay; **A12 once on macOS** | 🍎 `create_wallet_from_existing_mnemonic` stores the phrase via Keychain on macOS (DPAPI on Windows) — the atomic-restore change (A7) must roll that write back too on each platform. A12 on macOS can be the macOS field wallet's scratch profile |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1.25 h** — A12 live restore sitting on Windows, 1.0 h (SCOPE §7.1); report wording sign-off inside it; macOS repeat 0.25 h if the owner runs it |
| Human-bound rows | A12 (real money, visual judgement of the report). ⚠️ **Cannot be batched to the end** — later phases (P3, P4, T3b-P5/P6) read restored state (RELEASE_CYCLE §3.8) |
| Unknowns (K) | **Yes, 1:** the report's user-facing wording and what "resumable" means to a user are settled only with real coins (decision 7). Also a coordination unknown with T1-P6 (both edit the restore flow — order set at G5) |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | **T1-P3** | `money_utxos` + its rebuild function |
| needs | **T1-P4** | real, carried scripts + `script_length` (A4/A5's evidence without an indexer) |
| needs | **T1-P5** | `classify_output` on route I3; the Unknown display the report links to |
| ↔ | **T1-P6** | ⭐ T1 writes `reconcile_backup_tx` semantics, counter write-back, scan-after-restore; **T3 reviews** (A11). P2.3 owns the parsing bounds (A8). Whichever lands second rebases and re-runs A3/A5/A8 |
| needs | **T3a-P1** | mock chain with faults, `canonical()`, H15 first fixture (today's format) |
| needs (soft) | **T3a-P2.1** | the permission/payment tables the report counts (the report works without them; it counts fewer things) |
| gives | **T3b-P5** | tier ① (A1); the on-chain half of `P5-A4` (A3) — P5-A4 **closes after this phase** |
| gives | **T3b-P6** | the classify-then-rebuild path delta replay must also use |
| gives | **T5-P3** | the report states the detector restarts empty |
| gives | `R-RESTORE` | first full GREEN at this boundary |

## 12. Open questions for the owner

1. **The *Unavailable* message and the fallback.** Recommendation: on *Unavailable*, never offer the scan-only restore automatically; offer "Try again" and, only after an explicit second choice, *"Restore coins only (without your backup's history, tokens and settings)"* with that exact warning. Confirm at the A12 sitting.
2. No evidence that a G2 decision is wrong. ⚠️ One consequence worth knowing: decision 6's tier ① *"must work first"* was **not** true on today's code (§0) — this phase is what makes it true.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed (A1–A7, A10: RED designed by the second agent, name recorded)
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded (`R-RESTORE` first full run)
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
