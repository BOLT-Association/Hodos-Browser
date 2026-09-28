# B5-T2-P1 — Every ordinal and token that reaches the wallet is recognised and filed, never mistaken for money · PHASE CONTRACT

**Track:** B5-T2 1Sat Ordinals · **Tickets:** none directly; consumes T1's `../../tickets/TICKET_synced_outputs_store_a_fabricated_locking_script.md` fix · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent · **Platforms:** both (Rust only) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **8** (BSV-21 recognise/hold/show; `bsv21`, never `1sat` — BRC-163 MUST) · **2** (money = positively marked; a token is never stamped `change=1`) · **2a** (money index is derived — a filed token never enters it) · **7** (tiered rule: real script first; 1-sat unreadable ⇒ held; nothing hidden) · T2 **Q7** (patterns only).

> **Order.** Runs **after** P2 (decision 8) and **after T1-P4** for the sync routes (real scripts). The
> internalize and own-output routes already carry real scripts, so their rows can run first.

---

## 1. Goal

When an ordinal or a BSV-21 token arrives in the wallet by any route, it is filed where the ecosystem expects it (`1sat` or `bsv21`, with the standard tags and `customInstructions`), and anything token-shaped the wallet cannot fully read is held — never counted or spent as money.

## 2. Done means

- [ ] **Seam split, agreed with T1:** T1-P5 owns the classifier seam (the one function every ingest route calls, the `change=1` stamp, the `Unknown` state and its display). This phase owns the **token recognisers** that seam calls, and the **filing** (basket, tags, CI) of what they recognise. The recognisers run **before** decision 7's rule ② ("multi-sat plain P2PKH ⇒ money").
- [ ] Recognisers are pure functions over the **stored real locking script** and return one of: ordinal · BSV-21 value (JSON, BRC-161) · BSV-21 non-value (`deploy+auth` / `auth` / `burn`) · BSV-21 binary (BRC-162) · legacy BSV-20 (tick) · none.
- [ ] Filing, by recogniser result and satoshi value:
  - 1-sat + ordinal envelope (first `ord` field, content type not `application/bsv-20`) ⇒ basket `1sat`, tags per BRC-147 (`origin` / `origin:<txid_vout>`, `type:<mime>`), CI `{origin, …}` in underscore form.
  - 1-sat + BRC-161 **value** op ⇒ basket `bsv21`, tag `bsv21:<id>`, CI `{id, amt, op, sym?, dec?, icon?}` (BRC-163). ⛔ Never `1sat`.
  - BSV-21 non-value, BRC-162 binary (any satoshi value), legacy BSV-20, multi-sat with an envelope, unknown envelope ⇒ **held** (T1 `Unknown`, not money, shown), not filed. See §12 Q1 for binary.
