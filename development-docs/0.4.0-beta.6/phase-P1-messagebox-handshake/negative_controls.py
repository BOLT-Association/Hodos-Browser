"""B6-P1 negative controls: apply one mutation to rust-wallet/src/authfetch.rs (or messagebox.rs for
names ending "(MB)"), run the P1 tests, record which tests failed and with what message, restore the
file. Every mutation must turn red for its stated reason (A1-A4 REDs designed by a second agent,
Claude Opus 5.5, 2026-09-30; A6 by the author — not a crypto row).
Run from the repo root: python development-docs/0.4.0-beta.6/phase-P1-messagebox-handshake/negative_controls.py [name prefixes]
"""
import re, subprocess, shutil, sys, pathlib

SRC = pathlib.Path('rust-wallet/src/authfetch.rs')
ORIG = SRC.read_bytes().decode('utf-8')  # byte-exact: text mode would rewrite LF as CRLF on Windows
MB = pathlib.Path('rust-wallet/src/messagebox.rs')
MB_ORIG = MB.read_bytes().decode('utf-8')

ALL_NONCE = 'vector mismatches: [nonce:ascii,nonce:invalid_utf8,nonce:leading_bom,nonce:embedded_nul,nonce:valid_multibyte,nonce:truncated_tail,nonce:bom_not_at_start,nonce:lone_continuation,nonce:overlong,nonce:surrogate,nonce:truncated_mid,nonce:leading_zero_key] of 12'

MUTATIONS = [
    ('A1a no BOM strip', 'let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);', '',
     ['p1_a1_handshake_nonce_matches_sdk_vectors'], 'vector mismatches: [keyID:leading_bom,nonce:leading_bom] of 12'),
    ('A1b keep 32-byte key', 'let first_nonzero = key.iter().position(|b| *b != 0).unwrap_or(key.len());', 'let first_nonzero = 0;',
     ['p1_a1_handshake_nonce_matches_sdk_vectors'], 'vector mismatches: [nonce:leading_zero_key] of 12'),
    ('A1c keyID as hex', 'InvoiceNumber::new(SecurityLevel::CounterpartyLevel, "server hmac", sdk_to_utf8(first_half))',
     'InvoiceNumber::new(SecurityLevel::CounterpartyLevel, "server hmac", hex::encode(first_half))',
     ['p1_a1_handshake_nonce_matches_sdk_vectors', 'p1_a1_sdk_generated_nonces_verify'], ALL_NONCE),
    ('A1d security level 1', 'InvoiceNumber::new(SecurityLevel::CounterpartyLevel, "server hmac"',
     'InvoiceNumber::new(SecurityLevel::ProtocolLevel, "server hmac"',
     ['p1_a1_handshake_nonce_matches_sdk_vectors', 'p1_a1_sdk_generated_nonces_verify'], ALL_NONCE),
    ('A2 request nonce uses handshake format', 'let request_nonce = generate_nonce_base64();',
     'let request_nonce = create_handshake_nonce(&self.private_key)?;',
     ['p1_a2_wire_nonce_lengths'], 'request nonce decoded length'),
    ('A3a verification always fails', '.map_err(|_| fail("server signature does not verify"))?;',
     '.map_err(|_| fail("server signature does not verify"))?; return Err(fail("stubbed"));',
     ['p1_a3_sdk_peer_reply_verifies', 'p1_a3_high_s_form_of_genuine_signature_verifies', 'p1_a4_bad_replies_are_refused_each_for_its_own_reason', 'p1_a2_wire_nonce_lengths'], 'stubbed'),
    ('A3b no normalize_s', 'signature.normalize_s();', '',
     ['p1_a3_high_s_form_of_genuine_signature_verifies'], 'high-S form of a valid signature refused'),
    ('A3c double hash', 'Message::from_digest_slice(&Sha256::digest(&data))\n        .map_err(|_| fail("invalid digest"))',
     'Message::from_digest_slice(&Sha256::digest(Sha256::digest(&data)))\n        .map_err(|_| fail("invalid digest"))',
     ['p1_a3_sdk_peer_reply_verifies', 'p1_a3_high_s_form_of_genuine_signature_verifies', 'p1_a4_bad_replies_are_refused_each_for_its_own_reason', 'p1_a2_wire_nonce_lengths'], 'does not verify'),
    ('A4a no yourNonce check', 'if reply["yourNonce"].as_str() != Some(client_nonce) {', 'if false {',
     ['p1_a4_bad_replies_are_refused_each_for_its_own_reason'], "case 'yourNonce not ours': forged reply ACCEPTED"),
    ('A4b no messageType check', 'if reply["messageType"].as_str() != Some("initialResponse") {', 'if false {',
     ['p1_a4_bad_replies_are_refused_each_for_its_own_reason'], "case 'wrong messageType': forged reply ACCEPTED"),
    ('A4c missing signature treated as OK', 'let signature: Vec<u8> = reply["signature"].as_array()',
     'if reply["signature"].is_null() { return Ok((server_identity_key, server_nonce.to_string())); }\n    let signature: Vec<u8> = reply["signature"].as_array()',
     ['p1_a4_bad_replies_are_refused_each_for_its_own_reason'], "case 'signature missing': forged reply ACCEPTED"),
    ('A4d handshake ignores the verifier', 'let (server_identity_key, server_nonce) =\n            verify_initial_response(&self.private_key, &response_json, &client_nonce)?;',
     'let (server_identity_key, server_nonce) =\n            verify_initial_response(&self.private_key, &response_json, &client_nonce).unwrap_or_else(|_| (hex::decode(response_json["identityKey"].as_str().unwrap_or("")).unwrap_or_default(), response_json["initialNonce"].as_str().unwrap_or("").to_string()));',
     ['p1_a4_production_handshake_refuses_forged_reply'], 'fetch() ACCEPTED a forged handshake reply'),
    ('A4e empty server nonce allowed', '.filter(|n| !n.is_empty())\n        .ok_or_else(|| fail("missing initialNonce in response"))?;',
     '.ok_or_else(|| fail("missing initialNonce in response"))?;',
     ['p1_a4_bad_replies_are_refused_each_for_its_own_reason'], "case 'server nonce empty'"),
    # P1-A6 (messagebox.rs): restore the two pre-fix behaviours
    ('A6a ignore hasMore (MB)', 'if json.get("hasMore").and_then(|v| v.as_bool()) != Some(true) {', 'if true {',
     ['p1_a6_has_more_yields_every_message_and_the_next_offset', 'p1_a6_collects_every_page_once_with_advancing_offsets', 'p1_a6_endless_has_more_stops_at_the_page_cap'], 'nextOffset must be followed'),
    ('A6b unknown shape = empty inbox (MB)', 'let messages = json.get("messages").and_then(|v| v.as_array()).cloned().ok_or_else(|| {',
     'let messages = json.get("messages").and_then(|v| v.as_array()).cloned().or(Some(vec![])).ok_or_else(|| {',
     ['p1_a6_error_or_unknown_shape_is_an_error_not_an_empty_inbox'], "unknown shape must not read as 'no messages'"),
    ('A6c offset never advances (MB)', 'Some(n) => offset = n,', 'Some(_) => {},',
     ['p1_a6_collects_every_page_once_with_advancing_offsets'], 'offsets must advance'),
]

