# B5-T4 — 402 payments · SCOPE (G2)

**Written:** 2026-09-25 by the G2 research agent, at head `12b03d7` (`git rev-parse`, verified). **Status:** 📌 PROPOSED — candidate phases only; no contracts, no code.
**Read first:** `README.md` (this folder), `X402_INTEGRATION.md` §10 (the 2026-08-19 decision), the four tickets in `../tickets/`.

> **Claim labels used throughout.** **code reading** = read in this repo today, nothing run · **measurement** = a number somebody recorded in a phase doc or log · **doc says** = one of our own documents · **web (URL, date, version)** = fetched today, 2026-09-25. ⛔ Nothing in this file was executed against a wallet, a database or the browser.

---

## ✅ 0. G2 decisions applied (owner, 2026-09-27)

> The research below is the **evidence**; this block is the **decision**. Where they differ, this block wins.
> Full record: `../README.md` → "✅ Decisions as made".

| Question | Decided |
|---|---|
| §9 **Q3** x402 adapter (P3) | ✅ **Deferred** to the next release; re-check if #2890 merges, a live BRC-29 `exact` server appears, or the owner wants bsv.cx (decision 9) |
| **Q1** P1 changes release/re-mint | ✅ **Yes — P1 first.** Ask the chain before freeing or re-paying; can't tell ⇒ wait and tell the user |
| **Step 0** | ✅ **RUN 2026-09-27 — zero hits** (decision 10). Controls 200/404 first; production 1 failed row ⇒ 404; dev 49 failed ⇒ all 404. **P1 is hardening, not an incident** |
| **Q2** | ✅ Record, never broadcast ourselves · **Q4** ✅ pay exactly `amount` |
| Q5 TTL 20 s · Q6 body transport deferred · Q7 fix `X402_INTEGRATION.md` §4a · Q8 read #2890's comment | Per this doc's recommendations (approved) |

---

## 1. Goal

Make a BRC-121 payment **never lose track of whether it was paid** — the wallet asks the chain before it restores coins or mints a second payment, and tells the user plainly when delivery is unknown — then, only if there is something live to test against, add the x402 envelope over the same client.

---

## 2. Telescope — current state of the world (re-fetched 2026-09-25)

