# Advisory triage: the BSV reference stack's security advisories, checked against Hodos

**Written:** 2026-09-30 · **Method:** eight parallel agents, **code reading only**, each finding given two verdicts (below), then merged by the orchestrating session. Rule-7 ground-truth checks were run by the orchestrator where a trip-wire was named (§3). Full per-finding tables (437 rows + 1 added): `ADVISORY_TRIAGE_TABLES.md`.

**Sources:** ts-stack **GHSA-qp3j-h5xf-p2p7** (2026-09-23, high, **391 findings** TSA-001…431), **GHSA-2qqx-463q-2qhq** (2026-09-16, critical), **GHSA-5vmp-9hjc-rfwp** (2026-09-10, critical), **GHSA-622h-889m-w6w6** (2026-09-26, medium), **GHSA-36f9-7rg5-cpf8** (2026-07-06, high); go-sdk **GHSA-rh54-8fpg-8wwf** (2026-09-29, high).

**The two verdicts per finding:**
- **(A) COMPAT:** can the hardened server or SDK now **reject our client**, as MessageBox did with our nonce?
- **(B) VULN:** do **we** have the same weakness?

| Totals | P0 | P1 | P2 | none / N-A |
|---|---|---|---|---|
| 438 findings | 5 rows (**2 distinct issues**, below) | 43 | 83 | 306 |

Most of the "none" rows are software Hodos neither runs nor talks to (UHRP, CHIRP, Teranode, DID, marketplace CI, nginx…), or JavaScript object-aliasing bugs that cannot occur in Rust. All 29 script-interpreter rows are N/A: Hodos has no interpreter.

---

## 1. The distinct issues, merged by theme