def run():
    out = subprocess.run(['cargo', 'test', '--bin', 'hodos-wallet', '::tests::p1_'],
                         cwd='rust-wallet', capture_output=True, text=True, encoding='utf-8', errors='replace')
    text = out.stdout + out.stderr
    failed = sorted(set(re.findall(r'(?:authfetch|messagebox)::tests::(p1_\w+) \.\.\. FAILED', text)))
    return failed, text

ok_all = True
try:
    only = sys.argv[1:]  # optional name prefixes, e.g. A3c A4
    for name, old, new, want_failed, reason in MUTATIONS:
        if only and not any(name.startswith(o) for o in only):
            continue
        # Both files back to pristine before every mutation: only ONE mutation may be live.
        SRC.write_bytes(ORIG.encode('utf-8'))
        MB.write_bytes(MB_ORIG.encode('utf-8'))
        target, base = (MB, MB_ORIG) if name.endswith('(MB)') else (SRC, ORIG)
        assert base.count(old) == 1, f'{name}: anchor not unique/found'
        target.write_bytes(base.replace(old, new).encode('utf-8'))
        failed, text = run()
        if 'error[' in text and not failed:
            verdict = 'BAD (did not compile)'
        else:
            right = set(failed) == set(want_failed) and reason in text
            verdict = 'RED OK' if right else 'BAD'
        ok_all &= verdict == 'RED OK'
        print(f'{verdict:8} {name}: failed={failed} want={want_failed} reason_seen={reason in text}', flush=True)
finally:
    SRC.write_bytes(ORIG.encode('utf-8'))
    MB.write_bytes(MB_ORIG.encode('utf-8'))
failed, _ = run()
print('restored; green again:', failed == [])
sys.exit(0 if ok_all and failed == [] else 1)