- [ ] ⭐ **BRC-162 guard, new 2026-09-28:** an output whose script begins with the BRC-162 prefix is never classified money, **at any satoshi value** — BRC-162 (merged today) says 1-sat is *"convention … not a protocol rule"* and the prefix is followed by an ordinary P2PKH. ⇒ decision 7's "plain P2PKH" must be an **exact** 25-byte template match, never a suffix or contains match. (Stated here; enforced in T1-P5's rule — edge.)
- [ ] `internalizeAction` basket insertion stores `customInstructions` **verbatim** (BRC-37: *"store and forward the entire string unchanged"*), instead of today's `{"type":"basket_insertion","basket":…,"appData":<ci>}` wrapper; existing wrapped rows stay readable (reader accepts both).
- [ ] Sender-supplied tags and CI are kept and forwarded, but the wallet's own recogniser result wins for filing; a sender's `1sat` / `bsv21` basket request on an output that fails eligibility is held, not filed (BRC-147/163 "MUST NOT present … solely because of basket membership").
- [ ] `listOutputs` with basket `p 1sat <scope>` is routed to storage `1sat` (BRC-165 view), scopes exactly `all|collection|app|creator|id`, bare/extra/unknown scope rejected, the scope's tag filter mandatory, `id` allowed without an `all` grant; every other `p ` name still rejected. View is not covered by pay grants (`is_protected_basket` extended for asset baskets).
- [ ] The ingest-route list covered by this phase is **named**, and each route not covered is named with the reason (`R-CLASSIFY`).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-CLASSIFY` | Classification reaches every ingest route | This phase adds token recognition to that seam; a route that bypasses the seam files nothing and — before T1-P5 — lands spendable |
| `R-NOSPEND` | No automatic path spends an unclassified output | A recogniser that says "none" for a real token hands it to rule ② and it becomes money |
| `R-RESTORE` | Fail-closed survives recovery | Restore re-ingests; if the restore route skips the recognisers, tokens come back as money or vanish |
| `R-DUST` (beta.3) | 1-sat floor | Must not go vacuous: once tokens are filed, the floor's RED must be re-based on the classifier (R-DUST's own warning) — T1 owns that re-base |
| Balance display | `calculate_balance` counts only default-basket rows | Filing **money** into `1sat` by mistake hides real balance — eligibility is `satoshis === 1` **and** an envelope, never the value alone |

## 4. Evidence table

⛔ Classification decides spendability ⇒ these are money rows; RED by a second agent (`../../../RELEASE_CYCLE.md` §4.2).
**Subjects named before the run** (scratch profile; outpoints recorded here before the run): the mainnet ordinal `7faac48b…` fixture (script prefix `76a914…88ac 0063036f7264 5109 image/png`, from T2 SCOPE §5) `________`; a real BSV-21 **transfer** output `________`; a real BSV-21 **deploy+auth** output `________`; a BRC-162 binary output **if one exists on mainnet by the run** (`________`, else a hand-built script, stated as such); a legacy BSV-20 tick output `________`; a 1-sat plain P2PKH payment `________`; a multi-sat plain P2PKH payment `________`.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P1-A1` | Each fixture script through the recognisers returns the expected class (table above) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Recogniser output per fixture; fixtures are **real chain scripts** fetched by txid, length stated (not 25 B) | T1 | ⬜ |
| `P1-A2` | Ordinal fixture ingested by `internalizeAction` (real script from BEEF) ⇒ row in basket `1sat`, `origin`/`type:` tags, CI underscore form; not in `get_spendable_by_user`; not in `calculate_balance` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The stored `outputs` row (txid, vout, satoshis=1, basket, tags, CI) + both selector results | T2 | ⬜ |
| `P1-A3` | BSV-21 transfer fixture ⇒ basket `bsv21`, tag `bsv21:<id>`, CI `{id, amt, op}`; **never** `1sat` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Stored row; `SELECT … WHERE basket='1sat'` returns zero rows for this outpoint | T2 | ⬜ |
| `P1-A4` | deploy+auth, BRC-162 binary (1-sat **and** a multi-sat variant), BSV-20 tick, multi-sat-with-envelope ⇒ held (not filed, not money, listed as held) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Stored rows + selector + balance results; the multi-sat BRC-162 variant is the case decision 8's "held 1-sat coin" wording did not cover | T1 + T2 | ⬜ |
| `P1-A5` | Multi-sat plain P2PKH ⇒ passes to T1's rule ② (money); 1-sat plain P2PKH without envelope ⇒ held, **not** `1sat` (no envelope = not an ordinal) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Stored rows; proves the recognisers do not over-claim | T1 + T2 | ⬜ |
| `P1-A6` | **Vacuous-pass trap (decision 7):** the ordinal fixture ingested by the sync route **with the fabricated 25-byte script** must NOT be classified money or filed as a plain coin — it is held | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The row's stored `locking_script` length and the classifier's decision; states which field the classifier read and whether it is an observation (`R-CLASSIFY` note) | T2 | ⬜ |
| `P1-A7` | Every ingest route on T1-P5's list: an ordinal fixture arriving by that route ends filed or held — never spendable. Route list quoted in the result with the uncovered ones named | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Per route: the stored row. Named routes at minimum: `internalize_action` (wallet payment + basket insertion), `store_derived_utxo` (PeerPay), `create_action_internal` own outputs, sync (`task_sync_pending` / `utxo_fetcher`), restore / `reconcile_backup_tx`, recovery scan | T2 | ⬜ |
| `P1-A8` | `internalizeAction` with CI `{"origin":"…","name":"x"}` ⇒ `listOutputs(include customInstructions)` returns **exactly** that string | Re-enable the `appData` wrapper ⇒ the returned string differs (must be seen) | `outputs.custom_instructions` bytes + the `listOutputs` response for that outpoint | T1 + T2 | ⬜ |
| `P1-A9` | A pre-existing wrapped-CI row (made on today's build) is still read correctly after the change | Remove the wrapped-form reader ⇒ that row's CI reads as missing (must be seen) | A row created on the pre-change build, named by outpoint | T2 | ⬜ |
| `P1-A10` | `listOutputs` basket `p 1sat all` ⇒ routed to `1sat` and gated by a view prompt; `p 1sat` bare, `p 1sat foo`, `p 1sat all extra` ⇒ rejected; `p 1sat id` + `id:<k>` ⇒ allowed without an `all` grant, results limited to that id; `p other x` ⇒ still rejected | Restore today's blanket `p ` reject ⇒ `p 1sat all` is refused (must be seen); drop the mandatory tag filter ⇒ `p 1sat collection` + `tagQueryMode:"any"` returns out-of-scope rows | The `listOutputs` response + the Rust decision for the view call | T1 + T2 | ⬜ |
| `P1-A11` | A pay/auto-pay grant does not silence the `p 1sat …` view prompt (BRC-165 *"Item view is not covered by ordinary payment / auto-pay grants"*) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | `PermissionDecision` for the view call on a domain holding the broadest pay grant | T1 | ⬜ |
| `P1-A12` | A sender who asks for basket `1sat` on a **multi-sat** output, or `bsv21` on an ordinal, does not get it (eligibility is ours) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Stored row's basket vs the requested basket | T2 | ⬜ |

**Two-sided rows:** `P1-A4`↔`P1-A5` (token-shaped is held / plain money still passes) and `P1-A3`↔`P1-A2` (`bsv21` never `1sat` / ordinals always `1sat`).

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/database/basket_repo.rs :: validate_and_normalize_basket_name` | ✅ | Rejects every `p ` name and `default` — becomes route-if-`p 1sat` |
| `rust-wallet/src/database/basket_repo.rs :: BasketRepository::find_or_insert` | ✅ | Normalizes; used by internalize |
| `rust-wallet/src/database/tag_repo.rs :: assign_tag_to_output`, `get_tags_for_output`, `validate_and_normalize_tag` | ✅ | Tag vocabulary is strings — no change expected |
| `rust-wallet/src/database/output_repo.rs :: insert_output`, `get_spendable_by_user`, `get_spendable_confirmed_by_user`, `calculate_balance`, `remove_from_basket` | ✅ | Selectors exclude non-default baskets today; T1 adds `change=1` |
| `rust-wallet/src/handlers.rs :: internalize_action` | ✅ | Basket-insertion arm validates basket + tags, stores the **real** script from the tx, wraps CI under `appData`; `derivation_prefix/suffix = None` |
| `rust-wallet/src/handlers.rs :: store_derived_utxo` | ✅ | PeerPay ingest; real script |
| `rust-wallet/src/handlers.rs :: create_action_internal` | ✅ | Own basket outputs inserted with CI verbatim before broadcast |
| `rust-wallet/src/handlers.rs :: list_outputs` → `peek_scoped_grant_scope_basket` | ✅ | Basket gate peeks `basket` from the body; `p 1sat` routing goes before it |
| `rust-wallet/src/permission_service/request_gate.rs :: is_protected_basket` | ✅ | Extended for asset baskets on view |
| `rust-wallet/src/utxo_fetcher.rs :: generate_p2pkh_script_from_address`; same fn in `services/providers/whatsonchain.rs`, `services/providers/gorillapool_ordinals.rs` | ✅ | **Three** copies of the fabrication; T1-P4 replaces them |
| `rust-wallet/src/utxo_fetcher.rs :: is_token_reserved_value` | ✅ | beta.3 floor, untouched |
| `rust-wallet/src/monitor/task_sync_pending.rs :: cache_parent_transactions` | ✅ | Raw parent tx cached — a possible real-script source (T1-P4's choice) |
| `rust-wallet/src/reconcile.rs :: parse_tx_outputs` | ✅ | Returns `(value, script)` per output; fails closed on truncation |
| `rust-wallet/src/script/parser.rs :: parse_script_chunks` | ✅ | Chunk parser to build the envelope + BRC-162 prefix recognisers on (reuse, no new parser) |
| `rust-wallet/src/recovery.rs :: split_token_reserved` | ✅ | Recovery route (T1's list) |
| BRC-147 / 161 / 162 / 163 / 165 (`bsv-blockchain/BRCs` `master`, fetched 2026-09-28) | ✅ | ⚠️ **BRC-162 PR #273 merged 2026-09-28 04:05Z**; BRC-161 now calls itself the *"legacy JSON wire encoding"*; BRC-163 §"Binary encoding" lets binary outputs share basket `bsv21` |
| 1Sat SDK `packages/types/src/constants.ts` at `52cfe69` | ✅ | `ONESAT_BASKET='1sat'`, `BSV21_BASKET='bsv21'`, `BSV20_BASKET='bsv20'`; **no BRC-162 decoder** — its `templates/src/shrug/shrug.ts` is a *tagged* binary format, not the merged untagged BRC-162 |

## 6. Out of scope

- BSV-21 transfer (decision 8, deferred). BRC-176 proof building.
- Verifying origin (BRC-158/159/150 walk) — P3. Display — P4.
- The classifier seam itself, `change=1`, the Unknown display and "treat as money" — T1-P5.
- Filing BRC-162 binary into `bsv21` (§12 Q1 — recommended: hold only).
- Filing legacy BSV-20 into `bsv20`, OpNS into `opns` (held is safe; filing is a later call).
- Stamping BRC-164 `id:` tags (needed for `p 1sat input id <key>` resolution; not for safety).

## 7. Rollback

Revert the recogniser + filing commit and the `p 1sat` routing commit (two commits, no schema). Rows already filed stay in their baskets — excluded from spending either way; `remove_from_basket` would return them to held/default only by deliberate migration.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| Classifier reads the fabricated 25-byte script, finds "plain P2PKH", stamps money — green everywhere, protects nothing | `P1-A6`; `P1-A1` states fixture lengths |
| "Plain P2PKH" implemented as *ends with* P2PKH ⇒ a multi-sat BRC-162 token becomes money and is spent | `P1-A4` multi-sat variant; T1 edge |
| BSV-21 filed into `1sat` because the ord recogniser matches first | `P1-A3` |
| Only the internalize route tested; sync route files nothing | `P1-A7` route list |
| Sender puts a multi-sat payment in `1sat` to hide it from the balance | `P1-A12` |
| Unwrapping CI breaks readers of old wrapped rows (backups too) | `P1-A9`; T3 edge |
| `p 1sat` routing silently widens view to pay-granted sites | `P1-A11` |
| Recogniser tests use hand-typed hex that differs from real chain bytes | `P1-A1` subject: fetched by txid |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | Scratch profile |
| macOS | `P1-A2`, `P1-A3`, `P1-A7` (one route) | Rust-only phase; macOS confirms on its own dev profile that filing and selector exclusion hold (shared code, platform DB path). No C++ ⇒ no relay file note |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~0.5 h** — staging the real subject outputs on a scratch wallet (an ordinal, a BSV-21 transfer, a deploy+auth, a legacy BSV-20) from the owner's funded wallets; shared with P2's staging |
| Human-bound rows | none — every row is agent-run on a scratch profile |
| Unknowns (K) — uncertainty, not difficulty | **K = 1 phase.** (a) T1-P4's real-script shape (full script or a bounded prefix ≥ ~256 B — the BSV-21 JSON body sits ~100–200 B in; the 64 B option is not enough); (b) whether any BRC-162 output exists on mainnet to test against (merged today; the reference SDK does not decode it) |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| T1-P4 → T2-P1 | real scripts | **Blocking for the sync / restore rows** (`P1-A6`, `P1-A7`); ≥ ~256 B prefix or the full script |
| T1-P5 ↔ T2-P1 | classifier seam | ⭐ **T1 owns the seam, the stamp, `Unknown` and its display; T2 owns token recognisers + filing.** Order inside the seam: T2 recognisers → decision 7 ② (exact P2PKH template) → ③ held. T1-P5's contract should cite this row |
| T2-P1 → T1-P5 | BRC-162 | Rule ② must be an exact-template match (evidence: BRC-162 §"Satoshi value (convention)") |
| T2-P1 → T3a-P2.3 | restore | Restore re-ingests through the seam; tokens restored with the indexer down ⇒ held, filed once the real script is read. Wrapped + unwrapped CI both appear in old backups |
| T2-P1 → T2-P2 | view gate | `is_protected_basket` extension is shared; P2 lands first |

## 12. Open questions for the owner

1. **BRC-162 binary BSV-21 (merged 2026-09-28):** hold it (recognise, never money, show as held) — or also file it into `bsv21`? ⭐ Recommend **hold only** in this release: BRC-163 allows filing, but the reference SDK has no decoder for the merged format, so we would have nothing to test against (`WATCH_fungibles.md`: *"A capability we cannot test is not a capability we can claim"*).
2. ⚠️ **Evidence on decision 8's watch item, not against the decision:** the README records *"an unreadable binary BSV-21 output is a held 1-sat coin"*. BRC-162 as merged does not require 1 sat. The decision's outcome stays safe **only if** decision 7's "plain P2PKH" is an exact 25-byte match. No reopening needed — confirm the wording is carried into T1-P5.

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
