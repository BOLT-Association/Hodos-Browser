# B5-T4-P2 — a payment too big for the header fails once, cleanly, in Hodos's voice · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (G3, 2026-09-28). ⛔ Documents only — no code has been written for this phase.

**Track:** B5-T4 402 payments · **Tickets:** `../../tickets/TICKET_brc121_beef_header_exceeds_100kb_and_payment_is_lost.md` (both leftovers: fix step 3, verify step 4), `../../tickets/TICKET_brc121_client_has_no_body_transport_for_large_beef.md` (decision only — deferred, Q6) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent for T4 · **Platforms:** both (shared C++ shell + React page) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **9** (Q6 body transport **deferred**, no build; x402 adapter deferred). Carried, not reopened.

---

## 1. Goal

When a paid request is refused because its headers are too large, Hodos does not re-send the identical oversized request, and the user reads a Hodos page that says what happened — not the site's or Cloudflare's raw text.

## 2. Done means

- [ ] A 431 on a paid retry whose **assembled header block** (the page's own headers + the five `x-bsv-*`) exceeds the measured budget is **final**: zero auto-retries, and the log records the measured size and the budget.
- [ ] A 431 on a block **well under** the budget keeps today's single retry (the genuinely transient case the code comment describes).
- [ ] On the 2026-09-17 shape (431, then the user's tab), the tab shows the Hodos `/payment-failed` page, not the upstream body — and the mechanism by which it did not before is **recorded as measured**, with the CEF/Chromium contract cited (rule 4).
- [ ] The body-transport deferral and its re-check condition are recorded (below, §6) — no code.
- [ ] The existing 422 "too large" refusal text is not made to promise consolidation louder than T1 delivers.

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| **R-GOLD** | Gold pill on the originating tab for every auto-approved paid retry | P2 edits the `retryable` expression in `onUpstreamComplete`, two branches above the 2xx `firePaymentSuccessIpc()`. A transient 431 that now succeeds on its retry must still fire the pill; a final 431 must fire none |
| **P11-11-A7** (beta.3 phase row: 64 KB BEEF budget, refuse **before** minting) | A payment that cannot fit is not minted | P2 adds a **second** size measurement (the whole header block, in the shell). It must not replace or loosen the wallet's pre-mint refusal (`BRC121_MAX_BEEF_BYTES`), and it must not move the refusal to after the mint |
| **R-BEEFOUT** | A BEEF we hand a counterparty stands on its own | P2 does not build BEEF. Listed because the obvious "fix" for a too-large header — trimming ancestry — is exactly what this row forbids. ⛔ Not an option. Run at the boundary; state whether `MAX_BEEF_ANCESTORS` was reached |
| **B5-T4-P1** outcome states | The page tells the truth (paid / not paid / unknown) | P2's page change and P1's copy land on the same route; P2 must pass P1's outcome through, not overwrite it with a fixed "not broadcast" |
| `PaidContentCache` | Only a 2xx writes it | P2 must not cache the Hodos error page as the URL's paid content |

## 4. Evidence table

⛔ Money rows: RED designed by a **second agent** (`../../../RELEASE_CYCLE.md` §4.2). A1 changes whether an already-minted payment's delivery is retried — money-path behaviour — so its REDs are independent. A2 is a display/diagnosis row; its RED is written here.

**Rig:** the local demo server (`demos/brc121-402/server.js`) behind a header-size limit that answers **431** above a configurable total (rig-only — e.g. a Node `maxHeaderSize` listener or a reverse proxy set to 100 KB), plus a way to inflate the page's own headers (an oversized cookie set by the demo page) so the block exceeds the budget with a **small** BEEF. ⛔ This reproduces the 2026-09-17 failure without controlling coin selection and without a large-parent wallet.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P2-A1a` | **Size 431 is final.** Oversized cookie ⇒ the shell measures the assembled block over budget ⇒ on the 431 it logs the measured size + budget and makes **zero** retries; the paid retry proceeds to the refusal path (P1's release/probe) | (i) Revert to today's arm (431 or any 5xx retryable) ⇒ tab log `auto-retry 1/1` and the proxy logs **two** paid requests with byte-identical `x-bsv-*` ⇒ red on the proxy's count, independent of our log · (ii) Subject check first: the paid retry is a new `CefURLRequest` with flags `UR_FLAG_DISABLE_CACHE` only, and `cef_types.h` says cookies are sent only with `UR_FLAG_ALLOW_STORED_CREDENTIALS` ⇒ the demo page's oversized cookie may never reach the paid request; the proxy must log the Cookie header's size on the paid request — if absent, inflate through a header carried in `ctx_.originalHeaders` instead (see §4a) · Right reason: the 431 comes from the proxy's size limit (its log shows size > limit), not from the demo server or a CDN · Residue: one `nosend` payment per run, released by P1's path or the 10-min sweeper (scratch) — designed by controls-F (Opus), 2026-09-28 | Tab browser `debug_output-<pid>.log` (role-log verified): absence of `auto-retry 1/1` for this URL; demo proxy's request log shows **one** paid request, not two; the measured size equals the bytes the proxy counted (±header framing) | T2 | ⬜ |
| `P2-A1b` | **Transient 431 still retries once.** A block well under budget answered 431 once by the rig (then 200) ⇒ exactly one retry, 200, content served. Two-sided with A1a | (i) Drop 431 from `retryable` altogether ⇒ one request, final refusal, no pill, P1's release path runs ⇒ red · (ii) re-mint on retry (scratch build calling `/wallet/pay402` again) ⇒ the proxy logs two different `x-bsv-beef` hashes and two `transactions` rows ⇒ red — proves "same signed payment" is measured by the hash · Right reason: the rig's one-shot 431 is keyed on request count and fires on the **paid** request, not on the first 402 fetch · Residue: one completed real payment to the rig server (~1,150 sats), broadcast on the 200 — scratch — designed by controls-F (Opus), 2026-09-28 | Proxy log: two requests with **byte-identical** `x-bsv-*` values (same signed payment, no re-mint); gold pill on the tab (R-GOLD); one `transactions` row for the payment | T2 | ⬜ |
| `P2-A1c` | **The measurement is of what is actually sent.** The size the shell computes is the header block the network stack emits, not the five `x-bsv-*` values alone (the P10d lesson: measure the whole request, `10c772d`) | Switch (i): compute the size from the five `x-bsv-*` values only (the P10d trap) ⇒ at the near-budget cookie size the shell logs under budget while the proxy counts over ⇒ red · (ii) compute from what the shell set (`ctx_.originalHeaders` + `x-bsv-*`) ⇒ misses headers the network stack adds (Cookie from the jar, `Host`, `Accept-Encoding`, `Sec-CH-UA*`) ⇒ red if the gap exceeds the tolerance · The tolerance must be a number written before the run (e.g. ≤ 512 B), or the row cannot fail · Right reason: the proxy's byte count of the received header block is ground truth; the three cookie sizes bracket the budget · Residue: three scratch `nosend` payments — designed by controls-F (Opus), 2026-09-28 | Proxy-side byte count of the received header block vs the shell's logged number, for three cookie sizes (small / near budget / over) | T2 | ⬜ |
| `P2-A2a` | 📏 **Mechanism measured, not assumed.** Record why `/payment-failed` did not appear on 2026-09-17. Hypothesis from code reading today: `Async402ResourceHandler::GetResponseHeaders` hands CEF the upstream status (431/400) **with the upstream body**, so the navigation **commits** an error-status document and `SimpleHandler::OnLoadError` (branch 2, `ConsumeBrc121FailedUrl`) never fires — the swap runs only on a load **error**. Cite `CefLoadHandler::OnLoadError`'s contract (`cef_load_handler.h`) and Chromium's rule for HTTP-error responses with a body before building on it (rule 4) | Diagnosis row (written here): on the rig with today's binary, the tab log must show the paid retry's 431/400 **and no** `swapping failed-load for /payment-failed` line, and the tab shows the proxy's text — the defect reproduced. If instead the swap line **does** appear, the hypothesis is wrong and A2b's design changes | Tab browser log + CDP screenshot of the **tab** browser (not an overlay) + the header citations | T2 | ⬜ |
| `P2-A2b` | **The Hodos page reaches the screen.** On a final paid-retry refusal (A1a) the tab shows `/payment-failed` with the domain, sats, status and — from P1 — the outcome and txid; the upstream body is not shown | UI row (written here): revert the fix ⇒ the rig shows the upstream 431 text again (A2a's screenshot). Also: a refusal on a **non-paid** navigation must be unaffected (the swap is keyed on the failed-URL registry — a normal 404 page still renders the site's own 404) | CDP screenshot of the tab + the tab's committed URL; a control navigation to a plain 404 on the demo server | T2 + **T3 human** | ⬜ |
| `P2-A2c` | **The route is the paid URL's, and only once.** The registry entry is consumed once; a reload of the Hodos page does not re-trigger a payment silently (it goes through `pay_402` → P1's checks) | UI/flow row (written here): leave the entry un-consumed ⇒ a second unrelated load of the same URL shows the failure page — observe it, then restore | Tab log (`ConsumeBrc121FailedUrl` hit count per URL); `transactions` count for the URL | T2 | ⬜ |

