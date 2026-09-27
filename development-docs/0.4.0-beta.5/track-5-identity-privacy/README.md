# B5-T5 — Identity & privacy

**Status:** 📌 PROPOSED at G2 (2026-09-25) · **Scope doc:** owed at G2 — goal, integration check, telescope (re-fetch current BRCs/SDKs first), kaleidoscope

**Goal.** A site learns only the keys, identity and wallet surface the user chose to give it; servers prove who they are (BRC-103/104); and we can count users without identifying them.

## Tickets

Tickets stay in `../tickets/` (planning note D11); this list mirrors the register's Track column.

- [`TICKET_dapp_reachable_surface_is_a_denylist_not_an_allowlist.md`](../tickets/TICKET_dapp_reachable_surface_is_a_denylist_not_an_allowlist.md) — 🚨 An approved dApp reaches 30 internal wallet routes that cannot gate themselves, because exposure is a deny-list
- [`TICKET_derived_public_keys_have_no_prompt_and_can_match_across_sites.md`](../tickets/TICKET_derived_public_keys_have_no_prompt_and_can_match_across_sites.md) — 🔑 A connected site can fetch a derived public key with no prompt, and two sites can be handed the same one
- [`TICKET_well_known_auth_returns_a_key_it_cannot_sign_for.md`](../tickets/TICKET_well_known_auth_returns_a_key_it_cannot_sign_for.md) — 🔐 The wallet's own `/.well-known/auth` answers with an identity key that does not match its signature
- [`TICKET_wallet_backend_is_shared_across_os_accounts.md`](../tickets/TICKET_wallet_backend_is_shared_across_os_accounts.md) — 🔐 A second OS account's browser uses — and on macOS shuts down — the first account's wallet
- [`TICKET_active_user_count_without_identifying_users.md`](../tickets/TICKET_active_user_count_without_identifying_users.md) — 🔬 We cannot say how many people use Hodos, and the only number we have counts machines updating
- [`TICKET_brc103_server_identity_unverified.md`](../tickets/TICKET_brc103_server_identity_unverified.md) — 🔴 AuthFetch accepts the server's identity key without verifying it
- [`TICKET_derivation_params_unbound_to_origin.md`](../tickets/TICKET_derivation_params_unbound_to_origin.md) — TICKET — key-derivation parameters are taken from the app, unbound to the requesting origin
- [`TICKET_two_next_address_index_sources_can_reuse_addresses.md`](../tickets/TICKET_two_next_address_index_sources_can_reuse_addresses.md) — 🏷️ Two sources for the next address index can drift apart and reuse an address
- [`TICKET_createSignature_requires_counterparty.md`](../tickets/TICKET_createSignature_requires_counterparty.md) — TICKET — `createSignature` rejects a request with no `counterparty` (suspected BRC-100 conformance gap)
- [`TICKET_wallet_bridge_plumbing_is_advertised_to_every_site.md`](../tickets/TICKET_wallet_bridge_plumbing_is_advertised_to_every_site.md) — TICKET — the wallet bridge's **plumbing** is advertised to every https site, by name
- [`TICKET_deleting_a_site_in_the_advanced_wallet_does_not_reach_the_browser.md`](../tickets/TICKET_deleting_a_site_in_the_advanced_wallet_does_not_reach_the_browser.md) — 🔁 Deleting a site in the advanced wallet does not reach the browser — it keeps treating the site as approved

## Existing material

- 👤 Active-user count (D9): **adopt the best-privacy industry-standard approach — read how Brave does it — and do NOT design our own.** Document honestly how private it is and is not, for us and for users; redesign later only if we choose to