### G1 — BRC-103 handshake + MessageBox client · 🚨 **P0, users broken now**
| Issue | Findings | Evidence |
|---|---|---|
| **Handshake `initialNonce` is 32 bytes; the server requires 48 (SDK format)** | TSA-120, -131, -363 | **Measured** against live MessageBox (ticket). Owner approved the SDK-exact fix |
| Our own `/.well-known/auth` responder also issues a 32-byte nonce: do hardened TS clients reject it? | TSA-120, -131 | UNKNOWN: read the updated `Peer.ts` |
| Client never verifies the server's handshake signature / nonce echo / identity key | TSA-010, -132, -356, 2qqx, TSA-374/-417/-420 (P2) | code reading; already ticketed (`TICKET_brc103_server_identity_unverified`, T5-P1) |
| Client follows redirects with `x-bsv-auth-*` headers; no response-size cap | TSA-374, -417, -420 (P2) | code reading |
| **After the nonce fix:** MessageBox now pages `listMessages`; our parser treats an unknown shape as "no messages" | TSA-325 | code reading. Measure the live shape in the same change |
| **After the nonce fix:** MessageBox now enforces recipient fees; our send attaches none | TSA-047 | code reading. Measure |
| Local BRC-33 relay (`/sendMessage` etc.) takes identity from the unsigned `x-bsv-auth-identity-key` header | TSA-010, -325, 2qqx | code reading. Unknown whether a page can set that header (T5-P2's strip list does not name it) |

### G2 — **The next silent break: dApps on the new SDK strictly validate our answers** · P1 (compat, UNKNOWN)
The advisory added strict result validators to `@bsv/sdk`. Any dApp that upgrades will check Hodos's responses, and **any mismatch breaks that dApp with no change on our side**, the MessageBox pattern again.
| Suspected mismatch | Findings |
|---|---|
| ⭐ **The 1000-sat service-fee output**: "bound-action" / template-binding checks compare the final tx with what the dApp requested | TSA-082, -278, -401 |
| `listActions` returns legacy statuses (`confirmed`, `pending`) and extra keys | TSA-272 |
| `listOutputs` can return an output without its transaction; tag filter returns the whole basket when no tag matches | TSA-274, -289 |
| Error bodies are `{"error":…}`, not the BRC-100 envelope; `CreateActionResponse` has extra fields; result/BEEF/`noSendChange` consistency unconfirmed | TSA-061, -063, -065 |
| `/verifySignature`, `/verifyHmac` return `{valid:false}` where the SDK throws | TSA-040 |

**⭐ One offline test settles all of G2:** record real Hodos responses (dev wallet) and run them through the `b3155fa2` validators. No money needed. **Do this before scoping G2's fixes.**

### G3 — Money-path correctness · P1
| Issue | Findings | Ground truth (§3) |
|---|---|---|
| **`abortAction` re-locks the DB mutex it already holds ⇒ wallet hangs or panics.** It can also abort an already-broadcast action and mark its spent inputs spendable, swallows errors, and is not bound to the originating site | TSA-260 (P0 claim), -023, -044, -042 | code reading **confirmed**; **never fired** in either wallet's logs. Since `271dadb` (2026-09-16), so **in shipped beta.4** |
| `createAction` accepts negative / overflowing output amounts; the spending-cap sum can be fooled into about 1 sat | TSA-148, -069, -141 | no bad rows in either wallet |
| Broadcast success accepted for an empty or different txid; weak ARC statuses (RECEIVED…ANNOUNCED) count as on-chain; 200 with unknown status is success | TSA-192, -255, -250, -415 (P2) | code reading |
| Merkle proofs stored without verification (and "anyway" on network error) | TSA-188, -249 | see §3 |
| `sendWith` broadcasts partially | TSA-261 | code reading |
| Suspected double-spends become permanent after 6 h without evidence | TSA-346 | **0 rows** in either wallet |
| `/wallet/cleanup` turns a chain error into "spent" | TSA-306 | already in beta.7 T1-P2 |

### G4 — Certificates and identity · P1
| Issue | Findings |
|---|---|
| ⭐ **Recipient-name spoofing in Send:** `identity_resolver.rs` never verifies the certificate signature or trusted certifier, and never checks that the subject is the key asked about | TSA-077, -283, 5vmp F8d |
| Certificates stored unverified (signature check skipped for placeholder revocation outpoints; revocation failure proceeds) and not bound to the request | TSA-317, -284, 5vmp F8a/F8b (F8c P2) |
| Certifier-URL SSRF (issuance posts to a dApp-supplied URL; loopback treated as internal) | TSA-066 (P0 claim). **Measured: consent-gated** (202 prompt first) ⇒ P1. The prompt also shows a misleading "0-sat payment" |

### G5 — Overlay / SHIP · P1
Host advertisements are used with no signature or identity check; plain-http and loopback hosts are accepted; responses are unbounded. A forged advert can make an identity-certificate **unpublish** report success while the certificate stays public. TSA-073, -221, -341, -326, -334, -412, 5vmp F4/F7.

### G6 — Wallet HTTP surface · P1 (already planned)
Denylist, not allowlist; headerless loopback trusted as the wallet UI. TSA-057, -322. Covered by the beta.7 tickets `TICKET_dapp_reachable_surface_is_a_denylist_not_an_allowlist` (T5-P2) and `…headerless_loopback_requests_are_trusted_as_wallet_ui` (intake, beta.8).

### G7 — BRC-121 402 payments · P1
The paid retry follows redirects carrying the payment headers and cookies cross-origin, and a success from the redirect target triggers the broadcast; `http` payees are accepted; `PaidContentCache` ignores `no-store`/`private` and never expires without `Cache-Control`. TSA-121; a retry re-sends the same tx (TSA-369, P2).

---

## 2. Recommendation for `v0.4.0-beta.6` (👤 owner decides)

| Theme | Recommend | Why |
|---|---|---|
| **G1** | **In.** Nonce fix + measure paging/fees on the live server + server-signature verification | Users broken now; one function |
| **G2** | **Test first (offline), then decide.** Fix any confirmed break **in** | A confirmed break stops every upgraded dApp: same severity as G1 |
| **G3** abort · negative amounts/cap · broadcast substitution | **In** | Money-path; small, local fixes with clear tests |
| **G3** merkle-proof verification · double-spend 6 h · sendWith | beta.7 T1 (money path) unless the §3 proof check finds bad rows | Latent; no bad state found |
| **G4** recipient spoofing | **In** | Money to the wrong person |
| **G4** unverified certificates · certifier SSRF | beta.7 T5-P1 | Consent-gated; T5-P1 already rewrites this area |
| **G5** overlay/SHIP | beta.7 (T5) | Affects publish/unpublish truthfulness, not funds |
| **G6** | beta.7 T5-P2 (already planned) | Already scoped |
| **G7** | beta.7 T4 (402) | That track owns the paid retry |

## 3. Rule-7 ground-truth checks run 2026-09-30 (orchestrator, measured)

| Trip-wire named | Check | Result |
|---|---|---|
| TSA-260 abort re-lock | Code read (std `Mutex` re-entry: "will not return"); searched both wallets' logs for `/abortAction` | Confirmed in code; the only call in either log (dev, 2026-09-21) returned "not found" before the lock. **Never fired** |
| TSA-066 certifier SSRF | Dev wallet, harmless target `/health`, posing as an approved site | `202 pending` (payment-confirmation prompt); **no self-request made**. Residue: one expired approval in the dev wallet |
| TSA-346 double-spend verdicts | Read-only query, both wallets | **0** `double-spend-detected` rows |
| TSA-148 negative amounts | Read-only query, both wallets | **0** negative / >21M-BSV outputs |
| TSA-188/249 unverified proofs | Installed wallet, 12 of 532 `proven_txs` rows vs WoC headers | Block hash exists at the stored height **12/12**. ⚠️ `merkle_root` column is **empty** in every sampled row, so path→root was **not** verified. Owed: compute the root from `merkle_path` |

## Gaps

- **TSA-364** (certificate mutation / signing TOCTOU) was **omitted** by the chunk-6 agent despite its row count. Orchestrator triage, code reading only: the aliasing half is JavaScript-specific (Rust owns its values), so **B = NO** for that part. Whether our certificate signing preimage is unambiguous is **UNKNOWN**. **P2.**
- Every verdict is code reading by agents, except §3. Compat verdicts on the new SDK validators were reasoned from the advisory text, because the hardened ts-stack source was not available locally; G2's offline test replaces them.
