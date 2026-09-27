# TICKET — BRC-121 re-mints a NEW payment on every retry, and "funds preserved" is not a guarantee

> 🔎 **Re-verified 2026-09-25 (code reading, nothing run) — KEEP OPEN, NARROWED.**
> ✅ Retries **within 25 s** reuse the in-flight payment: the reuse query was fixed (`a337538` — it read a column that
> no longer existed) and the reuse/release race closed (`1ec1b8f`); see `handlers.rs :: pay_402`, `PAY402_REUSE_TTL_MS`.
> **Remaining:** (1) a **"possibly paid"** outcome — no response (`status == 0`), or a retry after 25 s — still mints a
> **new** payment even if the server already broadcast the first (this ticket's 4,600-sat scenario); (2) the log and UI
> still say *"NOT broadcasting (funds preserved)"* and nothing tells the user *"delivery unknown — you may have paid"*;
> (3) Option 2 (treat no-response as unknown, stop auto-retrying) not done; (4) Phase 11 row A3's known leftover — a
> second navigation can mint one extra, never-broadcast payment. ⚠️ beta.3's A1–A8 evidence was **Windows-only**, and the
> real-money row (A5) did not exercise the no-response-then-late-retry case. Note the 25 s reuse window sits close to
> BRC-121's 30 s freshness window (a reuse at ~33 s was rejected as stale).
> ⚠️ Related, **unverified**: `TICKET_brc121_release_restores_inputs_the_server_may_have_spent.md`.


**Found:** 2026-08-10, during the P6 money-path row (owner-driven test against
`now.bsvblockchain.tech`).
**Status:** OPEN. **Deliberately deferred** — owner decision 2026-08-10 to stay on the 0.4.0
sprint. Not scheduled. Do not start without sign-off.
**Severity:** real money, small amounts today, scales with payment size.

## What happened, measured

Eight paid retries against one flaky origin. **Six of them were for the SAME article**
(`/articles/agentpay-hackathon`), each minting a **fresh** 150-sat payment + the 1000-sat
service fee.

| txid | C++ decision | on chain | DB status |
|---|---|---|---|
| `1f8a2e4e` | 502 → NOT broadcasting | no | failed |
| `50c686f9` | 400 → NOT broadcasting | no | failed |
| `905af977` | status=0 → **NOT broadcasting** | **YES** | completed |
| `a15b58dd` | status=0 → **NOT broadcasting** | **YES** | completed |
| `5e07c6c5` | status=0 → **NOT broadcasting** | **YES** | completed |
| `1ef1e368` | status=0 → **NOT broadcasting** | **YES** | completed |
| `1973e4c5` | 200 → broadcast | YES | completed |
| `5b1d1477` | 200 → broadcast | YES | completed |

On-chain status verified against WhatsOnChain with a positive control (a known-broadcast
txid returns a body; the two `failed` ones return nothing).

**Net: 4 × 1,150 = 4,600 sats paid for an article that never rendered.**

## The two findings, and the second is the important one

### 1. Every retry mints a new payment instead of reusing the outstanding one

Cost scales linearly with retries. A slow origin — Cloudflare reported `cfOrigin;dur` of
**10,295 ms** and **19,939 ms** on the two attempts that did succeed — reliably produces
retries. Six mints for one article.

### 2. ⛔ "NOT broadcasting (funds preserved)" does not preserve funds

`pay_402` hands a **fully-signed BEEF to the server** as the payment. Once transmitted, the
**payee** holds a broadcastable transaction and can broadcast it whenever it likes — which is
exactly how BRC-29/BRC-121 is supposed to work. `status=0` means *we* never received the
response, **not** that the server never received the request. Four of them prove the server
both received and broadcast.

So the log line means only "*we* did not broadcast". It is not a statement about whether the
money moved, and it should not be read as one. `task_check_for_proofs` later observed these
in the mempool (`SEEN_ON_NETWORK`) and settled the DB row to `completed` — correctly, since
they really are on chain. Nothing is "restored" because nothing was recoverable.

## Options (none evaluated in depth — this is a starting point, not a plan)

1. **Reuse the outstanding payment per URL.** Keep a registry of minted-but-unsettled
   payments keyed by URL; on retry, resend the *same* BEEF rather than minting. Closest to
   correct, and matches the fact that the server may already hold it.
2. **Treat `status=0` as "delivery UNKNOWN — possibly paid"** and stop auto-retrying, surfacing
   it to the user instead of silently paying again.
3. **Probe reachability before minting.** Reduces but does not remove the window — the failure
   happens *after* transmission.

⚠️ Whatever is chosen, the fix likely lands in `pay_402` (`rust-wallet/src/handlers.rs`) and is
therefore **invariant #2/#3 territory** — present evidence and get approval before editing.

## Related, already fixed 2026-08-10

`Async402HTTPClient::OnRequestComplete` never read `GetRequestError()`, so every `status=0`
was undiagnosable — a timeout, a reset and an abort logged identically. It now logs the
`cef_errorcode_t`. That does not fix this ticket, but the next occurrence will at least say
*why* the response never arrived, which bears directly on option 2.

Related: `development-docs/0.4.0/IMPLEMENTATION_ROADMAP_0_4_0_BETA1.md` §7 money-path row.
