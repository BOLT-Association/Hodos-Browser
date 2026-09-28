# B5-T6-P5 — a Tools tab in the advanced wallet: "Claim a payment" (and "Scan my old addresses" when T1's scan lands) · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (G3, 2026-09-28). ⛔ Documents only — no code has been written for this phase.

**Track:** B5-T6 Browser shell · **Group:** 2 (should ship — and kept even if the release runs long: T6 Q8 keeps P5 with Group 1) · **Status:** ⬜ NOT STARTED
**Tickets / outline:** `../TOOLS_TAB_claim_a_payment.md` (owner decisions 2026-09-15 + card 2, 2026-09-25) · card 2's scan: `../../tickets/TICKET_rescan_cannot_find_payments_to_generated_addresses.md` (B5-T1)
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent for T6 (resumed run) · **Platforms:** both (Rust + React are shared; §9) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **T6 Q4** — **T6 builds the screen and the claim endpoint together; the endpoint carries T1's money-path harness rows** (chain amount only, never overwrite, negative controls designed by someone other than the author). **T6 Q8** — P5 is kept with Group 1. **2 / 2a / 7** — a claimed coin enters through a classified ingest route (T1-P5 route I5) and is filed by the money index (T1-P3). Outline decisions 1–4 (visible, advanced wallet, **input format fixed by `payment_claim_block`**). Carried, not reopened.

⛔ **Money phase.** Every row that can credit, refuse, or change the wallet's coins is **GREEN + SUBJECT by this author; its RED is designed by a second agent** (`../../../RELEASE_CYCLE.md` §4.2). Those rows carry exactly `⏳ independent control — second agent (RELEASE_CYCLE §4.2)`. UI-only rows carry controls authored here.

---

## 1. Goal

A user whose PeerPay sender could not deliver the notification pastes the sender's **Copy details** block into the wallet's new Tools tab and receives the payment — credited at the **chain's** amount, once, and only if it pays this wallet's keys.

## 2. Done means

