# B5-T5-P1 — Servers prove who they are (BRC-103/104) · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (copied 2026-09-28 from `../../../0.4.0-beta.3/PHASE_CONTRACT_TEMPLATE.md`, read-only).
> Written at G3 by the T5 track agent. ⛔ Documents only — no code, no schema, no issue, no endpoint stood up.

**Track:** B5-T5 Identity & privacy · **Tickets:** `../../tickets/TICKET_brc103_server_identity_unverified.md`, `../../tickets/TICKET_well_known_auth_returns_a_key_it_cannot_sign_for.md`, plus T5 SCOPE §4 findings **N1** (certificate path fails open) and **N2** (dropped port) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Fable 5.1), T5 track agent, repo head `65869b1` on `0.4.0` · **Platforms:** both (Rust is platform-neutral; the live smoke runs on each) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **T5 Q2** (`/.well-known/auth` — keep and fix to the protocol, low priority: send the identity key and sign with a child of that same key, per the SDK; proof = the ticket's SDK `Peer` handshake RED → GREEN plus a corrupted-byte control; no site-scoped variant here) · **T5 Q1** per the scope's recommendation (follow `@bsv/sdk` exactly: refuse a response carrying `x-bsv-*` headers without a valid signature; plain HTTP only for a host that never spoke BRC-103; MessageBox always speaks it, so refuse). ⚠️ Changing which key `/.well-known/auth` sends is identity/signing — **invariant 3, owner-approved in T5 Q2**. Nothing else here changes what is signed or how; the signing math in `sign_with_derived_key` and `derive_child_private_key` is untouched.

---

## 1. Goal

When the wallet talks BRC-103 to a server (MessageBox for PeerPay today, any certifier for certificate acquisition), a server that cannot sign for the identity key it claims is **refused**, on every path — and the wallet's own `/.well-known/auth` answers with an identity key a standard verifier accepts.

## 2. Done means

- [ ] `AuthFetchClient::handshake` reads and verifies the `initialResponse` signature over `[clientNonce ‖ serverNonce]` under `[2,'auth message signature']`, keyID `"<clientNonce> <serverNonce>"`, counterparty = the **claimed** key, and returns `Err` on a missing, malformed or non-verifying signature. A scratch server asserting a key it does not hold is refused (evidence `P1-A1`).
- [ ] One verifier function serves both callers; `acquire_certificate_issuance` no longer has its own copy and no longer proceeds when the signature is absent (`P1-A3`). The `"No server signature to verify (proceeding anyway)"` line no longer exists in the tree.
- [ ] `authenticated_request` verifies the BRC-104 §6.9 response signature (`x-bsv-auth-signature` over request-id, status, sorted signed headers, body) when the response carries any `x-bsv-*` header, and refuses a response that carries them without a valid signature (`P1-A5`, T5 Q1).
- [ ] `AuthFetchClient::fetch` builds the handshake base URL with the port (`P1-A4`).
- [ ] `well_known_auth` sends the **master** identity key as `identityKey` and signs with the child of that key derived against the client's identity key — so `@bsv/sdk` 2.8.7 `Peer` completes a handshake against the running wallet (`P1-A6`), and a corrupted signature byte makes it fail (`P1-A6`'s owner-decided control).
- [ ] PeerPay send + receive on the **live** MessageBox and one certificate acquisition from a real certifier still work end to end after the fix (`P1-A2`, `P1-A3b`) — ⛔ K1/K3 measured **before** landing.
- [ ] `rust-wallet/src/CLAUDE.md` §"App-Scoped Identity Keys" and `PROJECT_OVERVIEW.md`'s matching line corrected in the same commit as the `/.well-known/auth` change (they describe the app-scoped key as a working privacy feature; it was never verifiable).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-PEERPAY-DELIVERY` (beta.3 set) | A PeerPay either delivers its message or never leaves the wallet | Half 1 runs at every boundary. **Half 2 (real money) is directly at risk here**: `messagebox.rs :: send_message / list_messages / acknowledge_message` all go through `AuthFetchClient::fetch`. A verifier with the nonce order reversed, or a MessageBox that signs in a shape the SDK does not, makes every PeerPay send fail *after* the coin is selected. ⇒ `P1-A2` runs before landing, and the phase closes with Half 2 run, not cited |
| `R-PERIM` — sensitive cert fields | The four privacy-perimeter gates behave per `matrix_c.rs` | This phase edits `certificate_handlers.rs :: acquire_certificate_issuance` (acquisition), **not** `prove_certificate` / disclosure. `R-PERIM` T1 at the boundary; the cert-disclosure arm is not touched, asserted by `git diff` on `dispatch_cert_disclosure` call sites = 0 |
| `R-INTEXT` | Internal never prompts, external always gates | `well_known_auth` keeps `check_domain_approved` first; the change is which key goes in the response body. The SDK `Peer` test client is a **Node** process with no `X-Requesting-Domain` (internal path). A browser-page handshake still gates. Asserted in `P1-A6`'s SUBJECT |
| Invariant 1 (keys never in JS) | — | Nothing here reaches JavaScript. The `identityKey` is a **public** key |
| Invariant 3 (crypto/signing) | — | Owner-approved for the `/.well-known/auth` key change only (T5 Q2). Verification code is *added*; no derivation or signing function changes. `git diff --stat rust-wallet/src/crypto/` must be empty at close |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. A green result is reported with its red half or not at all.
⛔ Money, schema and crypto rows: the RED (negative control) is **designed by someone other than the assertion's author** — a second agent (`../../../RELEASE_CYCLE.md` §4.2). Record who designed it. Every signature-verification row below is a **crypto** row and carries the placeholder; `P1-A4` (port) and `P1-A7` (docs/tree) are not.

