"""B6-P3 negative controls: apply one mutation to rust-wallet/src/handlers.rs (Rust rows) or to
cef-native/include/core/PaymentCost.h (names ending "(C++)"), run the P3 tests, record exactly which
tests failed and with what message, restore the file. REDs designed by a second agent (Claude Opus 5.5,
2026-09-30) before the tests were final.
Run from the repo root: python development-docs/0.4.0-beta.6/phase-P3-amounts-cap-broadcast/negative_controls.py [name prefixes]
The C++ rows rebuild `hodos_tests` (cef-native/build, Release) for each mutation.
"""
import re, subprocess, sys, pathlib

H = pathlib.Path('rust-wallet/src/handlers.rs')
P = pathlib.Path('cef-native/include/core/PaymentCost.h')
ORIG = {H: H.read_bytes().decode('utf-8'), P: P.read_bytes().decode('utf-8')}  # bytes: no CRLF rewrite

AMOUNTS, BUILDER = 'p3_a1_amounts_must_be_real_money', 'p3_a1_create_action_internal_refuses_before_selecting'
ACK = 'p3_a3_unbound_acknowledgement_is_verified_and_never_confirms_ours'
LOOKUP = 'p3_a3_lookup_absent_only_on_the_not_found_variant'
NEG, IMPOSSIBLE, SINGLE = 'PaymentCostP3.NegativeOutputCannotShrinkThePrice', 'PaymentCostP3.ImpossibleOutputAmountsAreNotDerivable', \
    'PaymentCostP3.NegativeSingleAmountShapesAreNotDerivable'

# (name, [(old, new)], tests that must fail, text that must appear)
MUTATIONS = [
    # P3-A1 — Rust amounts
    ('A1a negatives allowed', [('        if sats < 0 {', '        if false {')],
     [AMOUNTS, BUILDER], 'negative accepted'),
    ('A1c sum not bounded', [('total = total.checked_add(sats).filter(|t| *t <= MAX_SATOSHIS)', 'total = total.checked_add(sats)')],
     [AMOUNTS, BUILDER], 'above max accepted'),  # the sum bound is also the per-output bound
    ('A1d 0-sat output refused', [('        if sats < 0 {', '        if sats < 1 {')],
     [AMOUNTS], 'Output 0: satoshis 0'),
    ('A1e max-exact refused', [('.filter(|t| *t <= MAX_SATOSHIS)', '.filter(|t| *t < MAX_SATOSHIS)')],
     [AMOUNTS], 'Outputs sum to more than'),
    ('A1f builder does not check', [('    let total_output = match validate_output_amounts(&req.outputs) {',
      '    let total_output = match Ok::<i64, String>(req.outputs.iter().filter_map(|o| o.satoshis).fold(0i64, |a, b| a.wrapping_add(b))) {')],
     [BUILDER], 'ERR_INVALID_OUTPUT_AMOUNT'),
    # P3-A3 — broadcast acknowledgement binding
    ('A3a every ack bound', [('        if br.txid != expected_txid {\n', '        if false {\n')], [ACK], 'must be verified once'),
    ('A3b verify asked about the provider txid', [('return match verify(expected_txid.to_string()).await {', 'return match verify(br.txid.clone()).await {')],
     [ACK], 'verify was asked about a txid that is not ours'),
    ('A3c unbound proof still cached', [('            return match verify(expected_txid.to_string()).await {',
      '            if let (Some(ref mp), Some(db)) = (&br.merkle_path_bump, db_for_cache) { cache_arc_merkle_proof(db, &br.txid, mp); }\n            return match verify(expected_txid.to_string()).await {')],
     [ACK], "(provider's proof cached, our status)"),
    ('A3d not-on-network accepted', [('                Ok(true) => Ok(format!(', '                Ok(_) => Ok(format!(')], [ACK], 'not on network'),
    ('A3f lookup verdict by text (adversarial review)', [('                broadcast_lookup_verdict(services.tx_status_unanimous(&txid).await)\n',
      '                tx_exists_verdict(services.tx_status_unanimous(&txid).await)\n'),
      ('        Err(crate::services::IndexerError::NotFound) => Ok(false),\n        Err(e) => Err(e.to_string()),\n        ok => tx_exists_verdict(ok),',
       '        r => tx_exists_verdict(r),')],
     [LOOKUP], "an error mentioning 'not found' read as absent"),
    ('A3g empty-txid ack verified (releases on index lag)', [('        if br.txid.is_empty() {\n', '        if false {\n')],
     [ACK], 'empty txid refused'),
    ('A3e inconclusive is an error (releases inputs)', [('''                    Ok(format!(
                        "{} acknowledged '{}' ({}); our {} unverified: {}",''', '''                    Err(format!(
                        "{} acknowledged '{}' ({}); our {} unverified: {}",''')],
     [ACK], 'verify inconclusive'),
    # P3-A2 — C++ cap
    ('A2a negatives read (C++)', [('if (i < 0 || i > kMaxSatoshis) return false;', 'if (i > kMaxSatoshis) return false;')],
     [NEG, IMPOSSIBLE, SINGLE], 'priced 2 sats'),
    ('A2b floats read (C++)', [('''        out = i;
        return true;
    }
    return false;''', '''        out = i;
        return true;
    }
    if (v.is_number_float()) { out = 1; return true; }
    return false;''')], [IMPOSSIBLE], '1.9'),
    ('A2c unsigned wraps (C++)', [('        if (u > static_cast<uint64_t>(kMaxSatoshis)) return false;\n', '')],
     [IMPOSSIBLE, SINGLE], '18446744073709551614'),  # positive JSON ints are unsigned in nlohmann
    ('A2d sum not bounded (C++)', [('                    if (total > kMaxSatoshis) return kAmountNotDerivable;\n', '')],
     [IMPOSSIBLE], '"satoshis":2100000000000000},{"satoshis":1}'),
    ('A2e single amount unchecked (C++)', [('''            if (!ReadSatoshis(json["amount"], amount)) return kAmountNotDerivable;''',
      '''            amount = json["amount"].get<int64_t>();''')], [SINGLE], '"amount":-5'),
]

