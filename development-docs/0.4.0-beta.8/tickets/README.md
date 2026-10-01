# beta.8 tickets — intake

Conventions follow `../../0.4.0-beta.7/tickets/README.md` (naming, statuses, method notes, negative
controls). ⛔ A ticket is not work until beta.8 planning assigns it to a track.

## Register

| Ticket | Status | Track | Filed |
|---|---|---|---|
| `TICKET_cicd_pipelines_unreviewed_and_nothing_builds_on_push.md` | ⬜ UNASSIGNED | — | 2026-09-25 |
| `TICKET_engine_move_to_cef_160_long_term_branch.md` | ⬜ UNASSIGNED | — | 2026-09-27 |
| `TICKET_centbee_import_creates_a_wallet_with_no_pin.md` | ⬜ UNASSIGNED | — (website audit (Marston session); 🟠 recovery phrase stored in plaintext) | 2026-09-30 |
| `TICKET_always_notify_lets_sub_cent_payments_through_silently.md` | ⬜ UNASSIGNED | — (website audit (Marston session)) | 2026-09-30 |
| `TICKET_verifyLegacyMessage_ignores_identityKey.md` | ⬜ UNASSIGNED | — (website audit (Marston session)) | 2026-09-30 |
| `TICKET_wallet_secrets_in_memory_are_never_wiped.md` | ⬜ UNASSIGNED | — (website audit (Marston session)) | 2026-09-30 |
| `TICKET_wallet_error_bodies_are_not_the_brc100_envelope.md` | ⬜ UNASSIGNED | — (beta.6 G2 test; dApps lose typed errors) | 2026-09-30 |
| `TICKET_verify_calls_return_valid_false_instead_of_an_error.md` | ⬜ UNASSIGNED | — (beta.6 G2 test (B4, TSA-040); low) | 2026-09-30 |
| `TICKET_verifySignature_rejects_counterparty_self.md` | ⬜ UNASSIGNED | — (beta.6 G2 test; pair with beta.7's `createSignature_requires_counterparty`) | 2026-09-30 |
| `TICKET_nosend_outputs_flagged_phantom_within_seconds.md` | ⬜ UNASSIGNED | — (beta.6 G2 test; may break `noSendChange` chaining) | 2026-09-30 |
| `TICKET_abortAction_not_bound_to_originating_site.md` | ⬜ UNASSIGNED | — (beta.6 triage (TSA-042); needs a schema change, deferred by owner) | 2026-09-30 |
| `TICKET_messagebox_recipient_fees_unmeasured.md` | ⬜ UNASSIGNED | — (beta.6 triage (TSA-047); 🔬 research, owner: not urgent) | 2026-09-30 |
| `TICKET_commission_rows_deleted_when_an_action_is_signed.md` | ⬜ UNASSIGNED | — (beta.6 P5 review; measured 3 rows / 636 outgoing; accounting loss, not poisoning) | 2026-09-30 |
| `TICKET_live_reservation_placeholders_fails_open_on_poisoned_lock.md` | ⬜ UNASSIGNED | — (beta.6 P5 review; code reading) | 2026-09-30 |
| `TICKET_privacy_shield_cookie_toggle_snaps_back.md` | ⬜ UNASSIGNED | — (beta.6 Mac smoke M-01c; shared C++ off-by-one; display only, allowance applied) | 2026-10-01 |