**Step 0 — reproduce on today's build first (measurement, not a control).** Stand up a scratch `/.well-known/auth` (Node, `@bsv/sdk` 2.8.7 `Peer` server side is the cheapest correct one) in three modes: (i) asserts a key it does not hold; (ii) valid key, one signature byte corrupted; (iii) no `signature` field. Point `AuthFetchClient::fetch` at it (`HODOS_DEV=1` wallet on `31401`, a `#[ignore]`d integration test or a one-off binary). Expected today: **all three accepted** — the `handshake OK — server key:` line appears. If any is refused today, the ticket is wrong somewhere and this contract is corrected before code is written (`../../tickets/TICKET_brc103_server_identity_unverified.md` §"Test").

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P1-A1` | After the fix, Step 0's three scratch modes (wrong key · corrupted byte · missing field) are each **refused** by `AuthFetchClient::handshake` with a distinct `AuthFetchError::Handshake(..)` naming the reason, and a correctly-signing scratch server still passes | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The Rust wallet log (`RUST_LOG=hodos_wallet=debug`, the **file** not stderr) line emitted by `authfetch.rs :: handshake` — the reason string, per mode. Not the HTTP status, not the certificate path (which has its own verifier). Dev port `31401` | T2 | ⬜ |
| `P1-A2` 👤 | PeerPay **send** and **receive** against the live `messagebox.babbage.systems` succeed end to end with the verifier on (`messagebox.rs` → `AuthFetchClient::fetch`; receive via `monitor/task_check_peerpay.rs :: MessageBoxClient::list_messages`). K1 answered: MessageBox signs its `initialResponse` in the SDK shape | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The recipient's `peerpay_received` row + `outputs` row, WhatsOnChain for the txid, and the sender's `peerpay_outbox` (0 rows) — `R-PEERPAY-DELIVERY` Half 2's subject, plus the sender's `handshake OK` line showing the verifier ran (a new `debug!` naming the verified key's first 16 hex) | T2 real money, cents (`PAYMENT_TEST_BATCH.md`) | ⬜ |
| `P1-A3` | `acquire_certificate_issuance` calls the shared verifier; a missing or unparseable `signature` in a certifier's `initialResponse` ⇒ `502` with a named error, **not** `"proceeding anyway"`. The `js_base64_to_array` second-decoder fallback either moves into the shared verifier or is deleted — one verifier, one decoder rule, stated in the diff | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The Rust log line from the **shared** verifier (one function name in both callers' logs), and `grep -c "proceeding anyway" rust-wallet/src` = 0 | T1 (unit on the verifier with SDK test vectors) + T2 | ⬜ |
| `P1-A3b` 👤 | One real certificate acquisition (the certifier the owner uses — CoolCert / SocialCert per `identity_resolver.rs` constants) succeeds with the fail-closed verifier. K3 answered | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The `certificates` row inserted + the certifier named in the log; run **before** landing, per §6.4 of the scope | T2 human (owner's certifier account) | ⬜ |
| `P1-A4` | `AuthFetchClient::fetch` against `http://127.0.0.1:<non-default port>/x` handshakes at `http://127.0.0.1:<that port>/.well-known/auth` | Revert the one-line `host_str()` → `host_str() + port` change: the handshake goes to `http://127.0.0.1/.well-known/auth` (port 80) and the scratch server on the real port logs **no** request — observe the scratch server's access log empty and the connection-refused error in the wallet log | The **scratch server's** request log (which port received the POST), not the wallet's own URL string | T1 (unit: `Url::parse` + base-URL builder) + T2 | ⬜ |
| `P1-A5` | `authenticated_request` verifies `x-bsv-auth-signature` on responses per BRC-104 §6.9 (`@bsv/sdk` `AuthFetch` behaviour, T5 Q1): a response carrying any `x-bsv-*` header with a missing/invalid signature ⇒ `Err`; a plain response with no `x-bsv-*` headers from a host that never spoke BRC-103 passes; for MessageBox (always BRC-103) a plain response is refused | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The wallet log line from `authenticated_request` naming which of the three branches fired, against the scratch server configured per branch. ⚠️ HARNESS §2: the control must fail **at** the signature check, with the body valid and headers present — not at a JSON parse | T1 + T2 | ⬜ |
| `P1-A6` | `@bsv/sdk` 2.8.7 `Peer` + `SimplifiedFetchTransport` (Node, no `X-Requesting-Domain`) completes `authenticateInitialResponse` against the running dev wallet's `/.well-known/auth`: the `identityKey` in our response equals the wallet's master public key (`get_master_public_key_from_db`) and the signature verifies | 👤 **Owner-decided control (T5 Q2):** (a) today's build ⇒ `Peer` throws `Unable to verify initial response signature` (RED first, confirms the ticket); (b) after the fix, a tiny Node proxy that replays the wallet's response with **one signature byte flipped** (JSON valid, nonces valid) ⇒ the same `Peer` error again. ⛔ No product-code seam for (b) | The **SDK's** verdict (`Peer.ts :: authenticateInitialResponse` throw vs `isAuthenticated`), not our own log; the client is the Node process on the internal path, so `domain_trust_mw` shows `<none:internal>` for it — say so, because a browser-page client would 202 first and confuse the reading | T2 | ⬜ |
| `P1-A7` | K2 answered on the owner's production log: the count of `Babbage auth request received` lines across the last ~30 days of `%APPDATA%/HodosBrowser/wallet` logs is recorded (0 expected). `rust-wallet/src/CLAUDE.md` §"App-Scoped Identity Keys" and `PROJECT_OVERVIEW.md` no longer describe the app-scoped key as a working privacy feature | Grep both docs for `app-scoped` / `prevent cross-app tracking` ⇒ 0 hits after; >0 before | The two files' text; the log count is a measurement and is recorded as one | T0 (grep) + measurement | ⬜ |
| `P1-A8` | Boundary regression: `R-PEERPAY-DELIVERY` Half 1 green; `R-PERIM` T1 green; `git diff --stat rust-wallet/src/crypto/` empty; `cargo test` green | Per `REGRESSION_SET.md`'s own REDs | Per `REGRESSION_SET.md` | T1 | ⬜ |