- [ ] **Tools** entry in the advanced wallet sidebar, just above Settings; one card per job, each with title, one "when to use this" sentence, the control, a result line, and a "What does this do?" expander (outline)
- [ ] **Card 1 — Claim a payment:** one paste box; parse → chain lookup → shows amount **from chain**, sender key prefix, date → **Claim** → one of four result lines: *Claimed N sats* · *Already in your wallet* · *Not yours: no output in this transaction pays your keys* · *Not found on chain yet — try again after it is mined*
- [ ] **A new first-party-only Rust endpoint** does all money work — fetch the tx and proof, build the BEEF (`beef_helpers.rs :: build_beef_for_txid`), derive, internalize through the hardened path. The frontend never builds a BEEF (invariant 1's spirit)
- [ ] The endpoint is **unreachable from a dApp** (added to `main.rs :: is_permission_surface`, the internal-only list) and goes through the shell's `isWalletEndpoint` table
- [ ] The claimed output is classified through T1-P5's seam and enters T1-P3's money index like any received payment
- [ ] **Card 2 — Scan my old addresses** (only when T1-P6 and T1-P4 `P4-A11` have landed): one button, progress line, result *found N payments / nothing new / incomplete*, and the NC-9 sentence that PeerPay payments are found through *Claim a payment*, not the scan
- [ ] `frontend/src/components/wallet/CLAUDE.md` corrected (its stale "Wallet Rescan in SettingsTab" lines — SCOPE §3) by whoever builds card 2

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-INTEXT` | Internal never prompts, external always gates | A new wallet endpoint. It must be internal-only (no dApp can trigger a claim) and the wallet UI's own call must not prompt |
| `R-NOSPEND` / `R-CLASSIFY` (beta.6 additions) | Every ingest route classifies; an unclassified output is not spendable | The claim is a new way in for a coin — it must pass T1-P5's seam, not write a spendable row directly |
| `R-PEERPAY-DELIVERY` | A PeerPay delivers its message or never leaves the wallet | The claim is the receiver half of the same flow; the block format is pinned by `payment_claim_block_tests` — the half-1 test set runs at this boundary |
| `R-GOLD` | Gold pill | Not touched: a claim is an internal receive, no payment IPC — asserted absent (`P5-U4`) |
| Service fee | 1000-sat fee on outgoing tx | Not touched: a claim spends nothing. Asserted: no transaction is broadcast by a claim |
| Invariant 1 | Keys and money logic stay in Rust | The frontend parses only for display; the endpoint re-parses and decides |
| Invariant 2 | Wallet schema | **No schema change planned.** If kickoff finds one is needed (e.g. a claim record), stop and ask |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. ⛔ Money rows: RED designed by a second agent — `⏳` until then; the designer is recorded in the row when filled.
⭐ `HARNESS_DELTA.md` §1.1: SUBJECT names the **output** — txid, vout, satoshis, basket.
⭐ Rig: a scratch profile + scratch wallet; a real PeerPay from a second wallet whose MessageBox delivery is suppressed (sender-side `HODOS_MESSAGEBOX_MAX_BODY_BYTES` shrink, as `R-PEERPAY-DELIVERY` half 1 does) so the receiver never hears of it and the sender's **Copy details** block exists.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P5-A1` (T-A1) | A block produced by `payment_claim_block` for a real undelivered payment to this wallet ⇒ one credited output, **satoshis = the chain's output value**, spendable, in the money index | **Switches:** (1) remove the endpoint's call into T1-P5's seam / T1-P3's index write ⇒ the `outputs` row exists but no money-index row ⇒ red on *in the money index* · (2) ⭐ attribution: run once WITHOUT suppressing MessageBox delivery ⇒ `task_check_peerpay.rs :: run` credits first and the claim answers *Already* — a green there is not A1; assert the credit row's `created_at` lies inside the claim's request/response window and no `peerpay_received` row existed before the claim · (3) ⚠️ *spendable* has two readings: `store_derived_utxo` inserts `confirmed=0, transaction_id NULL`, and `get_spendable_by_user` requires `transaction_id IS NOT NULL OR confirmed=1` — say which the row asserts (row flag now vs selectable after `TaskSyncPending` confirms); fail the confirm step (mock unspent query down) ⇒ the *selectable* reading must go red, the *row flag* reading must not · (4) value: compare `satoshis` to WoC's vout value read in the same run, never to the block · **Red for the right reason:** `outputs` row by `txid:vout` with `created_at`, its index row, the chain value · **Residue:** one real claimed coin in the scratch wallet — declared (mainnet, scratch profile) — designed by controls-C (Fable), 2026-09-28 | The receiver's `outputs` row (txid, vout, satoshis, basket, `change`), its money-index row (T1-P3), WhatsOnChain's value for that vout | T2 | ⬜ |
| `P5-A2` (T-A2) | The same block claimed twice ⇒ second answer *Already in your wallet*; balance and row unchanged (never overwrite — beta.3 10a) | **Switches:** (1) revert `store_derived_utxo`'s refuse-to-overwrite to the pre-10a UPDATE branch and claim a second block for the same outpoint with a DIFFERENT `derivationPrefix` ⇒ the row's derivation fields change (the coin is now unspendable by its real key) ⇒ red · (2) ⭐ a byte-identical second claim gets `Ok(())` from `store_derived_utxo` — make the endpoint treat `Ok` as *Claimed* ⇒ the *Already in your wallet* assertion goes red (the answer must come from detecting pre-existence, not from the write succeeding) · (3) `BalanceCache` has a 60 s TTL — read `/wallet/balance` after `invalidate()` or `calculate_balance` directly, else a double credit hides for a minute · (4) column equality must cover `custom_instructions`, `sender_identity_key`, `spendable`, `spent_by` — a *row count unchanged* assertion passes an overwrite · **Red for the right reason:** the full row dumped before/after and diffed · **Residue:** none beyond A1's coin — designed by controls-C (Fable), 2026-09-28 | Row count and every column of that output before/after the second claim; `/wallet/balance` | T2 | ⬜ |
| `P5-A3` (T-A3) | A block for **another wallet's** payment ⇒ *Not yours*; nothing credited; `recipientIdentityKey` mismatch shown as a warning, claim still attempted (outline) | **Switches:** (1) stub `verify_output_matches_derived_address` (or the seam's ownership check) to `true` ⇒ the stranger's output is credited ⇒ red — ownership is decided by derivation against the output script · (2) ⭐ wrong-reason guard: a block with a WRONG `recipientIdentityKey` but a correct derivation for THIS wallet must still credit (with the warning) — an endpoint that refuses on the mismatch passes A3 by never trying · (3) a block naming OUR txid with a stranger's remittance ⇒ *Not yours*, not *Already* · (4) `outputs` AND `peerpay_received` counts unchanged — a *Not yours* that still writes a notification row is red · **Red for the right reason:** response body + both table counts · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | Receiver `outputs` count unchanged; endpoint response body | T2 | ⬜ |
| `P5-A4` (T-A4) | Block with `amountSatoshis` edited **upward** ⇒ credited value is still the chain's; display shows the chain's amount before Claim | **Switches:** (1) revert to crediting `amountSatoshis` from the block ⇒ credited value = edited value ⇒ red · (2) display half: skip the chain fetch and render the block's amount ⇒ the pre-Claim display shows the edited value ⇒ red (hard reload first — HMR) · (3) edit DOWNWARD too ⇒ still the chain's value — a guard written as `min(block, chain)` passes only the upward case · (4) ⭐ the chain value must come from the transaction's own output (`build_beef_for_txid` / raw tx vout), not an indexer's address balance — a mock reporting a different address balance must not move the credit · **Red for the right reason:** three numbers side by side: block, credited row, WoC vout value · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | The credited row's `satoshis` vs the edited field vs WhatsOnChain | T2 | ⬜ |
| `P5-A5` | Block with an edited `derivationPrefix` / `derivationSuffix` / `senderIdentityKey` ⇒ no output derives ⇒ *Not yours*, nothing credited (the derivation, not the paste, decides) | **Switches:** (1) ⭐ this row passes with the feature absent (an endpoint that always answers *Not yours*) — it is two-sided with A1: the same test binary credits the UNEDITED block first, then refuses each of the three edits · (2) stub `derive_address_from_payment_remittance` to return the correct hash regardless of input ⇒ the edited-`derivationPrefix` block credits ⇒ red — the derivation, not a hash comparison, decides · (3) edit `senderIdentityKey` to another VALID 33-byte key ⇒ *Not yours* (an edit that only breaks hex parsing tests the parser) · (4) the log must show the derived pubkey and the compared script per output, or the SUBJECT's *logged* is unmet · **Red for the right reason:** response body, derived-vs-script log, `outputs` unchanged · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | Derived pubkey vs each output's locking script, logged; `outputs` unchanged | T1 + T2 | ⬜ |
| `P5-A6` | `outputIndex` wrong in the block ⇒ the endpoint scans every output and still finds the one that pays us (outline: hint only) | **Switches:** (1) pass the block's `outputIndex` straight through to `internalize_action`'s `outputs[].outputIndex` (its `remittance_map` is keyed by index, so the real vout is never derived against) ⇒ a block saying `1` for a payment at vout `0` (`PEERPAY_PAYMENT_OUTPUT_INDEX`) answers *Not yours* ⇒ red · (2) fixture: a real PeerPay tx with the recipient at vout 0 and change at vout 1, block edited to `outputIndex: 1` ⇒ credited vout must be 0 · (3) two matching outputs in one tx (the sender paid us twice with one remittance) ⇒ both credited, not only the hinted one · **Red for the right reason:** credited vout vs block hint, with the per-output derivation log · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | Credited vout vs block's `outputIndex` | T1 | ⬜ |
| `P5-A7` | Txid not on chain ⇒ *Not found on chain yet*; nothing credited; a lookup **error** (indexer unreachable) returns a distinct *couldn't check — try again* and **never** a verdict (rule 7 trip-wire 2) | **Switches:** (1) revert to today's `check_tx_exists_on_chain` reading (`NotFound ⇒ Ok(false)` AND `TxState::Unknown ⇒ Ok(false)`) with the provider chain pointed at a dead host ⇒ the card says *Not found on chain yet* ⇒ red — an error read as a verdict · (2) ⭐ the third state: a LIVE mock provider answering `TxState::Unknown` (Bitails' shape for a fresh tx) ⇒ must be *couldn't check*, not *Not found* (T4-P1's strict three-way) · (3) genuine `NotFound` from every provider ⇒ *Not found* — an endpoint that maps everything to *couldn't check* is over-strict and never tells the user to wait for mining · (4) fault the BEEF fetch (`build_beef_for_txid`) separately ⇒ also *couldn't check*, never *Not found* · **Red for the right reason:** response body per case + the mock's per-call log naming which provider answered what; `outputs` unchanged in all four · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | Response body per case; `outputs` unchanged; provider pointed at a dead host for the error case | T2 | ⬜ |
| `P5-A8` | Claim first, then the sender's delayed MessageBox message arrives and the PeerPay poller processes it ⇒ **one** credit; and the reverse order ⇒ the claim answers *Already in your wallet* | **Switches:** (1) ⭐ the poller dedupes by `message_id` only (`PeerPayRepository::is_already_processed`), never by outpoint: after a claim, the delayed message is NOT already processed, `store_derived_utxo` no-ops (`Ok`), and `insert_received` writes a SECOND `peerpay_received` row — run today's poller after a claim and expect this red on the `peerpay_received` count (`outputs` stays 1 by `UNIQUE(txid, vout)`); the fix's outpoint dedupe, whichever side owns it, is the switch — remove it ⇒ 2 rows · (2) reverse order: poller first, then claim ⇒ *Already in your wallet* and no new `peerpay_received` row · (3) the MessageBox message must be acknowledged after the claim path too, or it re-arrives every tick · **Red for the right reason:** `outputs` = 1 AND `peerpay_received` = 1 for that txid, and the badge shows one payment · **Residue:** none beyond A1's coin — designed by controls-C (Fable), 2026-09-28 | `outputs` rows for that outpoint (count = 1) and `peerpay_received` row after both paths | T2 | ⬜ |
| `P5-A9` | The endpoint is internal-only: a request carrying `X-Requesting-Domain` (any approved dApp, via the `wallet_call` bridge or HTTP) ⇒ **403**, handler not run | **Switches:** (1) remove the path from `main.rs :: is_permission_surface` ⇒ a dApp POST carrying `X-Requesting-Domain` reaches the handler (handler log line present) ⇒ red · (2) percent-encoding: revert the `decoded_path` arm and send `/wallet/%63laim-payment` ⇒ bypass ⇒ red · (3) ⭐ around the table: remove the path from `HttpRequestInterceptor::isWalletEndpoint` ⇒ a page-context `fetch('http://127.0.0.1:<port>/wallet/claim-payment', {method:'POST', headers:{'Content-Type':'text/plain'}})` (no CORS preflight) is not intercepted, arrives with NO `X-Requesting-Domain`, and Rust treats it as first-party ⇒ handler runs ⇒ red — so the GREEN must also assert the shell stamped the header for this path (interceptor log), not only the Rust 403 · (4) the gate is `is_mutation && hits_surface` — assert no GET binding exists for the path · **Red for the right reason:** wallet log shows the gate line and no handler line; `outputs` unchanged; the request was issued from a real dApp page origin, not curl · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | Wallet log: gate line and **no** handler line; `outputs` unchanged | T2 | ⬜ |
| `P5-A10` | The claimed coin passed T1-P5's classification seam (route I5) — money only by T1-P5's rule ②; a claimed output that is **not** a plain P2PKH to our key is not made spendable money | **Switches:** (1) bypass the seam (write the row as today's `store_derived_utxo` does, a direct INSERT) ⇒ no classification log line for the outpoint and no `change` stamp ⇒ red · (2) ⭐ the money rule with teeth: a sender constructs a 1-sat exact-25-byte P2PKH to our derived key with an inscription in the SAME tx's other output — T1-P5 rule ② calls that 1-sat output money; the row must state this is by design, and the control is the inverse: a non-25-byte script to our derived key is refused by `verify_output_matches_derived_address` (*Not yours*), never filed spendable · (3) route I5 must be named in the seam's log line — remove the route id ⇒ red · **Red for the right reason:** the seam log line for that `txid:vout`, the `change` column, index membership · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | The seam's classification log line for that outpoint; `change` stamp; money-index membership | T1 + T2 | ⬜ |
| `P5-A11` | A claim broadcasts **nothing** and records no commission (it is a receive) | **Switches:** (1) ⭐ today's `internalize_action` BROADCASTS when `check_tx_exists_on_chain` returns `Ok(false)` — claim a txid the mock reports `NotFound` while its raw bytes sit in `parent_transactions` ⇒ if the endpoint reuses `internalize_action` verbatim the mock ARC receives a transaction ⇒ red · (2) seed one unrelated `commissions` row and one `is_outgoing=1` transaction before the claim ⇒ counts unchanged (an empty-table *no rows* assertion is vacuous) · (3) the mock ARC's received-transaction log stays empty across the whole claim, including A7's error cases · **Red for the right reason:** the mock ARC log names any broadcast txid; `commissions` / `transactions` counts before and after · **Residue:** none — designed by controls-C (Fable), 2026-09-28 | `transactions` rows with `is_outgoing=1` and `commissions` rows before/after | T1 | ⬜ |
| `P5-U1` (T-A5) | The endpoint's and the card's parsers are tested against **`payment_claim_block`'s output** (golden-keys test fixture), not a hand-typed example | Rename one field in `payment_claim_block` ⇒ the parser tests fail while `payment_claim_block_tests` still pass | The fixture is produced by calling `payment_claim_block` in the test | T1 | ⬜ |
| `P5-U2` | Malformed paste (truncated, wrong `type`/`version`, not JSON) ⇒ one sentence naming the wrong part; no network call | Parser that ignores `type`/`version` ⇒ a foreign JSON reaches the chain lookup (seen in the log) | Card result line + wallet log shows no lookup | T1 + T2 | ⬜ |
| `P5-U3` | Tools entry sits above Settings; card copy has no jargon on its face; the four result lines render as specified | Remove the entry ⇒ card unreachable (today's state) | Owner reads the rendered card in the **wallet overlay's** browser (role-log resolved) | T3 👤 | ⬜ |
| `P5-U4` | No gold pill and no payment modal on a claim | Route the claim through a payment IPC by mistake (rig) ⇒ pill appears | Tab strip + notification overlay type log during a claim | T2 | ⬜ |
| `P5-C1` | **Card 2** (after T1-P6 + T1-P4 `P4-A11`): button calls T1-P6's scan; the result shows found / nothing new / **incomplete** (never "0 found" when the indexer was unreachable) | Point providers at a dead host ⇒ the card must say *incomplete*; showing "nothing new" = fail (T1-P4 A11's body, rendered) | The HTTP body T1 returns and the rendered line, same run | T2 | ⬜ |
| `P5-C2` | Card 2 copy carries NC-9: *PeerPay payments are found through Claim a payment, not the scan* | Delete the sentence ⇒ T1-P6's `P6-A9` string assertion fails | Rendered card text | T1 + T3 | ⬜ |

**Two-sided pairs:** A1 ↔ A3 (credits ours / never credits theirs) · A1 ↔ A2 (credits once / never twice) · A1 ↔ A4 (credits / at the chain's value only).

### 4a. Independent control notes (2026-09-28)

*controls-C (Fable), second agent per `../../../RELEASE_CYCLE.md` §4.2. Findings on GREEN/SUBJECT cells I did not rewrite. One line each: row — problem — suggested fix.*

- P5-A1 — *spendable* is ambiguous on this ingest: `store_derived_utxo` writes `confirmed=0, transaction_id NULL`, and `get_spendable_by_user` requires `transaction_id IS NOT NULL OR confirmed=1`, so the coin is a row-flag spendable but not selectable until `TaskSyncPending` confirms it. Say which the row asserts.
- P5-A5 — passes with the feature absent (an endpoint that always says *Not yours*); it is only meaningful paired with A1 in the same run.
- P5-A6 / K7b — `internalize_action`'s `remittance_map` is keyed by `outputIndex`; verbatim reuse answers *Not yours* whenever the block's hint is wrong. K7b must extract the derivation loop or call it per vout.
- P5-A8 — the poller dedupes by `message_id` only; a second `peerpay_received` row after claim-then-poller is expected on today's code, and §2 does not say who owns the outpoint dedupe. Assign it.
- P5-A9 — the GREEN asserts the Rust 403 only. The header is stamped by the shell's `isWalletEndpoint`; a path missing from that table is reachable header-free from any page via a `text/plain` POST (no preflight). Add the interceptor-log assertion to the GREEN.
- P5-A11 / K7b — `internalize_action` broadcasts on `Ok(false)` from `check_tx_exists_on_chain`; verbatim reuse contradicts A11 on a not-yet-visible txid. Same extraction as A6.
- P5-A7 — `check_tx_exists_on_chain` maps `TxState::Unknown ⇒ Ok(false)`; the endpoint must not consume it as-is (already the T4-P1 edge in §5 — repeated here because A7's wording *never a verdict* depends on it).

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/handlers.rs :: payment_claim_block` + `mod payment_claim_block_tests` | ✅ | the **only** writer of the block; fields: `type`, `version`, `txid`, `outputIndex`, `senderIdentityKey`, `derivationPrefix`, `derivationSuffix`, `amountSatoshis`, `recipientIdentityKey`. ⛔ Not redesigned |
| `rust-wallet/src/handlers.rs :: internalize_action` (`web::Data<AppState>`, `web::Json<InternalizeActionRequest>`; `protocol == "wallet payment"` arm) | ✅ | the hardened path the claim reuses — call it, or extract its core at kickoff (reuse-first) |
| `rust-wallet/src/handlers.rs :: store_derived_utxo` | ✅ | the PeerPay poller's writer — A8's double-credit pair |
| `rust-wallet/src/handlers.rs :: check_tx_exists_on_chain` | ✅ | ⚠️ T4 found `Unknown → Ok(false)` — A7 must not inherit a verdict from an error (T4-P1 edge) |
| `rust-wallet/src/beef_helpers.rs :: build_beef_for_txid`, `fetch_transaction_for_beef` | ✅ | BEEF from a txid — reuse, do not write a second builder |
| `rust-wallet/src/monitor/task_check_peerpay.rs :: run` | ✅ | the other route for the same payment (A8) |
| `rust-wallet/src/main.rs :: is_permission_surface` (nested fn; `/wallet/debug`, `/wallet/delete`, `/wallet/backup`, `/wallet/recover`, …) | ✅ | the new endpoint's path is added here (A9) |
| `rust-wallet/src/main.rs` route table (`/internalizeAction`, `/wallet/rescan`) | ✅ | new route registered here; 📏 no claim route exists today |
| `cef-native/include/core/HttpRequestInterceptor.h :: HttpRequestInterceptor::isWalletEndpoint` | ✅ | kickoff rule: new endpoints go through the table |
| `frontend/src/components/wallet/WalletSidebar.tsx` (tabs: Dashboard, Activity, Certificates, Tokens, Approved Sites, Settings) | ✅ | Tools inserted before Settings — ⚠️ tab ids are numeric; renumbering shifts Settings |
| `frontend/src/components/wallet/` tab files; `WalletDashboard.css` (`.wd-rescan-*` leftovers) | ✅ | new `ToolsTab.tsx` beside them; CEF input rule: native `<textarea>`/`<input>`, not MUI `TextField` |
| T1-P5 route list I4/I5 (`../../track-1-money-path/phase-P5-classifier-and-unknown-display/PHASE_CONTRACT.md`) | ✅ | names this card as an I5 caller |

## 6. Out of scope

Claiming non-PeerPay payments (paymail P2P, BRC-121). Receiving blocks automatically (link handlers, QR). Any block-format change beyond fields under a new `version`. A **"possibly paid" list** for BRC-121 payments (T4-P1 makes it available — decided **not** in this release's Tools tab, §12 Q1). Cards for `/wallet/consolidate-dust`, `/wallet/sync?full=true`, outbox retry — the outline's endpoint inventory is done at kickoff and anything beyond cards 1–2 is a ticket, not scope. `/wallet/debug/*` never get cards. The scan itself (T1-P6).

## 7. Rollback

Three commits: (1) Rust endpoint + internal-only listing, (2) Tools tab + card 1, (3) card 2. Each reverts alone; reverting (1) with (2) in place leaves a card whose calls 404 — revert (2) first. No schema, no data migration; a claimed coin stays a normal received output after a revert.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| The endpoint credits `amountSatoshis` from the paste instead of the chain | A4 |
| A claim overwrites an existing output row's derivation (pre-10a shape) | A2 |
| Claim and poller both credit the same outpoint ⇒ balance doubled | A8 |
| Indexer outage reads as "not found" and the user gives up on a real payment — or as "not yours" | A7 (error ≠ verdict) |
| A dApp discovers the endpoint and uses it to probe which txids pay this wallet (a privacy oracle) | A9 internal-only |
| The claim writes a spendable row that skipped classification — a token delivered as a PeerPay becomes spendable money | A10 (T1-P5 seam) |
| The parser drifts from the writer and old beta.3 blocks stop working | U1 (fixture from the writer) |
| Card 2 shows "nothing new" when the scan could not run | C1 |
| The Tools tab's textarea cannot take focus or paste in the CEF overlay | U3 (native input, owner pastes a real block) |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | U3 + one A1 claim via relay | Rust endpoint and React tab are shared code; the wallet overlay's paste/focus on mac is the only platform-specific risk (`cef_browser_shell_mac.mm` wallet overlay) — relay confirms, no mac code expected |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~2** — one real undelivered PeerPay between two of his wallets + T-A1…A4 observed; U3 by eye. Card 2 adds ~0.5 when it lands |
| Human-bound rows | `P5-U3`; A1 with a real payment (agent can drive the endpoint, owner confirms the wallet UI); mac relay |
| Unknowns (K) | **K7** answered by decision (T6 builds, T1 rows apply) · **K7b** whether `internalize_action` can be called as-is or its core must be extracted (kickoff read) — **K = 1** |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs ← | **B5-T1-P3** reservations + money index | a claimed coin must land in the index — P5 builds after T1-P3 |
| needs ← | **B5-T1-P5** classifier seam | route I5 ("T6-P5's claim card") — P5's endpoint calls the seam, A10 |
| needs ← | **B5-T1-P6** scan endpoint + **T1-P4 `P4-A11`** "incomplete" body | card 2 only; card 1 does not wait for them |
| needs ← | **second agent** (RELEASE_CYCLE §4.2) | designs the RED for A1–A11 before P5 opens |
| reviews ↔ | **B5-T4-P1** | shares `check_tx_exists_on_chain`; T4-P1 makes its three-way answer strict — A7 uses the strict form if it lands first |
| gives → | **B5-T1** harness | T1's money-path rows run against this endpoint (T6 Q4) |

## 12. Open questions for the owner

1. **A "possibly paid" list on the Tools tab?** T4-P1 now knows when a 402 payment *may* have been paid (three outcomes + txid) and offers it to any surface. ⭐ **My call at G3 (asked of me): not in this release** — T4-P1 already tells the user at the moment it happens, the outline's two cards are the owner-approved scope, and a list needs its own design (what can the user do with it?). ⭐ Recommend a beta.7 ticket. Say if you want it here.
2. **Endpoint path name** — working name `POST /wallet/claim-payment`. No preference needed unless you have one.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed — money-row REDs designed and recorded by the second agent (name in each row)
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded (R-INTEXT, R-PEERPAY-DELIVERY half 1, R-NOSPEND, R-CLASSIFY)
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] `frontend/src/components/wallet/CLAUDE.md` + `rust-wallet/src/CLAUDE.md` handler roster updated (invariant 11)
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
