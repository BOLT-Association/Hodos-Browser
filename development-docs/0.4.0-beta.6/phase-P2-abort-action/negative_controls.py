"""B6-P2 negative controls: apply one mutation to rust-wallet/src/handlers.rs (or services/collection.rs
for names ending "(COLL)"), run the P2 tests, record exactly which tests failed and with what message,
restore the file. Every mutation must turn red for its stated reason. REDs designed by a second agent
(Claude Opus 5.5, 2026-09-30) before the tests were final; the author added M3c and M4j.
Run from the repo root: python development-docs/0.4.0-beta.6/phase-P2-abort-action/negative_controls.py [name prefixes]
"""
import re, subprocess, sys, pathlib

H = pathlib.Path('rust-wallet/src/handlers.rs')
C = pathlib.Path('rust-wallet/src/services/collection.rs')
ORIG = {H: H.read_bytes().decode('utf-8'), C: C.read_bytes().decode('utf-8')}  # bytes: no CRLF rewrite

# Test names (all in handlers::abort_action_tests unless noted).
A1, A2, A3N, A3R, A3V = 'p2_a1_abort_of_an_unsigned_action_returns', 'p2_a2_non_abortable_actions_are_refused_and_untouched', \
    'p2_a3_nosend_is_released_only_when_the_chain_says_absent', 'p2_a3_status_change_during_chain_check_releases_nothing', \
    'p2_a3_only_not_found_is_absent'
A4, A4D, A4C = 'p2_a4_never_sent_actions_are_released', 'p2_a4_double_abort_isolation_and_no_pending_entry', \
    'p2_a4_action_with_a_spent_created_output_is_refused'
A5, A6, A7 = 'p2_a5_sign_action_after_abort_is_not_found', 'p2_a6_lookup_by_txid_and_errors_are_not_swallowed', \
    'p2_a7_sign_action_refuses_an_action_aborted_mid_flight'
U = 'p2_unanimous_not_found_only_when_every_provider_says_so'
A8 = 'p2_a8_abort_and_broadcast_nosend_wait_for_create_action_lock'

RELEASE = '(status, created output spendable, input spendable) after abort'

