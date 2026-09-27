# B5-T4 — 402 payments

**Status:** 📌 PROPOSED at G2 (2026-09-25) · **Scope doc:** owed at G2 — goal, integration check, telescope (re-fetch current BRCs/SDKs first), kaleidoscope

**Goal.** An x402 adapter over our BRC-121 client (x402 is BRC-121 in a different envelope — `pay_402` is already byte-compatible), plus the open 402 defects.

## Tickets

Tickets stay in `../tickets/` (planning note D11); this list mirrors the register's Track column.

- [`TICKET_brc121_client_has_no_body_transport_for_large_beef.md`](../tickets/TICKET_brc121_client_has_no_body_transport_for_large_beef.md) — 📦 Our 402 client can only carry a payment in a header, and the spec it speaks has no body path
- [`TICKET_brc121_remint_on_retry.md`](../tickets/TICKET_brc121_remint_on_retry.md) — TICKET — BRC-121 re-mints a NEW payment on every retry, and "funds preserved" is not a guarantee
- [`TICKET_brc121_beef_header_exceeds_100kb_and_payment_is_lost.md`](../tickets/TICKET_brc121_beef_header_exceeds_100kb_and_payment_is_lost.md) — A BRC-121 payment is minted, then cannot be delivered: the BEEF header exceeds 100 KB
- [`TICKET_brc121_release_restores_inputs_the_server_may_have_spent.md`](../tickets/TICKET_brc121_release_restores_inputs_the_server_may_have_spent.md) — 🚨❔ When a 402 server refuses after a payment, the wallet releases it without asking the chain — its coins may already be spent

## Existing material

- `X402_INTEGRATION.md` — research complete; the 2026-08-19 decision record (§10): build items 1, 2, 3, 5, 6; hold 4 (strict amount equality) and 7 (reuse cache vs freshness)
