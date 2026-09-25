# B5-T2 — 1Sat Ordinals: track scope (G2)

**Written:** 2026-09-25, by the G2 research agent for this track. **Status:** 📌 PROPOSED — candidate
phases only; no contracts, no code. **Standard:** `../../RELEASE_CYCLE.md` §2 (G2), §3.1a, §3.3, §4.2.
**Reads against:** head `4435ba1` on `0.4.0` (verified with `git log`).

> **How to read the labels.** Every claim is tagged **code reading** (our tree at head), **measurement**
> (something run or fetched and counted), **doc says** (a repo document), or **web (URL, date, version)**.
> Nothing here was executed against a wallet or browser. ⛔ Nothing in this file is a decision; §9 lists
> the decisions it asks for.

---

## 1. Goal

**Hold, show, receive and deliberately transfer 1Sat ordinals to the current BRC-147/150/159/160/165 text
— and, new in this scope, classify and show BSV-21 fungible tokens (BRC-161/163) so they are never
destroyed and never mistaken for collectables — with every token spend gated by a permission class that
no pay or auto-pay grant can satisfy.**

---

## 2. Telescope — the current state of the world (re-fetched 2026-09-25)

Per `RELEASE_CYCLE.md` §3.1a every source below was fetched fresh today. Where it differs from our repo
copies (`README.md` 2026-08-05, `RESEARCH_FINDINGS.md` 2026-05-05, `WATCH_fungibles.md` 2026-08-29,
`../../BSV-Tokens/` April–August), the difference is called out.

### 2.1 The BRCs — what is merged, what is open, what moved

