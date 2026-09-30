# B5-T1-P5 — a coin is money only when the wallet has shown it is, and what it cannot tell is shown, not spent · PHASE CONTRACT

**Track:** B5-T1 Money path · **Tickets:** the guard scope `../utxo-safety-guard/README.md` (items 1.1–1.5); `../../REGRESSION_ADDITIONS.md` `R-NOSPEND`, `R-CLASSIFY`, `R-RESTORE` · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **2** (option B — money only when **positively marked** `change=1`; tokens filed in BRC-147 baskets; unclassified = **Unknown: held, shown, never auto-spent**; the contract must carry (a) a one-time data migration stamping `change=1` on rows that pass the classifier, (b) the `change` predicate at every selector route **and** `send_max`, (c) every ingest route classifies — and the `main.rs` master fix-up's fate is decided here, (d) **the balance display includes Unknown coins**). **2a** (selection reads only the money index — P3 builds it; this phase switches its membership to the stamp). **7** (tiered rule: ① real script (P4) → ② multi-sat **plain** P2PKH, no inscription ⇒ money, released **on evidence, never on an indexer's word alone** → ③ 1-sat unreadable ⇒ held → ④ unresolved ⇒ **shown** under the balance, automatic retry, manual **"treat as money"** → ⑤ restore never fails because of unidentified coins). **8** (T2 recognisers run inside this seam). Owner: *"nothing stays unclassified for good."*

---

## 1. Goal

Every coin that enters the wallet — by sync, rescan, restore, PeerPay, a dApp or our own change — is classified once at the door; only a coin shown to be money can be spent by any path, and a coin the wallet cannot yet identify is visible to the user, retried, and never spent without the user saying so.

## 2. Done means

