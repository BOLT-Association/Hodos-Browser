"""B6-P5 negative controls (T1 rows B1/B2): apply one mutation to rust-wallet/src/handlers.rs (or
handlers/certificate_handlers.rs for names ending "(C)"), run the P5 tests, record exactly which tests
failed and with what message, restore the file. Author-designed (B1/B2 are response shapes, not
money/crypto). B3's RED is live: the pre-fix recording (g2-validator/recording_before_p5.json, #7).
Run from the repo root: python development-docs/0.4.0-beta.6/phase-P5-g2-fixes/negative_controls.py [name prefixes]
"""
import re, subprocess, sys, pathlib

H = pathlib.Path('rust-wallet/src/handlers.rs')
C = pathlib.Path('rust-wallet/src/handlers/certificate_handlers.rs')
ORIG = {H: H.read_bytes().decode('utf-8'), C: C.read_bytes().decode('utf-8')}

STATUS, SHAPE, DESC, TOTAL = 'p5_b1_status_is_the_brc100_word_as_stored', 'p5_b1_inputs_and_outputs_have_the_brc100_shape', \
    'p5_b1_descriptions_brc100_cannot_carry_become_empty', 'p5_b2_list_certificates_total_is_named_total_certificates'

MUTATIONS = [
    ('B1a legacy status words', [('        "referenceNumber": action.reference_number,\n        "status": status,\n',
      '        "referenceNumber": action.reference_number,\n        "status": action.status.to_string(),\n')],
     [STATUS], 'returned as'),
    ('B1b description passed through', [('        "description": brc100_description(action.description.as_deref()),',
      '        "description": action.description,')], [DESC], 'left: Null'),
    ('B1c sequence invented', [('in_tx.map(|i| i.sequence).unwrap_or(0xFFFF_FFFF)', '0xFFFF_FFFFu32')],
     [SHAPE], 'sequence from the stored raw tx'),
    ('B1d spendable not read', [('"spendable": ours.as_ref().map(|o| o.1).unwrap_or(false),', '"spendable": false,')],
     [SHAPE], 'Some(false)'),
    ('B1e old input shape', [('"sourceOutpoint": format!("{}.{}", input.txid, input.vout),', '"txid": input.txid,')],
     [SHAPE], 'called `Option::unwrap()` on a `None` value'),
    ('B2 snake_case total (C)', [('    // beta.6 P5 (B2): BRC-100 name; the SDK result validator rejects the snake_case one.\n    #[serde(rename = "totalCertificates")]\n', '')],
     [TOTAL], '"total_certificates":3'),
]

def run():
    out = subprocess.run(['cargo', 'test', '--bin', 'hodos-wallet', 'p5_'], cwd='rust-wallet', capture_output=True,
                         text=True, encoding='utf-8', errors='replace', timeout=1200)
    text = out.stdout + out.stderr
    return sorted(set(re.findall(r'::(p5_\w+) \.\.\. FAILED', text))), text

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
        target = C if name.endswith('(C)') else H
        body = ORIG[target]
        for old, new in edits:
            assert body.count(old) == 1, f'{name}: anchor not unique/found: {old[:60]!r}'
            body = body.replace(old, new)
        target.write_bytes(body.encode('utf-8'))
        failed, text = run()
        if 'error[' in text and not failed:
            verdict = 'BAD (did not compile)'
        else:
            verdict = 'RED OK' if set(failed) == set(want_failed) and reason in text else 'BAD'
        ok_all &= verdict == 'RED OK'
        print(f'{verdict:8} {name}: failed={failed} want={sorted(want_failed)} reason_seen={reason in text}', flush=True)
finally:
    restore()
failed, _ = run()
print('restored; green again:', failed == [])
sys.exit(0 if ok_all and failed == [] else 1)
