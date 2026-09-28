# B5-T1-P6 — a late payment to an address the wallet handed out is found, once, and a restore never invents a coin · PHASE CONTRACT

**Track:** B5-T1 Money path · **Tickets:** `../../tickets/TICKET_rescan_cannot_find_payments_to_generated_addresses.md` (⭐ owner priority) · `../../tickets/TICKET_two_next_address_index_sources_can_reuse_addresses.md` · the restore-side gaps G1/G2/G3/G7 (`../SCOPE.md` §5.3) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **T1 Q5** (owner, 2026-09-27): *the rescan stops writing BIP-32 addresses into `addresses` — nothing should; a BIP-32 hit is **swept into a BRC-42 address**; design here: automatic or confirmed; a sweep is a normal spend (network fee + the 1,000-sat service fee).* **T1 Q7**: the two-index-sources ticket is in this phase. **T1 Q6** (agent-level, approved): P6 edits T3's restore path — **T1 writes, T3 reviews**. **7** (every scan-found coin goes through the P5 classifier). Owner, 2026-09-25: *the phantom-coin cause is fixed at its root, not bypassed — research it, reproduce it, and write negative controls that go red when the fix is removed.*

---

## 1. Goal

A user who handed out an address the wallet generated — and was paid after the wallet stopped watching it, or after a restore dropped it — can press one button and have that payment found and credited **once**; and no scan or restore can ever make the wallet hold a coin the chain says is spent.

## 2. Done means