| BRC | Title | State today | Source |
|---|---|---|---|
| **147** | 1Sat Ordinals Basket Profile (`1sat`) | Merged (PR #216, 2026-08-13). Text stable since | web `raw.githubusercontent.com/bsv-blockchain/BRCs/master/tokens/0147.md`, 2026-09-25 |
| **150** | 1Sat Provenance Remittance (`provenance` v2 with `beefB64`) | Merged (PR #210, 2026-08-10). Stable | same path, `0150.md` |
| **159 / 160** | Origin tracking / inscription envelopes | Merged 2026-08-10. Stable | `0159.md`, `0160.md` |
| **165** | P1Sat Permission Scheme for basket `1sat` | Merged (PR #229, 2026-08-28) | `0165.md` |
| **161** | BSV-21 fungible tokens, JSON encoding | Merged (PR #213, 2026-08-28) | `0161.md` |
| **162** | BSV-21 binary/CBOR encoding | Merged 2026-08-28 — ⚠️ **being marked superseded/withdrawn** by open PR #267 (BRC-182 "Mandala extension", shruggr, 2026-09-23): *"That binary prefix was specified and never implemented."* | `gh api repos/bsv-blockchain/BRCs/pulls/267`, 2026-09-25 |
| **163** | BSV-21 Basket Profile (`bsv21`) | Merged (PR #217, 2026-08-28), after 10 review comments | `0163.md`; PR #217 comments |
| **176** | BSV-21 Validity Proofs (offline BEEF) | **Merged 2026-09-01** (PR #238, shruggr) — ⭐ **new since every repo doc** | `0176.md`; `gh api …/pulls/238` |
| **175** | 1Sat Fungible Basket Profile (`1sat-ft`) | **Still open** (PR #237, GenericCPU/HandCash, opened 2026-08-27). `mergeable_state = dirty` (conflicts with master), **last updated 2026-08-29, no activity for 27 days**, 3 comments, 0 review comments. shruggr's three questions on how a split's `amt` is committed are **unanswered**. PR body: *"Implemented in HandCash wallet for hold/send/receive"* | `gh api repos/bsv-blockchain/BRCs/pulls/237` + `issues/237/comments`, 2026-09-25 |
| **177** | Wallet-enforced expiry for `noSend` actions | Merged 2026-09-01 (ty-everett). Adjacent to our BRC-121 nosend path, not this track | `…/pulls/239` |
| **181** | Wallet-Enforced Autonomous-Agent Spend Policy | Open (PR #265, RexStarBSV, 2026-09-22). Relevant to the consent ticket (§4) | `…/pulls/265` |

**What changed versus our docs:**

1. **`WATCH_fungibles.md` said the amount-commitment question was open ("re-inscription vs remittance").
   For BSV-21 it is not open — it never was inside BSV-21 itself.** BRC-161 (merged): the token fields
   are *"a prefix on the locking script"* of **every** token output; a transfer output carries
   `{"p":"bsv-20","op":"transfer","id":…,"amt":…}` in its own `ord` envelope; *"Indexers admit or
   reject outputs; consensus miners do not enforce BSV-21 rules."* BRC-163's `customInstructions.amt`
   is a **remittance fast path over an inscription that is always present**, not a substitute for it —
   BRC-163: *"A wallet MUST NOT treat `amt` / token id as proven solely because they appear in CI or
   tags from an untrusted sender."* BRC-176 (merged 2026-09-01) then gives the indexer-free proof: an
   offline BEEF walked back to deploy, with *"A BRC-176 packet MUST NOT be validated, stripped, or
   round-tripped as ordinary BRC-95 Atomic BEEF"* (full bodies retained, so it is bigger than SPV BEEF).
   ⇒ **Model for BSV-21: inscription on every output + indexer validation, with BRC-176 as the offline
   alternative.** That is settled and implemented.
2. **BRC-175 is the thing that was contested, and it has stalled — and it converged.** The draft's
   *current* text (fetched from the PR branch `GenericCPU/BRCs:docs/brc-166-onesat-fungibles`) now says
   *"Conforming senders MUST inscribe each new colour tip (payee and change) as
   `application/1sat-ft+json`"* with `amt` in the envelope, and *"When remittance `amt` and the
   inscription disagree, the inscription wins."* So BRC-175 moved **to** re-inscription. Its remaining
   difference from BSV-21 is identity-by-origin (BRC-159 sat tracking) instead of identity-by-deploy
   outpoint with indexer admission. ⚠️ It is a **HandCash-only** profile with no second implementer, a
   dirty merge state and unanswered core questions. **Do not build it.** Keep the watch.
3. **BRC-162 is being withdrawn** (PR #267). The "two encodings" worry in `WATCH_fungibles.md` is
   resolving to **one: JSON (BRC-161)**. PR #267 is open; watch it, do not depend on it.
4. **BRC-147's "pay / auto-pay MUST NOT authorize ordinal spends" is a SHOULD in 147 and a MUST NOT in
   165.** Exact text — BRC-147: *"Conforming wallets SHOULD: … Not treat a general 'pay' or auto-pay
   grant as authorization to spend `1sat` tips."* BRC-165: *"MUST NOT treat payment / auto-pay grants
   as item access"* and *"Send approval is **per action** (inline prompt)"*. Our `README.md` §"Two
   rules" and `REGRESSION_ADDITIONS.md` `R-TOKENPERM` attribute the MUST to 147; the MUST lives in
   **165**, and it is stronger than we wrote: **per-action**, i.e. a standing token-spend grant is
   *also* non-conforming. (measurement — `curl … | grep -n auto-pay` on both files)
5. **BRC-165 normalization** is as `README.md` says: `p 1sat <scope>` is request-time only, storage is
   `1sat`, scopes are exactly `all | collection | app | creator | id`, values ride tags, and *"Bare
   `p 1sat` (no scope token) MUST be rejected"*. New detail: scope **`id`** MUST be auto-allowed without
   an `all` grant (§Scopes rule 7). Spend label: `p 1sat input id <key>`, `<key>` = the BRC-164 `id:`
   tag value. ⚠️ Our `basket_repo.rs :: validate_and_normalize_basket_name` rejects every `p ` name
   (code reading) — correct as a default, but it means a conforming caller's `listOutputs({basket:"p
   1sat all"})` is refused today rather than routed.

### 2.2 The 1Sat SDK — the de facto standard, as of its 2026-09-20 release

Repo `b-open-io/1sat-sdk`, default branch `master`, last push 2026-09-20 05:21Z, release commit
`52cfe69` *"Release: wallet 0.0.112, wallet-server 0.0.56, cli 0.0.121 and dependents"* (measurement,
`gh api`). npm `dist-tags.latest` on 2026-09-25: `@1sat/wallet` **0.0.112** (published 2026-09-20),
`@1sat/actions` **0.0.225** (2026-09-20), `@1sat/wallet-server` **0.0.56**, `@1sat/cli` **0.0.121**,
`@1sat/connect` **0.0.97`, `@1sat/core` 0.0.11 (last published 2026-03-21 — stale, superseded by
`actions`). All packages declare `"license": "MIT"` on npm; **the GitHub repo still reports
`license: null`** — unchanged since the consent ticket measured it (measurement, `gh api
repos/b-open-io/1sat-sdk`). 23 packages in `packages/`.

**How it handles BSV-21** (web, raw files at `master`, 2026-09-25):

| Question | Answer | File |
|---|---|---|
| Basket names | `ONESAT_BASKET = '1sat'`, `BSV21_BASKET = 'bsv21'`, `BSV20_BASKET = 'bsv20'`, `OPNS_BASKET = 'opns'`, `LOCK_BASKET = 'lock'`, `FUNDING_BASKET = 'default'`, plus `sigma`/`bsocial`/`bap`. All **plain** names — comment: *"not P-baskets. Permission routing for createAction stays on `p 1sat …` labels."* Legacy `p 1sat …` storage baskets are migrated once via `LEGACY_P1SAT_BASKET_MIGRATIONS`. One hard-coded exception: token `ae59f3b8…8127_0` (MNEE) files into basket `mnee` | `packages/types/src/constants.ts`; `packages/wallet/src/indexers/Bsv21Indexer.ts` |
| Tags on a BSV-21 output | `bsv21:<tokenId>` (exact-match filter), `bsv21:deploy`, `bsv21:auth`; legacy fallbacks `amt:`, `sym:`, `dec:`, `icon:` | `packages/actions/src/utils/bsv21Remittance.ts` |
| `customInstructions` on a BSV-21 output | `{ id, amt (string), op, sym, dec, icon, protocolID, keyID, counterparty }` — exactly BRC-163's recommended shape | same |
| Balance | `getBsv21Balances`: sum of CI `amt` per token id over `bsv21`-basket outputs, excluding auth-only rows (`isBalanceableBsv21Output`); reader priority CI → tags → script | `packages/actions/src/tokens/index.ts` |
| Validity | `Bsv21Indexer` decodes the inscription locally (`BSV21.decode`, range-checks `amt`), then asks the overlay `getTokenByTxid` for each input; status ladder `pending → valid / invalid`; a 404 from the overlay leaves the row **pending**, never invalid | `Bsv21Indexer.ts`; open PR #73 *"Keep unconfirmed BSV-21 inputs sweepable"* (rohenaz, 2026-09-12): *"Overlay state is advisory during migration; inscription amounts are the source data"* |
| Transfer | `sendBsv21`: inputs from `bsv21` basket matching the id, outputs re-inscribed via the `BSV21` template, change back to `bsv21` with a fresh `keyID`, action labels `buildTokenLabel(tokenId)` + `buildInputAssetLabel('bsv21', assetId)`, **an extra `fee:overlay` output** of `fee_per_output × tokenOutputCount` satoshis to the token's `fee_address` when the token is *active*, and a **BRC-176 BEEF** attached from `listOutputs` or a service lookup | `packages/actions/src/tokens/index.ts` |
| Ordinal transfer | inputs/outputs in `'1sat'`; tags carried from the source (`origin:`, `type:`); CI = `{protocolID, keyID: <outpoint>, counterparty: 'self', origin, content, app, collection, name}`; fees from ordinary coin selection, never from `1sat` | `packages/actions/src/ordinals/index.ts`, `utils/ordinalRemittance.ts` |
| ⚠️ **BRC-150 provenance** | **Not produced and not verified.** `grep beefB64` across the repo: **zero hits**. Their own plan: *"Optional: BRC-150 package build/verify (size vs CI limits — prefer omit / 156 for deep history)."* `internalizeOutpointBeef` walks a caller-supplied BEEF to the origin (BRC-158/159/160 checks: `not-a-1sat-tip`, `origin-has-no-ord-envelope`) but writes no `provenance` object | `docs/plans/2026-08-05-brc-alignment.md`; `utils/internalizeOutpointBeef.ts`; code search |
| Sweep classes (legacy import) | `bsv`, `ordinals`, `bsv21Tokens`, `bsv20Tokens`, `opnsNames`, `listings` (OrdLock, cancelled on sweep), `locked`, `run` — **all identified by the indexer**, not by local script parsing | `packages/actions/src/sweep/types.ts` |
| Their consent model for the HTTP endpoint | `1sat serve wallet-api` on `127.0.0.1:3321`: *"There are no prompts, no auto-approve, no interactive flag and no 'allow once'"*; a denied call is `400 {"error":"…run `1sat permissions grant …`"}`; origin = the `Origin` header only; audience *"agents and automation, which have no terminal to answer on"*; *"Users wanting interactive approval should use a graphical wallet instead"* | `packages/cli/skills/wallet-api/SKILL.md` |
| Their consent model for the GUI | The permission module **prompts per action** for asset spends (`promptHandler({kind:'transaction'})`, error *"1Sat permission module: user rejected the transaction."*), then caches `hashOutputs` + the approved input outpoints so a later `createSignature` for a different outpoint is refused: *"signature requested for outpoint … which was not part of the approved transaction"* | `packages/permission-module/src/handlers.ts` |

**Why the size limit matters for BRC-150.** In the BRC-163 review, its author records the reference
client's limits: *"tags: trim + lowercase, ≤ 300 UTF-8 bytes each; `customInstructions` on
`internalizeAction` basket insertion: ≤ 1000 UTF-8 bytes"* (web, PR #217 comment, GenericCPU,
2026-08-10). A `beefB64` for even one hop is larger than 1000 bytes. So through the reference SDK a
BRC-150 package **cannot travel inside `customInstructions` at all** — which is why nobody produces one.
BRC-150 itself allows this: *"Omit provenance when the package would exceed an implementation size
budget."* ⇒ **The ecosystem's provenance today is the BEEF handed on `internalizeAction` (BRC-158/176
style), not a BRC-150 object.**

### 2.3 Peer wallets

- **Yours Wallet** — active: repo pushed 2026-09-16, release **v5.1.0** on 2026-09-16 (measurement, `gh
  api repos/yours-org/yours-wallet`). Still the `window.yours` provider our `RESEARCH_FINDINGS.md`
  describes; our wallet already derives its `ordAddress` shape (`handlers.rs ::
  YOURS_LEGACY_ORD_RECEIVE_PROTOCOL = "2-yours-legacy-ord-receive"`, code reading). ⚠️ That means
  **ordinals may already be arriving at Hodos addresses from Yours-era dApps** — those addresses are
  in our address table only if `yours_legacy_addresses` inserted them; not verified.
- **1Sat wallet-desktop / CLI** — the SDK's own wallets (§2.2). A real counterparty to test against:
  the `RELEASE_PLAN.md` 2.6 worry *"we may have nothing to test against"* is **stale**.
- **HandCash** — implements BRC-175 per its author; not a BRC-100 wallet we can drive.

### 2.4 Indexers and content

Unchanged from `README.md` §"Indexer dependency": `shruggr/1sat-indexer`, `1sat-stack` and `1sat-sdk`
carry no LICENSE file on GitHub; BSV-21 indexing is metered (`fee:overlay` output per token output,
confirmed in the SDK code above); ORDFS (`ordfs.network`) is the content gateway; GorillaPool
`ordinals.gorillapool.io` is the address→ordinals API our provider chain already uses as the **UTXO
fallback** (`services/mod.rs`, `fetch_utxos: WoC → GorillaPool Ordinals`, doc says
`rust-wallet/src/CLAUDE.md`).

---

## 3. Kaleidoscope — do we already have this?

⭐ Most of the substrate exists. The track is wiring plus one new permission class, not a build.

| Need | Already have (code reading, head `4435ba1`) | Gap |
|---|---|---|
| Basket storage + normalization | `database/basket_repo.rs :: validate_and_normalize_basket_name`, `find_or_insert`; `output_baskets` (V1) | Rejects `p ` names outright — needs a *route* for `p 1sat <scope>` → `1sat` (BRC-165), not a reject |
| Tags on outputs | `database/tag_repo.rs :: TagRepository::assign_tag_to_output`, `get_tags_for_output`; `output_tags` + `output_tag_map` | None — BRC-147/163 tag vocab is just strings |
| `customInstructions` on outputs | `outputs.custom_instructions TEXT`; `OutputRepository::insert_output(.., custom_instructions)` stores it **verbatim** on the `createAction` path (`handlers.rs :: create_action_internal`, the `pending_basket_outputs` insert) | ⚠️ **The `internalizeAction` basket-insertion arm wraps it**: `{"type":"basket_insertion","basket":…,"appData":<ci>}` (`handlers.rs :: internalize_action`, ~line 13309). A BRC-147/163 reader looking for `customInstructions.origin` / `.amt` at the top level finds them nested under `appData`. This is a shape we would otherwise fix twice — fix once, in P1 |
| Exclusion from spending | `output_repo.rs :: get_spendable_by_user` / `get_spendable_confirmed_by_user` / `calculate_balance` — `basket_id IS NULL OR b.name = 'default'` | None for spending; **the gap is dApp-supplied inputs**, see below |
| Listing what the wallet holds | `handlers.rs :: list_token_outputs` (`GET /wallet/tokens`) — every spendable output in a non-`default`, non-`identity_certificates` basket, with tags | No CI, no content, no "verified vs claimed" flag; fine as the base query |
| BRC-100 `listOutputs` with basket gate | `handlers.rs :: list_outputs` → `dispatch_scoped_grant(ScopedCall::Basket{..})` → `PromptType::BasketPermissionPrompt` | View gating exists. BRC-165 scope parsing (`all/collection/app/creator/id`, `id` auto-allow) does not |
| A basket class that can never be silenced by a grant | `permission_service/request_gate.rs :: is_protected_basket` — `default`, `backup-*`, `admin *` force `scoped_grant_exists = false` | ⭐ **This is the R-TOKENPERM shape, already built — for VIEW.** The token-spend class extends this idea to **spend**, where nothing gates today |
| An unconditional prompt in the engine | `matrix_c.rs :: decide_privacy_perimeter` — `SensitiveCertField` always prompts, no opt-out | The token-spend decision is the same shape: always `Prompt`, never `Silent` |
| Scope-missing precedence on payments | `matrix_c.rs :: decide_payment` — `PaymentScopeKind::Basket` → `EngineReason::PaymentScopeBasketMissing` **before** cap checks | ⚠️ `context_builder.rs` sets `payment_scope_kind_missing: None` with a comment that scope-missing logic is not wired (line ~156/200, code reading). Verify at kickoff; it is the natural hook |
| Envelope / script parsing | `script/parser.rs`, `script/pushdrop.rs`; `reconcile.rs :: parse_tx_outputs(raw) -> (value, script)`; `monitor/task_sync_pending.rs :: cache_parent_transactions` stores the raw parent tx | ⛔ **`outputs.locking_script` is a fabrication for every synced output** (`tickets/TICKET_synced_outputs_store_a_fabricated_locking_script.md`, measured). T1 owns the fix; T2 cannot classify BSV-21 (needs the JSON body, ~100–200 B) on a 25-byte synthesised script |
| BEEF / AtomicBEEF | `beef.rs` (BRC-62/95 markers, `validate_beef_ancestry`), `beef_helpers.rs :: build_beef_for_txid` (`MAX_BEEF_ANCESTORS = 50`, silent truncation noted in `R-BEEFOUT`) | BRC-176 forbids Atomic-BEEF pruning and needs full bodies for every same-id token input; our builder is ARC-tuned |
| Per-request approval bound to the body | `PermissionService::consume_and_verify` (body sha256 bound to `X-User-Approved`) | The SDK additionally binds at the **signature** layer (`hashOutputs` + approved outpoints). Compare in P2 |
| Provider facade | `services/providers/gorillapool_ordinals.rs` (UTXO fallback only) | No inscription/content fetch; no ORDFS client |
| Wallet panel UI | `frontend/src/pages/WalletPanelPage.tsx` overlay | No token/ordinal view — a new section in the existing overlay, not a new HWND |
| Backup carries baskets/tags/CI | `backup.rs` — 22 `Backup*` row structs incl. `basket`, `output_tag`, `output_tag_map`, `output.custom_instructions` (doc says `rust-wallet/src/CLAUDE.md`) | None for schema; size is T3's question |

**The live defect this table exposes (code reading, `handlers.rs :: create_action_internal`):** dApp-
supplied `inputs` are resolved from `inputBEEF` (lines ~4957–5032) with **no lookup of the row's
basket**, then reserved with `mark_multiple_spent` for any tracked row (lines ~5434–5466). The payment
gate prices the call from **outputs only** — `cef-native/include/core/PaymentCost.h ::
ExtractOutputSatoshis` sums `outputs[].satoshis` — so a `createAction` that spends a `1sat`-basket
output into a 1-sat output is priced at 1 satoshi ≈ 0 cents and reaches `SilentWithinCaps`.
⇒ **An approved domain with a pay grant can spend a filed ordinal silently today.** That is
`R-TOKENPERM`'s RED, observable now, before any T2 code. 📏 The wallet already holds such rows: 9
non-default-basket outputs were measured on the dev wallet on 2026-09-16 (`output_repo.rs ::
calculate_balance` comment — a name token, a todo token, seven upvote tokens).

---

## 4. Ticket review

| Ticket | Still true at head? | Becomes | Why |
|---|---|---|---|
| `TICKET_brc100_consent_model_diverges_from_1sat_wallet_api.md` — **a peer wallet deleted the auto-approve layer we rely on** | **Yes, and half-answered by this pass.** Their argument is **about agents, not architecture**: the no-prompt endpoint is documented for *"agents and automation"*, and their **GUI wallet still prompts per action** for asset spends with a `hashOutputs` commitment cache (§2.2). So the removal does **not** argue against our auto-approve engine for payments; it argues that **asset spends are per-action prompts** — which is BRC-165's rule anyway. The port re-point (3321) is still unmeasured. The licence discrepancy (repo `null`, npm MIT) is unchanged | **Item** inside **P2 (token-spend permission class)** — the written comparison + the one-session port measurement. The ticket's own control still applies: the comparison must name one thing of theirs that is better (candidate: signature-layer binding of the approval) and one of ours (candidate: origin derived in-browser, a human at the keyboard) | It is a read, not a build; it sits exactly where the permission class is designed. Also watch **BRC-181** (open) — the agent-spend-policy BRC is the ecosystem's answer to the same question |
| `TICKET_token_outputs_lost_if_crash_between_broadcast_and_record.md` — **a crash between broadcast and record loses the record of an ordinal** | **Partly stale, and the true shape is different.** Code reading: `create_action_internal` inserts basket outputs (with CI) **before** signing and broadcast (~6178–6230), then the transaction row, then broadcasts — so for ordinals **we send**, record-before-broadcast already holds. What is genuinely non-recoverable is (a) the **annotation** (origin, name, CI, tags) on an output re-discovered by address sync — T1's ingest classifier must reproduce the basket but cannot reproduce sender-supplied CI; and (b) ordinals delivered to a **counterparty-derived** key (the SDK sends to `protocolID/keyID/counterparty`-derived keys, CI-only) — address sync **never finds these**, so a crash between `internalizeAction`'s ownership check and its insert loses the UTXO itself. Related: `TICKET_transaction_row_can_sit_at_created_while_its_coin_is_on_chain.md` (T1) — the `created`-status gap has no reconciler, protectively | **Item** inside **P3 (receive & deliberate transfer)**: write the per-token-type crash contract (self-derivable ⇒ re-discover; CI-only ⇒ write-ahead + affirmative cleanup), and prove it with a kill between insert and broadcast on a scratch profile | Must be settled before an ordinal can be received, per the ticket; it does not need its own phase because the ordering for the send path is already right |
| `WATCH_fungibles.md` — the BSV-21 watch | **Gate conditions 3 and 5 have fired**: (3) the amount-commitment question is answered for BSV-21 (inscription every hop + indexer / BRC-176); (5) the owner's D8. Conditions 1/4 (BRC-175 merged or reconciled) have **not** — it is dirty and idle | **Closes as a watch on BSV-21; stays open as a watch on BRC-175 and PR #267 (BRC-182 / 162 withdrawal).** The old "phase 2.6 review" is delivered by this §2 | The three re-check questions: one model for BSV-21 (yes); does it change T1's seam output (yes — a second basket, `bsv21`, see §7); is there something to test against (yes — 1Sat wallet-desktop/CLI, Yours 5.1.0) |

---

## 5. Candidate phases (ordered)

Sequencing follows `RELEASE_CYCLE.md` §3.7: **uncertain first**, then the dependency chain. P1 needs
T1's seam; P2 needs nothing from T1 and closes a live hole — so P2 can start first if T1 is not ready.

### B5-T2-P1 — Classify and file: ordinals into `1sat`, BSV-21 into `bsv21`, everything else held

**Objective.** Every 1-sat output entering the wallet by any ingest route is filed per BRC-147/163, with
tags and `customInstructions` stored in the spec's shape, and anything the wallet cannot positively
classify is held unspendable.

- Ordinal (first `ord` envelope, not `application/bsv-20`) ⇒ basket `1sat`, tags `ordinal`, `origin:…`,
  `type:…` (BRC-147). BSV-21 (`application/bsv-20`, `p:"bsv-20"`, value op) ⇒ basket `bsv21`, tag
  `bsv21:<id>`, CI `{id, amt, op, …}` (BRC-163). BRC-163: *"Wallets MUST NOT place ordinary payment
  change, `1sat` collectables, or authority-only outputs into `bsv21`."* Unknown mime / unparseable ⇒
  T1's `Unknown` (held, not spendable, not filed).
- Store CI **verbatim** on `internalizeAction` (unwrap the `appData` nesting) — or record the reason not to.
- Route `p 1sat <scope>` request baskets to storage `1sat` (BRC-165) instead of rejecting; keep the
  reject for every other `p ` scheme.
- Tickets: none directly; consumes T1's fabricated-script fix.
- **Unknowns (K):** whether T1 lands the true script (or a prefix long enough for the BSV-21 JSON body,
  ≥ ~256 B — the 64 B in the fabrication ticket's option A is enough for `ord`, **not** for BSV-21);
  whether the GorillaPool `…/inscriptions` endpoint is cheaper than parsing cached parent txs.
- **Negative control:** feed the two measured mainnet fixtures (the `7faac48b…` inscription prefix
  `76a914…88ac 0063036f7264 5109 image/png`, and a `bsv-20` transfer JSON) through the real ingest
  function with the classifier disabled → both land `basket_id IS NULL, spendable = 1`. Assert the
  **basket**, not "classification ran". Subject: the stored row.

### B5-T2-P2 — The token-spend permission class (`R-TOKENPERM`)

**Objective.** No `createAction` / `relinquishOutput` / `signAction` that consumes an output in an asset
basket (`1sat`, `bsv21`, and any non-`default` basket the user did not create for payments) can be
auto-approved. It prompts **per action**, regardless of pay grants, caps, `bundled_scope_grant`, quiet
mode or session opt-ins; the prompt names the asset (origin / token id + amount) and the destination.

- Engine: a new decision path in `matrix_c.rs` of the `SensitiveCertField` shape (always `Prompt`),
  driven by a context field the Rust side computes from the **inputs' baskets** — not from C++'s
  output-priced headers. Label `p 1sat input id <key>` is accepted and routed (BRC-165) but the gate
  keys on **what is spent**, so an unlabeled spend of a filed asset still prompts.
- `is_protected_basket` grows to cover asset baskets for the view path as well (BRC-165: *"Item view is
  not covered by ordinary payment / auto-pay grants"*), with scope `id` auto-allowed.
- The gold pill must **not** fire for a prompted token spend (it is not auto-approved) — and a token
  spend has no cents, so `OnWalletCallSuccess`'s no-floor rule is irrelevant; check it is not routed as
  `wasAutoApprovedPayment`.
- Tickets: the **consent-model comparison** (item), including the 3321 re-point measurement.
- **Unknowns (K):** whether `payment_scope_kind_missing` is genuinely unwired (code comment) — if so the
  cleanest hook is missing and needs building; how `signAction` (two-phase) learns the inputs' baskets.
- **Negative control:** `R-TOKENPERM`'s RED as written — grant the broadest pay permission the UI
  allows, then `createAction` spending a filed 1-sat row into a 1-sat output; with the class disabled it
  must go **silent** (observe `SilentWithinCaps` in the audit log), with it enabled it must prompt.
  Subject: the `PermissionDecision`, not the modal. `HARNESS_DELTA.md` §4's `cargo-mutants --in-diff` on
  the engine crate applies here. Adversarial panel per `TELESCOPE.md` §"T2.3".

### B5-T2-P3 — Receive and deliberately transfer an ordinal

**Objective.** A user can receive an ordinal from a 1Sat/Yours wallet, see it, and send it to an address
or identity key on purpose — with the asset never funding fees and never destroyed.

- Receive: `internalizeAction` with the sender's BEEF → BRC-158/159/160 walk to origin (the SDK's
  `internalizeOutpointBeef` checks, ported as pattern) → file per P1 → mark **verified** when the walk
  succeeds, **claimed** otherwise (BRC-147: *"A wallet MUST NOT treat a claimed `origin` as proven
  solely because it appears in tags or `customInstructions`"*). Verify a BRC-150 package **if present**;
  do not build one by default (§2.2 — nobody consumes them, and the reference SDK cannot carry one).
- Transfer: reuse `create_action_internal` — the 1-sat input comes from `1sat`, fees and dust from
  `default`, one 1-sat P2PKH output to the recipient, change to `default`; labels per BRC-165; CI on
  the recipient's output per BRC-147 (`origin`, `name`, `app`), derivation fields when self-kept.
- Crash contract (the ticket): kill between insert and broadcast on a scratch profile; the record must
  survive, and cleanup must be **affirmative** (proof of non-existence on chain), never absence-based.
- **Measurement rows for T3 (`HARNESS_DELTA.md` §1.3):** record real `customInstructions` sizes and
  BEEF ancestry depth per received ordinal. ⚠️ Record honestly that **BRC-150 `beefB64` rows will be
  ~zero** because the ecosystem does not emit them; T3's hypothesis must be re-stated against the BEEF
  that *does* travel (the internalize BEEF, which we cache as parent transactions).
- **Unknowns (K):** whether dApps send ordinals to Hodos at a **plain address** or a **counterparty-
  derived key** — the second is invisible to address sync and is the real crash/backup hazard (also
  T3's and `TICKET_rescan_cannot_find_payments_to_generated_addresses.md`'s question).
- **Negative control:** revert P1's classifier and re-run the transfer builder — the ordinal must be
  reached for as fee input (`R-NOSPEND` GREEN c, its own RED). And send to a Yours/1Sat wallet and
  confirm **it** shows the origin — the receiver's verdict, not ours (`R-BEEFOUT`'s rule).

### B5-T2-P4 — Hold and show: the wallet panel view, with BSV-21 balances

**Objective.** The wallet panel shows what the user holds: ordinals (thumbnail/name, origin, verified vs
claimed) and BSV-21 balances (Σ `amt` per token id, symbol, decimals) — with the token's validity shown
as **indexer-claimed** unless a BRC-176 proof was verified.

- Base query: extend `list_token_outputs` (`/wallet/tokens`) with CI, basket class and a status.
- Content: ORDFS `/content/{outpoint}` for ordinal media and BSV-21 `icon` (BRC-161: icon is an
  outpoint pointer; BRC-163: prefer BEEF-embedded icon bytes, HTTP as fallback). Cache locally.
- **Indexer posture decision** (the README's "decide this deliberately"): proposal — indexers are used
  for **display and discovery only**; nothing they return is a decision input for spending; when down,
  the panel shows held rows with "content unavailable", never hides them. Record it as a rule, then it
  is a `REGRESSION_ADDITIONS` candidate.
- No BSV-21 **transfer** in this phase (see P5).
- **Unknowns (K):** rendering arbitrary inscription mimes inside a CEF overlay safely (treat as
  untrusted content — separate origin / sandboxed iframe); whether `dec` display needs big-int handling
  in React (SDK uses `BigInt`).
- **Negative control:** point the content client at a dead host — every held row still lists; a hidden
  row is the failure.

### B5-T2-P5 — BSV-21 transfer *(candidate, gated — see §9 Q2)*

**Objective.** Send BSV-21 tokens conforming to BRC-161/163/176 and the SDK's wire behaviour.

- Needs: a BSV-21 inscription template (JSON body per BRC-161), conservation `T ≤ S` with excess burn
  awareness, change re-inscribed, the **`fee:overlay` output** (metered indexing — an ecosystem cost we
  pay per token output), a **BRC-176 BEEF builder** (full bodies, no Atomic pruning — our
  `build_beef_for_txid` is ARC-tuned and caps at 50 ancestors with silent truncation), and P2's prompt.
- **Unknowns (K):** BRC-176 packet sizes for real tokens (unmeasured anywhere); whether the overlay's
  admission is deterministic enough to call the send "done" (the SDK itself treats overlay state as
  *advisory* and keeps `pending`).
- Recommendation: **defer to beta.6 intake** unless the owner names a user need in beta.5. Hold/show
  (P4) already protects and displays; transfer is where the indexer economics and the largest new
  builder live.
- **Negative control:** a transfer with `T > S` must be refused locally before broadcast; with the
  conservation check disabled it must be built (and the overlay must then reject it — the receiver's
  verdict).

**Not a phase:** the old "2.6 BSV-20/21 review". Delivered by §2 of this document.

---

## 6. Integration check (`RELEASE_CYCLE.md` §3.3)

**1. Invariants at risk.**

| Invariant | Risk | Mitigation |
|---|---|---|
| **Privacy perimeter** (identity-key reveal, key linkage, sensitive cert fields, over-cap spends) | Untouched by design — the new class is added beside them. Risk is a refactor of `decide()`'s cascade order | Adversarial panel on P2; `cargo-mutants` on the crate; `R-PERIM` at the boundary |
| **Auto-approve engine** (`SilentWithinCaps`) | Wrong: token spends stay silent. Also wrong: ordinary payments start prompting because a `default`-basket input is misread as an asset | P2's RED (silent with class off) and an explicit GREEN that an ordinary paid `createAction` stays silent; `R-GOLD` at the boundary |
| **The gold pill** | A prompted token spend must not flash the pill; a silent one must never exist | Assert the pill does not fire on the prompted path; `R-GOLD` |
| **Fail closed** (`HARNESS_DELTA.md` §1.1) | A classifier that reads a fabricated script passes everything | P1 depends on T1's fix; the fixture-based RED asserts the basket |
| Balance display | Filing money into `1sat` by mistake hides a user's real balance (`calculate_balance` excludes non-default baskets) | BRC-147 eligibility (`satoshis === 1` + envelope) is the classifier's minimum; a 1-sat **payment** without an envelope stays `Unknown`, not `1sat` |

**2. What we touch that we did not write.** BRC-147/150/159/160/161/163/165/176 text (all merged; 162
being withdrawn); the reference SDK's tag (≤ 300 B) and `customInstructions` (≤ 1000 B on internalize)
limits — an interop ceiling we must respect when writing CI; GorillaPool ordinals API and ORDFS
(unlicensed hosted services); WhatsOnChain's unspent endpoints (which do not return scripts — the root
of the fabrication); `@bsv/sdk` BEEF/BUMP semantics; CEF overlay content rendering for untrusted media.
Nothing in the engine, Chromium or Sparkle.

**3. What we would un-ship if wrong.** Basket assignments are reversible (`remove_from_basket`, or a
migration back to `default`); the permission class is a Rust-only cascade branch and can be reverted
without a schema change. **Irreversible:** an ordinal spent into a >1-sat output (BRC-159: the chain
ends) and a BSV-21 output spent without its inscription (the overlay rejects the child; the value is
burned). Both are exactly what P1+P2 exist to prevent, which is why they precede P3/P4.

---

## 7. Feasibility inputs

**Owner-hours (N) — human-bound rows only:**

| Row | Why human | Estimate |
|---|---|---|
| Receive a real ordinal from Yours 5.1.0 or the 1Sat desktop wallet into Hodos | real asset, second wallet, visual check of the panel | ~1.0 h |
| Deliberate transfer back out, then confirm the receiver shows the origin | real money, receiver's verdict | ~1.0 h |
| Receive a real BSV-21 token and see the balance (P4) | second wallet, overlay-fee token | ~0.5 h |
| The token-spend prompt: broadest pay grant, then a spend — visual consent surface (`feedback_consent_surface_needs_human_eyes`) | native prompt, judgement | ~0.5 h |
| The destructive negative control with a real 1-sat asset on a scratch profile (`HARNESS_DELTA.md` §1.2) | staging real outputs | ~0.5 h |
| P5 if taken: a real BSV-21 send | real money, overlay admission | +1.0 h |

≈ **3.5 owner-hours without P5, ~4.5 with.** Everything else is agent-driven on a scratch profile.

**Unknowns (K) = 4 phases with a genuine unknown** — P1 (script availability), P2 (the scope-missing
hook), P3 (plain address vs derived key delivery), P5 (BRC-176 sizes / overlay determinism). P4 is
hard-but-understood.

**Cross-track dependencies:**

| Edge | What T2 needs | From |
|---|---|---|
| **T1 RQ-1** — what "classified" persists as | ⭐ **Baskets are the ecosystem's classification** (1Sat SDK `ONESAT_ASSET_BASKETS`; `wallet-toolbox` `TableOutput` has `basketId`, `type`, `customInstructions`, `scriptLength`, `scriptOffset`, web `bsv-blockchain/wallet-toolbox/src/storage/schema/tables/TableOutput.ts`, 2026-09-25). So T1's `Token` should persist as **a basket assignment (`1sat` / `bsv21`)**, and T1's `Unknown` needs a representation the ecosystem does not have — a held-not-spendable state that is **not** a basket name a dApp could request. T2 asks for: (a) the seam to emit `Token{basket}` rather than a bare `Token`; (b) the true script or a ≥ 256 B prefix + `script_length`; (c) both ingest tiers (WoC and GorillaPool fallback) classified; (d) the `identity_certificates` precedent kept | T1 microscope (`TELESCOPE.md` E1) |
| **T1** fabricated locking script | Blocking for P1; without it the BSV-21 classifier reads nothing | T1 front-loads it (its own README says so) |
| **T1** `created`-status reconcile ticket | P3's crash contract cites it; not blocking | T1 |
| **T3 backup** — measurement rows | P3 records CI sizes and BEEF depth; ⚠️ tells T3 that BRC-150 rows will be near zero and the workload is the internalize BEEF / cached parent tx (`parent_transactions.raw_hex` is already in the backup payload — a 2.6 MB inscription parent goes into every backup today, measured in the fabrication ticket §3.4) | T2 → T3 (`TELESCOPE.md` E2) |
| **T3 restore** — counterparty-derived ordinal keys | RQ-2's "held-but-unidentified" must cover outputs at keys only the CI can re-derive | T2 → T3 (`TELESCOPE.md` E3, `R-RESTORE`) |
| **T5 identity** — the consent comparison touches `domain_trust_gate` and origin attribution | Read-only in T2; any change to origin handling is T5's | T2 → T5 |

---

## 8. Prior-art rows for `development-docs/PRIOR_ART.md`

| Date | Question | Source(s) read | What we learned | Verdict | Landed in |
|---|---|---|---|---|---|
| 2026-09-25 | Is the BSV-21 amount-commitment question settled, and what does BRC-175 say now? | BRC-161/163/176 (merged text); PR #237 (BRC-175) body + 3 comments + head branch text; PR #267 (BRC-182 / 162 withdrawal); BRC-163 PR #217 discussion | **Settled for BSV-21**: inscription on every output + indexer admission (161), CI as fast path (163), BRC-176 offline BEEF as the indexer-free proof (merged 2026-09-01). **BRC-175 stalled** (dirty, idle since 08-29, core questions unanswered) and moved *to* re-inscription. BRC-162 being withdrawn ⇒ JSON only | 🟢 paid off — reopened the fungibles watch and reshaped T2 | this SCOPE §2.1, §5 |
| 2026-09-25 | How does the reference 1Sat SDK actually file, balance and transfer BSV-21 and ordinals? | `b-open-io/1sat-sdk` at `52cfe69`: `types/constants.ts`, `actions/tokens/index.ts`, `utils/bsv21Remittance.ts`, `wallet/indexers/Bsv21Indexer.ts`, `actions/ordinals/index.ts`, `utils/ordinalRemittance.ts`, `sweep/types.ts`, `docs/plans/2026-08-05-brc-alignment.md` | Plain baskets `1sat` / `bsv21` (never `p `); CI shapes match BRC-147/163; balances = Σ CI `amt`; validity is overlay-advisory (`pending` on 404); transfer adds a metered `fee:overlay` output and a BRC-176 BEEF. ⭐ **Nobody produces BRC-150** (`beefB64` zero hits; plan says "prefer omit") — and the reference client's 1000-byte CI cap makes it impossible to carry | 🟢 paid off — removed BRC-150 *production* from scope; sized P5 honestly | §2.2, P3, P5 |
| 2026-09-25 | Does the SDK's removal of auto-approve argue against ours? | `packages/cli/skills/wallet-api/SKILL.md`; `permission-module/src/handlers.ts`; BRC-181 PR #265 | The no-prompt endpoint is for **agents**; their **GUI prompts per action** for asset spends and binds the approval to `hashOutputs` + input outpoints at signature time. BRC-181 is the ecosystem's agent-policy answer (open) | 🟡 context — confirms our payment engine's position; gives P2 one pattern to consider (signature-layer binding) | §4, P2 |
| 2026-09-25 | Where does a conforming wallet keep "this output is a token"? (for T1 RQ-1) | `wallet-toolbox` `TableOutput.ts`; 1Sat SDK basket constants | `basketId` + `customInstructions` + `type` + `scriptLength`; asset baskets are plain names. No "unknown" state exists in either — ours must | 🟢 paid off — an answer for T1 to check, not to re-derive | §7 |

---

## 9. Open questions for the owner — each with a recommendation

| # | Question | Recommendation |
|---|---|---|
| **Q1** | **Does T2 grow to "ordinals + BSV-21"?** The specs BSV-21 rests on (161/163/176) are merged and implemented by the reference SDK; BRC-175 is not. | **Yes, for classify + hold + show (P1, P4).** The wallet needs BSV-21 code regardless of whether it ever sends one: a BSV-21 output is a 1-sat output that arrives at our addresses today, must be filed into `bsv21` and **not** `1sat` (a MUST in both 163 and 175), and cannot be shown as a balance without decoding its inscription. The old gate question — *"is it only the apps that talk to wallets?"* — answers **no**: the app cannot classify what the wallet has not stored |
| **Q2** | **BSV-21 transfer (P5) in beta.5 or beta.6?** | **Defer to beta.6 intake** unless there is a named user need. It carries the metered overlay fee, a BRC-176 builder our BEEF code is not shaped for, and an admission model the SDK itself calls advisory. Hold/show already removes the destruction risk |
| **Q3** | **BRC-150 provenance: verify-only, or also produce?** Nobody in the ecosystem produces a package and the reference client cannot carry one in CI. | **Verify if present; do not produce by default.** Revisit if a counterparty appears. Tell T3 plainly that its `beefB64` workload hypothesis has no data source; the real workload is the internalize BEEF / cached parents |
| **Q4** | **Per-action prompt for every token spend (BRC-165), or allow a standing "this site may move my ordinals" grant?** BRC-165 says per action; our permission model is built around standing grants. | **Per action, no standing grant, for beta.5.** It is the spec, it is what the peer GUI does, and a standing grant is how a permission surface widens silently (`TELESCOPE.md` E4). A "remember for this session" opt-in can be argued in later, deliberately |
| **Q5** | **Indexer posture:** display/discovery only, never a decision input, degrade to "content unavailable"? | **Yes**, record it as a rule in P4 and propose it for `REGRESSION_ADDITIONS.md` at track close |
| **Q6** | **Order:** P2 (the permission class) before P1 if T1's script fix is late? The hole P2 closes is live today (§3). | **Yes** — P2 has no T1 dependency and front-loads the release's most uncertain design |
| **Q7** | **The 1Sat SDK licence** — repo `license: null`, npm `MIT`. T2 takes no runtime dependency (Rust wallet, patterns only), so this only governs reading and porting patterns. | Proceed on patterns-only, as the ticket already says; no action unless we ever vendor TypeScript |

---

## Links

- `README.md` (this folder) — the 2026-08-05 decision this scope refines; `RESEARCH_FINDINGS.md`
  (2026-05-05, provider APIs — still accurate on GorillaPool endpoints, stale on "nothing to test against")
- `WATCH_fungibles.md` — updated verdict in §4 above; keep for BRC-175 / PR #267
- `../tickets/TICKET_brc100_consent_model_diverges_from_1sat_wallet_api.md`,
  `../tickets/TICKET_token_outputs_lost_if_crash_between_broadcast_and_record.md`,
  `../tickets/TICKET_synced_outputs_store_a_fabricated_locking_script.md` (T1, blocking P1)
- `../REGRESSION_ADDITIONS.md` — `R-NOSPEND`, `R-CLASSIFY`, `R-TOKENPERM`, `R-RESTORE`, `R-BEEFOUT`
- `../HARNESS_DELTA.md` §1.3 — the measurement rows this track owes T3
- `../TELESCOPE.md` — edges E1–E4
