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
| `P5-A1` | Seam table: exact 25-byte P2PKH to our key, >1 sat ⇒ Money; same with 1 sat ⇒ held; P2PKH + `ord` envelope ⇒ Token/held (T2); BRC-162 prefix + P2PKH ⇒ never Money; `script_length NULL` ⇒ Unknown; P2PKH to a key that is **not ours** ⇒ Unknown | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Seam output per fixture; fixtures are **real chain scripts** (P4's outpoints), length stated | T1 (money) | ⬜ |
| `P5-A2` | ⭐ **The default is refusal** (`R-CLASSIFY` RED, made observable by decision 2): a row inserted by a route that skips the seam lands `change=0` ⇒ not in the index ⇒ selected by **no** route | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The row by outpoint; the selection of every route S1–S6 | T1 | ⬜ |
| `P5-A3` | A fabricated 25-byte script (the pre-P4 value) with `script_length` unset does **not** classify as money (decision 7's named control: *the vacuous-pass trap*) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The row; which field the seam read, stated (REGRESSION_ADDITIONS `R-CLASSIFY` box) | T1 | ⬜ |
| `P5-A4` | Migration on a copy of the dev DB and a copy of a real beta.4 production-shaped DB: every row ends Money / Token / Unknown; **money + held + Unknown totals = the pre-migration visible total**; nothing vanished | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Per-class counts and satoshi sums before/after, and the outpoints now Unknown listed by name; **copies only** | T2 (schema/data) | ⬜ |
| `P5-A5a` | `R-NOSPEND` a: the dust consolidator's **timer fire** on a scratch wallet with 20+ dust coins **plus** one real Unknown/token output consolidates the dust and leaves the token **unspent** | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The token output (txid, vout, sats, basket, class) after the fire; the fire is the `Monitor` schedule with its timer shortened in a dev build — **say which** | T2 (money, destructive — scratch profile) | ⬜ |
| `P5-A5b` | `R-NOSPEND` b: `wallet_recover_external` on an address holding a token + coins sweeps the coins, **excludes and reports** the token | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The sweep transaction's inputs; the report shown | T2 (money, destructive) | ⬜ |
| `P5-A5c` | `R-NOSPEND` c: `select_utxos_with_preference` (every `dust_threshold_sats`) and `send_max` never take it | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Serialised transaction inputs | T1 + T2 | ⬜ |
| `P5-A6` | `R-CLASSIFY` per route I1…I9: an ordinal fixture arriving by that route ends Token or Unknown, never Money; a plain multi-sat payment arriving by that route ends Money | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) — per route: bypass the seam call ⇒ the multi-sat payment is **not** money (fail closed) and the ordinal is not money either (so the red is "no route defaults to money") | Per route, the stored row; the result lists every route **not** covered and why | T1 + T2 | ⬜ |
| `P5-A7` | The `main.rs` master fix-up is gone: a NULL-derivation spendable row survives a restart unchanged and unselectable; a genuine master-key output (service-fee style) whose observed script equals the master key's P2PKH is classified Money by the seam | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Both rows by outpoint across a restart | T1 | ⬜ |
| `P5-A8` | Display: with Unknown > 0 the panel shows the money balance **and** "N items being identified" listing each; the sum shown across both equals the pre-phase balance on an unchanged wallet | Hide the Unknown line (render money only) ⇒ the sum check fails and the reviewer sees a lower total (seen) | ⭐ Human eyes on the rendered overlay (`R-RESTORE` b in everyday use) + `/wallet/balance` JSON; ⚠️ Vite HMR fakes controls — hard-reload before each measurement | T3 (human) | ⬜ |
| `P5-A9` | `R-RESTORE` (T1's half): restore a scratch backup holding a 1-sat inscription with the indexer **down** ⇒ restore completes, the inscription is **held, shown, not spendable**, and the report says so (decision 7 ⑤ + its named control) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The **restored** DB row and the restore report | T2 | ⬜ |
| `P5-A10` | "Treat as money" on one Unknown row stamps exactly that row after a confirmation; refused on a T2-recognised token | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The row's `change` + the audit trace; the refusal response for the token | T1 + T3 | ⬜ |
| `P5-A11` | Activity feed: a received payment stamped `change=1` still appears as **received** with the right amount; our own change still does **not** appear as received | Keep `o.change = 0` in `wallet_activity` ⇒ the received payment disappears from the feed (seen) | `/wallet/activity` JSON for that txid, before and after the stamp | T1 + T2 | ⬜ |
| `P5-A12` | `R-DUST` re-based: bypass the **classifier** (not the floor) ⇒ R-DUST's RED fires; bypass the **floor** alone ⇒ the classifier still refuses the 1-sat coin | Each bypass is itself the red — two, not one | R-DUST's SUBJECT (serialised inputs) | T1 | ⬜ |
| `P5-A13` | CVE-2026-56744 question: every spend route that pays an external script lists where the script comes from (request, derivation, storage) and whether it is validated; any "from storage, unvalidated" route is a finding in §12 | Re-read one route with a planted storage-sourced script in a T1 test ⇒ the answer for that route must change (proves the review looked at the code path) | The written per-route table, pasted here | T1 (review) | ⬜ |
| `P5-A14` | T0 gate: `preflight.ps1` counts coin-choosing reads of `outputs` outside the index module = baseline 0 | `preflight.ps1 -NegativeControl` with one planted direct read ⇒ the gate fails | The preflight output; the baseline commit is **separate** from the code commit (rule 6) | T0 | ⬜ |

**Two-sided rows:** `P5-A1` money cases ⟷ non-money cases; `P5-A5a` dust consolidated ⟷ token untouched; `P5-A6` multi-sat payment is money ⟷ ordinal is not; `P5-A8` money shown ⟷ Unknown shown (a panel that hides either fails); `P5-A11` received shown ⟷ own change not shown.

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
