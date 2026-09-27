# B5-T6 — Browser shell

**Status:** 📌 PROPOSED at G2 (2026-09-25) · **Scope doc:** owed at G2 — goal, integration check, telescope (re-fetch current BRCs/SDKs first), kaleidoscope

**Goal.** Multi-window, tabs, import, update visibility and consent-UI defects — the browser around the wallet.

## Tickets

Tickets stay in `../tickets/` (planning note D11); this list mirrors the register's Track column.

- [`TICKET_update_is_visible_when_it_should_not_be.md`](../tickets/TICKET_update_is_visible_when_it_should_not_be.md) — TICKET — the user watches the update happen, twice, and neither moment needs to be visible
- [`TICKET_menu_exit_closes_primary_not_the_clicked_window.md`](../tickets/TICKET_menu_exit_closes_primary_not_the_clicked_window.md) — 🎫 Menu → Exit closes the primary window, not the window it was clicked in
- [`TICKET_multiwindow_session_restore_loses_all_but_last_window.md`](../tickets/TICKET_multiwindow_session_restore_loses_all_but_last_window.md) — 🎫 Session restore keeps only the last window's tabs — every other window's tabs are lost on quit
- [`TICKET_split_view_needs_multi_visible_tab_model.md`](../tickets/TICKET_split_view_needs_multi_visible_tab_model.md) — 🎫 Split view — our tab model allows exactly one visible tab per window
- [`TICKET_tab_pin_and_mute_need_model_changes.md`](../tickets/TICKET_tab_pin_and_mute_need_model_changes.md) — 🎫 Tab pin and SITE mute have no data model — deferred out of Phase 4
- [`TICKET_window_scoped_work_uses_process_globals.md`](../tickets/TICKET_window_scoped_work_uses_process_globals.md) — 🪟 Window-scoped work is performed against process-globals, so one window acts on another
- [`TICKET_chrome_import_bookmarks_history_passwords.md`](../tickets/TICKET_chrome_import_bookmarks_history_passwords.md) — TICKET — Chrome / Chromium-browser import (bookmarks, history, passwords)
- [`TICKET_profile_lock_misreports_missing_dir.md`](../tickets/TICKET_profile_lock_misreports_missing_dir.md) — TICKET — a MISSING profile directory is reported as "Profile is already in use by another instance"
- [`TICKET_fast_relaunch_attaches_to_dying_wallet_or_fails_port_bind.md`](../tickets/TICKET_fast_relaunch_attaches_to_dying_wallet_or_fails_port_bind.md) — 🔁 A fast relaunch can attach to a wallet that is shutting down, or lose the port race and start with no wallet
- [`TICKET_brand_remaining_permission_prompts.md`](../tickets/TICKET_brand_remaining_permission_prompts.md) — Brand the remaining 21 permission prompts
- [`TICKET_cef_file_thread_ids_share_one_thread.md`](../tickets/TICKET_cef_file_thread_ids_share_one_thread.md) — 🎫 Every `CefPostTask(TID_FILE_*)` in the browser process runs on ONE shared thread — a slow task stalls them all
- [`TICKET_edit_limits_modal_usability.md`](../tickets/TICKET_edit_limits_modal_usability.md) — Edit Limits modal (Approved Sites) — long, losable, and it discards work silently
- [`TICKET_modal_info_tooltip_overflows_modal.md`](../tickets/TICKET_modal_info_tooltip_overflows_modal.md) — TICKET — the info-icon tooltip in the permission modal overflows the modal and adds a dead scrollbar
- [`TICKET_wallet_quiet_detector_blind_to_long_polls.md`](../tickets/TICKET_wallet_quiet_detector_blind_to_long_polls.md) — The "is the wallet busy?" detector counts a long poll as silence
- [`TICKET_prompt_opens_behind_another_window_with_two_windows_open.md`](../tickets/TICKET_prompt_opens_behind_another_window_with_two_windows_open.md) — 🪟 With two browser windows open, a permission prompt opens BEHIND the other window — the site looks frozen
- [`TICKET_find_bar_does_not_follow_tab_switches.md`](../tickets/TICKET_find_bar_does_not_follow_tab_switches.md) — 🔍 The find bar does not follow a tab switch — the new tab shows no results until you retype

## Existing material

- `TOOLS_TAB_claim_a_payment.md` — an **outline** (not a ticket) for a Tools tab in the advanced wallet, starting with *Claim a payment* for PeerPay deliveries that never arrived; four owner decisions recorded