# (name, [(old, new), ...], test filter, tests that must fail, text that must appear)
MUTATIONS = [
    # P2-A1 — the pre-fix deadlock: re-take the DB mutex while holding it. Only A1 runs:
    # every other test that reaches the release would hang, not fail.
    ('A1 re-lock the DB mutex', [('let db = state.database.lock().map_err(|e| failed(&e))?;',
      'let db = state.database.lock().map_err(|e| failed(&e))?; let _again = state.database.lock();')],
     'p2_a1', [A1], 'abortAction HUNG'),
    ('A1b hold the DB lock across the chain call', [('    if status == "nosend" {\n',
      '    if status == "nosend" {\n        let _held = state.database.lock();\n')],
     'p2_a3_nosend', [A3N], 'DB lock was held during the chain call'),
    # P2-A2 — the gate
    ('A2a unproven abortable', [('[&str; 3] = ["unsigned", "unprocessed", "nonfinal"]', '[&str; 4] = ["unsigned", "unprocessed", "nonfinal", "unproven"]')],
     'p2_', [A2], 'status unproven'),
    ('A2b gate through from_str', [('!ABORTABLE_NEVER_SENT.contains(&status.as_str())',
      '!ABORTABLE_NEVER_SENT.contains(&crate::action_storage::TransactionStatus::from_str(&status).as_str())')],
     'p2_', [A2], 'status not-a-status'),
    ('A2c incoming allowed', [('    if !is_outgoing {\n', '    if false {\n')], 'p2_', [A2], 'incoming'),
    # P2-A3 — the chain check
    ('A3a any chain error = absent', [('Err(crate::services::IndexerError::NotFound) => AbortChainVerdict::Absent,',
      'Err(_) => AbortChainVerdict::Absent,')], 'p2_', [A3N, A3V], 'chain error'),
    ('A3b rejected/unknown = absent (check_tx_exists_on_chain semantics)',
     [('Err(crate::services::IndexerError::NotFound) => AbortChainVerdict::Absent,',
       'Err(crate::services::IndexerError::NotFound) => AbortChainVerdict::Absent,\n        Ok(s) if matches!(s.state, crate::services::TxState::Rejected | crate::services::TxState::Unknown) => AbortChainVerdict::Absent,')],
     'p2_', [A3N, A3V], 'rejected'),
    ('A3c nosend without the chain check', [('    if status == "nosend" {\n', '    if false {\n'),
      ('[&str; 3] = ["unsigned", "unprocessed", "nonfinal"]', '[&str; 4] = ["unsigned", "unprocessed", "nonfinal", "nosend"]')],
     'p2_', [A3N, A3R], 'in mempool'),
    ('A3d plain last-error semantics (COLL)', [('(Err(IndexerError::NotFound), Some(failure)) =>',
      '(Err(IndexerError::NotFound), Some(failure)) if false =>')], 'p2_', [U], '[2, 1, 1, 1]'),
    ('A3e a timed-out provider counts as a "no" (COLL)', [('failure = Some(format!("{}: soft timeout", provider.name()));', '')],
     'p2_', [U], '[3, 1, 1, 1]'),
    # P2-A4 — what is released
    ('A4a created outputs left spendable', [('let disabled = output_repo.disable_by_txid(&txid).map_err(|e| failed(&e))?;',
      'let disabled = 0;')], 'p2_', [A1, A3N, A4, A4D, A6], RELEASE),
    ('A4b placeholder reservation not restored', [('if let Some(ph) = placeholder.as_deref() {', 'if let Some(ph) = None::<&str> {')],
     'p2_', [A1, A3N, A4], RELEASE),
    ('A4c txid reservation not restored', [('        restored += output_repo.restore_by_spending_description(&txid).map_err(|e| failed(&e))?;\n', '')],
     'p2_', [A3N, A4, A6], RELEASE),
    ('A4d failed_at not set', [("SET status = 'failed', failed_at = ?1, updated_at = ?1", "SET status = 'failed', updated_at = ?1")],
     'p2_', [A1, A3N, A4, A6], 'failed_at not set'),
    ('A4e balance cache not invalidated', [('            state.balance_cache.invalidate();\n            log::info!("✅ Action aborted',
      '            log::info!("✅ Action aborted')], 'p2_', [A4], 'balance cache not invalidated'),
    ('A4h compare-and-set result ignored', [('        if changed != 1 {\n', '        if false {\n')], 'p2_', [A3R], 'ERR_NOT_ABORTABLE'),
    ('A4i compare-and-set without the status', [('WHERE txid = ?2 AND status = ?3"', 'WHERE txid = ?2 AND ?3 = ?3"')],
     'p2_', [A3R], 'ERR_NOT_ABORTABLE'),
    ('A4j child-spent created output not refused', [('        if spent_by_other > 0 {\n', '        if false {\n')],
     'p2_', [A4C], 'ERR_NOT_ABORTABLE'),
    # P2-A5 — the pending entry
    ('A5 pending entry kept', [('PENDING_TRANSACTIONS.lock().unwrap_or_else(|p| p.into_inner()).remove(&reference);', 'let _ = &reference;')],
     'p2_', [A1, A3N, A4, A5, A6], 'pending entry survived the abort'),
    # P2-A6 — lookup and errors
    ('A6a no txid lookup', [('Ok(None) if reference.len() == 64', 'Ok(None) if false && reference.len() == 64')],
     'p2_', [A6], 'by txid'),
    ('A6b restore error swallowed', [('restored += output_repo.restore_by_spending_description(&txid).map_err(|e| failed(&e))?;',
      'restored += output_repo.restore_by_spending_description(&txid).unwrap_or(0);')], 'p2_', [A6], 'ERR_ABORT_FAILED'),
    ('A4k spent_by not checked', [('spent_by IS NOT NULL\n               OR ', '')], 'p2_', [A4C], 'spent_by = ?2'),
    # P2-A8 — serialised with the wallet's own broadcasts of `nosend` rows (adversarial review)
    ('A8a abort without create_action_lock', [('    let _create_action_guard = state.create_action_lock.lock().await;\n\n    // 1. Find it', '\n    // 1. Find it')],
     'p2_', [A8], 'abortAction ran while create_action_lock was held'),
    ('A8b broadcast_nosend without create_action_lock',
     [('    let _create_action_guard = state.create_action_lock.lock().await;\n\n    // Fetch the raw tx', '\n    // Fetch the raw tx')],
     'p2_', [A8], 'broadcast_nosend ran while create_action_lock was held'),
    # P2-A7 — signAction after an abort
    ('A7 sign_action guard removed', [('                if action.status == ActionStatus::Failed {\n', '                if false {\n')],
     'p2_', [A7], 'ERR_ACTION_ABORTED'),
]

def run(filt):
    try:
        out = subprocess.run(['cargo', 'test', '--bin', 'hodos-wallet', filt], cwd='rust-wallet', capture_output=True,
                             text=True, encoding='utf-8', errors='replace', timeout=900)
        text = out.stdout + out.stderr
    except subprocess.TimeoutExpired as e:
        text = (e.stdout or b'').decode('utf-8', 'replace') if isinstance(e.stdout, bytes) else (e.stdout or '')
        text += '\n<<TIMEOUT>>'
    failed = sorted(set(re.findall(r'::(p2_\w+) \.\.\. FAILED', text)))
    return failed, text

def restore():
    for f, body in ORIG.items():
        f.write_bytes(body.encode('utf-8'))

ok_all = True
try:
    only = sys.argv[1:]
    for name, edits, filt, want_failed, reason in MUTATIONS:
        if only and not any(name.startswith(o) for o in only):
            continue
        restore()  # only ONE mutation live at a time
        target = C if name.endswith('(COLL)') else H
        body = ORIG[target]
        for old, new in edits:
            assert body.count(old) == 1, f'{name}: anchor not unique/found: {old[:60]!r}'
            body = body.replace(old, new)
        target.write_bytes(body.encode('utf-8'))
        failed, text = run(filt)
        if 'error[' in text and not failed:
            verdict = 'BAD (did not compile)'
        elif '<<TIMEOUT>>' in text:
            verdict = 'BAD (runner timed out)'
        else:
            verdict = 'RED OK' if set(failed) == set(want_failed) and reason in text else 'BAD'
        ok_all &= verdict == 'RED OK'
        print(f'{verdict:8} {name}: failed={failed} want={sorted(want_failed)} reason_seen={reason in text}', flush=True)
finally:
    restore()
failed, _ = run('p2_')
print('restored; green again:', failed == [])
sys.exit(0 if ok_all and failed == [] else 1)
