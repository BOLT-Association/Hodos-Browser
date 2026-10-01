# B6-P4 — Send shows a name for a key only if a trusted certifier said so, about that key · PHASE CONTRACT

**Release:** `v0.4.0-beta.6` · **Source:** triage G4 (`../ADVISORY_TRIAGE.md`): TSA-077, TSA-283, GHSA-5vmp F8d · **Status:** ✅ DONE (T1 + a live sample)
**Opened:** 2026-09-30 · **Author:** Claude (Opus 5.5) · **Platforms:** shared Rust, both
**Standard:** `../../0.4.0-beta.3/HARNESS.md` + `../../0.4.0-beta.7/HARNESS_DELTA.md`.
**Owner decisions carried (2026-09-30):** fix recipient-name spoofing: verify the certificate signature, the certifier, and that the subject is the key asked about. Unverified-certificate storage and certifier SSRF stay in beta.7 T5-P1.

---

## 1. Goal

A name the wallet shows next to a payment recipient was really issued by a certifier we trust, for that key.

**Where users see it today:** the Send form's name search (`TransactionForm.tsx` → `/wallet/recipient/suggest` → `IdentityResolver::search`, name → key). Choosing a suggestion pays `identity_key`, which is now the certificate's signed **subject**. `/wallet/recipient/resolve` (key → name) has **no UI caller** today (adversarial review); it is hardened too because the endpoint is reachable.

## 2. Done means

- [x] `identity_resolver.rs :: check_certificate` runs before any field is decrypted, for both `search` and `resolve`: the certifier is in `TRUSTED_CERTIFIERS` (Metanet Trust, SocialCert — the same constant the overlay query sends), and the certifier's BRC-52 signature verifies (`certificate::verifier::verify_certificate_signature_with_keyid`, keyed by the original base64 type/serial, as `acquireCertificate` does) (`P4-A1`, `P4-A2`).
- [x] In `resolve`, the subject must equal the key asked about (case-insensitive hex), and the identity returned is the **subject**, never the query string (`P4-A3`). Before, `ResolvedIdentity.identity_key` was the query, so any certificate about any key was shown as the typed key's name.
- [x] Real certificates from the live overlay still pass (`P4-A5`).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| Send UX | Resolution is best-effort and never blocks sending | A refused certificate returns `None`, like a lookup miss: Send shows no suggestion, never an error |
| Invariant 3 (crypto) | No signing/derivation logic changed | Only callers of the existing verifier are added; `verifier.rs` is not modified (its divergences from the SDK are reported in §8, not changed) |

## 4. Evidence table

⛔ REDs designed by a second agent (Claude Opus 5.5) before the tests were final; the field-swap vector and per-case reasons were added from that design.