**Two-sided pair:** A1a ⇄ A1b.

### 4a. Independent control notes (2026-09-28)

- `P2-A1a` / `P2-A1c` — ⭐ **The rig may not reach 431.** `Async402ResourceHandler` issues the paid retry as a fresh `CefURLRequest` with `SetFlags(UR_FLAG_DISABLE_CACHE)` only; `cef_types.h` says cookies are sent (and saved) only with `UR_FLAG_ALLOW_STORED_CREDENTIALS`. Unless the cookie was captured into `ctx_.originalHeaders`, the demo page's oversized cookie is not on the paid request. Suggested: the proxy logs the Cookie header size on the paid request before any A1 run; if absent, inflate via a forwarded header. ⚠️ Product side (code reading, not measured): the same flag means paid retries may go out **without the site's cookies**, contradicting the code comment that the cookie jar is shared — worth one measurement, since a site that ties payment to a session would see an anonymous request.
- `P2-A1c` — "the header block the network stack emits" may be unobservable from the shell: headers the network service adds after CEF (Cookie from the jar, `Host`, `Accept-Encoding`, `Sec-CH-UA*`) are not in the `HeaderMap` the shell set. Cite the CEF contract for what is visible (rule 4) and write the ± framing tolerance as a number; otherwise the row cannot fail.
- `P2-A1b` — make sure the rig's one-shot 431 lands on the **paid** request, not on the first unpaid fetch that returns the 402; otherwise A1b measures the pre-payment path.