- [ ] **One seam.** A single function every ingest route calls (working name `classify_output`) returning `Money` / `Token{basket, tags, customInstructions}` / `Unknown{reason}`. Order inside it (agreed with T2 — `../../track-2-1sat-ordinals/phase-P1-classify-and-file/PHASE_CONTRACT.md` §11 row "T1-P5 ↔ T2-P1"): **T2's token recognisers first** → rule ② → rule ③ → otherwise Unknown. T1 owns the seam, the stamp, Unknown and its display; T2 owns recognisers and filing.
- [ ] **Rule ② is an exact template:** money only when the observed script (P4) is **exactly** the 25-byte `OP_DUP OP_HASH160 <20> OP_EQUALVERIFY OP_CHECKSIG` template **and** `satoshis > 1` **and** the hash is one of ours (re-derived from the row's derivation — `reconcile.rs :: derive_receive_p2pkh_script` / `verify_receive_index`). "Starts with" or "ends with" P2PKH is **not** money (T2-P1: BRC-162 puts a P2PKH after a token prefix, at any satoshi value). A script `script_length IS NULL` (not observed, P4) is **never** money.
- [ ] **(a) Migration.** A one-time pass classifies existing rows using P4's observed scripts (fetching parents where needed), stamps `change=1` on money, files what T2 recognises, and leaves the rest Unknown **and shown**. It runs in the background after upgrade; until a row is classified it is **held and shown**, never silently dropped from view. Received payments that already carry `change=0` are the bulk of this (every `upsert_received_utxo*` writes `change=0`).
- [ ] **(b) Selection.** The P3 money index's membership becomes `change=1` (plus today's filters). With P3 in place this is **one** membership rule, and the six route controls from P3 (`P3-A2a…f`) are re-run against the new rule.
- [ ] **(c) Every ingest route classifies.** Starting route list (grep 2026-09-28; deliverable 1.1 is the proven-complete list, with every route **not** covered named):

  | # | Route (`file :: symbol`) | Writes via |
  |---|---|---|
  | I1 | `monitor/task_sync_pending.rs :: check_addresses_individually`, `check_addresses_bulk` | `upsert_received_utxo_with_confirmed` |
  | I2 | `handlers.rs :: wallet_rescan`, `wallet_recover` (seed scan) | `upsert_received_utxo_with_derivation` |
  | I3 | `handlers.rs :: wallet_recover_onchain` → `reconcile_backup_tx`; `backup.rs :: import_entities`; `handlers.rs :: wallet_import` | `insert_output` / raw `INSERT` |
  | I4 | `handlers.rs :: internalize_action` (wallet payment **and** basket insertion) | `store_derived_utxo` / `insert_output` |
  | I5 | `monitor/task_check_peerpay.rs :: run` (and T6-P5's claim card, which calls the internalize path) | `store_derived_utxo` |
  | I6 | `handlers.rs :: create_action_internal` own change; `do_onchain_backup` change; `unpublish_certificate_core` change; `task_consolidate_dust` output | `insert_output(is_change)` — money **by construction** (the wallet built the script); still passes through the seam so the route is on the list |
  | I7 | `handlers.rs :: wallet_sync`, `reconcile_spent_inputs` | `upsert_received_utxo*` |
  | I8 | `handlers.rs :: wallet_recover_external` (external-wallet sweep; `recovery.rs :: split_token_reserved`) | `upsert_received_utxo*` |
  | I9 | `handlers/certificate_handlers.rs :: admin_prepare_unpublish`; `handlers.rs :: debug_repair_nosend`, `debug_broadcast_nosend` | raw `INSERT` / `insert_output` (first-party-only `/wallet/debug*`) |
  | W1 | `main.rs :: main` — startup `UPDATE outputs SET derivation_prefix='master', derivation_suffix='-1' WHERE derivation_prefix IS NULL … AND spendable=1 AND satoshis != 546` | **Deleted.** It promotes unidentified rows to master-key signable on every boot (SCOPE suspicion 2). Rows it would have fixed are classified by the seam: money **only if** the master key's P2PKH equals the observed script |
  | W2 | `handlers.rs :: reconcile_backup_tx` — the "restore falsely marked external-spend" `UPDATE` flipping `spendable 0→1` for non-default-basket rows by string heuristic | Replaced by a chain answer (`reconcile.rs :: check_outpoint_spent`) or removed — P6 owns the edit (it is T3's restore path; T1 writes, T3 reviews) |
  | D1 | **dApp-named inputs** (`createAction.inputs` + `inputBEEF`) | Not ingest — a spend authority. Checked against this wallet's classification of that outpoint; a Token/Unknown outpoint named by a dApp goes to **T2's token-spend permission** (BRC-165: per action, pay grants never authorize it) |

- [ ] **(d) Balance and display.** The wallet panel shows **money** as the balance and, directly under it, *"N items being identified"* when Unknown > 0, with each Unknown coin listed (outpoint, value, reason) and a **"Treat as money"** action. `calculate_balance` and `/wallet/balance` keep the money figure; a separate field carries held/Unknown so no coin disappears from view. Automatic retry of Unknown coins runs on the sync cadence.
- [ ] **"Treat as money" is a confirmed user action**, never automatic: it stamps `change=1` on that one row, records who/when in `output_description` or a tag (no schema change), and is refused for a coin T2 recognised as a token.
- [ ] **The activity feed keeps "received" correct.** `handlers.rs :: wallet_activity` today identifies received payments by `o.change = 0` — the only non-backup reader of `change`. Stamping received payments `change=1` would drop them from the feed. The feed switches to a discriminator that does not change meaning under decision 2 (e.g. the output is not change **of one of our own outgoing transactions**: `o.transaction_id` NULL or `t.is_outgoing = 0`), proven by a row.
- [ ] **The beta.3 1-sat floor stays** as defence in depth (`utxo_fetcher.rs :: is_token_reserved_value`), and `R-DUST`'s RED is **re-based** on the classifier (T2-P1 asked T1 to own this): R-DUST must go red when the *classifier* is bypassed, not only when the floor is.
- [ ] **CVE-2026-56744 question answered in writing** (SCOPE T-10: wallet-toolbox did not verify recipient scripts supplied by storage in `createAction`): does any of our spend routes take a script from storage for an output it pays **to someone else** without re-deriving or validating it? Answer + evidence in the result column of `P5-A13`.
- [ ] **The candidate T0 gate** (`HARNESS_DELTA.md` §3 — "UTXO-selecting paths that do not consult the classification") is **accepted**: after P3+P5, no code outside the index module reads `outputs` to choose coins. Baselined **by `preflight.ps1` itself**, in **its own commit** (root `CLAUDE.md` rule 6), with `-NegativeControl` run.

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-NOSPEND` (a, b, c) | no automatic path spends an unclassified output | The guard this phase builds. Three paths, three reds (`P5-A5a–c`) |
| `R-CLASSIFY` | classification reaches every ingest route | The seam; `P5-A6` per route, the uncovered named |
| `R-RESTORE` (a, b) | fail-closed survives recovery; nothing silently lost | Restore routes I3; `P5-A9` is T1's half (T3a owns the full row) |
| `R-DUST` | no incidental 1-sat spend | Floor stays; RED re-based |
| `R-TOKENPERM` | pay grants never authorize a token spend | D1 hands dApp-named token inputs to T2's class |
| `R-GOLD` / `R-COUNT` | pill; counters | "Treat as money" and the migration must not look like a payment |
| balance shown to the user (SCOPE §8.1) | a reclassification that hides money looks like theft | (d) — `P5-A8` |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. A green result is reported with its red half or not at all.
⛔ Money, schema and crypto rows: the RED (negative control) is **designed by someone other than the assertion's author** — a second agent (`../../../RELEASE_CYCLE.md` §4.2). Record who designed it.
⭐ `HARNESS_DELTA.md` §1.1: every SUBJECT names the **output** — txid, vout, satoshis, basket, class. Record fixture outpoints here before the run.
⭐ **Pre-phase baseline:** run `R-NOSPEND` on today's code and record it **RED** (`REGRESSION_ADDITIONS.md` boundary table, first row).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P5-A1` | Seam table: exact 25-byte P2PKH to our key, >1 sat ⇒ Money; same with 1 sat ⇒ held; P2PKH + `ord` envelope ⇒ Token/held (T2); BRC-162 prefix + P2PKH ⇒ never Money; `script_length NULL` ⇒ Unknown; P2PKH to a key that is **not ours** ⇒ Unknown | Four switches, one per non-money case, each run alone on the same fixture set — a case whose verdict does not move under its switch is untested: ① replace the seam's exact-25-byte comparison with `ends_with(P2PKH)` ⇒ the BRC-162 fixture (real script, length stated ≠ 25) turns Money · ② stub the ownership check (`reconcile.rs :: verify_receive_index` / the hash re-derivation) to `true` ⇒ the not-ours P2PKH turns Money · ③ drop `satoshis > 1` ⇒ the 1-sat exact-P2PKH turns Money · ④ let a `script_length IS NULL` row fall through to `outputs.locking_script` ⇒ the NULL fixture (which carries the fabricated 25-byte script) turns Money · Right reason: the seam's per-fixture log names the rule that fired (`rule2:exact` / `rule2:owner` / `rule2:value` / `not_observed`); a fixture that goes Unknown because derivation errored is a harness fault, not a red · Residue: none (pure function, in-memory rows) — designed by controls-B (Fable), 2026-09-28 | Seam output per fixture; fixtures are **real chain scripts** (P4's outpoints), length stated | T1 (money) | ⬜ |
| `P5-A2` | ⭐ **The default is refusal** (`R-CLASSIFY` RED, made observable by decision 2): a row inserted by a route that skips the seam lands `change=0` ⇒ not in the index ⇒ selected by **no** route | Two switches, both required — the write default and the read predicate each guard the other: ① in a scratch build remove the `change = 1` term from the money-index membership (P3's index query) ⇒ the seam-skipping row (a raw `INSERT` mirroring route I9, `change=0`) is returned by every route S1–S6 — the row itself, by outpoint, in the serialised selection · ② keep the predicate and make the raw insert write `change=1` ⇒ same row selected — the stamp is a plain column any route can forge, which is why `P5-A14` exists (§4a) · Right reason: the selection log lists the fixture outpoint; seed one ordinary money coin so S1–S6 select *something* every run — a route returning nothing on an empty wallet is not a red · Residue: none (scratch DB) — designed by controls-B (Fable), 2026-09-28 | The row by outpoint; the selection of every route S1–S6 | T1 | ⬜ |
| `P5-A3` | A fabricated 25-byte script (the pre-P4 value) with `script_length` unset does **not** classify as money (decision 7's named control: *the vacuous-pass trap*) | Fixture = a pre-P4-shaped row: `script_length NULL`, `locking_script` = the fabricated 25-byte P2PKH to one of our keys (`utxo_fetcher.rs :: generate_p2pkh_script_from_address` output), 5 sats · Switch: make the seam read `locking_script` when `script_length` is NULL (today's implicit behaviour) ⇒ the row classifies Money and is stamped `change=1` (seen) · Right reason: the seam's log names the field it read (`observed_script` vs `locking_script`); a second fixture with `locking_script` also NULL must go Unknown under **both** builds — if it does not, the test read neither field · Residue: none (in-memory) — designed by controls-B (Fable), 2026-09-28 | The row; which field the seam read, stated (REGRESSION_ADDITIONS `R-CLASSIFY` box) | T1 | ⬜ |
| `P5-A4` | Migration on a copy of the dev DB and a copy of a real beta.4 production-shaped DB: every row ends Money / Token / Unknown; **money + held + Unknown totals = the pre-migration visible total**; nothing vanished | Three switches on the migration, run on the **copies**: ① make it abandon a row after a parent-fetch failure without writing its Unknown state ⇒ that outpoint is in no class and the totals no longer balance (nothing may vanish) · ② make it set `spendable=0` on Unknown (the "hide what we cannot tell" shortcut) ⇒ the held/Unknown sum drops · ③ point it at the fabricated `locking_script` instead of P4's observed script ⇒ every synced row lands Money and Unknown = 0 on a copy known to hold token rows — name those outpoints first; a copy with zero token rows cannot run this control · ⚠️ The equation must include the **Token** class (T2-P4 `A6`), or a filed token makes the GREEN fail for a mis-stated reason (§4a) · Right reason: the per-class table shows *which* outpoint moved; a copy/lock error is a harness fault · Residue: none (copies deleted) — designed by controls-B (Fable), 2026-09-28 | Per-class counts and satoshi sums before/after, and the outpoints now Unknown listed by name; **copies only** | T2 (schema/data) | ⬜ |
| `P5-A5a` | `R-NOSPEND` a: the dust consolidator's **timer fire** on a scratch wallet with 20+ dust coins **plus** one real Unknown/token output consolidates the dust and leaves the token **unspent** | Two fixtures in the scratch wallet: (i) a real ≥2-sat inscription (observed script, length ≠ 25) and (ii) a pre-P4-shaped Unknown row (`script_length NULL`, fabricated 25-byte script, 2–`DUST_THRESHOLD_SATS` sats) — (ii) is what tests P5, because `task_consolidate_dust.rs :: is_p2pkh_script` (len == 25) already refuses (i) once P4 stores real scripts; (i) surviving is **not** evidence for this row · Switch: in a scratch build revert `run_inner`'s coin source from the money index to `output_repo.rs :: get_spendable_confirmed_by_user` ⇒ (ii) appears in the consolidation inputs · Fire: `monitor/mod.rs` `consolidate_dust` shortened **and** `last_consolidate_dust` initial value set to 0 (it skips the first tick); `disable_dust_consolidation` unset; the result must be `Consolidated{..}` — a `Skipped` run is a green that measured nothing · ⛔ Run the RED with `broadcast_transaction` stubbed; assert on the serialised inputs · Residue: scratch DB with (ii) marked spent by an unbroadcast tx (trip-wire-1 shape), profile deleted after; if broadcast for real, (ii) is destroyed and this cell says so — designed by controls-B (Fable), 2026-09-28 | The token output (txid, vout, sats, basket, class) after the fire; the fire is the `Monitor` schedule with its timer shortened in a dev build — **say which** | T2 (money, destructive — scratch profile) | ⬜ |
| `P5-A5b` | `R-NOSPEND` b: `wallet_recover_external` on an address holding a token + coins sweeps the coins, **excludes and reports** the token | Fixture: an external key holding a ≥2-sat inscription plus ordinary coins — multi-sat so the 1-sat floor cannot mask the classifier · Two layers, two switches: ① revert `recovery.rs :: split_token_reserved` to value-only ⇒ the inscription is among the sweep's inputs · ② keep ① fixed and revert `build_sweep_transactions`'s own filter to value-only, calling it with the unfiltered slice ⇒ same (it is `pub` and must not trust its caller) · "Reports" half: drop the withheld list from the response ⇒ the headline balance equals coins + inscription (seen) · Right reason: the serialised inputs name the fixture outpoint; a sweep refused for fee/dust reasons is not a red · ⛔ RED with broadcast stubbed · Residue: none if stubbed; if broadcast, the inscription is destroyed — declared — designed by controls-B (Fable), 2026-09-28 | The sweep transaction's inputs; the report shown | T2 (money, destructive) | ⬜ |
| `P5-A5c` | `R-NOSPEND` c: `select_utxos_with_preference` (every `dust_threshold_sats`) and `send_max` never take it | Fixture: a ≥2-sat token-shaped row (BRC-162 prefix + P2PKH, observed) classified Token/Unknown, beside money coins · Switch: revert the money-index membership to P3's rule (no `change` term) in a scratch build ⇒ `select_utxos_with_preference` takes it at some `dust_threshold_sats`, and `send_max` takes it unconditionally (`selected_utxos = all` — the P8-A6 shape) · Run every `dust_threshold_sats` value the code uses and say at which the switch selected it — a threshold whose greedy pass never reaches the coin passes vacuously · Right reason: the fixture outpoint in the serialised inputs · Residue: none (T1 in-memory; T2 with broadcast stubbed) — designed by controls-B (Fable), 2026-09-28 | Serialised transaction inputs | T1 + T2 | ⬜ |
| `P5-A6` | `R-CLASSIFY` per route I1…I9: an ordinal fixture arriving by that route ends Token or Unknown, never Money; a plain multi-sat payment arriving by that route ends Money | Per route, one switch: bypass that route's seam call in a scratch build ⇒ the plain multi-sat payment lands `change=0` (not money) — the GREEN's money half fails; the ordinal must still not be money (the write default is refusal) · ⚠️ Route I6 cannot be controlled this way: `insert_output` writes `change = is_change` by construction, so bypassing its seam call changes nothing — I6's switch is a change output whose script is **not** exact P2PKH (`unpublish_certificate_core`'s PushDrop-shaped output, or a hand-built script) ⇒ the seam must refuse it; if it lands Money the seam is not on that route (§4a) · W1 is proved by `P5-A7`, D1 by T2-P2 · Right reason: per route the stored row's `change` and the seam log line naming the route; a route not reached in the run (e.g. I5 with no MessageBox message) is INCOMPLETE, not green · Residue: none (scratch DB / fake chain) — designed by controls-B (Fable), 2026-09-28 | Per route, the stored row; the result lists every route **not** covered and why | T1 + T2 | ⬜ |
| `P5-A7` | The `main.rs` master fix-up is gone: a NULL-derivation spendable row survives a restart unchanged and unselectable; a genuine master-key output (service-fee style) whose observed script equals the master key's P2PKH is classified Money by the seam | Half 1: re-add the `main.rs :: main` `UPDATE outputs SET derivation_prefix='master', derivation_suffix='-1' … spendable=1 AND satoshis != 546` in a scratch build and restart ⇒ the NULL-derivation fixture (7 sats, observed P2PKH to a key that is **not** the master) is tagged `master/-1` and appears in `get_spendable_by_user` (seen) · Half 2: stub the seam's master-key byte comparison to `true` ⇒ that same not-master fixture classifies Money — proves the seam compares scripts, not the `master` label; and delete the seam's master case ⇒ the genuine service-fee-style output (observed script == master P2PKH) goes Unknown (seen), the stranding the pre-mortem fears · Right reason: `SELECT derivation_prefix, change` for both outpoints before/after restart · Residue: none (scratch DB) — designed by controls-B (Fable), 2026-09-28 | Both rows by outpoint across a restart | T1 | ⬜ |
| `P5-A8` | Display: with Unknown > 0 the panel shows the money balance **and** "N items being identified" listing each; the sum shown across both equals the pre-phase balance on an unchanged wallet | Hide the Unknown line (render money only) ⇒ the sum check fails and the reviewer sees a lower total (seen) | ⭐ Human eyes on the rendered overlay (`R-RESTORE` b in everyday use) + `/wallet/balance` JSON; ⚠️ Vite HMR fakes controls — hard-reload before each measurement | T3 (human) | ⬜ |
| `P5-A9` | `R-RESTORE` (T1's half): restore a scratch backup holding a 1-sat inscription with the indexer **down** ⇒ restore completes, the inscription is **held, shown, not spendable**, and the report says so (decision 7 ⑤ + its named control) | ⚠️ An on-chain restore needs the indexer to *find* the backup, so "indexer down" must be injected **after** `fetch_onchain_backup` returns (or use `wallet_import` from a file) — say which · Three clauses, three switches: ① map the seam's `Err` (parent unfetchable) to Money ⇒ the 1-sat inscription lands `change=1` (the trip-wire-2 shape) · ② make the restore skip rows the seam cannot classify ⇒ the inscription is absent from the restored DB and the held line (R-RESTORE b) · ③ make an Unknown abort the restore ⇒ restore returns an error (decision 7 ⑤) · Right reason: the restored row by outpoint with class and reason `indexer unavailable`, and the report line; a restore that fails because the backup fetch itself was cut off is a harness fault · Residue: scratch profile, deleted — designed by controls-B (Fable), 2026-09-28 | The **restored** DB row and the restore report | T2 | ⬜ |
| `P5-A10` | "Treat as money" on one Unknown row stamps exactly that row after a confirmation; refused on a T2-recognised token | Fixtures: two Unknown rows sharing a txid (vout 0 and 1), one T2-recognised token, one **held token-shaped** row (BRC-162 prefix, reason "token this wallet cannot show yet" — T2-P4 `A6`/`A12`) · Switches: ① drop `vout` from the stamp's `WHERE` ⇒ both Unknown rows stamped (exactly-that-row fails) · ② skip the T2 class check ⇒ the recognised token is stamped · ③ skip the held-token-shaped reason check ⇒ the BRC-162 row is stamped — the GREEN as written does not cover this case (§4a) · ④ omit the audit write ⇒ the trace assertion fails · ⑤ call the endpoint without the confirmation ⇒ must refuse; remove the check ⇒ it stamps · Right reason: `SELECT change, output_description` for all four outpoints; the refusal body names the class · Residue: none (scratch DB) — designed by controls-B (Fable), 2026-09-28 | The row's `change` + the audit trace; the refusal response for the token | T1 + T3 | ⬜ |
| `P5-A11` | Activity feed: a received payment stamped `change=1` still appears as **received** with the right amount; our own change still does **not** appear as received | Keep `o.change = 0` in `wallet_activity` ⇒ the received payment disappears from the feed (seen) | `/wallet/activity` JSON for that txid, before and after the stamp | T1 + T2 | ⬜ |
| `P5-A12` | `R-DUST` re-based: bypass the **classifier** (not the floor) ⇒ R-DUST's RED fires; bypass the **floor** alone ⇒ the classifier still refuses the 1-sat coin | Each bypass is itself the red — two, not one | R-DUST's SUBJECT (serialised inputs) | T1 | ⬜ |
| `P5-A13` | CVE-2026-56744 question: every spend route that pays an external script lists where the script comes from (request, derivation, storage) and whether it is validated; any "from storage, unvalidated" route is a finding in §12 | Re-read one route with a planted storage-sourced script in a T1 test ⇒ the answer for that route must change (proves the review looked at the code path) | The written per-route table, pasted here | T1 (review) | ⬜ |
| `P5-A14` | T0 gate: `preflight.ps1` counts coin-choosing reads of `outputs` outside the index module = baseline 0 | `preflight.ps1 -NegativeControl` with one planted direct read ⇒ the gate fails | The preflight output; the baseline commit is **separate** from the code commit (rule 6) | T0 | ⬜ |

**Two-sided rows:** `P5-A1` money cases ⟷ non-money cases; `P5-A5a` dust consolidated ⟷ token untouched; `P5-A6` multi-sat payment is money ⟷ ordinal is not; `P5-A8` money shown ⟷ Unknown shown (a panel that hides either fails); `P5-A11` received shown ⟷ own change not shown.

### 4a. Independent control notes (2026-09-28)

*By controls-B (Fable). Findings only — the author's GREEN/SUBJECT cells are untouched. Most serious first.*

- `P5-A5a` — **GREEN can pass on the wrong guard.** Once P4 stores real scripts, `task_consolidate_dust.rs :: is_p2pkh_script` (len == 25) refuses any real inscription on its own; a real-script token surviving the consolidator proves that check, not the classifier. Fix: the fixture set must include a pre-P4-shaped Unknown row (fabricated 25-byte script, `script_length NULL`, 2–`DUST_THRESHOLD_SATS` sats) — the row `is_p2pkh_script` accepts and only the index refuses. Also state that `disable_dust_consolidation` is unset and the result is `Consolidated`, not `Skipped`.
- `P5-A4` — **GREEN mis-states the equation.** "money + held + Unknown = pre-migration visible total" omits the **Token** class; every row T2 files into `1sat`/`bsv21` leaves all three sums, so on any copy holding a recognisable token the row fails for a reason that is not a defect, and on a copy holding none the Token branch is untested. Fix: use T2-P4 `A6`'s four-way equation (money + held/Unknown + filed tokens = Σ all unspent owned, direct SQL) and require at least one named outpoint per class in the copies.
- `P5-A10` — **GREEN does not cover the integration item.** T2-P4 `A6`/`A12` ("one place per outpoint") require "Treat as money" to refuse **held token-shaped** rows (BRC-162 prefix, not T2-recognised, reason-coded), which are exactly the rows a user would be tempted to promote. The GREEN names only "a T2-recognised token". Fix: extend GREEN to "refused on a T2-recognised token **and** on a held token-shaped row"; the RED above carries switch ③ for it.
- `P5-A9` — **SUBJECT contradiction.** An on-chain restore must reach the indexer to find the backup, so "restore with the indexer down" is either a file import (`wallet_import`) or a fault injected after `fetch_onchain_backup` returns. Say which; the fault point decides what the report can say.
- `P5-A6` — **route I6 has no seam-bypass red.** `insert_output` writes `change = is_change`, so own change is money whether or not the seam runs; "bypass the seam on I6" stays green and the route's coverage is vacuous. Fix: I6's control is a non-exact-P2PKH change script (PushDrop-shaped) that the seam must refuse; or state that I6's classification is the constructor and drop it from the per-route red.
- `P5-A2` — **the stamp is not tamper-evident.** `change=1` written directly by a route is indistinguishable from a seam stamp; "the default is refusal" holds only because each route's write default happens to be 0. The compensating control is `P5-A14` (no coin-choosing read outside the index module) — but nothing gates a *write* of `change=1` outside the seam. Suggest a second T0 gate pattern: `change` may be assigned only in the seam/migration/"treat as money" handler (baseline in its own commit, rule 6).
- `P5-A1` — fixture "P2PKH + `ord` envelope ⇒ Token/held (T2)": if T2-P1 has not landed, rule ② rejects it as **Unknown**, a different verdict from Token. State the T2-P1 dependency or accept either verdict until it lands.
- `P5-X1` (missing row — dApp reach) — GREEN: `POST` to the "Treat as money" endpoint from an approved origin (`X-Requesting-Domain` present) is refused by the first-party-only list in `main.rs` (fourth class, no-`HttpRequest` fund-movers) · RED: remove it from the list ⇒ 200 and the row stamped · SUBJECT: response code + the row's `change` unchanged. Same family as the `/wallet/rescan` edge handed to T5-P2, but this endpoint is P5's own and must ship gated.
- `P5-X2` (missing row — the invariants table names it, no row measures it) — GREEN: across the migration and one "Treat as money", zero `payment_success_indicator` emits (`HttpRequestInterceptor.cpp :: OnWalletCallSuccess`) and `PermissionService.session_counters` unchanged · RED: route the stamp through the createAction silent-approve path in a scratch build ⇒ the pill fires · SUBJECT: the emit log + `/wallet/session` counters before/after.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| Ingest routes I1–I9 (table in §2) — `task_sync_pending.rs :: check_addresses_individually`, `check_addresses_bulk`; `handlers.rs :: wallet_rescan`, `wallet_recover`, `wallet_recover_onchain`, `reconcile_backup_tx`, `wallet_import`, `internalize_action`, `store_derived_utxo`, `create_action_internal`, `do_onchain_backup`, `wallet_sync`, `reconcile_spent_inputs`, `wallet_recover_external`, `debug_repair_nosend`, `debug_broadcast_nosend`; `backup.rs :: import_entities`; `task_check_peerpay.rs :: run`; `certificate_handlers.rs :: unpublish_certificate_core`, `admin_prepare_unpublish`; `task_consolidate_dust.rs :: run_inner` | ✅ each located by grep today | The seam's call sites |
| `rust-wallet/src/database/output_repo.rs :: upsert_received_utxo`, `upsert_received_utxo_with_confirmed`, `upsert_received_utxo_with_derivation`, `insert_output` (writes `change = is_change`), `calculate_balance` (already excludes non-default baskets, beta.3 P10e) | ✅ | Received rows are written `change=0` today |
| `rust-wallet/src/main.rs :: main` — the "Fix master key outputs" `UPDATE` | ✅ | Deleted (W1) |
| `rust-wallet/src/handlers.rs :: wallet_activity` — `AND o.change = 0` | ✅ | The only reader of `change` outside backup; must change (§2) |
| `rust-wallet/src/reconcile.rs :: derive_receive_p2pkh_script`, `verify_receive_index`, `check_outpoint_spent` | ✅ | Ownership evidence for rule ②; chain answer for W2 |
| `rust-wallet/src/utxo_fetcher.rs :: is_token_reserved_value`; `monitor/task_consolidate_dust.rs :: is_p2pkh_script` | ✅ | The floor stays. ⚠️ `is_p2pkh_script` inspects the **stored** script — until P4's data backfills, it cannot fail on a synced coin; not counted as a defence (guard README) |
| `rust-wallet/src/recovery.rs :: split_token_reserved` | ✅ | The external sweep's value split — becomes a seam call |
| `frontend/src/pages/WalletPanelPage.tsx` (overlay panel) and `frontend/src/components/wallet/DashboardTab.tsx` (advanced wallet) — both render a balance | ✅ files exist; the exact balance component is pinned at kickoff | The Unknown line. ⚠️ CEF input patterns: native `<button>`/`<input>`, no MUI `TextField` in the overlay |

## 6. Out of scope

- Token recognisers and filing (`1sat`, `bsv21`, tags, customInstructions) — T2-P1.
- The token-spend permission class for dApp-named inputs — T2-P2 (this phase only routes D1 there).
- The full restore report and its wording — T3a-P2.3 (the wording is settled at the P2 restore sitting with real coins, decision 7).
- Backup format changes — none; `change` is already in the payload.
- Late payments / scans — P6.

## 7. Rollback

Code: revert the seam + selection switch in one commit; the index membership returns to P3's rule. Data: the migration only **adds** evidence (`change=1` stamps, basket filing done by T2) and never deletes a row, so reverting the code leaves a wallet whose selectors (P3 rule) still see every money coin. The deleted `main.rs` fix-up is **not** restored by a rollback without its own review — it is the fail-open write this phase exists to remove.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Row that catches it |
|---|---|
| The classifier reads the fabricated 25-byte script and stamps every synced coin money — green, and useless | `P5-A3`; P4 must land first |
| After upgrade, received payments are all Unknown for hours while parents are fetched; the balance drops and the user thinks money was stolen | `P5-A4` totals equal; `P5-A8` shows held + Unknown |
| "Plain P2PKH" implemented as *ends with* P2PKH ⇒ a multi-sat BRC-162 token classified money and spent | `P5-A1` BRC-162 case |
| A route not on the list (found later) still inserts `change=1` by default | `P5-A2` (default is refusal) + `P5-A14` gate |
| Received payments vanish from the activity feed because it keyed on `change=0` | `P5-A11` |
| "Treat as money" becomes a one-click way to spend an ordinal | `P5-A10` refusal on tokens; confirmation |
| The consolidator test calls `run_inner` directly and claims the timer path is guarded | `P5-A5a` SUBJECT: the timer fire |
| Deleting the master fix-up strands genuine service-fee-style outputs that were only selectable because of it | `P5-A7` second half |
| A coin whose parent can never be fetched stays Unknown forever | decision 7 ④ — manual "treat as money"; `P5-A10` |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | A1–A14 | Primary |
| macOS | A1–A3, A6 (T1), A8 (the panel renders on macOS — human glance), A11, A14 + `cargo test` | Frontend change is shared React (no C++ expected). Relay round names this contract and the wallet panel file |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1.5 h** — receive a 1-sat and an inscribed output on a scratch wallet; watch a forced consolidator fire; send-max; look at the Unknown line; one "treat as money" |
| Human-bound rows | A5a–c (destructive, scratch profile, real outputs — record outpoints first), A8 (eyes), A10 (confirmation UX) |
| Unknowns (K) — uncertainty, not difficulty | **K = 1.** The migration's duration and parent-fetch failure rate on a real long-lived wallet (how many coins sit Unknown after upgrade, and for how long) — measured on a DB copy at kickoff, before the display copy is final |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs ← | B5-T1-P3, P4 | The index (membership switched here) and observed scripts |
| ↔ | **B5-T2-P1** (classify & file) | ⭐ Seam split per T2-P1 §11: **T1 owns the seam, the stamp, Unknown and its display; T2 owns recognisers + filing.** Order: T2 recognisers → rule ② (exact 25-byte template) → ③ held. BRC-162 guard: rule ② must be exact-template |
| gives → | B5-T2-P2 (token-spend permission) | D1: dApp-named Token/Unknown inputs are routed to that class |
| gives → | **B5-T3a-P2.3** (trustworthy backup / restore) | Restore **classifies before rebuilding** the money index (old backups carry `change=0` on received payments — T3 SCOPE §6a); `P5-A9` is T1's half of `R-RESTORE` |
| gives → | B5-T6-P5 (Tools tab claim) | A claimed payment enters through the internalize route (I4/I5) and is classified like any other — T1's money-path rows apply to the claim endpoint |
| gives → | B5-T1-P6, P7 | Scans and shedding write through the seam; shedding spends only index coins |

## 12. Open questions for the owner

| # | Question | Recommendation |
|---|---|---|
| **Q1** | The wording and place of the Unknown line (*"N items being identified"*) and the "Treat as money" confirmation | Show it **directly under the balance** in the wallet panel, list view behind a click; confirmation text names the value and says *"only do this if you know this coin is ordinary money"*. Final wording at the T3a-P2 restore sitting with real coins, as decision 7 says |
| **Q2** | Keep the beta.3 1-sat floor once the classifier lands? | **Keep** — cheap defence in depth; its RED is re-based on the classifier (`P5-A12`) |

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
