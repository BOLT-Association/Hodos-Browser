# B5-T2-P2 — No site can spend your ordinals or tokens without asking you, every time · PHASE CONTRACT

**Track:** B5-T2 1Sat Ordinals · **Tickets:** `../../tickets/TICKET_brc100_consent_model_diverges_from_1sat_wallet_api.md` (item) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent · **Platforms:** both (Rust engine is shared; the prompt surface is per-platform) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **8** (per-action token-spend prompt, no standing grant, BRC-165 MUST; this phase is built **first**; no T1 dependency) · **2 / 2a** (money = positively marked — this phase must not rely on it, because it lands before T1) · T2 **Q7** (1Sat SDK: patterns only, no code, no runtime dependency).

> **Phase order.** Decision 8: P2 is the first T2 phase. It closes a hole that is **live today** (T2 SCOPE §3,
> "the live defect this table exposes"), and it needs nothing from T1.

---

## 1. Goal

A website that the user has approved for payments — however broad its pay grant, caps or session opt-ins — can no longer move or detach an ordinal or BSV-21 token the wallet holds without a prompt for **that one action** that names the asset and where it is going.

## 2. Done means

- [ ] A `createAction` from an external origin whose `inputs` name an outpoint held in an **asset basket** (§12 Q1 fixes the set; recommendation `1sat`, `bsv21`, `bsv20`, `opns`) produces a Rust `PermissionDecision::Prompt` with a token-spend reason — with the broadest pay grant the UI allows, a zero-cost amount, and every session opt-in set.
- [ ] The same holds when the named input is **held-but-unclassified** (a 1-sat row in no basket today; T1's `Unknown` once T1-P5 lands) — fail closed, `HARNESS_DELTA.md` §1.1.
- [ ] An ordinary paid `createAction` that spends nothing from an asset basket still goes **silent within caps** and still lights the **gold pill** (the two-sided pair to the line above).
- [ ] `relinquishOutput` on an asset basket prompts per action and cannot be silenced by a basket grant or `bundled_scope_grant` — because today it sets `basket_id = NULL`, which moves the output into the default-basket pool that coin selection reads.
- [ ] Approval binding (BRC-165 *"A later `createSignature` … MUST NOT expand that set without a new approval"*): a `createSignature` whose BIP-143 preimage spends an asset-basket outpoint that is not in an approved, unexpired spend set is **refused**. ⚠️ Whether this item ships in P2 is §12 Q2.
- [ ] `p 1sat input id <key>` labels are accepted and do not error (a malformed one — empty key — is refused); the gate keys on **what is spent**, never on the label, so an unlabeled spend of a filed asset still prompts (BRC-165 §"Outpoint spend without labels").
- [ ] The prompt names: the site, the asset (origin, or token id + amount), and the destination output(s). Human-checked on both platforms.
- [ ] A prompted-then-approved token spend does **not** light the gold pill.
- [ ] `cargo-mutants --in-diff` run on `hodos_permission_engine` for the new branch; every surviving mutant explained or killed (`HARNESS_DELTA.md` §4).
- [ ] The consent-model comparison is written (ticket item): one table, their four decisions vs ours, **naming at least one decision of theirs that is better and one of ours** (the ticket's own control); plus the one-session measurement of whether a `1sat serve wallet-api` client survives our port-3321 re-point.

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-TOKENPERM` | Pay grants never authorize token spends (BRC-165) | This phase **creates** it; it must be seen RED on today's build first |
| `R-PERIM` | The four privacy-perimeter gates | The new branch goes into `matrix_c.rs :: decide`'s cascade next to them; a reorder changes their precedence |
| `R-GOLD` | The gold pill fires on every auto-approved payment | A misclassified default-basket input makes ordinary payments prompt (pill disappears); a prompted token spend could be routed as `wasAutoApprovedPayment` (pill lies) |
| `R-INTEXT` | Internal never prompts, external always gates | The gate must read `X-Requesting-Domain` the same way `dispatch_payment` does; the wallet UI's own sends (P3) must stay unprompted by the engine |
| `R-ONE-CLICK-ONE-SPEND` | One Approve signs exactly what it was shown | The token-spend approval must bind to the body (`PermissionService::consume_and_verify`), and the signature-layer binding must not widen it |
| `R-COUNT` | Per-session counters | A token spend must not increment or be silenced by the payment counters |
| `R-DUST` (beta.3) | The 1-sat floor on incidental paths | Untouched — its documented **excluded** path (dApp-named `user_inputs`) is exactly what this phase closes; update nothing in `REGRESSION_SET.md` (rule 6) |

## 4. Evidence table

⛔ Money / permission-on-spend rows: RED designed by a second agent (`../../../RELEASE_CYCLE.md` §4.2).
**Subjects named before the run** (`HARNESS_DELTA.md` §1.1): staged on a **scratch profile** — one real 1-sat inscription output filed in `1sat` (outpoint recorded here before the run: `________`), one real BSV-21 transfer output in `bsv21` (`________`), one unfiled 1-sat output (`________`), and ordinary multi-sat default-basket coins.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P2-A0` | **Baseline, before any code:** on today's build, an approved domain with the default caps spends the staged `1sat` outpoint via `createAction` `inputs` into a 1-sat output and the engine returns **`Silent{SilentWithinCaps}`** | Not applicable — this row **is** the RED of `P2-A1`, recorded first so the green that follows has a baseline (`REGRESSION_ADDITIONS.md` "run it once before and record it RED") | Rust audit log line for that request (`permission_service/audit.rs`) carrying `SilentWithinCaps` + the named outpoint; stop before broadcast (`noSend` / scratch) | T2 | ⬜ |
| `P2-A1` | Same request after the change ⇒ `Prompt` with the token-spend reason, for each of: broadest pay grant, `rate_limit`/caps at max, identity + key-linkage session opt-ins set, `bundled_scope_grant = 1` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The `PermissionDecision` kind + reason (`R-TOKENPERM` SUBJECT), not the modal; the named outpoint in the request | T1 + T2 | ⬜ |
| `P2-A2` | Unlabeled spend (no `p 1sat input …` label) of the `1sat` row ⇒ still `Prompt` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Decision + reason; request body shows no `p 1sat` label | T1 | ⬜ |
| `P2-A3` | Spend of the `bsv21` row ⇒ `Prompt` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Decision + reason; outpoint + basket `bsv21` | T1 | ⬜ |
| `P2-A4` | Spend of the **unfiled** 1-sat row (no basket; later T1 `Unknown`) ⇒ `Prompt` — fail closed | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Decision + reason; the row's `basket_id IS NULL`, satoshis = 1 | T1 | ⬜ |
| `P2-A5` | **Pair to A1:** an ordinary paid `createAction` (no `inputs`, or inputs that are default-basket multi-sat rows) under caps ⇒ `Silent{SilentWithinCaps}` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Decision + reason for a request whose inputs are named default-basket rows | T1 + T2 | ⬜ |
| `P2-A6` | `relinquishOutput` on the `1sat` row with a `read_write` basket grant **and** `bundled_scope_grant = 1` ⇒ `Prompt` every time; a second call after one approval prompts again | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Decision + reason from `dispatch_scoped_grant`; the row's `basket_id` unchanged after a denial | T1 + T2 | ⬜ |
| `P2-A7` | *(if §12 Q2 = yes)* `createSignature` with a BIP-143 preimage spending the `1sat` outpoint, no approved spend set ⇒ refused; the same after approving a **different** spend set ⇒ refused; after approving **this** set ⇒ allowed once | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The `createSignature` response + the Rust log naming the outpoint parsed from the preimage | T1 + T2 | ⬜ |
| `P2-A8` | One Approve on a token-spend prompt ⇒ exactly one `X-User-Approved consumed`, and a replay with an altered `inputs` array ⇒ 403 `body_mismatch` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | `X-User-Approved consumed` log lines + the 403 envelope; `transactions` rows created | T2 | ⬜ |
| `P2-A9` | A prompted-then-approved token spend lights **no** gold pill; an auto-approved ordinary payment on the same tab still does | Stub the new "was prompted" discriminator so the approved replay is treated as auto-approved ⇒ the pill appears on the token spend (must be seen) | The `payment_success_indicator` IPC emit at `HttpRequestInterceptor.cpp :: OnWalletCallSuccess`, on the tab found via `TabManager::GetTabIdForBrowserIdentifier` | T2 + T3 | ⬜ |
| `P2-A10` | The prompt shows site, asset (origin or token id + amount) and destination, readable at 100 % and 150 % DPI | Remove the asset field from the prompt payload ⇒ the human reviewer must mark the row FAIL (the reviewer is told a field *may* be missing, not which) | The rendered modal in the notification overlay, read by a person (`feedback_consent_surface_needs_human_eyes`) | T3 | ⬜ |
| `P2-A11` | `p 1sat input id <key>` labels accepted; `p 1sat input id` with an **empty** key rejected (BRC-165 §Held row rule 2: *"Non-empty, no spaces"*) | Remove the empty-key check ⇒ the empty-key label is accepted (must be seen) | The `createAction` response for each label form | T1 | ⬜ |
| `P2-A12` | `cargo-mutants --in-diff` on the engine crate: zero unexplained survivors in the new branch | A deliberately weakened test (assert only "no error") must leave mutants alive — proves the run can report survivors | `cargo mutants` report file | T1 | ⬜ |
| `P2-A13` | Consent comparison written; names ≥1 better-theirs and ≥1 better-ours; port-3321 measurement recorded (survives / fails / how) | A draft that is all one way is sent back (the ticket's control) | The document + the `debug_output-<pid>.log` re-point line | — | ⬜ |

**Two-sided rows:** `P2-A1`↔`P2-A5` (token spend prompts / ordinary spend stays silent) and `P2-A9` halves (no pill on the token spend / pill on the payment) are each other's controls.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/crates/hodos_permission_engine/src/matrix_c.rs :: decide` | ✅ | Cascade: trust → perimeter → scoped → payment → cert → generic. New branch sits after trust, before payment caps |
| `…/matrix_c.rs :: decide_privacy_perimeter` | ✅ | The `SensitiveCertField` always-prompt arm is the shape to mirror |
| `…/matrix_c.rs :: decide_payment` | ✅ | `payment_scope_kind_missing` short-circuit is the natural hook |
| `…/src/context.rs :: PermissionContext`, `CallKind` | ✅ | Needs one new input (inputs-touch-asset). No `TokenSpend` kind today |
| `…/src/decision.rs :: EngineReason` (`SilentWithinCaps`, `PaymentScopeBasketMissing`) | ✅ | New reason variant |
| `rust-wallet/src/permission_service/context_builder.rs :: build_payment_context` | ✅ | Sets `payment_scope_kind_missing: None` with a comment that it is not wired — **confirmed unwired** (SCOPE §3's K) |
| `rust-wallet/src/permission_service/request_gate.rs :: dispatch_payment_with_amount`, `dispatch_scoped_grant`, `is_protected_basket` | ✅ | `is_protected_basket` = `default` / `backup-*` / `admin ` only |
| `rust-wallet/src/permission_service/state.rs :: consume_and_verify` | ✅ | Body-sha256 binding reused as-is |
| `rust-wallet/src/handlers.rs :: create_action` | ✅ | Calls `dispatch_payment` before parsing; the inputs' outpoints are in the body |
| `rust-wallet/src/handlers.rs :: create_action_internal` | ✅ | User `inputs` resolved from `inputBEEF`, reserved via `OutputRepository::mark_multiple_spent`, **no basket lookup** |
| `rust-wallet/src/handlers.rs :: process_action` | ✅ | Gated; `ProcessActionRequest` has **no `inputs`** ⇒ cannot name an asset — out of this phase |
| `rust-wallet/src/handlers.rs :: sign_action` | ✅ | Signs the pending tx by `reference`; user inputs signed via `spends` or the row's derivation fields. No separate gate — covered by the gate at `create_action` |
| `rust-wallet/src/handlers.rs :: create_signature` | ✅ | Gated by `dispatch_scoped_grant(ScopedCall::Protocol)`; level 0 silent. ⚠️ Also edited by T5-P3 (decision 12) |
| `rust-wallet/src/handlers.rs :: relinquish_output` → `database/output_repo.rs :: remove_from_basket` | ✅ | Sets `basket_id = NULL` ⇒ row satisfies `get_spendable_by_user`'s `basket_id IS NULL` today |
| `rust-wallet/src/database/output_repo.rs :: get_by_txid_vout`, `get_spendable_by_user` | ✅ | Lookup used to classify an input; selector shows why relinquish is a spend path |
| `cef-native/include/core/PaymentCost.h :: IsPaymentEndpoint`, `ExtractOutputSatoshis` | ✅ | Prices by outputs only — a token spend into 1 sat is ~0 cents. C++ **not** changed: the decision is Rust's |
| `cef-native/src/core/HttpRequestInterceptor.cpp :: OnWalletCallSuccess` + IPC arm's local `wasAutoApprovedPayment = ok && isPaymentKind && !isErrorInResponse` | ✅ | ⚠️ Derived locally, not from Rust's decision — the `P2-A9` risk. Its comment still says "green-dot animation" (stale name; not this phase's to fix) |
| `cef-native/src/handlers/simple_render_process_handler.cpp` (`payment_success_indicator`), `frontend/src/hooks/useTabManager.ts` | ✅ | Pill chain, read-only here |
| `cef-native/include/core/PortConfig.h :: IsWalletOrigin`; `cef-native/src/handlers/simple_handler.cpp` (3321 re-point comment) | ✅ | ⚠️ The ticket cites `IsWalletOrigin` in `simple_handler.cpp`; it lives in `PortConfig.h` |
| BRC-165 text (`raw.githubusercontent.com/bsv-blockchain/BRCs/master/tokens/0165.md`, fetched 2026-09-28) | ✅ | §Spend items 3 + 4, §Security "Approval binding"; §Out of scope excludes BSV-21 |
| BRC-163 §"Payment separation" (fetched 2026-09-28) | ✅ | `bsv21` spend/relinquish: SHOULD require distinct authorization |
| 1Sat SDK `packages/permission-module/src/handlers.ts` at `52cfe69` (no commits since 2026-09-20, checked 2026-09-28) | ✅ | Per-action prompt; `createSignature` parses the BIP-143 preimage from `args.data` and checks outpoint + `hashOutputs` against the approved set. Pattern only (Q7) |

## 6. Out of scope

- A standing or session "this site may move my ordinals" grant (decision 8). No "remember" checkbox on this prompt.
- BRC-165 **view** scopes (`p 1sat all|collection|app|creator|id` routing of `listOutputs`) — belongs with filing (P1), where `validate_and_normalize_basket_name` is touched.
- Stamping BRC-164 `id:` tags. Without them `p 1sat input id <key>` cannot be *resolved*; this phase accepts the label and gates on outpoints.
- Spending app PushDrop baskets (todo, upvote, name markers) — unchanged unless §12 Q1 says otherwise.
- Coin-selection paths (dust consolidator, `send_max`, sweep) — T1 (`R-NOSPEND`).
- Changing the auto-approve engine for payments, or implementing the SDK's grant-then-retry 400 (ticket: out of scope).
- Fixing the "green-dot" comment wording.

## 7. Rollback

Revert the one engine + gate commit: the new cascade branch, the context field and the `relinquish_output` / `create_signature` checks are Rust-only, no schema. Behaviour returns to `P2-A0`.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| Gate reads the **label**, so an unlabeled outpoint spend stays silent | `P2-A2` |
| Gate reads the basket from a row lookup that misses when the dApp names the outpoint in `txid.vout` vs `txid_vout` form, or the row is `spendable = 0` / reserved | `P2-A1` run with both outpoint forms; the lookup must not filter on `spendable` |
| Every `createAction` with *any* input starts prompting (misread default rows) — pill vanishes, users complain | `P2-A5`, `P2-A9` second half |
| dApp skips `createAction`: builds the tx itself and calls `createSignature` with the ordinal's derivation triple (it knows it — it sent the ordinal) | `P2-A7` — ⚠️ only if §12 Q2 = yes; otherwise a **known open hole**, written into §12 and the track README |
| dApp calls `relinquishOutput` on `1sat`, then any coin-selection path spends the now-unbasketed 1-sat row | `P2-A6`; and T1-P3/P5's `change=1` rule closes the second half (edge) |
| `hashToDirectlySign` only (no preimage) ⇒ binding cannot parse the outpoint | `P2-A7` must include this case and record the outcome; §10 K |
| Approved token spend replayed with different inputs | `P2-A8` |
| Prompt renders behind another window (beta.4 two-window Z-order bug) and times out ⇒ looks like "denied" | `P2-A10` run in two-window setup; edge to T6 |
| New branch placed after `decide_payment` so a cap prompt (not a token prompt) appears — user approves a "payment" | `P2-A1` asserts the **reason**, not just `Prompt` |
| Test drives the engine with a hand-built context that production never builds | `P2-A1`/`A5` run **T2** through `create_action` over the real IPC bridge (`feedback_test_subject_must_be_the_production_call`) |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | Scratch profile under `HodosBrowserDev` |
| macOS | `P2-A1`, `P2-A5`, `P2-A8`, `P2-A9`, `P2-A10` | Rust rows are shared code; A9/A10 exercise the macOS notification overlay + pill. No C++ change planned ⇒ no MAC_RELAY file note unless one lands |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~0.75 h** — prompt legibility (A10) both DPI cells ~0.4 h; pill check on a real approved token spend (A9) ~0.2 h; staging the three real subject outputs ~0.15 h (shared with P1) |
| Human-bound rows | `P2-A10` (T3, visual); `P2-A9` second look (T3) |
| Unknowns (K) — uncertainty, not difficulty | **K = 1 phase.** (a) Whether `createSignature` binding is buildable for the `hashToDirectlySign`-only case; (b) whether a `1sat serve wallet-api` client survives the 3321 re-point |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| T2-P2 → T1-P3/P5 | money index + classifier | This phase guards dApp-named inputs and relinquish **without** T1. Once T1 lands, `change=1` makes a relinquished token non-money too; T1's `Unknown` state must be readable by this gate (`P2-A4` re-run at T1→T2 boundary) |
| T2-P2 ↔ T5-P3 | `create_signature` | Both edit `handlers.rs :: create_signature` (decision 12's counterparty default; this phase's binding). **Serialize at G5**; T5 lands first or both rebase |
| T2-P2 → T6 | prompt surfaces | The prompt lives in the shared notification overlay (`BRC100AuthOverlayRoot.tsx` type dispatch — a new case, no new HWND). T6's two-window Z-order work affects whether it is seen |
| T2-P2 → T3 (E4) | OpNS names | If §12 Q1 includes `opns`, names reuse this class — T3 verifies rather than assumes (TELESCOPE E4) |
| T2-P2 → T2-P3 | wallet-UI transfer | Internal calls carry no `X-Requesting-Domain` and skip the engine (`R-INTEXT`); P3 supplies its own confirm step |

## 12. Open questions for the owner

1. **Which baskets are "assets"?** Two readings of the scope's *"any non-default basket the user did not create for payments"*: (a) **every** non-default basket — this would prompt on every action of PushDrop apps that spend their own tokens (todo, upvote), a UX regression; (b) a **named set**. ⭐ Recommend (b): `1sat` + `bsv21` (the two baskets this release files) + `bsv20` + `opns` (the SDK's other 1-sat carriers, `ONESAT_ASSET_BASKETS`), plus any **unfiled 1-sat** row. App baskets keep today's behaviour.
2. **Signature-layer binding in this phase?** BRC-165 makes it a MUST, and without it a dApp that knows an ordinal's derivation (it sent it) can spend it by calling `createSignature` directly, never touching `createAction`. ⭐ Recommend **yes, in P2**, following the SDK's pattern (parse the BIP-143 preimage, check the outpoint against the approved set). It adds one K (the hash-only case). The alternative is shipping P2 with a written, known hole.
3. `relinquishOutput` on an asset basket: prompt per action (recommended — BRC-163 lists relinquish beside spend), or refuse outright?

⚠️ No evidence that decision 8 is wrong. Evidence that its *mechanism* is larger than the scope said: the `createSignature` path (Q2) and `relinquishOutput` (Q3) are spend paths the scope did not list.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded
- [ ] Adversarial review of the evidence complete, four questions answered in writing (TELESCOPE T2.3: **adversarial panel**)
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