## 5. Blast radius

All verified 2026-09-28 by Read/Grep at `65869b1`.

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `cef-native/src/core/HttpRequestInterceptor.cpp :: Async402ResourceHandler::onUpstreamComplete` | ✅ | `bool retryable = (status == 431) || (status >= 500 && status < 600);` then `retryAttempts_ < MAX_UPSTREAM_RETRIES` ⇒ re-sends the same context after `RETRY_DELAY_MS`. The comment still justifies a 431 retry as transient |
| `… :: Async402ResourceHandler::MAX_UPSTREAM_RETRIES` (= 1) | ✅ | Unchanged; only the 431 arm is narrowed |
| `… :: Async402ResourceHandler::GetResponseHeaders` | ✅ | When `responseStatus_ > 0`: `SetStatus`, `SetHeaderMap(responseHeaders_)`, MIME from upstream `content-type` (defaults to `text/html`) — the basis of A2a's hypothesis |
| `… :: RegisterBrc121FailedUrl`, `s_brc121_failed_urls` | ✅ | Registered on every non-2xx final refusal |
| `… :: TryHandleBrc121_402`, the `x-bsv-beef` / `x-bsv-*` header insert | ✅ | Where the block is assembled; A1's measurement point |
| `cef-native/src/handlers/simple_handler.cpp :: SimpleHandler::OnLoadError` (branch 2, `ConsumeBrc121FailedUrl`) | ✅ | Builds `http://127.0.0.1:5137/payment-failed?domain&sats&status&originalUrl`. `127.0.0.1:5137` is the frontend origin in dev **and** release (`include/core/PortConfig.h` comment: not a backend port, no dev offset) — same construction as the sibling `/payment-pending` URL |
| `frontend/src/pages/PaymentFailedPage.tsx` | ✅ | Shared with P1 (copy) |
| `rust-wallet/src/handlers.rs :: BRC121_MAX_BEEF_BYTES` (64 KiB), `large_parent_bytes_for_budget`, `pay_402`'s pre-mint size refusal (422, owner-approved message) | ✅ | Untouched by P2; A1 is a second, shell-side measurement |
| `demos/brc121-402/server.js` | ✅ | Rig host; no production effect |

## 6. Out of scope

- ⛔ **Body transport for large BEEF (Q6, decision 9): deferred, no build.** Re-check condition — any one of: a reply on BRCs **#261** (open, 1 comment — ours, last update 2026-09-18, checked 2026-09-28); a change to BRC-121's text adding a transport or a size bound; a live BRC-121 server advertising `x-bsv-payment-transports`. ⚠️ The ticket file still reads "⬜ UNASSIGNED / waits on an upstream answer" — adding this re-check condition to the ticket is a shared-doc edit for the ticket's owner, not made here.
- ⛔ **x402 adapter** — deferred (decision 9).
- Raising or lowering `BRC121_MAX_BEEF_BYTES`, or changing `prefer_small_parents` — the wallet's pre-mint refusal is correct and stays.
- Trimming BEEF ancestry to fit (forbidden by R-BEEFOUT).
- Consolidating large parents so a wallet can pay again — **B5-T1** (`TICKET_wallet_cannot_shed_large_parents.md`).
- PeerPay's size limit (a different transport, already handled by beta.3 P10d).

