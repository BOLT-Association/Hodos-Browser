# beta.5 tickets

**Opened:** 2026-08-29.

This folder replaces beta.3's flat `TICKET_*.md` at folder root. The difference is not cosmetic:

> ⛔ **A ticket is not work until the owner has assigned it to a track.**
> This folder is a **review queue**, not a backlog anyone picks from.

---

## How it works

1. Anyone (session or owner) files a ticket here from `TICKET_TEMPLATE.md`.
2. It sits at **Track: unassigned**.
3. **The owner reviews and assigns it** to a track, to the misc bucket, or closes it.
4. Only then does it become work, and only inside the track it was assigned to.

⚠️ The owner has items on paper that are not here yet. ⛔ **Do not chase them.** They arrive when he
files them.

## Naming

`TICKET_<short_snake_case_description>.md` — the beta.3 convention, kept, because it greps well and
existing links use it. Describe the **symptom or the defect**, not the fix:
`TICKET_recovery_sweep_ignores_classification.md`, not `TICKET_add_classification_to_sweep.md`.

## Status values

| Status | Meaning |
|---|---|
| ⬜ **UNASSIGNED** | Filed, not yet reviewed by the owner. The default. |
| 📌 **ASSIGNED** | Owner has put it in a track. Names which one. |
| 🚧 **IN PROGRESS** | Being worked, inside its track's phase structure. |
| ✅ **CLOSED** | Fixed, with evidence, or closed with a written reason. |
| ❄️ **DEFERRED** | Deliberately not now, **with a re-check condition** — the `WATCH_fungibles.md` pattern. A deferral with no condition is a ticket rotting. |

## Rules

- ⛔ **State method.** Say whether a claim is a **code reading** or a **measurement**
  (`HARNESS.md` §8). Both are legitimate; mislabelling one as the other is not. The dust-consolidator
  ticket does this well — it opens with a method note naming the one thing it did *not* verify.
- ⛔ **Say what you did not check.** A ticket that reads as complete when it is partial is worse than
  a short one.
- **Every ticket that proposes a fix proposes its negative control.** Per `HARNESS.md`: not done
  until the check has been *seen* to fail.
- **Do not size a fix you have not scoped.** "Small" is a claim.

## Related tickets living elsewhere

| Ticket | Where | Why it is not here |
|---|---|---|
| `TICKET_token_outputs_destroyed_by_dust_paths.md` | 📌 **ASSIGNED 2026-09-27** | ✅ **Closed** (2026-09-25 review): its minimal defensive floor shipped in beta.3 (`383bf4f`, Phase 8a). The **full classification guard** it motivated is beta.5 Track 1 scope, not an open ticket. Stays with the beta.3 folder. |

---

## Register

