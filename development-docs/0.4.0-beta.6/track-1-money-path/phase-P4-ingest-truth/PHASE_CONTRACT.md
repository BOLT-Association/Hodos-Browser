# B5-T1-P4 — what the wallet records about a coin is what the chain says · PHASE CONTRACT

**Track:** B5-T1 Money path · **Tickets:** `../../tickets/TICKET_synced_outputs_store_a_fabricated_locking_script.md` (first) · `../../tickets/TICKET_bulk_utxo_sync_truncates_at_20_per_address.md` · the rule-7 row *"the rescan says nothing found when it could not look"* (`../SCOPE.md` "Rule 7 check") · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **7** (tiered rule, step ①: *fetch the coin's **real** script and classify it — today's synced scripts are fabricated from the address*; T1-P4 goes **first**, before the classifier). **2** (money is positively marked — the mark is only as good as the evidence under it). `HARNESS_DELTA.md` §1.1 (fail closed; SUBJECT names the output).

---

## 1. Goal

For every coin the wallet finds on its own addresses, what it stores is an **observation of the chain** — the real locking script (or a marked "not observed"), all of the coins rather than the first twenty, and "I could not look" rather than "there is nothing there".

## 2. Done means

- [ ] **No fabricated script is stored as if observed.** `generate_p2pkh_script_from_address` (three copies: `utxo_fetcher.rs`, `services/providers/whatsonchain.rs`, `services/providers/gorillapool_ordinals.rs`) no longer produces a value written to `outputs.locking_script` for a synced coin. The script comes from the parent transaction's raw bytes (`reconcile.rs :: parse_tx_outputs(raw)[vout]`), fetched **before** the row is written (today `task_sync_pending.rs :: cache_parent_transactions` runs **after** the insert).
- [ ] **The raw parent is checked before it is believed:** `double-SHA256(raw) reversed == txid`, and `parse_tx_outputs(raw)[vout].value == satoshis the indexer claimed`. A mismatch ⇒ the row is written **not observed** and logged — never with the indexer's value silently trusted over the bytes.
- [ ] **Bounded storage, toolbox shape (prior art, re-read today):** `outputs.script_length` and `outputs.script_offset` are **always** written from the parsed transaction; `outputs.locking_script` holds the full script only when `script_length ≤ cap`; above the cap it holds nothing and readers fetch the script from the raw transaction. This is exactly `@bsv/wallet-toolbox`'s rule (`ts-stack` `packages/wallet/wallet-toolbox/src/storage/methods/processAction.ts`: *"Remove long lockingScript data from outputs table, will be read from rawTx"* when `offset.length > settings.maxOutputScript`). Our schema already has all three columns and `settings.max_output_script` (`migrations.rs`, default 500000, **never read today**) ⇒ **no schema change**. The cap value is §12 Q1.
- [ ] **"Not observed" is representable without a new column:** a row whose script was not obtained has `script_length IS NULL`; every reader that needs a script (the classifier in P5) treats `NULL` as **not observed** ⇒ fail closed. ⚠️ A synthesised P2PKH may still be *used* for spending a coin already known to be ours, but it is never *written* as an observation.
- [ ] **A raw transaction a long script depends on is not purged:** `monitor/task_purge.rs` deletes confirmed `parent_transactions` after 7 days; a parent referenced by an output whose `locking_script` was offloaded is exempt (or the script is re-fetched on demand and the re-fetch path is tested — §12 Q1 chooses).
- [ ] **Twenty is not "all":** an address for which the WhatsOnChain bulk call (`utxo_fetcher.rs :: WOC_BULK_CONFIRMED`, via `fetch_bulk_chunk`) returns **exactly 20** coins is re-fetched singly (`fetch_utxos_for_address`); the result used is the single-address answer. Bulk stays the default (the ticket: do not switch everything to single fetches).
- [ ] **An error is not an answer:** `recovery.rs :: recover_wallet_from_mnemonic` no longer turns a failed `fetch_all_utxos` into an empty list (`.unwrap_or_default()`), nor a failed `utxo_fetcher.rs :: address_has_history` into "never used" (`if let Ok(true) = …`). The scan returns **incomplete, with the failed index ranges**, and the gap counter does not advance over a range it could not read. Its callers (`handlers.rs :: wallet_rescan`, `wallet_recover`) report "incomplete — N ranges could not be checked" instead of a count.

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-CLASSIFY` | classification reaches every ingest route | This phase **makes it possible**: the row's own note says it goes vacuous while the script is fabricated. After P4 the classifier has an observation to read — or an explicit "not observed" |
| `R-DUST` / `R-NOSPEND` | no incidental token spend | A script-fetch failure must never make a coin *more* spendable than today. Today's rows keep their current status; P5 decides classification |
| `R-PEERPAY-DELIVERY` half 1 | PeerPay delivers or never leaves | Not touched (MessageBox path), but `TaskSyncPending` runs beside it — half 1 at the boundary |
| memory: bulk sync blind to mempool | `addresses/unspent` is confirmed-only | The single-address re-fetch must not silently change the confirmed/unconfirmed meaning of what is stored |
| memory: caches must not self-poison | a failed fetch must not write | A failed parent fetch must not write a "not observed" that later reads as final — it is retried |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. A green result is reported with its red half or not at all.
⛔ Money, schema and crypto rows: the RED (negative control) is **designed by someone other than the assertion's author** — a second agent (`../../../RELEASE_CYCLE.md` §4.2). Record who designed it.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P4-A1` | An inscription output synced from its real parent is stored with `script_length` = the true length (the ticket's measured fresh inscription: **2,596,810 B**) and is **not** stored as a 25-byte P2PKH | Two switches · (i) the ticket's own: reinstate the synthesis path ⇒ `script_length` 25 / NULL with a 25-byte script ⇒ red on the **length** (⛔ assert the length differs, never that a classification 'succeeded') · (ii) the ordering defect on its own: keep the real parse but write the row **before** the parent is fetched (today's order — `task_sync_pending.rs :: cache_parent_transactions` runs after the upsert) ⇒ the row as first read has `script_length` NULL/25 ⇒ red — distinct from (i) because the fetch is correct and the row is still wrong · Fixture T1: the ticket's real prefix bytes; T2: a real inscription received at a scratch address · Residue: T2's inscription lives at a scratch address (record the outpoint; it is an asset — fake broadcaster only, never spend it) — designed by controls-A (Fable), 2026-09-28 | ⭐ The **stored row** by outpoint (the ticket's reproducible mainnet outpoints), never the API response or a log line — the fabrication happens between them. Fixture: the ticket's real prefix `76a914…88ac 0063036f7264 5109 image/png` (no network needed) for T1; a real inscription at a scratch wallet address for T2 | T1 + T2 | ⬜ |
| `P4-A2` | An OrdLock listing (**860 B**, not P2PKH-shaped) synced from its parent ⇒ `script_length` 860 and the script stored (under the cap) byte-for-byte equal to the chain's | (i) flip one byte inside the fetched raw tx's output script (scratch fixture) with the A4 checks intact ⇒ the txid check refuses it and the row is *not observed* — the byte-compare is never reached; with the checks removed (scratch build) the stored 860 bytes ≠ chain ⇒ red — run both so the row is seen to depend on the compare and not only on A4 · (ii) cap set below 860 ⇒ `locking_script` empty, `script_length`=860; a compare that reads the column instead of the accessor reports ∅ ≠ chain ⇒ red for the wrong reason — the GREEN must compare through the accessor · Residue: none (T1); T2 reads a public listing outpoint, scratch DB only — designed by controls-A (Fable), 2026-09-28 | Stored bytes vs WhatsOnChain's raw tx output script for that outpoint | T1 + T2 | ⬜ |
| `P4-A3` | A plain 25-byte P2PKH coin is stored with the chain's script (equal to today's value — proves no regression for money) | Swap in a script from a different output of the same parent ⇒ the byte comparison fails (seen) | Stored bytes vs the parent's output bytes at that vout | T1 | ⬜ |
| `P4-A4` | Raw parent whose double-SHA256 ≠ txid, or whose `[vout]` value ≠ the indexer's satoshis ⇒ row written **not observed** (`script_length NULL`), one warning line | (i) remove the `reconcile.rs :: verify_raw_txid` call (scratch build) ⇒ the one-byte-flipped raw tx parses and its `[vout]` script is stored as observed (`script_length` set) ⇒ red · (ii) keep the hash check, remove the value check, feed a raw tx that hashes correctly while the fixture's indexer-claimed satoshis are altered ⇒ row written observed with the indexer's value trusted over the bytes ⇒ red · Right reason: the warning line names the txid **and which check failed**; a generic 'parse error' line is not this row · Residue: none, T1 — designed by controls-A (Fable), 2026-09-28 | The row and the log line; fixture = a real raw tx with one byte flipped | T1 | ⬜ |
| `P4-A5` | Parent fetch fails (all providers down) ⇒ the row is written **not observed** and the fetch is retried on the next sync tick; once it succeeds the row is completed | Fake providers: tick 1 all `Err`, tick 2 `Ok(raw)` · (i) make the failed-fetch arm write the synthesised 25-byte script (today's behaviour) ⇒ the row reads observed and tick 2 has nothing to complete ⇒ red · (ii) make 'not observed' final — the retry candidate query only sees new rows (today `upsert_received_utxo*` is `INSERT OR IGNORE` and the `Ok(_)` arm re-fetches nothing) ⇒ after providers return the row stays `script_length NULL` forever ⇒ red = the pre-mortem's 'Unknown forever' · Residue: none, T1 — designed by controls-A (Fable), 2026-09-28 | The row before/after the second tick | T1 | ⬜ |
| `P4-A6` | A script longer than the cap: `locking_script` empty, `script_length`/`script_offset` set, and reading the script back through the new accessor returns the full chain script — **also after `TaskPurge` has run** with its clock moved 8 days forward | (i) remove the purge exemption (scratch build) and force the purge — ⚠️ `task_purge.rs :: run` deletes only `cached_at < now-7d AND txid IN (SELECT txid FROM proven_txs)`, so the fixture **must** seed a `proven_txs` row for the parent and backdate `cached_at`, or the purge cannot fire and the control is vacuous ⇒ accessor returns empty/Err ⇒ red · (ii) read `outputs.locking_script` directly instead of through the accessor ⇒ ∅ ≠ chain ⇒ red for the wrong reason — the GREEN must name the accessor it read · Residue: scratch DB — designed by controls-A (Fable), 2026-09-28 | Accessor output vs chain bytes, after a forced purge | T1 | ⬜ |
| `P4-A7` | The 22-coin address: bulk returns 20 ⇒ re-fetch singly ⇒ **22** rows, all marked with the height the single call reports | (i) the ticket's own: remove the exactly-20 re-fetch ⇒ 20 rows ⇒ red on the count · (ii) SUBJECT control for 'the single answer is the one used': fake indexer returns 20 from bulk and 22 from single with a **different `height`** on one coin ⇒ the stored height must be the single call's; a pipeline that unions bulk ∪ single also reaches 22 rows but keeps the bulk height ⇒ red on the height, count fine · (iii) 21 from single (a coin spent between the two calls) ⇒ stored 21, never 22 — the single answer replaces, it does not add · Residue: T2 — one real 22-output tx to a scratch address (cents; record the txid) — designed by controls-A (Fable), 2026-09-28 | Row count for that address, by outpoint; ⚠️ the ticket's M5 carrier was hand-marked confirmed once — use a **fresh** 22-output transaction (a scratch wallet paying one of its own addresses) | T1 (fake indexer) + T2 (money, cents) | ⬜ |
| `P4-A8` | An address with 19 coins is **not** re-fetched (bulk stays the path) | Force the threshold to 19 ⇒ a re-fetch log line appears (proves the log can see it) | The per-address re-fetch log line, absent | T1 | ⬜ |
| `P4-A9` | `recover_wallet_from_mnemonic` with the fake fetcher returning `Err` for batch k ⇒ returns *incomplete* with that range; gap counter unchanged over it (SCOPE §5.5 **NC-5**) | (i) restore `.unwrap_or_default()` on `fetch_all_utxos` in `recovery.rs :: recover_wallet_from_mnemonic` ⇒ batch k reads as empty, 'N found' with no range ⇒ red · (ii) gap counter: keep the `Err` but let `unused_count += 1` run over the unread range ⇒ the scan stops 'gap limit reached' inside a range it never read and a real coin at index k·20+25 is missed ⇒ red on `addresses_found` — the fake fetcher must place a coin **after** the failed batch or (ii) cannot be seen · Residue: none, T1 — designed by controls-A (Fable), 2026-09-28 | The function's return value (ranges + found list) | T1 | ⬜ |
| `P4-A10` | `address_has_history` returning `Err` ⇒ that index counts as "unknown", never as "never used" | Restore `if let Ok(true) = address_has_history(..)` ⇒ an `Err` index counts as unused; fake fetcher returns `Err` for 20 consecutive indices ⇒ the scan terminates 'gap limit reached' ⇒ red · SUBJECT: `Err` for indices 5–24 and a real coin at 30 — the green must still reach 30 (or stop and report 5–24 as unknown); ⚠️ an `Err` on one index only can never be told apart from 'unused' because a single unknown never reaches the gap limit — that control is vacuous · Residue: none, T1 — designed by controls-A (Fable), 2026-09-28 | The scan's gap counter and return value | T1 | ⬜ |
| `P4-A11` | `/wallet/rescan` with the indexer unreachable ⇒ response says **incomplete**, never "0 found" | Point the providers at a dead host on today's code ⇒ response reads as "nothing found" (the defect, seen) | The HTTP response body the Tools tab would render (T6 card 2 consumes it) | T2 | ⬜ |
| `P4-A12` | No write site passes a synthesised script as observed: `grep -rn "generate_p2pkh_script_from_address" rust-wallet/src` lists only call sites that do **not** feed `outputs.locking_script` for a synced coin (each justified in the commit) | Add one synthesised write in a scratch build ⇒ the per-site review list grows and `P4-A1`'s T1 test goes red | The grep output + the justification per remaining site | T0 | ⬜ |

**Two-sided rows:** `P4-A1`/`A2` (non-P2PKH scripts are stored truly) ⟷ `P4-A3` (a P2PKH coin is unchanged) — a pipeline that stores "unknown" for everything passes neither. `P4-A7` (20 ⇒ re-fetch) ⟷ `P4-A8` (19 ⇒ no re-fetch).

### 4a. Independent control notes (2026-09-28)

- `P4-A6` — `task_purge.rs :: run` purges a parent only when `txid IN (SELECT txid FROM proven_txs)`. A fixture whose parent has no `proven_txs` row makes **both** halves vacuous: the purge never fires, GREEN and RED look identical. The row must seed the proven row and say so.
- `P4-A11` — `wallet_rescan` today returns **503** `Scan failed` when the scanner returns `Err`; the defect is the scanner returning `Ok` with 0 found because `fetch_all_utxos`' `Err` is swallowed inside the batch loop. The GREEN's "incomplete" must be a named field distinguishable from today's 503, or T6 card 2 cannot tell the two apart.
- `P4-A12` — the grep counts sites, but the RED ("add one synthesised write ⇒ A1's T1 test goes red") only holds if A1's T1 test reads the row the added site writes. Name which test / which write site pair closes it, or the grep grows and A1 stays green.
- `P4-X1` (proposed) — an offloaded row (`locking_script` empty, `script_length` set) reaching a sighash path. §5's last row says "to enumerate at kickoff"; this row makes the enumeration falsifiable. GREEN: every signing path that receives such a row fails closed (refuses to sign) or reads the script through the accessor; RED: hand such a row to the sighash builder on today's code ⇒ it signs over an empty prevout script and the network rejects with `mandatory-script-verify` — the failure surfaces at the miner, not in the wallet; SUBJECT: the prevout script length in the sighash preimage.
- `P4-X2` (proposed, small) — `output_repo.rs :: upsert_received_utxo_with_confirmed` hard-codes `type='P2PKH'`; after P4 a 2.6 MB inscription row still says `P2PKH`. Either `P4-A1` also asserts `type`, or the row states the column is P5's and why.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/utxo_fetcher.rs :: generate_p2pkh_script_from_address`, `fetch_utxos_for_address`, `fetch_bulk_chunk`, `fetch_utxos_bulk`, `fetch_all_utxos`, `address_has_history`, `WOC_BULK_CONFIRMED`, `BULK_BATCH_SIZE` (= 20 **addresses per request**, not the cap) | ✅ | The fetch side |
| `rust-wallet/src/services/providers/whatsonchain.rs :: generate_p2pkh_script_from_address`, `services/providers/gorillapool_ordinals.rs :: generate_p2pkh_script_from_address` | ✅ | The other two copies of the fabrication |
| `rust-wallet/src/monitor/task_sync_pending.rs :: run`, `cache_parent_transactions` (called after the upsert today) | ✅ | Order changes: parent first, then write |
| `rust-wallet/src/cache_helpers.rs :: fetch_parent_transaction_from_api`; `database/parent_transaction_repo.rs` | ✅ | The existing parent fetch + cache — reused |
| `rust-wallet/src/reconcile.rs :: parse_tx_outputs` (+ its tests `parse_tx_outputs_reads_all_outputs`, `parse_tx_outputs_fails_closed_on_truncation`) | ✅ | Existing parser — reused |
| `rust-wallet/src/database/output_repo.rs :: upsert_received_utxo`, `upsert_received_utxo_with_confirmed` (hard-codes `2-receive address`, `type='P2PKH'`), `upsert_received_utxo_with_derivation`, `insert_output` | ✅ | Gain the observed script / length / offset |
| `rust-wallet/src/database/migrations.rs` — `outputs.script_length`, `script_offset`, `locking_script`; `settings.max_output_script INTEGER NOT NULL DEFAULT 500000`; `database/settings_repo.rs :: SettingsRepository::get` | ✅ | No schema change |
| `rust-wallet/src/monitor/task_purge.rs :: run` (confirmed `parent_transactions` > 7 days deleted) | ✅ | Exemption or on-demand re-fetch |
| `rust-wallet/src/recovery.rs :: recover_wallet_from_mnemonic` (`fetch_all_utxos(..).await.unwrap_or_default()`, `if let Ok(true) = address_has_history`) | ✅ | Error honesty |
| `rust-wallet/src/handlers.rs :: wallet_rescan`, `wallet_recover` | ✅ | Callers that must report "incomplete" |
| `rust-wallet/src/backup.rs :: collect_payload` (`script_length, script_offset, locking_script` in the outputs select) | ✅ | Backups get smaller for long scripts — a T3a edge, not a T3 format change (the columns already exist) |
| Signing: any path that uses `outputs.locking_script` as the prevout script for a sighash | ⬜ to enumerate at kickoff | ⚠️ ForkID SIGHASH needs the **full** prevout script. Money coins are 25 B (under any cap) — but an offloaded script must be read through the accessor, never from the empty column. T2-P3 (ordinal transfer) depends on this |

## 6. Out of scope

- The classifier itself and any stamping of `change=1` — P5. This phase only makes the evidence exist.
- A cross-referenced second explorer (the owner's 2026-09-16 proposal; "banana blocks" not identified) — §12 Q2.
- Timeout/partial bulk failures (the owner's second complaint): **not measured** ⇒ not fixed from a sentence (ticket). If P4-A7's work reproduces one, it becomes a row here; otherwise a ticket.
- Rewriting `TaskSyncPending`'s tiering.

## 7. Rollback

Revert the ingest commit: new rows go back to the fabricated script; rows written by P4 keep true data, which the old code reads correctly (a longer `locking_script` or an empty one with `script_length` set). ⚠️ An old binary reading an **offloaded** row (empty `locking_script`) cannot spend it — acceptable only because offloaded rows are, by construction, non-P2PKH (not money). State that in the revert commit.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Row that catches it |
|---|---|
| The "real" script is still derived from the address somewhere, the classifier sees 25 bytes for everything, `R-CLASSIFY` goes green vacuously | `P4-A1` asserts the **length**; `P4-A12` grep |
| An indexer returns a wrong raw tx (or wrong vout) and we store someone else's script as ours | `P4-A4` (txid hash + value check) |
| We store 2.6 MB inscriptions in `outputs` and every on-chain backup balloons | `P4-A6` (cap) |
| `TaskPurge` deletes the parent of an offloaded script; seven days later the ordinal cannot be shown or transferred | `P4-A6` after a forced purge |
| The re-fetch of 20-coin addresses hits the rate limit and the whole sync slows for a wallet with many busy addresses | `P4-A8` (only exactly-20 addresses) + the sync duration recorded at the boundary |
| A failed parent fetch writes "not observed" and nothing ever retries it — the coin is Unknown forever (decision 7: *nothing stays unclassified for good*) | `P4-A5` |
| The rescan now honestly says "incomplete" but the Tools tab (T6) renders it as an error dialog with no next step | T6 edge; `P4-A11` SUBJECT is the body T6 renders |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | A1–A12 | Primary |
| macOS | A1–A6, A9, A10 (T1) + `cargo test` | Rust only; the T2 money rows (A7's real 22-output tx, A1's real inscription) run once, on Windows — no platform code in this phase. Relay round names this contract |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1 h** — a 22-output transaction to one scratch address; receive one real inscription at a scratch address |
| Human-bound rows | A7 (T2 half), A1 (T2 half) — real outputs, cents; scratch profile (HARNESS_DELTA §1.2) |
| Unknowns (K) — uncertainty, not difficulty | **K = 1.** (a) The cap value vs T2's need (≥ ~256 B for the BSV-21 JSON body; T2-P1's own K) and the purge-exemption vs re-fetch choice; (b) whether WhatsOnChain's bulk endpoint pages (**unverified**; the exactly-20 re-fetch does not depend on it) |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| gives → | **B5-T2-P1** (classify & file) | ⭐ **Real scripts**, full up to the cap (T2 needs ≥ ~256 B of prefix for the BSV-21 JSON body; recommend a cap ≥ 1,024 B, §12 Q1) and an explicit "not observed". **Blocking** for T2-P1's sync/restore rows |
| gives → | B5-T1-P5 | The observation the classifier reads, and "not observed" ⇒ Unknown |
| gives → | B5-T1-P6 | Error-honest scans (NC-5), >20 coins per address (NC-8) |
| gives → | B5-T2-P3 (transfer) | Offloaded scripts are read through the accessor for signing |
| gives → | B5-T3a | Backups carry `script_length`/`script_offset` and a bounded `locking_script` — no new fields, smaller payloads |
| gives → | B5-T6 (Tools card 2) | `/wallet/rescan`'s "incomplete" response shape |

## 12. Open questions for the owner

| # | Question | Recommendation |
|---|---|---|
| **Q1** | Largest script kept inline in `outputs` (the toolbox's `maxOutputScript`), and for longer ones: keep their raw transaction from being purged, or re-fetch it when needed? | **1,024 bytes inline; exempt referenced parents from purge.** 1 KB covers every money coin, OrdLock listings (860 B) and T2's BSV-21 prefix; pinning is simpler to test than a re-fetch that can fail when the user is offline. Our unused `settings.max_output_script` (default 500000) becomes the knob, set to 1,024 |
| **Q2** | The second explorer you named on 2026-09-16 ("banana blocks" as transcribed) — which service? | Not needed for this phase (the exactly-20 re-fetch fixes the measured case). If you want a cross-check for completeness, name it and it becomes its own ticket with a defined disagreement rule (fail closed) |

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded
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