def run_rust():
    out = subprocess.run(['cargo', 'test', '--bin', 'hodos-wallet', 'p3_'], cwd='rust-wallet', capture_output=True,
                         text=True, encoding='utf-8', errors='replace', timeout=1200)
    text = out.stdout + out.stderr
    return sorted(set(re.findall(r'::(p3_\w+) \.\.\. FAILED', text))), text, 'error[' in text

def run_cpp():
    b = subprocess.run(['cmake', '--build', 'build', '--config', 'Release', '--target', 'hodos_tests'], cwd='cef-native',
                       capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=1800)
    if b.returncode != 0:
        return [], b.stdout + b.stderr, True
    t = subprocess.run([str(pathlib.Path('cef-native/build/bin/Release/hodos_tests.exe').resolve()), '--gtest_filter=PaymentCost*:ExtractOutputSatoshis*:ComputePaymentCost*'],
                       cwd='cef-native', capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=300)
    text = t.stdout + t.stderr
    return sorted(set(re.findall(r'\[  FAILED  \] (\w+\.\w+) \(', text))), text, False

def restore():
    for f, body in ORIG.items():
        f.write_bytes(body.encode('utf-8'))

ok_all = True
try:
    only = sys.argv[1:]
    for name, edits, want_failed, reason in MUTATIONS:
        if only and not any(name.startswith(o) for o in only):
            continue
        restore()
        cpp = name.endswith('(C++)')
        target = P if cpp else H
        body = ORIG[target]
        for old, new in edits:
            assert body.count(old) == 1, f'{name}: anchor not unique/found: {old[:60]!r}'
            body = body.replace(old, new)
        target.write_bytes(body.encode('utf-8'))
        failed, text, broken = run_cpp() if cpp else run_rust()
        if broken and not failed:
            verdict = 'BAD (did not compile)'
        else:
            verdict = 'RED OK' if set(failed) == set(want_failed) and reason in text else 'BAD'
        ok_all &= verdict == 'RED OK'
        print(f'{verdict:8} {name}: failed={failed} want={sorted(want_failed)} reason_seen={reason in text}', flush=True)
finally:
    restore()
rf, _, _ = run_rust()
cf, _, cb = run_cpp()
print('restored; green again:', rf == [] and cf == [] and not cb)
sys.exit(0 if ok_all and rf == [] and cf == [] and not cb else 1)