| Ticket | Status | Track | Filed |
|---|---|---|---|
| `TICKET_e2e_specs_wrong_subject_and_never_run.md` | 📌 **ASSIGNED 2026-09-27** | **Background — instruments** | 2026-08-29 |
| `TICKET_dapp_reachable_surface_is_a_denylist_not_an_allowlist.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T5 Identity & privacy** — (⭐ suggest track 1) | 2026-09-19 |
| `TICKET_brc100_consent_model_diverges_from_1sat_wallet_api.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T2 1Sat Ordinals** — (⭐ suggest track 2) | 2026-09-21 |
| `TICKET_derived_public_keys_have_no_prompt_and_can_match_across_sites.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T5 Identity & privacy** | 2026-09-21 |
| `TICKET_well_known_auth_returns_a_key_it_cannot_sign_for.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T5 Identity & privacy** | 2026-09-21 |
| `TICKET_brc121_client_has_no_body_transport_for_large_beef.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T4 402 payments** — (decision at the microscope pass; waits on BRCs #261) | 2026-09-23 |
| `TICKET_mkcert_dev_private_key_is_tracked_and_public.md` | 📌 **ASSIGNED 2026-09-27** | **Background — hygiene** — (🟡 low; regenerate, do NOT rewrite history) | 2026-09-23 |
| `TICKET_chromium_default_debug_log_lands_in_install_root.md` | ✅ **CLOSED 2026-09-25** | — (fixed `9559191`, shipped in beta.4) | 2026-09-23 |
| `TICKET_g1_gate_cannot_see_chromiums_own_log_target.md` | 📌 **ASSIGNED 2026-09-27** | **Background — instruments** — (⭐ the gate PASSED through the defect it was written for) | 2026-09-23 |
| `TICKET_update_is_visible_when_it_should_not_be.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** — (👤 owner-raised; ⭐ tiers 1+2 give Chrome's UX without Chrome's architecture) | 2026-09-24 |
| `TICKET_wallet_backend_is_shared_across_os_accounts.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T5 Identity & privacy** — (🔐 measured once on macOS; mechanism is code reading) | 2026-09-24 |
| `TICKET_active_user_count_without_identifying_users.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T5 Identity & privacy** — 👤 owner wants it **this release** (marketing, fundraising); a phase: *count users without identifying them* | 2026-09-17 |
| `TICKET_brc103_server_identity_unverified.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T5 Identity & privacy** — (🔴 AuthFetch trusts the server's claimed key) | 2026-09-16 |
| `TICKET_brc140_key_shares_vs_bip39.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T3 Backup & sync (research phase)** — (🔬 research-and-decide) | 2026-09-16 |
| `TICKET_menu_exit_closes_primary_not_the_clicked_window.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** | beta.3 Phase 3.5 |
| `TICKET_multiwindow_session_restore_loses_all_but_last_window.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** | beta.3 Phase 3.5 |
| `TICKET_split_view_needs_multi_visible_tab_model.md` | ❄️ **DEFERRED 2026-09-27** | **next release** (decision 14) — re-check: after T6-P2 session restore lands, own scoping run | beta.3 Phase 4 |
| `TICKET_tab_pin_and_mute_need_model_changes.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** | beta.3 Phase 4 |
| `TICKET_window_scoped_work_uses_process_globals.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** | beta.3 Phase 3 |
| `TICKET_derivation_params_unbound_to_origin.md` | ✅ **CLOSED 2026-09-27 — duplicate** | — merged into `derived_public_keys_have_no_prompt…` (T5 SCOPE Q3; its three unique points carried there at G3) | 2026-09-02 |
| `TICKET_chrome_import_bookmarks_history_passwords.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** — slices A + B (bookmarks, history) only; **slice C passwords ❄️ deferred to the next release, CSV only** (decision 14) | 2026-09-02 |
| `TICKET_brc121_remint_on_retry.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T4 402 payments** — ✅ verified 2026-09-25: **still open, narrowed** (possibly-paid case; see file) — (real money; beta.3's `TICKET_brc121_paid_retry_aborts_and_mints_a_payment_each_time.md` was **closed** by Phase 11 item 11 — in-flight payment reuse — which likely closes this too. **Verify, then close with evidence**) | 2026-08 |
| `TICKET_debug_log_unfiltered_in_production.md` | 📌 **ASSIGNED 2026-09-27** | **Background — hygiene** — ✅ verified 2026-09-25: **still open, narrowed** (a few INFO URL lines; macOS log pruning) — (beta.3 `CRITICAL_UPDATES.md` reads it as fixed by `fa0c143`, and beta.3's twin `TICKET_production_debug_logging_unbounded.md` is **closed** by `fa0c143` + `c3604f9` — **verify, then close with evidence**) | 2026-08 |
| `TICKET_knowledge_and_memory_architecture.md` | ✅ **CLOSED 2026-09-25 — adopted** | — (now `../../KNOWLEDGE_AND_MEMORY.md`; open items in its §6) | 2026-09-08 |
| `TICKET_logged_in_screenshots_in_public_history.md` | ✅ **CLOSED 2026-09-25 — accepted risk** | — (owner: content is public anyway; no history rewrite) | 2026-08-13 |
| `TICKET_profile_lock_misreports_missing_dir.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** — (small) | 2026-08 |
| `TICKET_reservation_ownership_converge_on_spent_by.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T1 Money path** — 📌 placed as **Track 0.5** in `../README.md` (owner, 2026-09-15; to confirm at kickoff) | 2026-08-22 |
| `TICKET_engine_behind_its_own_cef_branch_and_upstream_stable.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T0 Engine** — (⭐ proposed as a track, likely Track 0; **target is an owner decision**) | 2026-09-24 |
| `TICKET_auto_unlock_accepts_another_wallets_mnemonic.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T1 Money path** — (🔑 **money**: a valid phrase from another wallet passes auto-unlock; from Wallet-Hardening) | 2026-09-25 |
| `TICKET_confirmed_tx_never_rechecked_after_reorg.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T1 Money path** — (money; medium, low probability; from Wallet-Hardening) | 2026-09-25 |
| `TICKET_token_outputs_lost_if_crash_between_broadcast_and_record.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T2 1Sat Ordinals** — (⭐ **settle before ordinals ship**; from Wallet-Hardening) | 2026-09-25 |
| `TICKET_fast_relaunch_attaches_to_dying_wallet_or_fails_port_bind.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** — (availability; from Wallet-Hardening FIX_B) | 2026-09-25 |
| `TICKET_two_next_address_index_sources_can_reuse_addresses.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T1 Money path** — moved from T5 (T1 Q7 = T5 Q10; same counter as the rescan ticket) | 2026-09-25 |
| `TICKET_db_lock_held_across_await_is_unenforced.md` | 📌 **ASSIGNED 2026-09-27** | **Background — instruments** — (hygiene; carries H-1/H-2/H-4 notes) | 2026-09-25 |
| `TICKET_dead_cert_tx_builder_marks_coins_spent_on_404.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T1 Money path** — (hygiene; dead code, grep-verified only) | 2026-09-25 |
| `TICKET_final_mvp_efficiency_leftovers_never_scheduled.md` | 📌 **ASSIGNED 2026-09-27** | **Background — split at triage (fuzzing → instruments)** — (five items; ⭐ fuzz testing's "post-launch" trigger has fired) | 2026-09-25 |
| `TICKET_appcast_missing_minimum_system_version.md` | ✅ **CLOSED 2026-09-25** | — (owner: no pinned Big Sur feed item; live feed already declares macOS 12.0) | 2026-08-17 |
| `TICKET_brand_remaining_permission_prompts.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** — (moved from beta.3; 21 prompts still stock Chrome UI) | beta.3 |
| `TICKET_brc121_beef_header_exceeds_100kb_and_payment_is_lost.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T4 402 payments** — (moved from beta.3; HTTP 431 still retried; money) | beta.3 |
| `TICKET_bulk_utxo_sync_truncates_at_20_per_address.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T1 Money path** — (moved from beta.3; ⚠️ **money, high**: wallet sees only 20 coins per address) | beta.3 |
| `TICKET_cdp_port_open_in_release.md` | 📌 **ASSIGNED 2026-09-27** | **Background — two release checks** — (moved from beta.3; macOS installed-build check + F12 row) | beta.3 |
| `TICKET_cef_file_thread_ids_share_one_thread.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell (measure first)** — (moved from beta.3; leave until a stall is measured) | beta.3 |
| `TICKET_createSignature_requires_counterparty.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T5 Identity & privacy** — (moved from beta.3; check against the spec first; signing code — invariant 3) | beta.3 |
| `TICKET_edit_limits_modal_usability.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** — ⛔ **item 4 dropped** (owner 2026-09-27: do not store a declined permission; the user is simply re-prompted) | beta.3 |
| `TICKET_loopback_host_form_wallet_routing.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T5 Identity & privacy** — moved from T1 (T1 SCOPE Q3; rows W4/W6/W7/W8 are trust-boundary work) | beta.3 |
| `TICKET_modal_info_tooltip_overflows_modal.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** — (moved from beta.3; low) | beta.3 |
| `TICKET_signaction_response_not_brc100_shape.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T1 Money path** — (moved from beta.3; ⚠️ **money**: a fatal broadcast failure returns success) | beta.3 |
| `TICKET_synced_outputs_store_a_fabricated_locking_script.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T1 Money path** — (moved from beta.3; ⭐ would fool the Track 1 classifier) | beta.3 |
| `TICKET_transaction_row_can_sit_at_created_while_its_coin_is_on_chain.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T1 Money path** — (moved from beta.3; reasoned, not measured) | beta.3 |
| `TICKET_wallet_bridge_plumbing_is_advertised_to_every_site.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T5 Identity & privacy** — (moved from beta.3; fingerprint surface; medium) | beta.3 |
| `TICKET_wallet_cannot_shed_large_parents.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T1 Money path** — (moved from beta.3; money; medium → high) | beta.3 |
| `TICKET_wallet_quiet_detector_blind_to_long_polls.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** — (moved from beta.3) | beta.3 |
| `TICKET_brc121_release_restores_inputs_the_server_may_have_spent.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T4 402 payments** — ❔ **unverified hypothesis**, money; ground-truth check against the chain is step 1 | 2026-09-25 |
| `TICKET_rescan_cannot_find_payments_to_generated_addresses.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T1 Money path** — the scan; its button is Tools-tab card 2 (T6). ⚠️ today's rescan scans BIP32 while generated addresses are BRC-42 | 2026-09-25 |
| `TICKET_prompt_opens_behind_another_window_with_two_windows_open.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** — from the 2026-09-25 beta.4 scare (Lead C); buggy, not broken | 2026-09-25 |
| `TICKET_deleting_a_site_in_the_advanced_wallet_does_not_reach_the_browser.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T5 Identity & privacy** — consent surface (Lead B); IPC role gate blocks our own wallet tab | 2026-09-25 |
| `TICKET_find_bar_does_not_follow_tab_switches.md` | 📌 **ASSIGNED 2026-09-27** | **B5-T6 Browser shell** — adopt Chromium's per-tab find state | 2026-09-27 |

👤 **Owner approved every proposed placement 2026-09-27** (G2 sitting) — the Track column is the assignment (D11); the ticket files' own `Status: UNASSIGNED` headers are superseded by this register.

📏 **Reconciled 2026-09-27: 57 tickets, 57 rows** (the last five filed during triage and the 2026-09-25 beta.4 scare). 28 from the 2026-09-24 consolidation (19 with the
old beta.4 folder, 1 loose, 1 from the old beta.5 folder, 6 from `development-docs/` root, 1 engine
bump) · **16** still-open tickets moved from `../../0.4.0-beta.3/` after a file-by-file review · **7**
from the Wallet-Hardening review · **1** from the Final-MVP-Sprint review.
`../track-6-browser-shell/TOOLS_TAB_claim_a_payment.md` is a **feature outline with recorded owner decisions**, not a
ticket, and stays at the release root — a beta.3 phase contract links to it there.

📏 **The beta.3 review, for the record:** of 51 files, 16 were open (moved here), **17 read "open" but
were already fixed** — each now carries a *CLOSED — 2026-09-25 open-ticket review* note with its
evidence, and stays in beta.3 — 16 were closed, 1 accepted as no-fix, 1 was a triage record. ⚠️ Rows
marked 🟡 *leftover* are fixed except for a named remainder, stated at the top of each file.

⚠️ `TICKET_e2e_specs_wrong_subject_and_never_run.md` is **second-hand** — reported by research (c),
**not independently verified**. Its first proposed step is verification. That is deliberate: a ticket
may record an unverified report as long as it says so.

⚠️ `TICKET_dapp_reachable_surface_is_a_denylist_not_an_allowlist.md` is **code reading, not
measurement** — the owner was using the app, so nothing was executed. Its RED is written to be run
first, before the fix, because it is also what confirms the finding.
