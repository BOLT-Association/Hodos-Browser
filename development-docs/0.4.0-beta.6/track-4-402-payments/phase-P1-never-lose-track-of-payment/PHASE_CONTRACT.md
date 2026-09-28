# B5-T4-P1 — a 402 payment never loses track of whether it was paid · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (G3, 2026-09-28). ⛔ Documents only — no code has been written for this phase.

**Track:** B5-T4 402 payments · **Tickets:** `../../tickets/TICKET_brc121_release_restores_inputs_the_server_may_have_spent.md` (fix), `../../tickets/TICKET_brc121_remint_on_retry.md` (fix, as narrowed 2026-09-25), `X402_INTEGRATION.md` §10 item 7 (absorbed) and item 4 (closed here as a unit assertion — P3 is deferred) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent for T4 · **Platforms:** both (Rust wallet + shared C++ shell + React page) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **9** (P1 first; chain-before-release/re-mint; Q2 record-never-broadcast; Q4 pay exactly `amount`; Q5 TTL → 20 s with A3; Q6 body transport deferred — that is P2's; Q7 verified; Q8 read), **10** (Step 0 run — zero hits ⇒ hardening, not an incident). Carried, not reopened. Evidence that bears on decision 9's wording is in §12, not routed around.

---

## 0. Step 0 — ALREADY RUN (decision 10). Recorded here, not re-planned.

| | |
|---|---|
| **When / who** | 2026-09-27, orchestrating session, owner-approved, read-only |
| **Method** | SQLite `mode=ro`; `transactions ⋈ tx_labels_map ⋈ tx_labels` where label ∈ {`pay402`,`brc121`}, non-deleted, `status IN ('failed','nosend')`; one `GET api.whatsonchain.com/v1/bsv/main/tx/hash/{txid}` per row |
| **Positive control (run first)** | `73eab7534e1514…` (known broadcast) ⇒ **200**; `1f8a2e4e516c…` (known failed) ⇒ **404** — the method can tell the two apart |
| **Production wallet** | 3 completed + **1 failed** ⇒ the failed one **404** |
| **Dev wallet** | 110 completed + **49 failed** ⇒ all 49 **404**; zero `nosend` |
| **Errors** | none — no lookup returned an error/unknown |
| **Verdict** | **Zero "freed but actually spent" rows ⇒ no rule-7 stop. P1 is a hardening phase.** |
| **Limit (carried)** | A 404 is WhatsOnChain's view. For rows days-to-weeks old a mined tx would be indexed, so the limit is small; it is not zero for a tx that was broadcast and then dropped |
| **Residue** | none (read-only) |

## 1. Goal

When a site refuses or never answers a paid request, the wallet asks the network whether that payment exists before it frees its coins or pays again — and when it cannot tell, it does neither and tells the user *"you may have paid — here is the txid — we won't pay again until this settles."*

## 2. Done means

- [ ] A refused paid retry whose transaction **is on the network** leaves its inputs spent and its outputs live; the row moves `nosend → sending` (the existing promotion); **no broadcast by us** (Q2); the user is told "paid" with the txid.
- [ ] A refused paid retry whose transaction is **provably not on the network** is released exactly as today (`release_unbroadcast_transaction`), no behaviour change.
- [ ] A refused paid retry whose chain answer is **unknown** (transport failure, all providers down, a non-terminal provider state) changes **nothing**: row stays `nosend`, inputs stay reserved, outputs stay as they are; the user sees the "you may have paid" state with the txid.
- [ ] `pay_402` never mints a second payment for the same reuse key (domain + server key + URL) while an earlier payment for it is on the network **or unknown**; it mints exactly once when the earlier one is provably absent.
- [ ] "Unknown" ends only on **terminal evidence** (chain shows it ⇒ paid; the existing sweepers' verdict ⇒ settled). Until then the user is not charged again for that page (decision 9 wording; persistence scope — §12 Q-P1-3).
- [ ] `PaymentFailedPage` and the shell log no longer claim *"not broadcast / funds preserved"* unconditionally; the page states one of three outcomes that matches the wallet's answer.
- [ ] `PAY402_REUSE_TTL_MS` = **20 000** in the same commit as the re-mint check (Q5).
- [ ] A unit assertion pins `pay_402`'s payment output to exactly `req.satoshis` (Q4; closes `X402_INTEGRATION.md` §10 item 4).
- [ ] The BRC-121 paid-retry half of `R-GOLD` — owed since beta.3 — is observed green with a real payment (A5).
- [ ] A `status == 0` paid retry logs whether the request body left the machine, or the contract records that CEF cannot say (A1).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| **R-GOLD** (`REGRESSION_SET.md`) | Gold pill on every auto-approved payment, on the **originating tab** | The pill's BRC-121 emit is `Async402ResourceHandler::firePaymentSuccessIpc()` → `OnWalletCallSuccess`, on the **2xx** branch of `onUpstreamComplete`. P1 edits the **else** branch next to it. ⚠️ Two-sided: a "chain says paid but the site refused" outcome delivered **no content** and must **not** fire the pill (A7); the 2xx branch must still fire it (A5) |
| **R-COUNT** | Per-session counters, `PermissionService.session_counters` | `dispatch_payment` runs **before** the reuse cache in `pay_402` (code reading today), so the spend is metered on entry. A3's new "don't mint" returns must not add a metered call that mints nothing without saying so; A2's promotion must not record a second spend (A8) |
| **R-ONE-CLICK-ONE-SPEND** | One Approve signs exactly what it was shown | The direct subject of A3: one page, one payment, however many retries |
| **R-INTEXT** | Internal never prompts, external always gates | Both endpoints are called only by the shell's own tasks (no `X-Requesting-Domain`). ⚠️ Read today: `/wallet/broadcast-nosend` is on `main.rs`'s internal-only surface list (rejects a request carrying `X-Requesting-Domain`); **`/wallet/release-nosend` is not on that list**, and neither is in the shell's wallet route table. **Kickoff item:** establish whether any external origin can reach `release-nosend` before P1 puts a network probe and a state change behind it; if it can, adding it to the internal-only list is in scope (one line, its own row at kickoff). Not claimed as a defect — not verified |
| **R-NODOUBLE** (sweeper rule, beta.3) | Release only on positive on-chain observation | P1 applies the same rule at the decision point; it must **strengthen**, never weaken, it — in particular the sweepers (`TaskCheckForProofs`, `TaskUnFail`) keep running unchanged as the crash backstop |
| **R-BEEFOUT** (`REGRESSION_ADDITIONS.md`) | A BEEF we hand a counterparty stands on its own | P1 does not change BEEF building. Listed because A3 re-sends a **stored** BEEF on reuse — it must be the byte-identical BEEF, not a rebuild. Run at the boundary; state whether `MAX_BEEF_ANCESTORS` was reached |
| **HARNESS_DELTA §1.1** fail closed | "Could not tell" never turns into an action | ⛔ The load-bearing rule of this phase: `Err` / unknown ⇒ do nothing. Root `CLAUDE.md` rule 7 trip-wire 2 |
| `PaidContentCache` | Only a 2xx writes it | "Paid, no content" must cache nothing (a later load under BRC-121 pays again — the page copy must say so) |

## 4. Evidence table

⛔ Money rows: RED designed by a **second agent** that did not write the GREEN (`../../../RELEASE_CYCLE.md` §4.2). Record the designer's name in the Result cell when it is written.
🎯 Every SUBJECT names the **output** (HARNESS_DELTA §1.1): the payment txid, its input outpoints, its change vout, sats.
🧪 Scratch profile / dev wallet only (HARNESS_DELTA §1.2). Never the production profile.

**Rig (built in this phase, rig-only):** `demos/brc121-402/server.js` gains a `BROADCAST_ON_RECEIPT=1` switch that broadcasts the received BEEF via ARC **and then** returns **502** (and a `BROADCAST_DELAY_MS` knob that returns 502 first and broadcasts after N ms — for A10). Plus a dev-only probe override (`HODOS_DEV=1` only, same family as `HODOS_402_UPSTREAM_DELAY_MS`) that forces the chain answer to `exists` / `absent` / `error`. ⛔ The override must be unreadable in a production binary — its own row (A9b).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P1-S0` | Step 0 result recorded (§0): zero failed/nosend BRC-121 rows exist on chain in either wallet | Positive control ran first: known-broadcast ⇒ 200, known-failed ⇒ 404 (§0) | Owner's production DB + dev DB, `mode=ro`; WoC `tx/hash` per txid | T2 | ✅ 2026-09-27 (decision 10) — carried, not re-run |
| `P1-A1a` | A paid retry that ends `status == 0` logs, beside the existing `GetRequestError()` code, **whether the request body was sent** (CEF upload progress) — *or* the contract records, citing the CEF header, that `CefURLRequest` cannot distinguish "never sent" from "sent, no reply" (rule 4) | Instrument row, not money: on a request that **completes** (200) the new line must be **absent**; on a rig-cancelled request (navigate away mid-delay with `HODOS_402_UPSTREAM_DELAY_MS`) it must be **present**. Revert the log call ⇒ absent in both | `debug_output-<pid>.log` of the **tab** browser that made the request (role log, not an overlay's); the cited CEF header (`cef_urlrequest.h` / `CefURLRequestClient::OnUploadProgress`) | T2 | ⬜ |
| `P1-A1b` | A refused paid retry (`status > 0`) logs the response's status, reason headers and a bounded body preview **before** the release decision, so the "first 402" has a reason code | Instrument row: stub the log ⇒ the refusal leaves only the status line (today's state, captured once as the baseline) | Same log; one rig refusal (demo server returns 402 with a JSON reason) | T2 | ⬜ |
| `P1-A2a` | **Chain says the payment exists ⇒ keep it.** Rig `BROADCAST_ON_RECEIPT=1`: server broadcasts then 502 ⇒ `release_nosend` probes, gets `Ok(true)`, does **not** call `release_unbroadcast_transaction`; row `nosend → sending` via the existing `conn_update_status` promotion; response to the shell says `paid` + txid | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The payment txid (WoC 200); its **input outpoints** (`spent_by`/`spending_description` unchanged, `spendable=0`); its change output (vout, sats, `spendable` unchanged); `transactions.status` before/after — dev DB snapshot, not the HTTP code | T2 (real sats, ~1,150/run) | ⬜ |
| `P1-A2b` | **Chain says it is absent ⇒ release exactly as today.** Rig server returns 502 **without** broadcasting ⇒ probe `Ok(false)` ⇒ `release_unbroadcast_transaction` runs; outputs disabled, inputs restored, status `failed` with `failed_at` stamped. Two-sided with A2a: each is the other's control | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Same txid: WoC **404**; input outpoints restored (`spendable=1`, reservation cleared); change output disabled; `failed_at` non-NULL | T2 | ⬜ |
| `P1-A2c` | **Chain cannot answer ⇒ nothing changes.** Probe forced to `Err` (override `error`, and separately: all four providers unreachable) ⇒ row stays `nosend`, inputs stay reserved, outputs unchanged, `failed_at` NULL; response = `unknown` + txid; shell shows the "you may have paid" state. ⛔ Rule 7 trip-wire 2: the error must never be read as "absent" | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The DB rows for the txid and its inputs, before and after — byte-for-byte unchanged except the log; the wallet log line naming the probe **error**, not a verdict | T1 (mapping) + T2 (real providers blocked) | ⬜ |
| `P1-A2d` | **Non-terminal provider states are unknown, not absent.** For P1's decisions, `TxState::Unknown` (JungleBus returns it for any unconfirmed tx), ARC `SEEN_IN_ORPHAN_MEMPOOL`, ARC `MINED_IN_STALE_BLOCK` ⇒ **unknown** (A2c behaviour); `DoubleSpendAttempted` ⇒ **do not restore inputs** (a competing tx spends them). Only an all-providers `NotFound` or a terminal `Rejected` is "absent". ⚠️ Today `check_tx_exists_on_chain` returns `Ok(false)` for all five — see §12 Q-P1-1 | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Unit table over each `TxState` / `raw_provider_status` value → P1 verdict; T2: JungleBus-only chain (ARC/WoC/Bitails blocked) against a **mempool** txid ⇒ unknown, not release | T1 + T2 | ⬜ |
| `P1-A2e` | **No race with the sweepers or a concurrent `pay_402`.** Between the probe (an `await`, seconds) and the release, the row's status is re-checked inside the release's own SQLite transaction; a row a sweeper promoted meanwhile is left alone. No DB lock is held across the probe `await` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | A test that promotes the row to `sending` while the probe is suspended (a test seam on the probe future), then asserts zero outputs disabled / inputs restored; `db_lock_held_across_await` pattern checked by code review of the diff | T1 | ⬜ |
| `P1-A2f` | **The tab never hangs and the shared FILE thread is not starved.** Worst-case probe time is bounded below the shell's `SyncHttpClient` timeout for `/wallet/release-nosend` (today 30 000 ms; the probe chain is 4 providers × 8 s soft timeout = 32 s, **over** it); a shell-side timeout is reported to the user as **unknown**, never as "not paid"; `releaseCefCallbacks()` fires on every path. The release call moves off `TID_FILE_USER_BLOCKING` or is bounded so it cannot stall other FILE-thread work (all three `TID_FILE_*` ids are one thread — 2026-09-14 lesson) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Tab browser log: time from refusal to callbacks released; a concurrent balance poll's latency during a forced 35 s probe; the page state shown after a shell-side timeout | T2 | ⬜ |
| `P1-A3a` | **Re-mint asks the chain — exists ⇒ no mint.** A paid retry that ended `status == 0`, then a re-click **after** the reuse TTL, with the first payment on the network (rig broadcast) ⇒ `pay_402` returns a distinct `possibly_paid`/`paid` outcome with the **first** txid and **zero** new `createAction` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Count of `transactions` rows labelled `pay402` for the reuse key before/after (= unchanged); the first txid (WoC 200); wallet log `pay_402` decision line | T2 | ⬜ |
| `P1-A3b` | **Absent ⇒ mint exactly once.** Same sequence, first payment provably absent ⇒ exactly **one** new payment, and the stale one is released. Two-sided with A3a | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Row count +1 exactly; new txid ≠ old; old txid WoC 404 and its inputs restored | T2 | ⬜ |
| `P1-A3c` | **Unknown ⇒ no mint, however many clicks, until it settles.** Probe forced to `Err` ⇒ `unknown` + txid, zero `createAction`; repeated re-clicks past the TTL still mint nothing; after the probe answers `absent` (override flipped) the next click mints once | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Row count per click (0,0,0 then +1); the outstanding txid carried in each response | T2 | ⬜ |
| `P1-A3d` | **Reuse entry survives an unknown release.** After A2c (release probe `Err`), the outstanding txid is still findable by A3 for that reuse key — today `release_nosend` evicts the `pay402_reuse` entry **first**, which would make the next click mint (see §8 story 4) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | `AppState.pay402_reuse` / the outstanding-payment lookup after an `Err` release; the next click's `createAction` count (= 0) | T1 + T2 | ⬜ |
| `P1-A3e` | **TTL = 20 s (Q5).** `PAY402_REUSE_TTL_MS == 20_000`; a re-click at 22 s after a `status == 0` goes through the chain check (A3a/b/c) instead of reusing; a re-click at 15 s reuses the **byte-identical** BEEF | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The constant (unit); the `REUSE` vs chain-check log line per click; the `x-bsv-beef` bytes the demo server logged for both sends (hash equal at 15 s) | T1 + T2 | ⬜ |
| `P1-A3f` | **Pay exactly `amount` (Q4).** `pay_402`'s payment output (vout 0, `randomize_outputs=false`) carries exactly `req.satoshis`; the 1,000-sat service fee and change are separate outputs | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The built transaction's outputs parsed from the returned BEEF (not the request struct): vout 0 value, the fee output to `HODOS_FEE_ADDRESS`, change | T1 | ⬜ |
| `P1-A4a` | **The page tells the truth, three ways.** `/payment-failed` renders **paid** (chain shows it: "the site did not deliver; your payment went through — txid …; reloading will pay again under BRC-121") · **not paid** ("not broadcast; your coins are back") · **unknown** ("you may have paid — txid …; we won't pay again for this page until it settles"). The unconditional "not broadcast" sentence is gone | UI row (written here): serve each state with the wallet answer **stubbed to the opposite** ⇒ the page text must change with it. Baseline RED, cheap: today's build on the A2a rig renders "Your sats are safe — the transaction was **not broadcast**" for a txid WoC shows as broadcast — capture that screenshot before the fix | The **tab's** rendered document (CDP screenshot of the tab browser, verified by the role log — not an overlay), per state, with the txid visible and matching the DB | T2 (render) + **T3 human legibility** | ⬜ |
| `P1-A4b` | **The shell log stops lying.** The `"NOT broadcasting (funds preserved)"` warning is replaced by the wallet's three-way answer; the in-flight banner (`Brc121PaymentBannerTask`) is hidden on refusal as today | UI/log row (written here): grep the built binary's log after an A2a run for the **behavioural** token of the old claim — it must be absent; revert ⇒ present. (Grep a runtime log line, never a source comment) | Tab browser `debug_output-<pid>.log` from the A2a run | T2 | ⬜ |
| `P1-A4c` | The Activity row for the payment (`description "Paid content — <host>"`) shows the same outcome as the page (paid / released / pending-unknown) | UI row (written here): force the row's status by hand in a scratch DB ⇒ the Activity label follows the status, not a cached string | Wallet panel Activity list read over CDP from the wallet overlay browser (role-log verified), matched to `transactions.status` | T2 | ⬜ |
| `P1-A5` | 👤 **Real site, real money, one sitting.** Against `now.bsvblockchain.tech` with `HODOS_402_UPSTREAM_DELAY_MS` > 30 s: stale 402 ⇒ **no** second mint and the correct page state; then a normal paid load ⇒ **gold pill on the paying tab** (the `R-GOLD` BRC-121 half beta.3 left owed) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Chain + dev DB for every txid minted in the sitting; `OnWalletCallSuccess fired (cefBrowserId=… -> tabId=…)` addressed by `Tab::id`; owner's eyes on the pill | T3 (owner) + T2 | ⬜ |
| `P1-A6` | **We never broadcast a refused payment (Q2).** In every A2a/A3a outcome, zero calls reach `broadcast_transaction` / `/wallet/broadcast-nosend` for that txid | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Wallet log (`broadcast-nosend called` absent for the txid) + ARC submission count from the services stats snapshot for the run | T2 | ⬜ |
| `P1-A7` | **No pill without content.** A2a (paid, refused) fires **no** `payment_success_indicator`; the same rig returning 200 fires exactly one. Two-sided | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | `OnWalletCallSuccess fired` count per run in the tab log; the tab's pill state | T2 | ⬜ |
| `P1-A8` | **Counters honest.** Promote (A2a), release (A2b), unknown (A2c) and possibly-paid (A3a/c) each leave `session_counters` spend unchanged beyond the one spend recorded at the original mint; if the no-mint returns pass through `dispatch_payment`, the contract records how many spends that meters and why it is acceptable | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | `PermissionService.session_counters` for the domain/browser read before/after each step (not a UI total) | T1 + T2 | ⬜ |
| `P1-A9a` | **The rig is real.** With `BROADCAST_ON_RECEIPT=1` the demo server's received txid is on WoC; with it off, 404 — so A2a/A3a are measured against an actually-broadcast payment | Instrument row (written here): switch off ⇒ WoC 404 for the same flow's txid — seen once per rig build | Demo server log + WoC for the txid | T2 | ⬜ |
| `P1-A9b` | **The dev probe override cannot reach production.** It is read only under `HODOS_DEV=1` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | A release-profile binary started with the override set: the probe still calls the providers (log line) | T1 + T2 | ⬜ |
| `P1-A10` | 📏 **Measurement, not a pass — the late-broadcast race.** Rig `BROADCAST_DELAY_MS` = 0 / 5 / 15 / 40 s after the 502: record what P1 decides at refusal time and how long until the sweepers (`TaskCheckForProofs`, `TaskUnFail`) put the row right. Feeds §12 Q-P1-2 | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Per delay: probe verdict + timestamp, `transactions.status` timeline, inputs' `spendable` timeline, WoC first-seen time | T2 | ⬜ (a number; blank = INCOMPLETE) |

**Two-sided pairs:** A2a⇄A2b · A3a⇄A3b · A7⇄(A5 pill half) · A4a paid⇄not-paid. Each is the other's control.

## 5. Blast radius

All verified 2026-09-28 by Read/Grep at `65869b1`.

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/handlers.rs :: check_tx_exists_on_chain` | ✅ | Returns `Result<bool,String>`. ⚠️ Maps `TxState::Unknown`, `Rejected`, `DoubleSpendAttempted`, ARC orphan/stale **all to `Ok(false)`**; `Err` only when the chain ends on a non-NotFound error; NotFound detected by **string match** on the error text. One definition, **three callers today**: `internalize_action`, `wallet_cleanup` (`.unwrap_or(false)` ⇒ marks outputs spent as `ghost-cleanup`), `monitor/task_check_peerpay.rs` (not a duplicated copy of the helper — the *calling pattern* is duplicated) |
| `rust-wallet/src/services/mod.rs :: WalletServices::tx_status`, `soft_timeouts::TX_STATUS` (8 s) | ✅ | Provider chain ARC → WoC → JungleBus → Bitails |
| `rust-wallet/src/services/collection.rs :: ProviderCollection::call` | ✅ | First `Ok` wins (so an `Ok(Unknown)` from JungleBus ends the chain); `NotFound` advances; final error = the **last** provider's |
| `rust-wallet/src/services/providers/junglebus.rs :: parse_jb_tx_status` | ✅ | `block_height` absent ⇒ `TxState::Unknown` — i.e. every unconfirmed tx |
| `rust-wallet/src/services/providers/arc_gorillapool.rs` (status map) | ✅ | Orphan/stale ⇒ `Rejected`; unrecognised ⇒ `Unknown` |
| `rust-wallet/src/handlers.rs :: release_nosend` | ✅ | Status read uses `.ok()` (a DB error reads as "not found ⇒ alreadyGone success"); evicts `pay402_reuse` **before** releasing (`P11-11-A5`); calls `release_unbroadcast_transaction`; then a second `UPDATE … 'failed'` with `let _ =` |
| `rust-wallet/src/handlers.rs :: release_unbroadcast_transaction` | ✅ | One SQLite txn; no status re-check inside it; its doc says the "never broadcast" gate is held **by construction at every call site** — the premise P1 makes true for `release_nosend` |
| `rust-wallet/src/handlers.rs :: broadcast_nosend`, `conn_update_status` | ✅ | The existing `nosend → sending` promotion A2a reuses |
| `rust-wallet/src/handlers.rs :: pay_402`, `Pay402ReuseEntry`, `pay402_reuse_key`, `PAY402_REUSE_TTL_MS` (25 000), `PAY402_REUSE_STATUS_SQL` | ✅ | `dispatch_payment` runs **before** the reuse check; past-TTL branch removes the entry and falls through to mint; output `satoshis: Some(req.satoshis)`, `randomize_outputs: false`, labels `brc121`/`pay402`, description `"Paid content — <host>"`, `no_send: true` |
| `rust-wallet/src/main.rs :: AppState.pay402_reuse` | ✅ | In-memory `HashMap` — lost on wallet restart (§12 Q-P1-3) |
| `rust-wallet/src/database/transaction_repo.rs :: set_transaction_status` | ✅ | Stamps `failed_at` on `Failed` — `TaskUnFail` depends on it |
| `rust-wallet/src/monitor/task_check_for_proofs.rs :: NOSEND_TIMEOUT_SECS` (10 min), `mark_failed` | ✅ | The crash backstop; unchanged by P1. ⚠️ **G3 integration:** T3a-P2.2 proposes changing `mark_failed` so an inconclusive oracle no longer restores inputs (BRC-177); ownership is owner question T3a-P2.2 Q1 — if it lands, re-run this phase's release rows against it |
| `rust-wallet/src/monitor/task_unfail.rs :: recover_transaction` | ✅ | Recovers `failed` rows within 6 h only when mined with a proof; unchanged |
| `rust-wallet/src/reconcile.rs :: check_outpoint_spent`, `SpentStatus` | ✅ | Not P1's primitive (it answers the input side); the audit tool if a promoted row's inputs are ever questioned |
| `rust-wallet/src/permission_service/request_gate.rs :: dispatch_payment` path (`decide_and_record_payment`) | ✅ | Spend recorded at decision time (A8) |
| `cef-native/src/core/HttpRequestInterceptor.cpp :: Async402ResourceHandler::onUpstreamComplete` | ✅ | `retryable = 431 || 5xx`, `MAX_UPSTREAM_RETRIES = 1`; 2xx ⇒ `broadcastNosendAsync()` + `firePaymentSuccessIpc()` + `PaidContentCache::Put`; else ⇒ the "funds preserved" warning, `RegisterBrc121FailedUrl`, banner hide, and on `status > 0` `releaseNosendThenContinue()`; `status == 0` keeps the txid for reuse |
| `… :: Async402ResourceHandler::ReleaseTask` | ✅ | Posted to `TID_FILE_USER_BLOCKING`; `SyncHttpClient::Post(…/wallet/release-nosend, 30000)`; fires `ContinueTask` unconditionally |
| `… :: Async402ResourceHandler::firePaymentSuccessIpc` → `OnWalletCallSuccess` | ✅ | The gold-pill emit (R-GOLD) |
| `… :: Async402HTTPClient::OnRequestComplete` | ✅ | Logs `GetRequestError()` on `status == 0` (A1 extends) |
| `… :: Brc121PaymentBannerTask`, `Brc121PaymentBannerHideTask`, `RegisterBrc121FailedUrl`, `HODOS_402_UPSTREAM_DELAY_MS` seam | ✅ | Reused |
| `cef-native/src/handlers/simple_handler.cpp :: SimpleHandler::OnLoadError` (branch 2, `ConsumeBrc121FailedUrl`) | ✅ | Builds `/payment-failed?domain&sats&status&originalUrl` — A4a adds the outcome + txid. ⚠️ P2 A2 shows this branch may not fire on an error **document**; P1's page must reach the screen by P2's route |
| `frontend/src/pages/PaymentFailedPage.tsx` | ✅ | Copy: *"Your sats are safe — the transaction was **not broadcast**"* — unconditional today |
| `frontend/src/App.tsx` routes `/payment-failed`, `/payment-pending` | ✅ | Reused |
| `demos/brc121-402/server.js` | ✅ | "We don't broadcast here" — the rig switch is added rig-only |

## 6. Out of scope

- ⛔ **x402 adapter (T4-P3)** — deferred by decision 9; re-check if #2890 merges, a live BRC-29 `exact` server appears, or the owner wants bsv.cx. Q8 read today: nothing moved (§12).
- Body transport for large BEEF (Q6) — P2 records the deferral.
- Any payer-side broadcast of a refused payment (Q2 — record only).
- An overpayment tolerance knob (Q4).
- Changing the sweepers (`TaskCheckForProofs`, `TaskUnFail`, `TaskSweepReservations`) — they stay the crash backstop.
- Changing `check_tx_exists_on_chain`'s behaviour for its existing callers (`internalize_action`, `wallet_cleanup`, PeerPay) — see §12 Q-P1-1 and §11.
- A Tools-tab list of "possibly paid" payments — T6 (`TOOLS_TAB_claim_a_payment.md`) is its natural home.
- Per-invoice idempotency ("already paid, serve again") — that is a **server** feature (BRC-166 has it; BRC-121 does not).
- Tempted by: a new outcome enum (the kaleidoscope says four already exist — map to them, do not add a fifth); a persistent "outstanding payments" table (schema — only if §12 Q-P1-3 is answered that way, and then invariant 2 applies).

## 7. Rollback

Revert the phase's commits: the probe call sites in `release_nosend` and `pay_402` are additive, the page copy and TTL are one-line reverts. Behaviour returns to today's (immediate release on refusal; sweepers as the only reconciliation).

## 8. Pre-mortem (adversarial review — before)

Assume P1 shipped and a user lost money or trust. Why?

| # | Failure story | Caught by |
|---|---|---|
| 1 | The helper said "absent" when it meant "don't know": JungleBus answered `Unknown` for a mempool tx after ARC/WoC timed out, and P1 released spent coins — the exact bug, now with a check that looks like it prevents it (trip-wire 4, an instrument that cannot fail) | `P1-A2d` |
| 2 | ARC said `DOUBLE_SPEND_ATTEMPTED`; P1 read "absent" and restored inputs a competing tx had spent | `P1-A2d` |
| 3 | Probe took 32 s; the shell's 30 s timeout fired, logged "sweepers will reconcile", released the page callbacks with the refusal, and the page said "not paid"; seconds later Rust promoted the row | `P1-A2f`, `P1-A4a` |
| 4 | `release_nosend` evicted the reuse entry first (as today), the probe errored, row stayed `nosend` — and the user's next click found no entry and minted a second payment: "we won't pay again" was false | `P1-A3d`, `P1-A3c` |
| 5 | A sweeper promoted the row during the probe `await`; the release then disabled outputs of a live transaction | `P1-A2e` |
| 6 | The probe ran on the shared FILE thread and stalled balance polls and every other `TID_FILE_*` task for 30 s per refusal | `P1-A2f` |
| 7 | "Not found" at refusal time was indexer lag or an origin that broadcast after its gateway gave up (34.5 s origin measured); P1 released, the payee broadcast a minute later | `P1-A10` (measured) → §12 Q-P1-2 |
| 8 | A3 blocked re-minting forever because an outstanding row never settled (sweeper stuck, provider down for hours) — user can never buy the page | `P1-A3c` (settles when override flips); §12 Q-P1-3 asks the bound |
| 9 | The pill fired on "paid but refused", telling the user a paywall opened when it did not | `P1-A7` |
| 10 | The copy was right in code and wrong on screen — the `/payment-failed` swap never fires for an error **document** (P2's hypothesis), so the user saw the site's raw text anyway | `P1-A4a` SUBJECT is the rendered tab; depends on P2-A2 (§11) |
| 11 | The rig override leaked into a release build and forced "absent" | `P1-A9b` |
| 12 | A3's "don't mint" return still passed `dispatch_payment` and metered a spend, so the session cap was hit by a page that was never paid for | `P1-A8` |
| 13 | Evidence gathered on Windows only (beta.3's A1–A8 were) and macOS diverged in the shared C++ | §9 macOS rows |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | Primary development platform |
| macOS | `P1-A1a/b`, `A2a–c`, `A2f`, `A3a–c`, `A4a–c`, `A5` (pill half at least), `A7` | Shared C++ (`HttpRequestInterceptor.cpp`, `simple_handler.cpp`) and the React page. 🍎 **Relay note owed** in `MAC_RELAY_BETA5.md` naming both files; macOS rebuilds after its next rebase (root `CLAUDE.md` branch rule). Rust rows (A2d/e, A3e/f, A8 T1 half, A9b) are platform-neutral: run once, cite. The ReleaseTask threading fix (A2f) must use CEF task/thread APIs, not a Win32 primitive (invariant 9) |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **≈ 2 h** — `P1-A5` real-money sitting ~1.5 h (one driver, cannot share with a T1 money row); `P1-A4a` legibility read of the three page states ~0.5 h. Step 0's 0.5 h is spent |
| Human-bound rows | `P1-A5` (owner at the keyboard, real site, pill), `P1-A4a` T3 half (is the copy understood?) |
| Unknowns (K) — uncertainty, not difficulty | **K = 2.** (i) Whether CEF can report "bytes left" for a cancelled `CefURLRequest` (A1a — may end as "cannot, cited"). (ii) How often "not found at refusal time" is lag rather than absence (A10 — a number that may reopen Q-P1-2). *(Resolved since SCOPE: Step 0 — zero hits; the fail-closed reach — code reading shows the helper does **not** fail closed, so it is now a design question (§12 Q-P1-1), not an unknown.)* |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs / coordinate | **B5-T1-P3** reservations (+ money-index table, decision 2a) | `release_unbroadcast_transaction` restores inputs via `restore_by_spending_description`; T1-P3 converges reservation ownership on `spent_by` and adds the money index. P1 edits a **caller** (`release_nosend`) and must not depend on the placeholder mechanism T1-P3 is replacing. ⇒ **Serialise P1 after T1-P3**, or agree the seam: P1 calls only `release_unbroadcast_transaction` and never touches reservation columns itself. T1-P3's per-selector controls must include "a coin P1 kept (paid) is not re-selectable" |
| gives / ask owner | **B5-T1** (money path) | `check_tx_exists_on_chain`'s non-terminal-states-as-absent behaviour and `wallet_cleanup`'s `.unwrap_or(false)` (which **marks outputs spent** when the chain is unreachable — trip-wire 2 shape, code reading only; no UI caller found, route `/wallet/cleanup` is registered). Recommended split in §12 Q-P1-1: T4-P1 owns a **strict** tri-state for its own two call sites; T1 owns the lenient callers' review |
| gives | **B5-T1** PeerPay (`monitor/task_check_peerpay.rs`) | Same helper; P1 must not change its behaviour (a PeerPay `Ok(false)` triggers a broadcast, which tolerates a false "absent") |
| needs | **B5-T4-P2** | The `/payment-failed` page reaching the screen (P2-A2). P1-A4a's GREEN is only observable on a refusal that produces a load error, or after P2's fix |
| gives | **B5-T6** (`TOOLS_TAB_claim_a_payment.md`, prompt/notice surfaces) | The three-way outcome + txid is available to any surface T6 builds; P1 adds no overlay and no HWND. T6 decides if "possibly paid" belongs in a Tools-tab list |
| needs | **B5-T0 Build 1** | Browser-level evidence (A1, A4, A5, A7) gathered once, on the refreshed engine |
| gives | **R-GOLD** standing row | A5 pays the BRC-121 half beta.3 left owed; record in `REGRESSION_ADDITIONS.md`'s boundary table at the T4 boundary |

## 12. Open questions for the owner

| # | Question | Recommendation |
|---|---|---|
| **Q-P1-1** | ⚠️ **Evidence against the premise "reuse `check_tx_exists_on_chain`", as written.** Read today: the helper turns `TxState::Unknown` (which JungleBus returns for **every** unconfirmed tx), ARC orphan/stale, `Rejected` and `DoubleSpendAttempted` into **`Ok(false)` = absent**. For P1 that is exactly the verdict-instead-of-error rule 7 forbids, and `DoubleSpendAttempted` would restore inputs another tx spent. Changing the helper itself changes three other callers (`internalize_action` would stop broadcasting on `Unknown`; `wallet_cleanup`; PeerPay) | **Keep "reuse", tighten the reading:** P1 adds a **strict** reading of the same `services.tx_status` answer for its two call sites — present ⇒ paid; all-providers NotFound or terminal `Rejected` ⇒ absent; everything else ⇒ unknown; `DoubleSpendAttempted` ⇒ never restore inputs. The lenient helper is untouched for its existing callers, and `wallet_cleanup`'s `.unwrap_or(false)` goes to **T1** as a separate trip-wire-2 item. Needs your OK because it narrows "reuse" |
| **Q-P1-2** | Decision 9 says *"not found ⇒ free as today"*. A "not found" at the instant of a gateway 5xx is not terminal evidence (x402 #3387: *"Reconciliation closes the unknown state only when it produces terminal evidence"*): the origin may still be internalizing (34.5 s measured) and broadcast after its gateway gave up | **Carry the decision as written**; measure it (`P1-A10`). If A10 shows a broadcast landing after a "not found" release inside the sweepers' window, bring the number back with one option: on a **5xx** refusal, hold as unknown for one re-probe (e.g. 30–60 s) before releasing; a **402/4xx** refusal (the server said no) releases on "not found" as decided |
| **Q-P1-3** | *"We won't pay again until this settles"* — must that survive a **wallet restart**? The reuse registry is in memory; after a restart an outstanding `nosend` payment is invisible to `pay_402` | **Yes, without a schema change:** on a reuse-registry miss, look for an outstanding `nosend` row labelled `pay402` for the same host (`description "Paid content — <host>"`) before minting. If that proves too coarse (two pages on one host), return here before adding a column (invariant 2) |
| **Q-P1-4** | The unknown state has no upper bound today except the sweepers (nosend 10 min ⇒ their verdict) | Accept the sweepers as the bound (a nosend row settles within ~10 min by `TaskCheckForProofs`); state the bound in the page copy ("usually within 10 minutes") |

**Q8 (agent-level, read 2026-09-28):** PR #2890's 38th comment — the one the G2 sweep did not read — is **our own** implementer report (BSVArchie, 2026-09-18: BEEF-in-header hits a 100 KB wall; 779 / 92,837 / 849-byte BEEFs; 431 on the large one). No maintainer or TSC comment. PR still open, 38 commits (head `36d6098`, 2026-09-28, all bot merges of `upstream/main`); BSV paths unchanged since `92fc793` (2026-09-03). x402 #3387 still open (last update 2026-09-06). ⇒ **Decision 9's deferral stands; nothing re-opens its re-check condition.**
**Q7 verified, not redone:** `X402_INTEGRATION.md` §4a carries the 2026-09-27 correction banner (BRC-121's 30 s window). ⚠️ Residual stale text outside §4a: §4's intro ("only under x402"), the unbannered §4a body, and §10's item-7 row ("HOLD — blocked on spec") — recommended as a one-line pointer by whoever owns that file; not edited here.

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
- [ ] 🍎 Relay note in `MAC_RELAY_BETA5.md` naming `HttpRequestInterceptor.cpp` and `simple_handler.cpp`
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
