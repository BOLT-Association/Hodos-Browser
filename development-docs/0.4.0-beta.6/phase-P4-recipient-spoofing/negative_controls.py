"""B6-P4 negative controls: apply one mutation to rust-wallet/src/identity_resolver.rs (or
certificate/verifier.rs for names ending "(V)"), run the P4 tests, record exactly which tests failed and
with what message, restore the file. REDs designed by a second agent (Claude Opus 5.5, 2026-09-30).
Run from the repo root: python development-docs/0.4.0-beta.6/phase-P4-recipient-spoofing/negative_controls.py [name prefixes]
"""
import re, subprocess, sys, pathlib

R = pathlib.Path('rust-wallet/src/identity_resolver.rs')
V = pathlib.Path('rust-wallet/src/certificate/verifier.rs')
ORIG = {R: R.read_bytes().decode('utf-8'), V: V.read_bytes().decode('utf-8')}  # bytes: no CRLF rewrite

A1, A2, A3, A4 = 'p4_a1_genuine_certificate_names_its_subject', 'p4_a2_unsigned_untrusted_or_forged_certificates_name_nobody', \
    'p4_a3_certificate_about_another_key_is_not_that_keys_name', 'p4_a4_production_trusts_exactly_the_two_known_certifiers'

MUTATIONS = [
    ('M1 trust check off', [('if !trusted_certifiers.iter().any(', 'if false && !trusted_certifiers.iter().any(')],
     [A2, A4], 'untrusted_certifier shown as a name'),
    ('M2 signature ignored', [('.map_err(|e| format!("certifier signature does not verify: {:?}", e))?;', '.ok();')],
     [A2], 'forged_certifier shown as a name'),
    ('M3 subject check off', [('        if !expected.eq_ignore_ascii_case(&subject) {', '        if false {')],
     [A3], 'was shown as the name of the key asked about'),
    ('M4 case-sensitive subject', [('        if !expected.eq_ignore_ascii_case(&subject) {', '        if expected != subject {')],
     [A1], 'genuine certificate refused (asked Some('),
    ('M5 identity taken from the query', [('        let identity_key = subject.as_str();', '        let identity_key = expected_subject.unwrap_or(subject.as_str());')],
     [A1], 'left: ('),
    ('M6 search path skips the check', [('    let certifier = cert.get("certifier").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();\n    if !trusted',
      '    if expected_subject.is_none() { return Ok(cert["subject"].as_str().unwrap_or("").to_lowercase()); }\n    let certifier = cert.get("certifier").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();\n    if !trusted')],
     [A2, A4], 'shown as a name (asked None)'),
    ('M7 verifier refuses everything (V)', [('    let is_valid = secp.verify_ecdsa(&message, &signature, &public_key).is_ok();',
      '    let is_valid = { let _ = secp.verify_ecdsa(&message, &signature, &public_key); false };')],
     [A1], 'genuine certificate refused'),
]

def run():
    out = subprocess.run(['cargo', 'test', '--bin', 'hodos-wallet', 'p4_'], cwd='rust-wallet', capture_output=True,
                         text=True, encoding='utf-8', errors='replace', timeout=1200)
    text = out.stdout + out.stderr
    return sorted(set(re.findall(r'::(p4_\w+) \.\.\. FAILED', text))), text

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
        target = V if name.endswith('(V)') else R
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