- [ ] **Two pure scan functions**, separately tested, **no DB writes** (SCOPE §5.4): `scan_bip32(seed, 0..=hw+margin)` and `scan_brc42_self(master, 0..=hw+margin)` with invoice `2-receive address-{i}`, each returning *found coins* + *failed ranges* (P4's error honesty). `scan_brc42_self` revives `recovery.rs :: derive_brc42_address` (**zero callers** today — not a third copy); `scan_bip32` is extracted from `recovery.rs :: recover_wallet_from_mnemonic`.
- [ ] ⛔ **BRC-42 is scannable only for counterparty = self.** A PeerPay / BRC-29 key (sender's key, random prefix/suffix) cannot be found by any scan; the Tools-tab copy says so (T6 card 2, NC-9).
- [ ] **One writer** shared by both scans. For each hit it (1) re-verifies ownership against the **chain's** locking script (P4's observed script; `reconcile.rs :: verify_receive_index` for BRC-42, the BIP-32 equivalent via `recovery.rs :: derive_private_key_bip32`); (2) asks `reconcile.rs :: check_outpoint_spent` and inserts only a coin the chain says is **unspent** (`Unknown` ⇒ do nothing, report it); (3) inserts **only unknown outpoints** — `output_repo.rs :: upsert_received_utxo_with_derivation`'s `INSERT OR IGNORE` on `UNIQUE(txid, vout)`, with a negative control this time; (4) sends each row through P5's seam; (5) records `derivation_prefix/suffix` from **what was verified**, never from the index; (6) sets `purpose` to a scan-specific value so every scan-written row is **findable afterwards** (SCOPE §8.3 — rollback is a query).
- [ ] > **The rule the writer enforces (SCOPE §5.2):** *a scan may only ever ADD a coin the chain says is unspent and that we can prove we can sign for. It may never mark a known coin spendable again, and it may never report "none" when it could not ask.*
- [ ] **BIP-32 (decision Q5):** no scan or restore writes a BIP-32 row into `addresses` (today `wallet_rescan` and `wallet_recover` both do, colliding with BRC-42 in `UNIQUE(wallet_id, "index")` — SCOPE suspicion 3 / G3). A verified BIP-32 hit becomes a coin row labelled `bip32`, **held** (not in the money index), and is **swept to a fresh BRC-42 receive address** by a **user-confirmed** action (§12 Q1) that is a normal spend: network fee **plus** the 1,000-sat service fee. A hit too small to cover both is shown, not swept.
- [ ] **One next-index source.** `handlers.rs :: generate_address` (`current_index + 1`) and change derivation in `create_action_internal` (`address_repo.rs :: get_max_index() + 1`, with self-heal) read **one** function; the scan's high-water mark is that same number (`max(wallets.current_index, MAX(addresses.index ≥ 0))`) plus a margin of 20. No gap limit below the mark.
- [ ] **Restore restores the counter.** `handlers.rs :: wallet_recover_onchain` writes the payload's `wallets.current_index` back (today the restored row is created with a literal `0` in `wallet_repo.rs` and the payload's value is thrown away); `handlers.rs :: wallet_import`'s `UPDATE wallets … WHERE id = payload.wallet.id` targets the **backup's** old id and discards the result — it updates the **restored** row instead. (Confirmed by the orchestrating session; **not yet measured** — `P6-A10` measures it.)
- [ ] **The derivation scan runs once after every restore** and on the Tools-tab button; the restore report says whether it ran, and what it could not check.
- [ ] **`reconcile_backup_tx` is completed (G1, G2):** its failure is no longer *"sync will fix later"* (the named healer `TaskValidateUtxos` no longer exists — removed in `9ba106b`) — a failed parse is retried and, if it still fails, the restore report says the wallet may hold a coin the backup itself spent, and **the scan does not run** until it succeeds (the scan would otherwise surface the phantom's twin). Its change lookup **re-derives** with `verify_receive_index` instead of reading `addresses` (a stripped address row today ⇒ `change_addr_index = -1` ⇒ the backup's change coin **silently not inserted**). Its "restore falsely marked external-spend" `UPDATE` (P5 route W2) answers from `check_outpoint_spent`, not a string heuristic.
- [ ] **The March phantom is reproduced, then shown fixed** (NC-11, scratch wallet, residue declared).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| rule 7 trip-wire 1 | no money-path row in a state no code expects | The phantom **is** this shape — a spendable row the chain says is spent. NC-1, NC-2, NC-11 |
| `R-RESTORE` (a, b) | fail-closed survives recovery; nothing silently lost | Restore path edited (`reconcile_backup_tx`, counter); NC-3 is the "silently lost" twin |
| `R-CLASSIFY` | classification on every ingest route | Scan and restore-scan writers are routes I2/I3 (P5) — NC-7 |
| `R-NOSPEND` | no automatic path spends an unclassified output | The BIP-32 sweep is a spend; it is **user-confirmed**, and it spends only verified plain P2PKH `bip32` coins, never a token found at a BIP-32 address |
| `R-DUST` | no incidental 1-sat spend | Sweep inputs obey the floor |
| service fee (root `CLAUDE.md`) | every outgoing transaction pays 1,000 sats | The sweep is a new spend path — it must pay it (owner, Q5) and appear in the fee table (§11) |
| address privacy | no address reuse | The counter fix exists to stop reuse after restore |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. A green result is reported with its red half or not at all.
⛔ Money, schema and crypto rows: the RED (negative control) is **designed by someone other than the assertion's author** — a second agent (`../../../RELEASE_CYCLE.md` §4.2). Record who designed it. ⭐ `SCOPE.md` §5.5 already asks that the phantom controls be designed by someone other than whoever writes the fix — the second agent designs NC-1…NC-11's reds from the SCOPE's sketches, it does not inherit them.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P6-A1` (NC-1 ⭐ the March bug) | Import payload P (X spendable at addr i) → `reconcile_backup_tx(T)` → BRC-42 scan (fake chain: Y at addr i). **Exactly one** spendable row at addr i (Y); balance = Y; X `spendable=0`, `spent_by` = T's row | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Rows X and Y by outpoint + the balance figure | T1 (money) | ⬜ |
| `P6-A2` (NC-2 never resurrect) | Row Z `spendable=0`, `spent_by` set; stale fake indexer lists Z unspent; after the scan Z is still spent | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Row Z | T1 | ⬜ |
| `P6-A3` (NC-3 stripped change address) | Payload lacks the address row for Y's index; after restore, Y is present and spendable | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Row Y | T1 | ⬜ |
| `P6-A4` (NC-4 signable or not at all) | For every row a scan inserts, the key re-derived from `(prefix, suffix)` produces a P2PKH equal to **the chain's** locking script | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | ⚠️ Compare against the **chain's** script (P4), never the stored one — a fabricated stored script makes this pass vacuously (`feedback_check_whether_a_stored_value_is_an_observation`) | T1 (crypto: derivation) | ⬜ |
| `P6-A5` (NC-5 an error is not an answer) | Fake fetcher `Err` for batch k ⇒ scan returns *incomplete* with the range; gap counter not advanced | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The scan's return value (shared with `P4-A9`, re-run on the new functions) | T1 | ⬜ |
| `P6-A6` (NC-6 high-water, not current_index) | `current_index=5`, `MAX(addresses.index)=12`, payload counter restored = 30, coin at 28 ⇒ found | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The coin at 28 | T1 | ⬜ |
| `P6-A7` (NC-7 classify on scan) | A 1-sat inscribed output at addr i lands Token/Unknown, not money | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The row's class (P5 route list gains "rescan" and "restore scan") | T1 | ⬜ |
| `P6-A8` (NC-8 >20 coins) | An address with 22 coins ⇒ 22 rows | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Count by outpoint | T1 | ⬜ |
| `P6-A9` (NC-9 self only) | A BRC-29 payment (foreign sender, random prefix) is **not** found, and the Tools-tab card text and the rescan response both say PeerPay payments are found through *Claim a payment*, not the scan | Delete the sentence from the card ⇒ the string assertion fails (seen) | The rendered card copy (T6) and the response's `note` field | T1 + T3 | ⬜ |
| `P6-A10` | After an **on-chain** restore **and** after a **file** import, `wallets.current_index` of the restored row = the payload's value, and the next `generate_address` returns an index **above** every index the old wallet used | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The restored row's `current_index` and the next generated index; ⭐ measure today's value **first** on a scratch restore (the orchestrator's "not yet measured") — that is the red | T2 (scratch) | ⬜ |
| `P6-A11` | `generate_address` and change derivation return the same next index from one function; two sequential calls of each never repeat an index | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The indices handed out, across a mixed sequence of generate + send | T1 | ⬜ |
| `P6-A12` | No scan or restore writes a BIP-32 row into `addresses`; a BIP-32 hit becomes a held `bip32` coin row | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | `SELECT` rows of `addresses` whose address does not re-derive under `2-receive address-{index}` ⇒ 0; the hit's row class | T1 | ⬜ |
| `P6-A13` | BIP-32 sweep (user-confirmed): inputs = only verified `bip32` coins; outputs = one fresh BRC-42 receive address + 1,000-sat fee to `HODOS_FEE_ADDRESS` (+ change if any); the swept coin arrives, classified money | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The **serialised transaction** (inputs, outputs, fee output) and WhatsOnChain for it; scratch wallet seeded by paying `m/i` from outside | T2 (money) | ⬜ |
| `P6-A14` | `reconcile_backup_tx` failure ⇒ retried; persistent failure ⇒ restore report says so and the scan does not run | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The restore report and the absence of scan log lines | T1 | ⬜ |
| `P6-A15` (NC-10 ⭐ owner's acceptance) | Pay an old generated address that the backup stripped; restore onto a scratch profile; the scan credits it **once** | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) — the ticket's own RED: today's `/wallet/rescan` finds **nothing** (run it first and record it) | `/wallet/balance` + the `outputs` row + WhatsOnChain for the outpoint | T2 (money) | ⬜ |
| `P6-A16` (NC-11 reproduce March) | Scratch wallet: backup T1 → backup T2 (funded by T1's change) → restore with a **dev-only** switch skipping `reconcile_backup_tx`, scan on ⇒ X + Y (the phantom); switch back ⇒ Y only | This row **is** the red half; the second agent confirms the switch is compiled out of release builds (`HODOS_DEV` gate, cf. `P1-A9b` in T4) | ⛔ **Ask the chain first:** WhatsOnChain outspend for X must show it spent by T2 before X is called a phantom (rule 7). 🚨 **Residue declared:** this row deliberately creates a trip-wire-1 row; scratch profile, deleted afterwards, and this cell says so | T2 (money, fault injection) | ⬜ |

**Two-sided rows:** `P6-A1` (phantom gone) ⟷ `P6-A3` (real coin kept) — deleting the scan passes A1 and fails A3, which is exactly what March did. `P6-A15` (late payment credited) ⟷ its "once" half and `P6-A2` (nothing resurrected). `P6-A13` sweep happens ⟷ sweep refused for a coin too small to pay the fees.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/recovery.rs :: recover_wallet_from_mnemonic` (BIP32 only), `derive_brc42_address` (**0 callers**), `derive_private_key_bip32`, `scan_external_wallet`, `build_sweep_transactions`, `split_token_reserved` | ✅ | Scans extracted/revived. ⚠️ `build_sweep_transactions` (external sweep) pays **no** service fee today (grep: no `HODOS_SERVICE_FEE` in `recovery.rs`) |
| `rust-wallet/src/reconcile.rs :: derive_receive_p2pkh_script`, `verify_receive_index`, `check_outpoint_spent`, `parse_tx_outputs` | ✅ | The verification primitives (review finding "D-K1" comment above `derive_receive_p2pkh_script` names the collision hazard) |
| `rust-wallet/src/handlers.rs :: wallet_rescan`, `wallet_recover` (both `address_repo.create` BIP32 rows + `upsert_received_utxo_with_derivation`) | ✅ | Stop writing BIP32 address rows |
| `rust-wallet/src/handlers.rs :: wallet_recover_onchain` (calls `reconcile_backup_tx`, logs *"sync will fix later"* on failure), `reconcile_backup_tx`, `wallet_import` | ✅ | T3's restore path — **T1 writes, T3 reviews** |
| `rust-wallet/src/handlers.rs :: generate_address` (`current_index + 1`), `create_action_internal` change index (`get_max_index` + self-heal via `update_current_index`), `do_onchain_backup` backup change (`get_max_index` → `2-receive address-{max}`) | ✅ | One next-index function |
| `rust-wallet/src/database/address_repo.rs :: get_max_index`, `get_by_wallet_and_index`, `clear_stale_pending_addresses`; `wallet_repo.rs :: update_current_index` | ✅ | |
| `rust-wallet/src/database/output_repo.rs :: upsert_received_utxo_with_derivation` (`INSERT OR IGNORE`) | ✅ | The never-resurrect property, now with a control |
| `rust-wallet/src/backup.rs :: collect_payload` (carries `wallets.current_index`; address time-tiered strip) | ✅ | Read, not changed |
| `rust-wallet/src/main.rs` — first-party-only list | ✅ | ⚠️ `POST /wallet/rescan` is **not** on it (T5 denylist ticket lists it among the 30) — a site could trigger a scan. Edge to T5-P2 |

## 6. Out of scope

- A general BRC-42 recovery tool (other counterparties) — impossible by construction; card 1 (*Claim a payment*) covers PeerPay.
- Moving backup change off the user's newest receive address (G8 — a privacy issue: the backup is linkable to an address the user handed out). Flagged to T5; no change here.
- Watching BIP-32 addresses for 90 days (the alternative to Q5 that the owner declined).
- The Tools-tab UI itself — T6 card 2 (this phase provides the endpoint and its response).
- Backup format changes — T3.

## 7. Rollback

Every scan-written row carries a scan-specific `purpose` ⇒ `SELECT … WHERE purpose = '<scan>'` finds them; a rollback is a revert commit plus that query reviewed by the owner (never an automatic delete — rule 7). The counter fix and `reconcile_backup_tx` changes revert with their commits; ⚠️ a counter already restored stays restored (that is the correct value).

## 8. Pre-mortem (adversarial review — before)

| Failure story | Row that catches it |
|---|---|
| The scan is switched back on and the March phantom returns because `reconcile_backup_tx` failed silently on one restore | `P6-A14` (no scan until reconcile succeeds) + `P6-A1` |
| The fix is "don't insert at backup change addresses" — the phantom goes and so does the real change coin (March's mistake, again) | `P6-A3` |
| A stale indexer revives a spent coin through an upsert that sets `spendable=1` | `P6-A2` |
| A BIP-32 hit is stored as `2-receive address` and the wallet cannot sign for it (G3) | `P6-A4`, `P6-A12` |
| The scan reports "nothing found" when the indexer was down | `P6-A5`, `P4-A11` |
| After restore, generate-address hands out an old address again (privacy) | `P6-A10`, `P6-A11` |
| The sweep builder is reused without the service fee — a silent revenue gap and a fee-table lie | `P6-A13` asserts the fee output |
| NC-11's dev switch ships in a release build | `P6-A16` RED cell |
| An approved site calls `/wallet/rescan` repeatedly and hammers the indexer / writes rows | Not caught here — T5-P2 edge (§11) |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | A1–A16 | Primary |
| macOS | A1–A8, A11, A12 (T1) + one on-chain restore on a scratch macOS profile (A10) | Rust only; the money rows (A13, A15, A16) run once, on Windows. Relay round names this contract |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~2.5–3 h** — backup → restore on a scratch profile; pay an old generated address; the March reproduction (NC-11); one BIP-32 sweep |
| Human-bound rows | A13, A15, A16 (real money, scratch profiles); A9's card text (with T6) |
| Unknowns (K) — uncertainty, not difficulty | **K = 1.** (a) Where "a restore happened, run the scan" lives if the scan does not complete in the restore call (§12 Q2 — no-schema answer recommended); (b) the T3 edge — T3a-P2 may restructure the restore flow this phase edits |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs ← | B5-T1-P4, P5 | Error-honest fetch, >20 coins, observed scripts; the classifier (nothing writes scan results before P5 — SCOPE §7) |
| ↔ | **B5-T3a-P2.3** (trustworthy backup / restore) | ⭐ **T1 writes, T3 reviews** (Q6): `reconcile_backup_tx`, the counter write-back, the scan-after-restore hook, the address strip it depends on. T3's restore must **classify before rebuilding** the money index (old backups carry `change=0`). If T3a-P2 moves backup change off the receive address (G8), NC-1/NC-11's fixture changes |
| gives → | **B5-T6-P5** (Tools tab card 2) | The scan endpoint + its response (found / incomplete ranges / the self-only note). Card copy must carry NC-9's sentence |
| gives → | **B5-T5-P2** (dApp-reachable surface) | `POST /wallet/rescan` is reachable by an approved site; recommend first-party-only |
| gives → | B5-T5 (privacy) | G8: backup change goes to the user's newest receive address — linkable. Flag only |
| gives → | root `CLAUDE.md` "Wallet Service Fee" table | The BIP-32 sweep becomes a **fifth** builder that pays the fee — the table is updated in the same commit (invariant 11) |

## 12. Open questions for the owner

| # | Question | Recommendation |
|---|---|---|
| **Q1** | BIP-32 hit ⇒ sweep to BRC-42: **automatic** or **user-confirmed**? | **User-confirmed.** It spends the user's money (network fee + 1,000-sat service fee) on a coin they may not know about; the scan result shows *"found N sats at an old-style address — move it into your wallet?"*. Automatic would be a scheduled fee-paying spend the user never triggered (the dust consolidator's known wart) |
| **Q2** | "Run the scan once after a restore" needs a trigger that survives a crash between restore and scan. A new settings column is a schema change | **No schema:** the restore call writes the counter back and runs the scan in the same flow; the restore report says whether it ran; if it did not, the Tools-tab button is the recovery. Choose a persisted marker (column) only if you want the scan to resume by itself after a crash |
| **Q3** | Reuse the external-sweep builder (`build_sweep_transactions`) for the BIP-32 sweep and add the service-fee output to it — or build the sweep through the normal send path? | **Reuse the sweep builder, add the fee output** — it already signs with non-BRC-42 keys and already splits token-reserved coins; the normal send path would need BIP-32 signing added to it. Either way the fee table in root `CLAUDE.md` gains a row |

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