| ID | 🟢 GREEN | 🔴 RED (mutation in `negative_controls.py`) | 🎯 SUBJECT | Tier | Result |
|---|---|---|---|---|---|
| `P4-A1` | SDK-made genuine certificate ⇒ name "alice", identity = subject, asked lower-case, upper-case, or by search | **M4** case-sensitive subject compare · **M5** identity taken from the query · **M7** verifier refuses everything (`verifier.rs`) | `extract_identity_from_certificate` (production function; trusted list = the vectors' certifier) over `tests/fixtures/identity_cert_vectors.json`, made by `@bsv/sdk` 2.8.11 (`g2-validator/gen_identity_cert_vectors.mjs`), which also asserts the SDK's own verdict on every vector | T1 | 🟢🔴 |
| `P4-A2` | Untrusted certifier (valid signature) ⇒ refused "is not trusted"; forged certifier (claims the trusted key, signed by another), altered signature, and the victim's signed certificate with the attacker's fields + keyring swapped in ⇒ refused "does not verify"; none shown, by key or by search. Every case checked before failing. The SDK decrypts all four, and so does our code (M2 proves it) | **M1** trust check off · **M2** signature ignored ⇒ "forged_certifier shown as a name" · **M6** search path skips the check | same + `check_certificate` | T1 | 🟢🔴 |
| `P4-A3` | Genuine certificate about S, asked about K ⇒ no name | **M3** subject check off | same | T1 | 🟢🔴 |
| `P4-A4` | Production trust list is exactly Metanet + SocialCert, and the vectors' certifier does not pass it | **M1**, **M6** | constant + production list | T1 | 🟢🔴 |
| `P4-A5` | Live (2026-09-30, `#[ignore]`d network test, `cargo test -- --ignored p4_a5`): every raw certificate `overlay-us-1` returned for "john"/"bsv"/"a" — **5 pass, 0 refused**; `search` shows 4 verified identities | (verifier refusing everything ⇒ fails here and in A1) | `check_certificate` over real overlay output | T3 (read-only network) | 🟢 |

**Controls run:** `negative_controls.py` (this folder): **7/7 RED OK**, green after restore (2026-09-30).

**Not falsifiable / not measured (declared):**
- **M8**, keying the verifier by re-encoded base64: SDK base64 is canonical, so re-encoding gives the same bytes.
- **Live sample:** 5 certificates, all "Email via SocialCert". **No Metanet Trust certificate could be found** on any of the three hosts (15 queries), and no X/Discord type came back. Whether those pass is unmeasured; a refusal fails closed (no suggestion).
- `process_output` / `process_search_output` (BEEF → PushDrop → JSON) are exercised only by the live test.

## 5. Blast radius (verified 2026-09-30)

| Cited code | Note |
|---|---|
| `rust-wallet/src/identity_resolver.rs :: check_certificate`, `TRUSTED_CERTIFIERS`, `extract_identity_from_certificate` | The change. Callers `process_output` (resolve) and `process_search_output` (search) |
| `certificate/parser.rs :: parse_certificate_from_json`, `certificate/verifier.rs :: verify_certificate_signature_with_keyid` | Reused unchanged |
| `handlers.rs :: recipient_suggest` (UI), `recipient_resolve` (no UI caller) | Consumers; unchanged. Both build `IdentityResolver::new()` per request, so no cache outlives a request; `transactions.recipient_name` has no writer, so no unverified name was ever stored |

## 6. Out of scope

- PushDrop locking key and the subject's field signature (prior art `identityUtils.ts` checks them; they prove the subject chose to publish, not who the subject is), revocation, chain evidence for the overlay output: beta.7 T5.
- Certificates stored unverified by `acquireCertificate`, certifier-URL SSRF: beta.7 T5-P1.

## 7. Rollback

One commit; `git revert`. No schema, no data.

## 8. Pre-mortem and residuals

| Failure story | Caught by |
|---|---|
| Our verifier disagrees with real certifiers ⇒ names vanish from Send | `P4-A5` live (sample of 5) |
| A forgery is refused only because decryption fails, not because of the check | M2 goes red: the forgeries decrypt in our code |
| The refusal log records the key the user is about to pay | Logged at `debug` (production runs at `warn`) — fixed from the review |

**Residuals (stated, not fixed here):**
1. **Lookalike names.** A genuine certificate for a lookalike handle ("elonmusk" on Discord) shows with a trusted label; a hostile host chooses which certificates to return and in what order. Signatures cannot fix this.
2. **Avatars (pre-existing).** `TransactionForm.tsx` renders `<img src={avatar_url}>` for any http(s) URL from the certificate, so typing a name in Send fetches subject-chosen URLs (IP and search leak to the image host). No injection (React escapes; `javascript:` excluded).
3. **Our shared BRC-52 verifier diverges from the SDK in three places, each failing closed** (a real certificate is refused): field names are sorted byte-wise where the SDK uses `localeCompare` (differs for mixed-case/punctuated names); a txid-only `revocationOutpoint` (the SDK default) is refused by `parser.rs`; high-S signatures are refused by `secp256k1::verify_ecdsa`. `verifier.rs` is shared with `acquireCertificate`; changing it is crypto-adjacent (invariant 3) ⇒ 👤 owner, with beta.7 T5-P1.
4. A malicious **subject** could in theory craft a field ciphertext that decrypts under two keys (AES-GCM is not key-committing); the SDK has the same exposure.

## 9. Platforms

Shared Rust; T1 via `cargo test`.

## 12. Open questions for the owner

Residual 3 (verifier vs SDK) — reported for beta.7 T5-P1; not blocking.

---

## Sign-off

| Item | Result | Date | By |
|---|---|---|---|
| preflight `-Full` | PASS — all checks ran | 2026-09-30 | Claude (Opus 5.5) |
| preflight `-NegativeControl` | PASS — every gate seen to fail | 2026-09-30 | Claude (Opus 5.5) |
| `cargo test --workspace` | 0 failed across 20 targets | 2026-09-30 | Claude (Opus 5.5) |
| P4 negative controls | 7/7 RED OK (`negative_controls.py`), green after restore | 2026-09-30 | Claude (Opus 5.5); REDs designed by a second agent |
| adversarial review (after) | Second agent (Claude Opus 5.5). No fail-open path found. **Closed:** contract claimed a UI path that does not exist (`resolve` has no caller; §1 rewritten); refusal log leaked the payee key at `warn` (→ `debug`); stale test comment. **Recorded:** lookalike names, avatar fetches, verifier/SDK divergences (fail closed), live sample limits (§4, §8) | 2026-09-30 | second agent + author |