**Two-sided rows:** `P1-A1` (refuse the forgery) and `P1-A2` / `P1-A6` (accept the genuine) are each other's control — a verifier that always fails passes A1 and fails A2; one that never runs passes A2 and fails A1. Report them together.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/authfetch.rs :: AuthFetchClient::fetch` | ✅ | Builds `base_url` from `parsed.scheme()` + `parsed.host_str()` — **no port** (N2) |
| `rust-wallet/src/authfetch.rs :: AuthFetchClient::handshake` | ✅ | Reads `identityKey`, `initialNonce`; **never reads `signature`**; logs `handshake OK — server key:` and returns `AuthSession { server_identity_key, server_initial_nonce, client_initial_nonce }` |
| `rust-wallet/src/authfetch.rs :: AuthFetchClient::authenticated_request` | ✅ | Signs with invoice `2-auth message signature-{request_nonce} {server_initial_nonce}` via `sign_with_derived_key(&session.server_identity_key, …)`; checks only `401`/`403` on the way back — no response verification |
| `rust-wallet/src/authfetch.rs :: AuthSession` | ✅ | The struct that holds the claimed key; `server_identity_key: Vec<u8>` |
| `rust-wallet/src/handlers/certificate_handlers.rs :: acquire_certificate_issuance` | ✅ | The only existing verifier: parses `signature` as a byte array, derives `server_child_pubkey` via `derive_child_public_key`, `secp.verify_ecdsa`; tries a second `js_base64_to_array` decoder on failure; **`None` ⇒ `"No server signature to verify (proceeding anyway)"`** (N1). Also logs `"❌ CRITICAL: Certifier public key differs from server's identityKey!"` and continues — **its own ticket**, out of scope here |
| `rust-wallet/src/crypto/brc42.rs :: derive_child_public_key` / `derive_child_private_key` | ✅ | Reused as-is. ⛔ Not edited |
| `rust-wallet/src/crypto/brc43.rs :: InvoiceNumber`, `SecurityLevel`, `normalize_protocol_id` | ✅ (via `well_known_auth`'s use) | Reused for the invoice |
| `rust-wallet/src/handlers.rs :: well_known_auth` | ✅ | `check_domain_approved` first; `identityKey` = `derive_child_public_key(master_priv, app_identity_key, "2-identity")` (the **app's** child key); signs with `derive_child_private_key(master_priv, app_identity_key, "2-auth message signature-<theirNonce> <ourNonce>")`. The fix changes the **first** to `master_pubkey_hex`; the second is already what the SDK does |
| `rust-wallet/src/messagebox.rs :: MessageBoxClient` (`send_message`, `list_messages`, `acknowledge_message`) | ✅ | The only production callers of `AuthFetchClient::fetch`; `MESSAGEBOX_URL` = `https://messagebox.babbage.systems` (port 443 — why N2 is latent) |
| `rust-wallet/src/monitor/task_check_peerpay.rs :: run` | ✅ | Receive side: `MessageBoxClient::new` + `list_messages("payment_inbox")` every 60 s |
| `cef-native/src/core/HttpRequestInterceptor.cpp` — the `/.well-known/auth` arm in `Open()` | ✅ | Re-points a page's **loopback** `/.well-known/auth` to `hodos::WalletBaseUrl()` via `hodos::IsLoopbackAuthority(hodos::OriginFromUrl(url))`; external sites' own `/.well-known/auth` are not intercepted. ⛔ Not edited |
| `cef-native/include/core/PortConfig.h :: IsWellKnownAuthRequest`, `IsLoopbackAuthority` | ✅ | Routing predicates; not edited |

Not touched: `prove_certificate`, `dispatch_cert_disclosure`, `auth_session.rs` (our **server-side** session store for `/.well-known/auth` clients), the C++ layer, the schema.

## 6. Out of scope

- Adding an inbound BRC-103 **verifier role** (`processGeneralMessage`, the 2.7.1 session-identity binding) — we never take the verifier role today; noted for the future crate (T5 Q2's "all functionality" is about the *server-role handshake*, not a full peer).
- The certifier-mismatch `"CRITICAL … continues anyway"` at `acquire_certificate_issuance` — its own ticket (named in the source ticket's "out of scope").
- Rewriting `certificate_handlers.rs`'s hand-rolled header path to share `AuthFetchClient` (N3 — per-request handshakes are a performance matter, noted not chased).
- A site-scoped identity key in `/.well-known/auth` — T5 Q2: stays a BRC-draft proposal (`originator-scoped-authentication-keys`), not code.
- Nonce replay tracking in `well_known_auth` (its own `TODO`); not this phase.
- Tempted by: making `AuthFetchClient` keep a session across calls. No — N3, performance, separate.

## 7. Rollback

One commit per item, all additive except the one-line `identityKey` change in `well_known_auth`; `git revert` of the phase's commits restores the accept-everything client and the app-scoped key. No schema, no data.

## 8. Pre-mortem (adversarial review — before)

| Story: it shipped and failed because… | Row that catches it |
|---|---|
| The nonce order in the preimage was reversed (`server ‖ client`) — the check **always** fails, so PeerPay dies quietly after coin selection | `P1-A2` (genuine server must pass) run **before** landing; `P1-A1`'s two-sided pairing |
| MessageBox signs its `initialResponse` in a shape the SDK's `Peer` would also reject (K1) — a conformant client cannot talk to it | `P1-A2` measured live first; §6.4 of the scope: never land item 2 without it. If it fails, the phase stops and the owner decides (per-host exception vs upstream report) |
| The verifier was extracted but the certificate path kept the `js_base64_to_array` fallback as a *second* verifier — two truths that drift | `P1-A3` SUBJECT (one function name in both logs) |
| The corrupted-byte control went red **upstream** (invalid JSON, invalid base64) and looked like a verifier refusal | `P1-A6` control spec: JSON valid, nonces valid, only signature bytes flipped; the SDK's exact error text recorded (HARNESS §2 "fail AT the subject") |
| Response verification (`P1-A5`) refuses MessageBox because it omits `x-bsv-auth-signature` on some responses (e.g. 4xx) | `P1-A5` branch table recorded against the live server for 200 **and** an error response; Q1 says refuse — if the live server omits it, that is evidence for §12 and the owner decides |
| `/.well-known/auth` now sends the master key but the SDK test drove a **browser page**, which 202'd on domain trust, and the "RED" was a connect prompt, not a signature failure | `P1-A6` SUBJECT names the Node client and the `<none:internal>` trust line |
| The fail-closed certifier verifier breaks acquisition from a certifier that never signed (K3) | `P1-A3b` before landing |
| The port fix changed the base URL for MessageBox (443) and broke it | `P1-A4` + `P1-A2` |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all; `P1-A2` and `P1-A3b` on the dev wallet | Rust code is shared; the wallet's DPAPI auto-unlock path is the one exercised |
| macOS | `P1-A2` (PeerPay receive via the Keychain-unlocked wallet), `P1-A6` against the mac dev wallet | Same Rust; the smoke re-run is owed because the mac wallet is a separately built binary and the Keychain unlock path differs. No C++ change ⇒ no relay-round rebuild note needed, but the relay round names the smoke |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1.5 h**: one PeerPay send + receive on the live MessageBox between two wallets (`P1-A2`, ~40 min incl. funding checks), one certificate acquisition with the owner's certifier account (`P1-A3b`, ~20 min), the K2 log count on the production machine (~10 min), reading the §12 answers (~20 min) |
| Human-bound rows | `P1-A2` (real money, two wallets), `P1-A3b` (owner's certifier account) |
| Unknowns (K) — uncertainty, not difficulty | **K1** MessageBox signs in the SDK shape (both handshake and responses) — measured by `P1-A2`/`P1-A5` before landing · **K2** anything calls our `/.well-known/auth` — `P1-A7` · **K3** the certifier signs `initialResponse` — `P1-A3b` · **K18 (new)** which error responses from MessageBox carry `x-bsv-*` headers (decides `P1-A5`'s branch behaviour on 4xx) |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | none | No T1/T2/T3/T4/T6 dependency. Rust-only |
| gives | T6 (prompt surfaces) | Nothing — no new prompt |
| gives | future crate / server-side wallet (post-release) | A conformant server-role `/.well-known/auth` |
| shares | `R-PEERPAY-DELIVERY` Half 2 (release boundary) | This phase **runs** Half 2 at close; the RC run can cite it if nothing touches PeerPay afterwards |

## 12. Open questions for the owner

1. **`P1-A5` on MessageBox error responses (K18):** if the live server returns 4xx/5xx **without** `x-bsv-*` headers, Q1's "MessageBox always speaks it ⇒ refuse" would turn every server error into a signature error. Recommendation: apply the signature check only when the response carries `x-bsv-*` headers **or** is 2xx; surface a headerless non-2xx as the HTTP error it is. This follows `@bsv/sdk` `AuthFetch`'s "rejects a plain response that carries any `x-bsv-*` header" literally without inventing a stricter rule. Yes / no?
2. No evidence that a G2 decision is wrong.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