## 7. Rollback

Revert the commit: the 431 arm returns to `retryable = (status == 431) || 5xx`; the page-routing change is additive and reverts with it.

## 8. Pre-mortem (adversarial review — before)

| # | Failure story | Caught by |
|---|---|---|
| 1 | The measurement counted only the `x-bsv-*` headers; a site with 20 KB of cookies still got a futile retry (the P10d trap, an order of magnitude tighter) | `P2-A1c` |
| 2 | The narrowed arm also stopped a **genuinely transient** 431 from retrying, so pages that used to load now fail and trigger P1's release path | `P2-A1b` |
| 3 | The fix was built on the `OnLoadError` hypothesis without measuring it; the real cause was elsewhere (e.g. the registry keyed on a URL that differs after redirect), and the page still never showed | `P2-A2a` (measure first; if the swap line appears, re-design) |
| 4 | The Hodos page replaced **every** non-2xx paid response, including a site's legitimate paid 404 page with useful content | `P2-A2b` control navigation; design keys on the failed-URL registry, not on status |
| 5 | The failure page's Retry silently re-paid | `P2-A2c` + P1's A3 checks |
| 6 | The fix worked on the dev build and not on a release build (different frontend serving) | A2b's T3 run is on a release build |
| 7 | Evidence gathered on Windows only | §9 |
| 8 | The 431 path's failure page promised "consolidating your coins" that T1 has not built | §2 last line; copy reviewed in A2b's T3 run |

### 8a. Prior art to read at kickoff (added 2026-09-30, from the owner's morning report)

⛔ **A reading assignment, not a scope change.** Log it in `../../../PRIOR_ART.md` (rule 5).

**BRC-105 amendment, 2026-09-29.** Spec `https://github.com/bsv-blockchain/BRCs/blob/master/payments/0105.md`, rationale in BRCs PR #285. Unread by a T4 session; the figures are the PR author's measurements, not ours.
- A new optional 402 response header, `x-bsv-payment-known-txids`, lists transactions the service already holds, and the payer may omit them from the payment's ancestry. The PR reports the payment header staying **flat at 594 bytes** instead of growing **302 bytes per payment**, and payers hitting a **hard stop at 13 consecutive payments** without it. ⭐ That growth is the same pressure that produces this phase's oversized-header 431.
- ⛔ **It collides with `R-BEEFOUT`** (§3: trimming ancestry is forbidden). The amendment is a protocol-sanctioned exception to trimming, limited to txids the service has **declared** it holds. If it is ever adopted, `R-BEEFOUT` needs that exception written in, not quietly relaxed, and the amended spec's §8 item 6 failure (an omitted ancestor the service cannot resolve ⇒ refused after broadcast) becomes a `B5-T4-P1` row (its §8a).
- ⚠️ We are a **BRC-121** payer; the header is BRC-105's. Adoption waits on the same place as body transport: our BRCs issue #261. 👤 Owner's call; this phase does not implement it.
- Related fixes cited in the PR: ts-stack #660 (payer), go-wallet-toolbox #1049 (service). Unread.

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | `P2-A1a`, `A1b`, `A2a`, `A2b` | Shared C++ (`HttpRequestInterceptor.cpp`, `simple_handler.cpp`). 🍎 **Relay note owed** in `MAC_RELAY_BETA5.md` naming both files. Any page-routing change must work in the macOS shell's load-error path too (invariant 9) |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **≈ 0.5 h** — one look at the Hodos failure page on the rig and on a release build (A2b T3). The real-world 431 (large-parent wallet from 2026-09-17) is **not** required: the rig inflates the page's own headers instead |
| Human-bound rows | `P2-A2b` T3 half (the page is legible and says the right thing) |
| Unknowns (K) — uncertainty, not difficulty | **K = 1** — A2a's mechanism (the hypothesis is now supported by today's code reading of `GetResponseHeaders`, but not measured) |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| gives | **B5-T4-P1** | The route by which P1's three-state page actually reaches the screen (P1-A4a depends on A2b). **Order: P2-A2a (measure) can run first; P1-A4 and P2-A2b land together or P2 first** |
| needs | **B5-T1** (`TICKET_wallet_cannot_shed_large_parents.md`) | The consolidation the owner-approved 422 message alludes to. P2 must not make that promise louder until T1 ships it |
| needs | **B5-T0 Build 1** | Browser-level evidence on the refreshed engine |
| gives | **B5-T6** | Nothing new; the failure page is an existing route, not a new overlay |

## 12. Open questions for the owner

None — decision 9 (Q6) settles the only owner-level question. If `P2-A2a` disproves the hypothesis, the redesign is agent-level and recorded here before code.

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