| # | Source | What it says today | What changed vs our docs |
|---|---|---|---|
| T1 | **BRC-121 text** — web (`raw.githubusercontent.com/bsv-blockchain/BRCs/master/payments/0121.md`, 2026-09-25; last commit `42ceeb2` 2026-08-10 "Fix legacy repository links", last substantive `fb14783` 2026-04-24) | Seven headers exactly as we send them. **"differs from the server's current time by more than 30 seconds, the server MUST reject the request and respond with 402."** Replay = the 30 s check + the wallet's `isMerge`. **"A client that receives a `402` after submitting payment headers SHOULD NOT automatically retry without user confirmation to avoid double-spending."** No size bound, no body transport, nothing on overpayment, nothing on what a client does when it gets *no* response | 🚨 **`X402_INTEGRATION.md` §4a is wrong against today's text.** It says *"Plain BRC-121 specifies no freshness window at all, so `PAY402_REUSE_TTL_MS = 25_000` is not a live bug in what we ship today."* BRC-121 **does** specify 30 s, the code comment on the constant already knew it (**code reading** `handlers.rs` :: `PAY402_REUSE_TTL_MS` — *"conservative within the BRC-121 30s freshness window"*), and `P11-11-A3`'s rig **measured** the real server rejecting a 33 s-old reuse as stale (**doc says** `PHASE_CONTRACT_item11_brc121_feedback.md` A3 run record). ⇒ §3 item 7 ("reuse cache vs freshness — BLOCKED on x402 spec") is **not blocked on x402**: it is a BRC-121 fact we already ship against. Re-scoped into P1 below. ⚠️ I may not edit `X402_INTEGRATION.md`; the correction is owed at G3 |
| T2 | **x402-F PR #2890** — web (`api.github.com/repos/x402-foundation/x402/pulls/2890`, 2026-09-25) | `state: open`, `merged_at: null`, **36 commits**, head `c14619c`, `updated_at 2026-09-25T09:08Z` (the daily bot merge), `review_comments: 0`, **`comments: 38`** | Was 26 commits / 37 comments at the 09-14 sweep. **BSV files unchanged since `92fc793` (2026-09-03, wording only)** — web (path-filtered commit lists for `typescript/packages/mechanisms/bsv` and `specs/schemes/exact/scheme_exact_bsv.md`, 2026-09-25). `DEFAULT_PAYMENT_WINDOW_MS = 30_000` still — web (`constants.ts` on `feat/bsv-exact-scheme`, 2026-09-25). Spec still says **"carries exactly `PaymentRequirements.amount` satoshis … stricter than plain BRC-121, which accepts overpayment"** and **contains no guidance on client retries, idempotency, or what a client does when no response arrives** — web (`scheme_exact_bsv.md`, 2026-09-25). ⚠️ **One new issue comment since 09-14 that I did not read** (the comments endpoint returned the oldest five); a G3 sweep should read it before assuming nothing moved. Still no TSC engagement visible |
| T3 | **x402 supported networks** — web (`docs.x402.org/core-concepts/network-and-token-support`, 2026-09-25) | 26 EVM networks + Solana, TON, Algorand, Stellar, Aptos, Hedera, Keeta, NEAR, Concordium, XRPL, Cardano. **BSV absent** | Unchanged. Cardano landed via the three-PR route (`X402_INTEGRATION.md` §7); BSV still has two competing spec PRs (#1844 still open, web 2026-09-25) |
| T4 | ⭐ **x402-F PR #3387 "docs: explain request timeouts and safe payment retries"** — web (`patch-diff.githubusercontent.com/raw/x402-foundation/x402/pull/3387.diff`, 2026-09-25; **open, not merged**; `main`'s `client-server.md` has no such section yet) | Verbatim: *"If neither this nor an earlier attempt has submitted a payment authorization or broadcast a transaction, retry only when the HTTP operation itself is safe to repeat."* · *"Treat the payment outcome as unknown. Do not automatically create or authorize a second payment, because the server may have received and settled the first one."* · *"Reconciliation closes the unknown state only when it produces terminal evidence. If the expected payment is confirmed, recover the original operation's result through the server rather than paying again."* · *"Re-run payment policy checks or request user approval when terms such as `scheme`, `network`, `amount`, `asset`, or `payTo` change."* | ⭐ **New since our last sweep, and it is exactly our `TICKET_brc121_remint_on_retry.md` written by the other side.** It is the rule P1 adopts. Note it is an open PR, not the spec — cite it as convergent practice, not authority |
| T5 | **BRC-166** (andyrowe, P2PKH 402) — web (`api.github.com/repos/bsv-blockchain/BRCs/pulls/231` → **merged 2026-08-28**, head `38d7e6e`; `payments/0166.md` present on master, 2026-09-25) | **"Overpayment MUST be accepted."** Settlement **idempotent per invoice** (*"If the invoice is already settled, treat the request as paid and serve the resource again"*). **Produce-then-broadcast** (*"The origin broadcasts … only after it has produced the resource"*). On `500 broadcast_unavailable` the payer **"SHOULD retry the same signed transaction rather than paying a second fee"**; on `402 broadcast_rejected` with `retryable:false`, do not. Discriminator `extra.payloadFormat.kind = "p2pkh-rawtx"` | Was "PR #231 open" in our doc → **now merged**. ⭐ Two BSV 402 specs now **disagree on amount** (166: overpayment accepted; #2890: exact equality) and **agree on retry** (resend the *same* signed tx). Per working rule 5 the disagreement is the design question — see §9 Q4 |
| T6 | **BRCs issue #261** (ours: BEEF-in-header hits intermediary limits) — web (`api.github.com/repos/bsv-blockchain/BRCs/issues/261`, 2026-09-25) | Open since 2026-09-18; **1 comment, ours**; no reply from anyone in 7 days | The body-transport ticket "waits on #261". Nothing to wait for yet |
| T7 | **ts-stack PR #569** (BRC-118 multipart for BRC-105) — web (`api.github.com/repos/bsv-blockchain/ts-stack/pulls/569`, 2026-09-25) | Open, 29 commits, 171 files, head `f02333a`, updated 2026-09-24, CI green per its description | Still unmerged; still says BRC-121 wire behaviour unchanged (per the ticket's reading) |
| T8 | **bsv.cx** — web (`bsv.cx/`, 2026-09-25) | Still advertises a 402 challenge on `POST /n/batch` (*"returns a 402 payment challenge first (~1 sat/hash)"*), pay in BSV or MNEE. No wire-level detail on the page | Still the only live x402-on-BSV server, still the **P2PKH variant** (BRC-166), not our BRC-29 payload. **No BRC-29 `exact` server exists to pay** — the 2026-08-19 gate is still shut |
| T9 | **now.bsvblockchain.tech** — web (2026-09-25) | Reachable, "Paid µicro-parody", articles priced 75–200 sats | Target A (plain BRC-121 non-regression) is alive |
| T10 | **Peers on "outcome unknown"** — web: `winsznx/routedock` issue #386 (2026-09-25, open) · `coreyphillips/beignet` PR #992 (2026-09-25, **merged 2026-09-23**) · LND `TrackPaymentV2` docs (2026-09-25) | routedock: *"a payment is signed at most once per `pay()` call… every retry resends exactly that header"* — one `pay()` had settled **up to 4** payments against a flaky route. beignet (Lightning): a payment stays **PENDING** while its HTLC is in flight; user text *"Dispatch timed out while its HTLC may still be in flight; outcome unknown. Check the payment for this invoice before paying it again."* LND: `PaymentStatus ∈ {INITIATED, IN_FLIGHT, SUCCEEDED, FAILED}` — a fourth, non-terminal state is first-class | ⭐ Three independent ecosystems reached the same shape: **a non-terminal "in flight / unknown" state, and never a second signature until it resolves.** We have two terminal states (`nosend` → `failed`/`sending`) and a log line that says "funds preserved" |

**Not fetched / not verifiable today:** `Vibewatch-io/vibewatch-mcp` issue #27 (facilitator settled ~5 min after an upstream 502) — 404 on both the page and the API; only the search-result title is known. The #2890 38th comment (above).

---

## 3. Kaleidoscope — do we already have this?

| Shape we would otherwise duplicate | Already exists | Use it for |
|---|---|---|
| "Is this txid on the network?" (mempool **or** mined) | **code reading** `handlers.rs :: check_tx_exists_on_chain(&services, txid) -> Result<bool,String>` — Services chain ARC → WoC → JungleBus → Bitails; `Ok(true)` = Mined or InMempool; orphan/stale collapse to `false`; transport failure = `Err`. Three callers today: `internalize_action` (~12989), a sweep at ~18172, `monitor/task_check_peerpay.rs :: 402` | ⭐ **The P1 primitive.** A BRC-121 payment the server may have broadcast is a *txid* question, not an *outpoint* question — this is the right helper, and `Err` must map to "unknown, do nothing" |
| "Is this outpoint spent, and by whom?" | **code reading** `reconcile.rs :: check_outpoint_spent` (WoC + GorillaPool, owner-locked decision table, `Unknown` fails closed) and `check_outpoint_unspent` (positive listing only) | Not P1's primary — it answers the *input* side. Useful as the **ground-truth audit** in P1 Step 0 (were the released inputs actually spent?), and as the belt-and-braces check before restoring inputs if the owner wants two signals |
| Release a minted-but-dead transaction | **code reading** `handlers.rs :: release_unbroadcast_transaction` (one SQLite txn: disable outputs, restore inputs by placeholder/txid, status → failed **with `failed_at` stamped** via `set_transaction_status`). Callers: `resolution_failed_response`, `abort_action`, `pay_402`'s too-large refusal, `release_nosend` | ⛔ Do not write a fourth cleanup. P1 adds a **chain check in front of** `release_nosend`'s call to it, nothing else |
| The backstops that already resolve "possibly paid" — late | **code reading** `monitor/task_check_for_proofs.rs`: a `nosend` row older than 60 s is checked on WoC and **promoted if found**; not found by `NOSEND_TIMEOUT_SECS` (10 min) → `mark_failed`. `monitor/task_unfail.rs`: `failed` rows with `failed_at` within 6 h are re-checked and recovered to `completed` **only when mined with a proof** (`recover_transaction` re-marks inputs spent, re-enables outputs) | ⭐ This is why the 4,600-sat rows in the remint ticket ended `completed` — the sweeper got there. **The gap is the ~10-min window in which released inputs are `spendable=1` and a second payment can be minted**, plus the fact that the *user* is told "funds preserved" in that window. P1 closes the window at the decision point; the sweepers stay as the crash backstop |
| Reuse-don't-recreate for the same (domain, server key, URL, sats) | **code reading** `handlers.rs :: pay402_reuse_key`, `AppState.pay402_reuse`, `PAY402_REUSE_TTL_MS = 25_000`, `PAY402_REUSE_STATUS_SQL` (fixed `a337538` — *"the pay402 reuse cache has never once worked — it queried a column that does not exist"*, verified `git show -s`) | P1 extends the miss path (past TTL / entry evicted): **ask the chain about the outstanding txid before minting**. Same registry, one new branch |
| Auto-retry of the paid request | **code reading** `HttpRequestInterceptor.cpp :: Async402ResourceHandler::onUpstreamComplete` — `retryable = (status == 431) \|\| (5xx)`, `MAX_UPSTREAM_RETRIES = 1`, re-sends the **same** headers | Consistent with BRC-166's "retry the same signed transaction" and does **not** violate BRC-121's SHOULD NOT (that sentence is about a **402**, which we never auto-retry). P2 only narrows the 431 arm |
| Tell the user a payment is in flight | **code reading** `Brc121PaymentBannerTask` / `Brc121PaymentBannerHideTask` (A1/A2 banner, page DOM); `/payment-failed` route (`frontend/src/pages/PaymentFailedPage.tsx`, copy *"Your sats are safe — the transaction was never broadcast"*); `/payment-pending` route; Activity row via `create_action` description `"Paid content — <host>"` | P1's "delivery unknown" state is a **third banner text + a third page state**, not a new surface. ⚠️ `PaymentFailedPage`'s copy asserts "never broadcast" — under P1 that claim must be conditional on the chain answer |
| Hodos-voice failure page on the 431 path | **code reading** `simple_handler.cpp :: OnLoadError` branch 2 (`ConsumeBrc121FailedUrl` → `/payment-failed?…`) | Exists. The ticket says it did not reach the screen on the real 431 — see P2 A2 for the hypothesis |
| The 402 demo server | **code reading** `demos/brc121-402/server.js` (express, `@bsv/sdk`, `/paid` and `/paid-session`, replay guard by txid, **does not broadcast**) | Target A local. P1's rig needs a **broadcast-on-receipt** switch added to it (rig-only) to manufacture "paid, then refused" |
| Size budget for the header channel | **code reading** `BRC121_MAX_BEEF_BYTES = 64*1024`, `large_parent_bytes_for_budget`, the `P11-11-A7` hard stop in `pay_402` with the owner-approved 422 message | P2 measures the **assembled header block** against it instead of a constant, per the ticket's own recommendation |

**Shape found at open, to note for close:** the same three-way outcome (`paid` / `not paid` / `unknown`) appears in `check_tx_exists_on_chain` (`Ok(true)` / `Ok(false)` / `Err`), in `reconcile::SpentStatus` (`Spent` / `Unspent` / `Unknown`), in `UnspentProbe`, and in the sweepers' `OracleVerdict`. P1 should **not** add a fifth enum; it should map `Err` → do nothing, exactly as `SpentStatus::Unknown` does.

---

## 4. Ticket review

| Ticket | Still true today? | Proposed disposition | Why |
|---|---|---|---|
| `TICKET_brc121_release_restores_inputs_the_server_may_have_spent.md` 🚨❔ | **Mechanism confirmed by code reading, occurrence unverified.** `onUpstreamComplete` calls `releaseNosendThenContinue()` on **any `status > 0` non-2xx** after the single retry — including a **5xx from a gateway in front of an origin that may already have internalized and broadcast**. `release_nosend` checks only `status == 'nosend'` in our own DB, then restores inputs to `spendable=1`. It never asks the chain. ⭐ The realistic trigger is the one the owner already hit: the site's origin took **34.5 s** (`cfOrigin;dur=34481`, **measurement**, item-11 §0); a Cloudflare 5xx at the origin timeout while the origin finishes internalizing is exactly this path. **Mitigations that exist:** `set_transaction_status(Failed)` stamps `failed_at`, so `TaskUnFail` re-checks within 6 h — but only recovers **once mined with a proof**; in the meantime the inputs are spendable and a later send that picks them fails as a double-spend attempt (first-seen rule protects the payee; the user gets "Missing inputs" and a confusing failure, not a loss) | **P1 A2** (fix) after **P1 Step 0** (ground truth, owner's wallets, read-only) | Trip-wire 1 shape (a row disagreeing with the chain), so rule 7 applies: **verify cheaply first, escalate on evidence.** The check is designed in P1 Step 0 below |
| `TICKET_brc121_remint_on_retry.md` (narrowed 2026-09-25) | **Yes, as narrowed.** Within 25 s the reuse works (`a337538`, `1ec1b8f`). Past 25 s, or after a `status == 0`, `pay_402` mints again with no chain check (**code reading**: the miss path falls straight to `create_action`). The log/UI still say "NOT broadcasting (funds preserved)" (**code reading** `onUpstreamComplete` warning string; `PaymentFailedPage` copy). The A3 residual (a second navigation installs a second handler and mints one extra never-broadcast payment) is recorded and un-fixed. The A5 correction's three hypotheses for the "first 402" are **still un-instrumented** | **P1 A1 (instrument) + A3 (chain-check before re-mint) + A4 (the unknown state)**. Merge the freshness half of `X402_INTEGRATION.md` item 7 into A3 | The x402 #3387 rule and routedock #386 are this ticket, solved the same way: sign once; on unknown, reconcile, never re-sign |
| `TICKET_brc121_beef_header_exceeds_100kb_and_payment_is_lost.md` (leftover) | **Yes, both leftovers.** `retryable = (status == 431) \|\| …` still retries a size-431 byte-identically (**code reading**). The 4th step (Hodos wording) is **unverified**: the route exists (`OnLoadError` branch 2) and the ticket reports it did not appear | **P2 A1 + A2** | Small, understood, and the fix shape is in the ticket. The open question is only *why* the swap did not fire — P2 A2 must measure it before fixing (see hypothesis there) |
| `TICKET_brc121_client_has_no_body_transport_for_large_beef.md` (decision) | **Yes — and nothing upstream moved** (#261: no replies; #569 open; BRC-121 silent). Exposure still low: the 64 KB refusal ships, no server we pay advertises multipart | **P2 A3 — a decision item, not a build item.** Recommend **defer to beta.6 with a re-check condition** (any reply on #261, or BRC-121 text change, or a 121 server advertising `x-bsv-payment-transports`) | Building a transport no server accepts is speculative scope (working rule 2). Keep the ticket; add the re-check condition at G3 |
| *(no ticket)* `X402_INTEGRATION.md` §10 items 1, 2, 3, 5, 6 | Decision stands on its own terms, but its stated gate — *"merges to a release when either PR #2890 merges or a real server advertises the BRC-29 scheme"* — is **still unmet** after 6 weeks; the only live server is the BRC-166 variant our §3 does not cover | **P3, last, and the first thing cut** (G1 text: *"then the x402 adapter"*) | See §9 Q3 |
| *(no ticket)* §10 item 4 — strict amount equality, HOLD | **Resolvable from code today.** `pay_402` builds one output of exactly `req.satoshis`; the 1,000-sat service fee is a separate output, change is separate (**code reading** `create_req.outputs`, `HODOS_SERVICE_FEE_SATS`). So the payment output **is already exact**; a client that pays exactly `amount` satisfies both #2890 (equality) and BRC-166 (≥ amount) | Close the hold with a **unit assertion** in P3 (or P1 if P3 is cut) — no behaviour change | The two specs' disagreement is a *server* rule; as a payer, exactness satisfies both |
| *(no ticket)* §10 item 7 — reuse cache vs freshness, HOLD | **The premise is stale** (T1). The window is BRC-121's, live today, measured by A3's rig | **Absorbed into P1 A3** — no longer a separate hold | See T1 |

---

## 5. Candidate phases (ordered — uncertain first, cut candidate last)

### B5-T4-P1 — a BRC-121 payment never loses track of whether it was paid

**Objective (one sentence):** before the wallet restores a payment's coins or mints a second payment for the same page, it asks the network whether the first payment exists, and if it cannot tell, it waits and says so.

**Tickets:** release-restores-inputs (fix), remint-on-retry (fix), §10 item 7 (absorbed).

**Step 0 — the cheap ground-truth check** (👤 owner's real wallets; **read-only**; designed here, not run):

| | |
|---|---|
| **Rows** | `SELECT t.txid, t.status, t.failed_at, t.created_at FROM transactions t JOIN tx_labels_map m ON m.transaction_id = t.id JOIN tx_labels l ON l.id = m.label_id WHERE l.label IN ('pay402','brc121') AND t.status = 'failed'` — the released BRC-121 payments (labels from **code reading** `pay_402` → `labels: ["brc121","pay402"]`; table names from `database/CLAUDE.md`; ⚠️ column names of the join to be confirmed against `tx_label_repo.rs` at G3). Also `WHERE t.status = 'nosend'` for anything still hanging |
| **Chain query, per txid** | `GET https://api.whatsonchain.com/v1/bsv/main/tx/hash/{txid}` — 200 = exists (the payee broadcast it), 404 = never seen. One `curl` per row; the wallets that have used BRC-121 have tens of rows, not thousands |
| **Positive control** | a known-broadcast BRC-121 txid from `P11-11-A5` (`73eab7534e1514…`, **measurement**) returns 200; a known-failed one from the remint ticket (`1f8a2e4e…`) returns 404 |
| **Verdict** | any `failed` row that exists on chain ⇒ **poisoning confirmed** — stop, tell the owner, and the row's restored inputs are the ones to audit with `check_outpoint_spent`. Zero ⇒ hardening item, proceed as planned |
| **Residue** | none — read-only |

**Items:**

| Item | What | Negative-control idea |
|---|---|---|
| **A1 — instrument** | The A5 leftover: log whether a `status == 0` attempt's bytes left the machine (`CefURLRequest` upload progress / `GetRequestError` — ⚠️ **genuine unknown**: CEF may not be able to say; cite the header before relying on it), and capture the 402 body/headers on a refusal so the "first 402" has a reason code | Not a behaviour; the control is that the log line appears on a rig-cancelled request and not on a completed one |
| **A2 — release asks the chain** | In `release_nosend`: before `release_unbroadcast_transaction`, call `check_tx_exists_on_chain`. `Ok(true)` ⇒ **do not release**; promote the row the way `TaskCheckForProofs` does for a found `nosend` (`nosend → sending`, reuse `conn_update_status` / the existing promotion) and tell the shell "paid". `Ok(false)` ⇒ release as today. `Err` ⇒ **do nothing**, 202/409 to the shell, leave it to the sweepers. ⛔ Rule 7 trip-wire 2: `Err` must never become a verdict | Stub the probe to `Ok(false)` (rig seam, dev-only, same pattern as `HODOS_402_UPSTREAM_DELAY_MS`) and run the rig below ⇒ the row flips to `failed` and its inputs to `spendable=1` while the txid is in the mempool — the ticket's defect, seen |
| **A3 — re-mint asks the chain** | In `pay_402`'s reuse-miss path (past TTL, or entry evicted after a `status == 0`): if an outstanding txid exists for this reuse key, ask the chain first. Exists ⇒ do not mint; return a distinct `"possibly_paid"` outcome with the txid. Not found ⇒ mint. `Err` ⇒ do not mint; return "unknown" | Same stub ⇒ two `createAction`s for one URL after a 30 s hold (the remint ticket's shape) |
| **A4 — the user sees "delivery unknown"** | Replace the *"NOT broadcasting (funds preserved)"* log/UI claim with a three-way message: **paid** (chain shows it) / **not paid** (chain does not) / **unknown — you may have paid; here is the txid; we will not pay again until this settles** (the beignet/LND shape). Banner + `/payment-failed` copy + Activity row. ⚠️ `PaymentFailedPage` must stop asserting "never broadcast" unconditionally | Screenshot of the rendered page in each of the three states; RED = the old page text on a chain-confirmed payment |
| **A5 — 👤 real money, one sitting** | Against `now.bsvblockchain.tech` with the dev delay seam: hold > 30 s ⇒ real 402 stale ⇒ assert **no** second mint and the correct user message; then a normal paid load ⇒ gold pill on the paying tab (`R-GOLD` half for this route, which `REGRESSION_SET.md` still lists as **owed**) | Two-sided with A3: the same hold on the pre-fix binary mints twice |

**Rig for "paid, then refused" (T2, agent-run, real sats):** add a rig-only `BROADCAST_ON_RECEIPT=1` switch to `demos/brc121-402/server.js` that broadcasts the BEEF via ARC and then returns **502**. That is the exact sequence the ticket fears, reproducible on demand for ~1,150 sats a run.

**Genuine unknowns (K):** (i) whether Step 0 finds anything — changes urgency, not the fix; (ii) whether CEF can report "bytes left" for a cancelled request (A1); (iii) how far `check_tx_exists_on_chain`'s `Err` arm reaches in practice (all four providers down) — the fail-closed path must be seen to hold, not assumed.

**Reuse-TTL side question:** A3's rig measured accept at 25 s and reject at 33 s against a server clock ±30 s. `PAY402_REUSE_TTL_MS = 25_000` leaves ~5 s for RTT and skew. With A3 in place a stale reuse costs one round trip, not a double payment, so the TTL becomes a tuning knob — recommend 20 s, owner's call (§9 Q5).

### B5-T4-P2 — the 431 path fails once, cleanly, in Hodos's voice

**Objective:** a payment that cannot fit the header is refused before minting, is never retried byte-identically, and the user reads a Hodos page, not Cloudflare's text.

**Tickets:** beef-header-exceeds-100kb (both leftovers), body-transport (decision only).

| Item | What | Negative-control idea |
|---|---|---|
| **A1 — size 431 is not retryable** | Measure the **assembled header block** the shell is about to send (original headers + five `x-bsv-*`) against the budget; a 431 on a block that exceeds it is final — no retry; a 431 on a block well under it keeps the one retry (transient) | Rig: inject an oversized cookie into `originalHeaders` ⇒ pre-fix binary logs `auto-retry 1/1` then 400; post-fix logs the refusal with the measured size and **zero** retries |
| **A2 — the Hodos page reaches the screen** | Establish *why* `/payment-failed` did not show on 2026-09-17. **Hypothesis (unverified, code reading only):** `Async402ResourceHandler::GetResponseHeaders` hands CEF the real 431/400 **with Cloudflare's body**, so the navigation *succeeds* with an error-status document and `OnLoadError` never fires — the swap only runs on a load *error*. If so the fix is to serve the Hodos page from the handler itself (or a redirect to `/payment-failed`), not to fix `OnLoadError`. ⚠️ Rule 4: cite CEF's `CefLoadHandler::OnLoadError` contract and Chromium's HTTP-error-page rule before building on this; today's web search did not settle it | The RED is the 2026-09-17 log itself (`debug_output-34216.log` 6260–6286, **measurement**, ⚠️ rotated) reproduced on the rig with the fix reverted |
| **A3 — decision: body transport** | Record the deferral with a re-check condition (any #261 reply · BRC-121 text change · a live 121 server advertising `x-bsv-payment-transports`). No code | — |

**Unknowns:** A2's mechanism only. **Dependency:** the 422 message promises consolidation that `TICKET_wallet_cannot_shed_large_parents.md` (**B5-T1**) has not built — P2 must not make that promise louder.

### B5-T4-P3 — the x402 envelope over the same client *(cut first if the cycle runs long)*

**Objective:** a page that answers 402 with a JSON `PaymentRequired` naming `bsv:mainnet` is paid with the same BRC-29 payment, serialised as `PAYMENT-SIGNATURE`, behind a flag that leaves the header form byte-identical when off.

**Items:** §10's 1, 2, 3, 5, 6 as written; item 4 as a unit assertion (the output is already exact); **plus a decision on the BRC-166 P2PKH branch** (`payloadFormat.kind == "p2pkh-rawtx"`), which is the only branch a live server would exercise today. Test matrix `X402_INTEGRATION.md` §9 A → C → B stands.

**Unknowns (K):** (i) **no live BRC-29 `exact` server** — target C is Deggen's example server from a branch whose CI has never run; (ii) whether the owner wants the P2PKH branch (a second addressing mode, and the one that actually has a merchant); (iii) whether #2890's wire surface still matches upstream after 36 bot merges. **Negative-control idea:** flag off ⇒ target A's five headers and paid page are byte-identical to today (hash the request the demo server logs); flag on against a `bip122:…` challenge ⇒ refused **without a mint** (count `createAction` calls = 0).

---

## 6. Integration check (RELEASE_CYCLE §3.3)

**1. Invariants at risk**

| Invariant | Where this track touches it | Guard |
|---|---|---|
| ⭐ **Gold pill** (`R-GOLD`) — `HttpRequestInterceptor.cpp :: OnWalletCallSuccess`, reached on this path from `Async402ResourceHandler::firePaymentSuccessIpc()` on the 2xx branch (**code reading**) | P1 A2/A4 edit the **else** branch of `onUpstreamComplete` and the release task; P3 edits `TryHandleBrc121_402` ahead of it. ⚠️ A "chain says paid" promotion (A2) delivers **no content** and must **not** fire the pill — the pill means *content was bought*, and firing it on a refused response would tell the user a paywall opened when it did not. Conversely, the x402 path must fire it on 2xx exactly as the header path does | `R-GOLD` both halves at P1 and P3 close; `REGRESSION_SET.md` already lists the BRC-121 route as **owed** — P1 A5 pays it |
| **Per-session counters** (`R-COUNT`) | Spend is recorded **at mint** in `dispatch_payment` (**code reading** `pay_402` comment: *"The money is committed here, at the mint, which is where it is metered"*). A2's promotion and A3's "possibly paid" must **not** record a second spend; a release does not un-record today and P1 does not change that | Assert counter unchanged across a promote / a refuse / a possibly-paid |
| **`PaidContentCache`** | Only the 2xx branch writes it (**code reading**). "Paid but no content" caches nothing — correct; a later successful load will (rightly) pay again under BRC-121, which has no per-invoice idempotency. A4's copy must say so | none needed; note in A4's copy |
| `R-ONE-CLICK-ONE-SPEND` | The direct subject of P1 | P1 A3/A5 |
| `R-INTEXT` | `release_nosend` and `broadcast_nosend` are internal (no `X-Requesting-Domain`); adding a chain probe does not change their gating | unchanged |
| **Invariants 2/3** (schema, crypto) | P1 changes **release logic and a status transition**, not schema or crypto — but it is money-path behaviour: **ask before editing** (§9 Q1) | owner sign-off at G3 |
| `R-NODOUBLE` (sweeper rule: release only on positive on-chain observation) | P1 A2 is the same rule applied at the decision point — it **strengthens** it. ⚠️ It also means `release_nosend` gains a network call inside a handler that today is DB-only; the DB lock must not be held across the `await` (`db_lock_held_across_await` ticket is background) | code review at G3 |

**2. What we touch that we did not write**

- **Cloudflare's 100 KB header-block limit** and its 431 body (P2) — an intermediary's default, not a spec.
- **BRC-121's 30 s clock rule** and the origin's internalize-then-broadcast timing (P1) — the payee decides when money moves, not us.
- **CEF `CefURLRequest` status-0 semantics** (P1 A1) — whether "no response" can be distinguished from "never sent".
- **Chromium's HTTP-error-document behaviour** (P2 A2) — whether an error status with a body is a "load error".
- **Four third-party indexers** behind `check_tx_exists_on_chain` (P1) — their `NotFound` vs transport-failure distinction is what makes `Err` safe.
- **x402-F's wire surface** (P3) — moving weekly under a PR whose CI has never run.

**3. What we would un-ship if wrong**

- P1: revert the chain-probe call sites (two functions, additive); the sweepers resume as the only reconciliation. Behaviour returns to today's.
- P2: revert the 431 arm to `retryable = (status == 431) || 5xx`; the Hodos page fix is additive.
- P3: the flag. Off = today's bytes.

---

## 7. Feasibility inputs (§3.7)

| | |
|---|---|
| **N — owner-hours** | **≈ 4–5 h.** P1 Step 0 ground truth on the owner's wallets ~0.5 h (read-only; agent can run the queries if given the DB path, owner reads the result) · P1 A5 real-money sitting ~1.5 h · P2 A2 visual (a real 431 needs the large-parent wallet from 2026-09-17) ~0.5 h · P3 target B (bsv.cx, real sats) + target C ~1.5 h — **0 if P3 is cut** |
| **K — unknowns** | P1: 3 (Step 0 result; CEF "bytes left"; the fail-closed reach) · P2: 1 (the error-page mechanism) · P3: 3. **All P3's are external; all P1's are measurable in a day** — which is why P1 goes first |
| **Cross-track** | **B5-T1** owns `release_unbroadcast_transaction`'s reservation semantics (`TICKET_reservation_ownership_converge_on_spent_by.md`) and the consolidation the 422 promises (`TICKET_wallet_cannot_shed_large_parents.md`) — P1 A2 edits a *caller*; serialise after T1's reservation phase or agree the seam. **B5-T6** `TOOLS_TAB_claim_a_payment.md` is the natural home for surfacing "possibly paid" txids. **T0** none. 🍎 P1/P2 touch shared C++ ⇒ relay note (root `CLAUDE.md` build rule) |
| **Serial constraints** | One real-money driver at a time (§3.5); P1 A5 and any T1 money row cannot share a sitting |
| **Test money** | dev wallet + the 2026-09-17 large-parent wallet (P2); ~2–3 k sats per P1 rig run |

---

## 8. Prior-art rows — ready for `development-docs/PRIOR_ART.md` §3

| Date | Question | Source(s) read | What we learned | Verdict | Landed in |
|---|---|---|---|---|---|
| 2026-09-25 | What should a payment client do when the paid request gets no response, or an error after the payment left? | **x402-F PR #3387** (open docs PR, `docs/core-concepts/client-server.md`), **`winsznx/routedock` #386**, **`coreyphillips/beignet` PR #992** (merged 09-23), **LND `TrackPaymentV2` PaymentStatus** | ⭐ Three ecosystems converge: *"Treat the payment outcome as unknown. Do not automatically create or authorize a second payment"*; *"a payment is signed at most once per `pay()`"*; Lightning keeps a first-class **IN_FLIGHT** state and tells the user *"outcome unknown. Check the payment … before paying it again."* We have two terminal states and a log line that says "funds preserved" | 🟢 paid off — it set P1's rule and A4's wording | B5-T4 SCOPE §5 P1 |
| 2026-09-25 | Does BRC-121 itself bound payment freshness, or only x402? | **BRC-121** current text (`payments/0121.md` @ `42ceeb2`) | **BRC-121 mandates the 30 s `x-bsv-time` window** ("MUST reject … respond with 402"). Our `X402_INTEGRATION.md` §4a said plain BRC-121 has none — wrong, and our own A3 rig had already measured the rejection at 33 s | 🟢 paid off — it un-blocked §10 item 7 and moved it into P1 | B5-T4 SCOPE §2 T1, §4 |
| 2026-09-25 | Do the two BSV 402 specs agree on amount and on retry? | **BRC-166** (merged 2026-08-28) vs **#2890 `scheme_exact_bsv.md`** | **Disagree on amount** (166: *"Overpayment MUST be accepted"*; #2890: *"exactly … stricter than plain BRC-121"*); **agree on retry** (resend the *same* signed tx; 166 adds idempotent-per-invoice and produce-then-broadcast). As a payer, paying exactly `amount` satisfies both — and our output already is exact | 🟢 paid off — closed §10 item 4 as a unit assertion | B5-T4 SCOPE §4 |
| 2026-09-25 | Has anything moved on x402-on-BSV since 09-14? | **#2890** API, path-filtered commit history, `constants.ts`, `docs.x402.org` networks page, **#1844**, **#261**, **ts-stack #569**, **bsv.cx** | 36 commits (all bot merges), BSV files untouched since 09-03, window still 30 s, BSV still absent from the supported list, #1844 still open, #261 unanswered, #569 open, bsv.cx still P2PKH-only. **The 2026-08-19 gate (a paying BRC-29 `exact` server) is still shut** | 🟡 context — nothing changed the decision, which is itself the finding | B5-T4 SCOPE §2, §9 Q3 |

---

## 9. Open questions for the owner — each with a recommendation

| # | Question | Recommendation |
|---|---|---|
| **Q1** | **May P1 change money-path release/re-mint behaviour** — *never restore a BRC-121 payment's inputs or mint a second payment without asking the network; "cannot tell" means wait*? It is not schema or crypto, but it is the wallet deciding whether coins are spendable | **Yes, with Step 0 first.** It is the sweeper's own rule (`R-NODOUBLE`, positive observation before release) applied where the decision is actually made. Rule 7 says verify cheaply first: run the read-only ground-truth check on your BRC-121 wallets before any code, and if it finds a released-but-on-chain row, that becomes the phase's first item and everything measured since 09-17 on those wallets is re-run |
| **Q2** | When the chain shows the payment but the site refused, should **we** broadcast or record? | **Record only, never broadcast ourselves.** BRC-121 puts broadcast with the payee (it internalizes then serves); a payer broadcast re-creates the `isMerge` race the nosend design exists to avoid (**code reading** `broadcast_nosend` header comment). Promote the row (`nosend → sending`) so the Monitor proves it, and tell the user the txid |
| **Q3** | **Build the x402 adapter (P3) in beta.5, or defer?** Six weeks on: zero maintainer engagement, BSV files unchanged, no BRC-29 `exact` server to pay, and the only live merchant speaks the *other* addressing mode | **Defer P3 to beta.6 with a re-check condition** (#2890 merged, or a live BRC-29 `exact` server, or the owner wanting bsv.cx specifically). G1 already names it the second thing cut. If you want *something* x402-shaped this cycle, the **BRC-166 P2PKH branch** is the half with a real merchant — but it is a second addressing mode, not the adapter §10 decided, so it would be a new decision, not a continuation |
| **Q4** | Two BSV 402 specs disagree on overpayment. Which do we follow? | **Neither needs following as a payer: pay exactly `amount`**, which both accept. Close §10 item 4 with a unit assertion that `pay_402`'s payment output equals `req.satoshis`. Do not add a tolerance knob |
| **Q5** | Shorten `PAY402_REUSE_TTL_MS` from 25 s? A3's rig: accepted at 25 s, rejected at 33 s, server window ±30 s | **20 s, after A3 lands** — once a stale reuse costs a round trip rather than a second payment, the number is a tuning choice, not a safety one. Do it in the same commit as A3 so the negative control (a 25 s hold now mints fresh, and mints **once**) covers both |
| **Q6** | The body-transport ticket: keep waiting on #261? | **Yes — defer with a re-check condition**, no build. Seven days, no reply, no server advertising it. Working rule 2 |
| **Q7** | `X402_INTEGRATION.md` §4a contradicts BRC-121 and our own code comment. Who fixes it and when? | **At G3, by whoever writes P1's contract** — one paragraph, citing T1. I did not edit it (one-file rule) |
| **Q8** | The 38th comment on #2890 was not read today | Read it in the P1 kickoff sweep (2 minutes) before the P3 deferral is recorded as final |

---

*End of SCOPE. No contracts, no code, no other file touched.*
