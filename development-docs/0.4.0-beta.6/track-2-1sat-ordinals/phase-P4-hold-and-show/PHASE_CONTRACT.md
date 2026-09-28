# B5-T2-P4 — You can see every ordinal and token you hold, with BSV-21 balances, and none of it is ever hidden or counted as money · PHASE CONTRACT

**Track:** B5-T2 1Sat Ordinals · **Tickets:** none directly · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent (finishing run) · **Platforms:** both (Rust + shared React; no C++ planned) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **8** (BSV-21 **recognise, hold, show** — P4 is the "show"; `bsv21` basket, never `1sat`; ⛔ **no BSV-21 transfer**, no send button of any kind for tokens) · **2(d)** (the balance display includes Unknown coins — nothing silently drops out of view) · **7 ④** (anything unresolved is shown, never hidden) · T2 **Q5** (indexers are for display and discovery only, never a spend input; when down, rows show "content unavailable" and stay listed) · **Q3** (BRC-150 verify-only; P4 only *displays* P3's verified/claimed flag) · **Q7** (1Sat SDK: patterns only).

> **Order.** Last T2 phase. Needs **P1** (filing: `1sat` / `bsv21` rows exist), **P3** (the verified/claimed flag),
> **T1-P4** (real scripts ≤ 1,024 B inline — BSV-21 transfer inscriptions fit; amounts are read from them),
> **T1-P5** (the Unknown line this phase must agree with). **T0 Build 1** before any browser-level (T3) row.

> ⚠️ **Watch item (decision 8), carried:** BRC-162 binary BSV-21 (PR #273) **merged 2026-09-28** (P1 §5).
> Under decisions 2 + 7 and P1 §12 Q1 (recommended **hold only**), a binary BSV-21 output is **held, not filed** —
> so this phase shows it as a held item, **never** as a balance. No BRC-162 decoder is built here.

---

## 1. Goal

In the wallet's Tokens tab the user sees every ordinal they hold (name, type, origin, verified or only claimed) and a balance per BSV-21 token (symbol, decimals, exact amount, marked *not verified*), and nothing they hold is ever missing from view or counted in their money balance.

## 2. Done means

- [ ] **`/wallet/tokens` extended, not paralleled** (reuse-first): `handlers.rs :: list_token_outputs` returns, per row, basket class (`ordinal` / `bsv21` / `app`), CI, and P3's verified/claimed flag; plus a `bsv21Balances` array: one entry per token id with `amount` (exact decimal **string**, Σ of unspent `bsv21` rows of that id), `sym`, `dec`, `metaSource` (`deploy-tx` / `sender-stated` / `unavailable`) and `validity: "not verified"`.
- [ ] **Amounts come from the chain, not the sender.** Each `bsv21` row's `amt` is read by P1's recogniser from the row's **stored real script**, never from its `customInstructions`. A CI `amt` that disagrees is ignored for the sum and the row is flagged `ciMismatch`.
- [ ] **Exact arithmetic.** Rust sums in `u128` and returns decimal strings; the frontend never converts an amount to a JS `Number` (BRC-161 amounts are u64; `Number` loses precision above 2⁵³). `dec` (0–18, BRC-161) is applied by string placement.
- [ ] **Symbol and decimals** (a transfer inscription carries neither — they live in the token's deploy output, whose outpoint **is** the token id): fetched once per id via the existing `WalletServices::get_raw_tx`, accepted only if `double-SHA256(raw) reversed == txid`, parsed with the same recogniser, cached locally. Fallbacks, labelled on screen: CI values (*"as stated by the sender"*) → *"decimals unknown — raw units"*. This lookup is **display only** (Q5).
- [ ] **Never hidden.** A DB error in `list_token_outputs` returns an error and the tab says it could not load — never `200` with an empty list (today: `Err(_) => vec![]` and `filter_map(|r| r.ok())`, a verdict where an error is owed — rule 7 trip-wire 2, on a display path). A metadata fetch failure never drops a row. A filed ordinal reserved by an in-flight P3 transfer is listed as **sending**, not hidden by the `spendable = 1` filter.
- [ ] **One place per outpoint** (decision 2(d), consistency with T1-P5): every unspent output the user owns appears in **exactly one** of — the money balance (`calculate_balance`), T1-P5's held/Unknown line, or the Tokens tab's filed rows (`1sat`, `bsv21`, app baskets). Held **token-shaped** rows (BRC-162 binary, BSV-21 deploy+auth/auth/burn, legacy BSV-20, multi-sat with an envelope — P1 `P1-A4`) live in T1-P5's line with the reason *"token this wallet cannot show yet"*; the Tokens tab shows only a count linking there (§12 Q2).
- [ ] BSV-21 rows appear **only** as balances, never in the ordinals list; ordinals never contribute to a balance.
- [ ] **No inline media in the wallet overlay this release** (§12 Q1): an ordinal shows name (CI `name` if present, labelled *"as stated by the sender"*), content type, origin outpoint, verified/claimed; a **View** button opens `ordfs.network/content/<origin>` in an ordinary browser tab. BSV-21 shows its symbol, no icon.
- [ ] **No send, list, burn or "treat as money" action** is offered on any filed token or ordinal from this tab (transfer lives in P3's panel flow for ordinals only).
- [ ] **dApps cannot read this view.** `GET /wallet/tokens` from an external origin is refused — it enumerates every asset and would bypass the BRC-165 `p 1sat` view prompt (`P1-A10`/`P1-A11`). Measured on today's build first (`P4-A0`).
- [ ] The indexer posture is written as a rule in this contract's result (*"indexers: display and discovery only; never a spend input; down ⇒ 'unavailable', row stays"*) and **proposed** for `REGRESSION_ADDITIONS.md` at track close — as its own commit (working rule 6), not in this phase's diff.
- [ ] Measurement rows (`HARNESS_DELTA.md` §1.3) recorded for every real BSV-21 token received (`P4-M1`).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| Balance display (decision 2(d), beta.3 P10e) | `calculate_balance` counts default-basket rows only; Unknown is shown beside it | A token view that also sums satoshis, or a held row listed in two places, makes the user's total disagree with the chain; a row in **no** place is a silent loss |
| `R-NOSPEND` | No automatic path spends an unclassified output | A convenience action on the Tokens tab ("send", "treat as money", "clean up dust") would be a new spend path over held/filed rows |
| `R-TOKENPERM` | Pay grants never authorize token spends | Not a spend — but enriching `/wallet/tokens` hands a dApp the full asset list that BRC-165 puts behind a per-scope view prompt |
| `R-CLASSIFY` | Classification reaches every ingest path | The display must **read** P1's classification, never re-derive it (e.g. from the `type:` MIME tag) — a second classifier drifts |
| `R-RESTORE` | Fail-closed survives recovery | After a restore, every filed token must re-appear; a restored row with metadata missing must still list |
| `R-GOLD` | Gold pill | Untouched — this phase makes no payment; stated so a reviewer can check the diff touches no IPC |

## 4. Evidence table

⛔ Rows that total a balance or decide what may be spent: RED designed by a second agent (`../../../RELEASE_CYCLE.md` §4.2). Display rows: RED designed here.
**Subjects named before the run** (scratch profile, `HARNESS_DELTA.md` §1.1 — outpoints recorded here before the run): one real ordinal filed `1sat` from P3 (`________`, flag **verified**), one filed **claimed** (`________`); two real BSV-21 transfer outputs of the **same** id (`________`, `________`) and one of a second id with `dec > 0` (`________`); one held BRC-162 or deploy+auth output (`________`, or a hand-built script, stated as such); one T1 Unknown 1-sat plain P2PKH (`________`); ordinary default-basket money.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P4-A0` | **Baseline, before any code:** on today's build, an **approved** external domain requests `GET /wallet/tokens` through the real bridge — record whether it gets the list | Not applicable — this row **is** the RED of `P4-A1` if it returns the list; if it is already refused, record the refusing layer and `P4-A1` becomes a regression pin | Response body + the Rust log line for that request carrying `X-Requesting-Domain`. Code reading only so far: `request_gate.rs :: domain_trust_gate` returns `Proceed` for any endpoint once `trust_level = 'approved'`, and `main.rs`'s first-party-only refusal applies to POST/DELETE only; the C++ bridge's path handling is **not** traced (§10 K) | T2 | ⬜ |
| `P4-A1` | External origin ⇒ `/wallet/tokens` refused; the wallet overlay (first-party, no `X-Requesting-Domain`) still gets it | Remove the refusal ⇒ the approved dApp receives the list (must be seen; `P4-A0` may already be this) | Both responses, over the production call path (`feedback_test_subject_must_be_the_production_call`) | T2 | ⬜ |
| `P4-A2` | BSV-21 balance per id = Σ inscription `amt` over that id's **unspent** `bsv21` rows, as a decimal string; a row spent by a transfer out drops from the sum | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | `bsv21Balances` vs a hand sum of the named outpoints' inscriptions decoded from their stored scripts (lengths stated); the spent row named | T1 + T2 | ⬜ |
| `P4-A3` | A `bsv21` row whose sender CI says `amt: "1000000"` while its inscription says `5` contributes **5**, flagged `ciMismatch` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The row's stored CI string, stored script and its contribution to `bsv21Balances` | T1 | ⬜ |
| `P4-A4` | Exact display: two rows of one id with `amt` `18446744073709551615` and `1` show `18446744073709551616`; `dec = 8`, amount `123456789` shows `1.23456789` | Convert amounts with `Number()` in the component ⇒ the rendered digits differ (seen) | Rendered text in a component test fed the API's JSON, **and** the Rust sum for the same fixtures | T1 | ⬜ |
| `P4-A5` | `sym`/`dec` from the deploy tx only when the fetched bytes hash to the id's txid; a byte-flipped raw tx ⇒ not used, falls back to CI (*"as stated by the sender"*) or *"decimals unknown — raw units"* | Skip the hash check ⇒ the flipped fixture's `sym` is displayed (seen) | `metaSource` per id + the rendered label; fixture = a real deploy tx with one byte flipped | T1 | ⬜ |
| `P4-A6` | **One place per outpoint:** Σ money balance + Σ held/Unknown line + Σ sats of Tokens-tab filed rows = Σ sats of all unspent outputs the user owns (direct SQL), and each outpoint appears in **exactly one** list | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The three API responses (`/wallet/balance`, T1-P5's held field, `/wallet/tokens`) vs one `SELECT` over `outputs`, with every subject outpoint located by name | T2 | ⬜ |
| `P4-A7` | With every indexer provider pointed at a dead host, the Tokens tab lists every filed row with its verified/claimed flag; missing metadata shows *"unavailable"* | Make the handler skip a row whose metadata lookup failed ⇒ that row is missing from the response (seen) | `/wallet/tokens` response + provider log showing the failed lookups; row count vs `P4-A6`'s SQL | T2 | ⬜ |
| `P4-A8` | A DB error in `list_token_outputs` ⇒ HTTP error, tab shows *"could not load"* | Today's code: `Err(_) => vec![]` ⇒ `200` with `"count": 0` (seen on today's build) | The HTTP status + rendered message, with the fault injected by a test-only failing connection (say which) | T1 + T2 | ⬜ |
| `P4-A9` | Ordinals list shows only `1sat` rows with name/type/origin and the correct verified/claimed label; `bsv21` rows appear **only** under balances | Group rows by the `type:` MIME tag instead of the basket ⇒ a `bsv21` row appears in the ordinals list (seen); swap the flag mapping ⇒ the claimed ordinal reads "verified" (seen) | Rendered component for the named subjects; the claimed subject is the one from `P3-A1`'s RED | T1 + T2 | ⬜ |
| `P4-A10` | Rendering the Tokens tab makes **zero** requests to any content host; **View** opens the ORDFS URL in an ordinary tab | Add an `<img src="https://ordfs.network/content/…">` ⇒ the request appears (seen) | Network requests of the **wallet overlay's** browser, identified by the shell's role log (not the active tab — `reference_resolve_tab_single_candidate_hole`); ⚠️ hard-reload before measuring (Vite HMR) | T2 | ⬜ |
| `P4-A11` | An ordinal reserved by an in-flight P3 transfer lists as **sending**; after the transfer is recorded it is gone | Keep today's `spendable = 1` filter ⇒ the ordinal vanishes mid-send (seen) | `/wallet/tokens` for that outpoint before broadcast, during, after | T2 | ⬜ |
| `P4-A12` | Held token-shaped rows (BRC-162 / deploy+auth / BSV-20 / multi-sat envelope) appear once, in T1-P5's held line with reason *"token this wallet cannot show yet"*; *"Treat as money"* is **refused** for them; the Tokens tab shows only a count | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The held-line entries + the refusal response for each subject (edge: T1-P5-A10) | T2 | ⬜ |
| `P4-A13` | Human: Tokens tab readable at 100 % and 150 % DPI on both platforms; every BSV-21 balance visibly says *not verified*; sender-stated names/metadata are visibly labelled | Remove the *not verified* label ⇒ the reviewer must mark FAIL (told a label *may* be missing, not which) | The rendered advanced-wallet overlay, read by a person (`feedback_consent_surface_needs_human_eyes`) | T3 | ⬜ |
| `P4-M1` | **Measurement** per real BSV-21 received: CI bytes, BEEF bytes, BEEF ancestry depth, `beefB64` bytes (0 if absent), stored script length | A row with no number is INCOMPLETE, not green (`HARNESS_DELTA.md` §1.3) | The inbound `internalizeAction` body and the stored row | — | ⬜ |

**Two-sided rows:** `P4-A1` (dApp refused ↔ overlay served); `P4-A2`↔`P4-A3` (chain amount counted ↔ sender amount ignored); `P4-A6` (each outpoint in one place ↔ in no second place); `P4-A9` (ordinals only in the list ↔ tokens only in balances).

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/handlers.rs :: list_token_outputs` | ✅ | Extended here. Today: `spendable = 1`, excludes `default` / `identity_certificates` / backup marker; no CI; **`Err(_) => vec![]`** and `filter_map(|r| r.ok())` swallow errors into an empty list |
| `rust-wallet/src/main.rs` — route `/wallet/tokens` → `handlers::list_token_outputs` | ✅ | GET, no per-handler gate |
| `rust-wallet/src/main.rs` — first-party-only middleware (`is_mutation` / `hits_surface`) | ✅ | Refuses POST/DELETE on the permission surface only — a GET is not covered |
| `rust-wallet/src/permission_service/request_gate.rs :: domain_trust_gate` | ✅ | `trust == "approved"` ⇒ `Proceed` for any endpoint (the `P4-A0` hypothesis) |
| `cef-native/src/core/HttpRequestInterceptor.cpp :: HttpRequestInterceptor::isWalletEndpoint` | ✅ | Any `/wallet/` path counts as a wallet endpoint. Read only — no C++ change planned; if `P4-A1`'s refusal lands in C++, add a MAC_RELAY note |
| `rust-wallet/src/database/output_repo.rs :: calculate_balance` | ✅ | Default-basket only (beta.3 P10e comment). **Not changed** — `P4-A6` reads it |
| `rust-wallet/src/handlers.rs :: wallet_balance`, `wallet_activity` | ✅ | ⭐ **P4 does not read `wallet_activity`.** Its received query uses `o.change = 0` (T1-P5-A11 changes it); token receipts appear there today as "+1 sat received" — cosmetic, out of scope (§6) |
| `rust-wallet/src/database/tag_repo.rs :: get_tags_for_output` | ✅ | Reused for `bsv21:<id>` grouping |
| `rust-wallet/src/services/mod.rs :: WalletServices::get_raw_tx` | ✅ | Deploy-tx fetch for `sym`/`dec`; 4-tier chain; reuse, no new client |
| `rust-wallet/src/database/parent_transaction_repo.rs :: ParentTransactionRepository::get_by_txid` | ✅ | Local raw-tx cache — checked before the network |
| `rust-wallet/src/reconcile.rs :: parse_tx_outputs` | ✅ | `(value, script)` per output; fails closed on truncation — reused to reach the deploy output |
| `rust-wallet/src/script/parser.rs :: parse_script_chunks` | ✅ | Under P1's recogniser; reused, no new parser |
| `frontend/src/components/wallet/TokensTab.tsx :: TokensTab`, `fetchTokens` | ✅ | Groups by basket today; renders `satoshis` via `toLocaleString()`. Extended with the ordinals list + balances section |
| `frontend/src/pages/WalletOverlayRoot.tsx` — `TAB_TITLES` `'Tokens'`, lazy `TokensTab` | ✅ | Host page; no new overlay, no new HWND |
| `frontend/src/services/walletApi.ts :: walletFetch` | ✅ | Transport, unchanged |
| T1-P5 contract §2 (d), `P5-A8`, `P5-A10`, `P5-A11` | ✅ read 2026-09-28 | The held/Unknown line and "Treat as money" this phase must agree with |
| T1-P4 contract §12 Q1 | ✅ read 2026-09-28 | 1,024 B inline cap; `script_length IS NULL` = not observed ⇒ such a row is not filed by P1, so never in a balance |
| BRC-161 (amounts u64 strings, `dec` ≤ 18, `id` = deploy outpoint), BRC-163 (basket `bsv21`, CI shape) — `bsv-blockchain/BRCs` `master` | ✅ fetched 2026-09-28 by this track (P1 §5) | Not re-fetched by this run; P1's read of the same day stands |

## 6. Out of scope

- ⛔ BSV-21 transfer, send, burn, list (decision 8). ⛔ BRC-176 proof verification (with P5 — so every balance says *not verified*).
- An indexer/overlay validity lookup for BSV-21 (§12 Q3 — recommended none).
- Inline media / thumbnails / BSV-21 icons in the overlay (§12 Q1).
- Filing or decoding BRC-162 binary; `bsv20` / `opns` sections (held is safe; P1 §6).
- The activity feed's labelling of token receipts as "+1 sat received" (`wallet_activity`) — ticket candidate for beta.7 intake.
- The T1-P5 held line itself, its wording, and "Treat as money" (T1 owns them).
- The same dApp-readability question for `/wallet/balance` and `/wallet/activity` — reported to T5-P2, not fixed here.

## 7. Rollback

Revert the `list_token_outputs` extension commit and the `TokensTab` commit (Rust + React, no schema; the metadata cache, if it needs storage, reuses `parent_transactions` — if it needs a new table that is invariant 2 and §12 Q4 decides it first). The dApp refusal (`P4-A1`) is its own commit so it can stay if the view is reverted.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| Balance summed from the sender's CI — a scam token "airdrops" 1,000,000 of a real token's id | `P4-A3` |
| Amount summed in JS `Number` — large supplies display wrong digits | `P4-A4` |
| `dec` taken from a lying CI or a spoofed deploy fetch — balance off by 10⁸ | `P4-A5` (hash check, labelled fallback) |
| A held BRC-162 token shown in both T1's line and the Tokens tab — user thinks they have two, or "treats as money" from one view | `P4-A6`, `P4-A12` |
| Indexer outage ⇒ Tokens tab empty ⇒ user thinks the wallet lost their NFTs | `P4-A7`, `P4-A8` |
| Ordinal disappears during a transfer, user retries, sends again | `P4-A11` |
| BSV-21 shows in the ordinals grid because both carry `type:application/bsv-20` | `P4-A9` |
| Richer `/wallet/tokens` hands every approved dApp the user's full asset list | `P4-A0`, `P4-A1` |
| Inscription bytes rendered in the wallet overlay's renderer — a decoder exploit lands in the process that can call the wallet first-party | `P4-A10` (no content request at all) |
| The "no media" check measured the page tab, not the overlay browser | `P4-A10` SUBJECT via role log |
| Vite HMR serves the old component during a RED | `P4-A10` / `P4-A13` hard-reload note |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | Scratch profile under `HodosBrowserDev` |
| macOS | `P4-A1`, `P4-A2`, `P4-A6`, `P4-A10`, `P4-A13` | Shared Rust + React; A10/A13 exercise the macOS advanced-wallet overlay. No C++ planned ⇒ no relay file note unless `P4-A1`'s refusal lands in C++ |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~0.75 h** — receive a real BSV-21 token into the scratch wallet (~0.5 h, second wallet, overlay-fee token — SCOPE §7) + the visual pass at two DPI cells (~0.25 h; macOS side separate) |
| Human-bound rows | `P4-A13` (T3); funding for the `P4-A2`/`P4-M1` subjects |
| Unknowns (K) — uncertainty, not difficulty | **K = 1 phase.** (a) Whether a dApp can reach `GET /wallet/tokens` through the C++ bridge today (`P4-A0` answers it); (b) whether real BSV-21 deploy txs are small enough to fetch per id without a size guard (a deploy may carry a large icon inscription in another output — the fetch is whole-tx) |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| T2-P1 → T2-P4 | filing + recogniser | `1sat` / `bsv21` rows and the `amt` decoder; needed first |
| T2-P3 → T2-P4 | verified/claimed flag; "sending" state | Needed first |
| T1-P4 → T2-P4 | real scripts | BSV-21 amounts are read from stored scripts (≤ 1,024 B inline) |
| T1-P5 ↔ T2-P4 | held/Unknown line | ⭐ **One place per outpoint** (`P4-A6`): held token-shaped rows live in T1's line with reason *"token this wallet cannot show yet"*; T1-P5-A10's "Treat as money" refusal must cover them (`P4-A12`). T1-P5's contract should cite this row |
| T1-P5-A11 → T2-P4 | `wallet_activity` | **No dependency:** P4 does not read the feed |
| T2-P4 → T5-P2 | dApp-reachable surface | `GET /wallet/tokens` (and likely `/wallet/balance`, `/wallet/activity`) readable by approved dApps — code reading, `P4-A0` measures. T5-P2's allow-list should decide the GET side; P4 lands the tokens refusal if T5-P2 has not |
| T2-P4 → T3a-P2 | restore | After restore every filed token re-lists (`R-RESTORE`); `P4-A6` re-run on a restored wallet |
| T2-P4 → T3a-P0 | measurement | `P4-M1` numbers for BSV-21 (joins `P3-M1`) |
| T0 Build 1 → T2-P4 | engine | Browser-level rows (`P4-A10`, `P4-A13`) run on the new engine |
| T2-P4 → REGRESSION_ADDITIONS | indexer posture rule | Proposed at track close, own commit (rule 6) |

## 12. Open questions for the owner

1. **No inline media this release?** Showing inscription images means feeding attacker-chosen bytes to an image decoder **inside the wallet overlay**, the most privileged renderer we have. ⭐ Recommend **no inline media**: name, type, origin, verified/claimed, and a **View** button that opens the content in a normal tab. Thumbnails come later with a separate-origin, process-isolated frame (needs a site-isolation check in our CEF build). The alternative is a thumbnail feature with an unmeasured security boundary.
2. **Held tokens in one place or two?** ⭐ Recommend **one place** — T1-P5's held line (with a clear reason and "Treat as money" refused), and a count in the Tokens tab pointing there. Two listings make the totals ambiguous (`P4-A6`).
3. **BSV-21 validity:** no BRC-176 in this release. Show every balance as *not verified* (recommended — no indexer call, nothing implied), or also ask the overlay/indexer and show *"indexer says valid"*? The reference SDK treats overlay state as advisory.
4. **Metadata cache:** reuse `parent_transactions` for the deploy tx (recommended — no schema) versus a small per-id cache table (invariant 2). Decide at kickoff with the code open.

⚠️ No evidence that decision 8 is wrong. One piece of evidence **on its mechanism**: `/wallet/tokens` appears reachable by approved dApps (code reading only), which would already let a site read what BRC-165 puts behind a view prompt — `P4-A0` measures it before anything is built.

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
