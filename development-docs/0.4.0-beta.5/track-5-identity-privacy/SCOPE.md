# B5-T5 — Identity & privacy · track scope (G2)

**Written:** 2026-09-25 by the G2 research agent for beta.5 Track 5. **Repo head read:** `399cc4c`
(branch `0.4.0`). **Status:** 📌 PROPOSED. The owner reviews it at G2. Phase contracts are G3's job,
and none are written here.

**How each claim is labelled:**
- **code**: code reading at `399cc4c`
- **measured**: something was actually run
- **doc**: a repo document says so
- **web**: fetched 2026-09-25, with the URL and version given

⛔ **Nothing was run against a live wallet or browser for this document.** Every "still true" below
is code reading. Each candidate phase therefore opens with a RED measurement.

---

## ✅ 0. G2 decisions applied (owner, 2026-09-27)

> The research below is the **evidence**; this block is the **decision**. Where they differ, this block wins.
> Full record: `../README.md` → "✅ Decisions as made".

| Question | Decided |
|---|---|
| Derived keys (P3; §9 Q4/Q5/Q6) | ✅ **Parts 1 + 2 (decision 11).** Part 1: levels 1–2 derived `getPublicKey` needs the `createSignature` grant. **Part 2 is IN** (overrides **Q5**'s defer): `requesting_domain` on `derived_key_cache`, **warn + log** when two sites get the same `(invoice, counterparty)` — schema change approved. Part 3 → BRC draft only. Level 0 stays open, documented. ⚠️ **Shape:** `derived_key_cache.derived_pubkey` is `UNIQUE` + `INSERT OR REPLACE`, so a plain column would overwrite the first site with the second — use a child table or change the uniqueness (T3 SCOPE §6a) |
| **Q7** missing counterparty | ✅ **Approved (decision 12):** SDK per-call defaults — `createSignature` ⇒ `anyone`, `createHmac`/`verifySignature` ⇒ `self`; not via `resolve_counterparty_pubkey`. Validate before prompting |
| **Q8 / Q9** user count | ✅ **Opt-out Brave-style ping with three conditions** (decision 13): first-run notice; the "every byte we send" page ships first; wire-level zero-requests control. No install date. Switch-on returns to the owner |
| **Q2** `/.well-known/auth` | ✅ **Keep, fix to the protocol, low priority.** It is the wallet's BRC-103 **server-role** path (never used — the browser wallet is always the client, and that path is correct). Fix = send the identity key and sign with the same key (SDK behaviour); SDK `Peer` RED → GREEN + corrupted-byte control. 👤 *"we just want all functionality"* |
| **Q3** merge tickets · **Q10** two-index → T1 · loopback W4/W6/W7/W8 → **here** | ✅ Done in the register 2026-09-27 |
| Q1 BRC-103 no-auth responses · Q11 · Q12 | Per this doc's recommendations (approved) |

---

## 1. Goal

A site learns only the keys, identity and wallet surface the user chose to give it. Servers we talk
to prove who they are (BRC-103/104). And we can count users without identifying them.

---

## 2. Telescope: the world as of 2026-09-25

### 2.1 What was re-fetched

| Source | Version / date | What it says that matters here |
|---|---|---|
| **BRC-103** `bsv-blockchain/BRCs` `peer-to-peer/0103.md` | last changed `fb14783`, 2026-04-24 (**web**) | *"When A receives this `initialResponse`, it verifies B's authenticity via the signature over `(A_Nonce + B_Nonce)`, marking the session as authenticated."* The verify steps are: recompute the preimage, *"verify the signature with the alleged identityKey"*, and *"otherwise, reject"*. |
| **BRC-104** `peer-to-peer/0104.md` | same commit (**web**) | §6.9 defines the **response** preimage: request-id, status, sorted signed headers, then body. `x-bsv-auth-*` headers MUST be excluded from the signed set. ⚠️ The spec has **no explicit client-side MUST** to verify responses. |
| **`@bsv/sdk`** (TypeScript), now in `bsv-blockchain/ts-stack/packages/sdk` | **2.8.7**, published 2026-09-25. `Peer.ts` at `40dad06` (**web**) | `authenticateInitialResponse` checks the nonce, then the signature over `[ourNonce, theirNonce]` with protocol `[2,'auth message signature']`, keyID `"<ourNonce> <theirNonce>"` and counterparty = the **claimed** key. Only after that does it set `isAuthenticated`. **`processGeneralMessage`** verifies every later message and, **since 2.7.1** (`134e4ec`, 2026-09-16), requires that message to carry the session's verified key. **`AuthFetch`** verifies response signatures. It falls back to plain HTTP only for a server known not to speak BRC-103, and it **rejects** a plain response that carries any `x-bsv-*` header. |
| **go-sdk** | **v1.6.0**, 2026-09-24. `auth/peer.go` at `f810a16` (**web**) | Same initial-response verification, same nonce order. ⚠️ **Inference:** it has no equivalent of TS's 2.7.1 session-identity binding. The two reference SDKs disagree on that half. |
| **`CreateSignatureArgs`** in `Wallet.interfaces.ts` | 2.8.7 (**web**) | `counterparty?` is **optional**. `ProtoWallet` defaults: **`createSignature` → `'anyone'`**; `encrypt`/`decrypt`/`createHmac`/`verifyHmac`/`getPublicKey`/`verifySignature` → `'self'`. |
| **`@bsv/wallet-toolbox`** `WalletPermissionsManager.ts` | **2.14.2**, file at `fc6ebef` 2026-09-25 (**web**) | `getPublicKey` with a `protocolID` runs `ensureProtocolPermission({usageType:'publicKey'})`, controlled by `seekPermissionsForPublicKeyRevelation` (**default `true`**). The identity key is a separate check under protocol `[1,'identity key retrieval']`, with its own flag (default `true`). **Level 0 is always open** (`if (level === 0) return true`). **No originator is bound into derivation**: `Wallet.ts` takes `_originator` unused, so the app picks protocol, keyID and counterparty freely. |
| **BRC index** | README at master (**web**) | **No BRC binds a derived key to the requesting origin.** BRC-43 has security levels only. BRC-100 uses the originator for authorisation only. **BRC-179** (method allow-list manifest) exists but is aimed at SDK/tool servers. |
| **Brave usage ping** | wiki "Brave's Capture of Usage Data", last edited 2026-03-02. `brave-core` `browser/brave_stats/brave_stats_updater_params.cc` (**web**) | One daily HTTP GET, **no identifier**. The device keeps its own "last ping" dates and sends `daily` / `weekly` / `monthly` / `first` flags plus platform, channel and version. The server counts requests that carry each flag. Brave says *"it contains no personal data"*. The CDN derives the country, then *"removes IP address from data sent to Brave"*. On by default, with a settings switch. |
| **Brave P3A / STAR** | P3A wiki; STAR post 2022-07-19 (**web**) | Built for **feature questions**, not user counts. It needs a randomness server and an aggregator, which is too much infrastructure for our question. |
| **Firefox `usage-reporting` ping** | `pings.yaml` on `main` (**web**) | Kept apart from other telemetry, but it still sends a **persistent** `profile_id` UUID, so it can be linked across days. |
| **Tor metrics** | `metrics.torproject.org/reproducible-metrics.html` (**web**) | *"we actually don't count users, but we count requests"*. It gives an estimate only. |
| **Chrome/Omaha** | `ServerProtocolV3.md` (**web**) | Counts by date: the server hands out the day number and the client echoes it back. No ID unless the user opts in. |

### 2.2 What changed against our repo's own docs

- **`TICKET_brc103_server_identity_unverified.md` cites `reference/ts-stack` read on 2026-09-16.**
  That copy predates 2.8.7. The initial-response rule has not changed. The **session-identity
  binding** is now shipped in TS (2.7.1+) and is still absent in Go.
- **`TICKET_createSignature_requires_counterparty.md`'s guess is confirmed** (**web**). The field is
  optional, and the default for signing is `'anyone'`, not `'self'`. ⚠️ Our shared helper
  `handlers.rs :: resolve_counterparty_pubkey` defaults `None` to **self** (**code**). A naive fix
  that reuses it would sign with the wrong key.
- **`TICKET_derived_public_keys_have_no_prompt…` says "the identity key always prompts, and
  auto-approve cannot silence it."** ⛔ **That is wrong** (**code** + **doc**). Identity-key reveal is
  silent when `identity_key_disclosure_allowed=1`, which is on by default (V19,
  `AUTO_APPROVE_ENGINE.md` §2 branch 2). The merged ticket must carry the correction.
- **`architecture/AUTO_APPROVE_ENGINE.md` §2 branch 3 does not list `SilentProtocolLevelZero`**,
  which `matrix_c.rs:199` now returns first (added by **`10b2916` — level-0 protocols are open**,
  2026-09-21) (**code**). The doc has drifted. Reported here, not edited.
- **`PROJECT_OVERVIEW.md:446` and `rust-wallet/src/CLAUDE.md` §"App-Scoped Identity Keys"** describe
  `/.well-known/auth` as a working privacy feature (**doc**). Per the ticket's measured maths it is
  not. Correct both when that ticket lands.

---

## 3. Kaleidoscope: do we already have this?

| We need | We already have | Reuse, or it would duplicate |
|---|---|---|
| Verify a BRC-103 `initialResponse` signature | ⭐ **`handlers/certificate_handlers.rs :: acquire_certificate_issuance`** already does it for certifiers: it derives the server's child public key, uses the nonce order client-then-server, and calls `secp.verify_ecdsa` (**code**, ~`:1271–1566`) | **Extract it into one function** in `authfetch.rs` or `crypto/`, and call it from both. A second hand-rolled copy is the duplicate to avoid. ⚠️ See the new finding in §4: this copy **fails open when the signature is missing**. |
| Derive the verifying child public key | `crypto/brc42.rs :: derive_child_public_key` (**code**) | Reuse. No new crypto. |
| A method allow-list for **external** callers | ⭐ **`simple_handler.cpp :: IpcMessageAllowedFromWebPage`** already default-denies IPC **message names** from web pages, allowing four (P0.5 C2) (**code**). `main.rs :: domain_trust_mw` is already the single Rust chokepoint and already has the path-normalisation work (**code**) | The Rust fix is the **same shape one layer down**: an allow-list of **endpoints** in `domain_trust_mw`. Do not build a third mechanism. |
| The list of endpoints dApps legitimately call | `CWIShimScript.h`: `'/' + methodName` for BRC-100, plus `/wallet/address-to-script`, `/wallet/broadcast`, `/wallet/bsv-price`, `/wallet/yours-legacy-addresses` (**code**) | Derive the allow-list from this and the BRC-100 route list. ⚠️ Two of these (`address-to-script`, `bsv-price`) are **on the denylist ticket's list of 30**, so a naive allow-list would break the Yours/Panda shims. |
| A scoped-grant prompt for derived keys | `permission_service::dispatch_scoped_grant`, already called by `create_hmac`, `encrypt`, `decrypt`, `create_signature`, `list_outputs`, `relinquish_output` (**code**). The engine's `ProtocolUse` + `SilentProtocolLevelZero` | `getPublicKey` gets the **same call**. No new `CallKind` and no new table. |
| A record of which derived keys were handed out | `derived_key_cache` (`derived_pubkey`, `invoice`, `counterparty_pubkey`), `forSelf` only, **no requesting site** (**code**) | Owner's idea 2 (log per site) would add a column here. ⛔ Schema change: invariant 2. |
| "Don't stop a wallet we didn't start" | Windows `StopWalletServer` guards on `g_walletServerProcess.hProcess` (**doc**, per ticket) | Port the guard to macOS `StopServers`. |
| A daily outbound call we already make | `AutoUpdater.cpp` / `AutoUpdater_mac.mm`: the appcast check to `hodosbrowser.com` every 86 400 s (**code**) | ⚠️ **Do not piggy-back the count on the update check.** The owner declined counting appcast hits (ticket, 2026-09-17), and Sparkle's request is not ours to shape. A separate endpoint on the same host is fine. Honesty note: the update check **already** shows the server our IP and version once a day. |
| Telemetry of any kind | **None.** `simple_handler.h:236` says *"we ship no telemetry"* (**code**) | That comment becomes false when the ping lands. It must change in the same commit. |

---

## 4. Ticket review

Every ticket was re-read against `399cc4c`. "Still true" means the load-bearing lines were found.

| Ticket (what it is) | Still true today? | Phase / item | Why |
|---|---|---|---|
| **`brc103_server_identity_unverified`**: our client accepts the server's identity key without checking its signature | ✅ **Yes** (**code**). `authfetch.rs :: handshake` reads `identityKey` and `initialNonce` and never reads `signature`. `authenticated_request` checks only 401/403. ⭐ The ticket's open question is now **answered by tracing**: `server_identity_key` is never displayed or stored. It lives only for one `fetch()` and is used as the signing counterparty. The only consumer is MessageBox/PeerPay. | **P1**, items: handshake verify (floor), response verify (system) | Owner's focus today. Conformance with both SDKs (**web**). |
| **`well_known_auth_returns_a_key_it_cannot_sign_for`**: our own `/.well-known/auth` answers with a key that does not match its signature | ✅ **Yes** (**code**). The handler still sends `derive_child_public_key(master, app_key, "2-identity")`, which is the **app's** child key, while signing with the master-derived key. The handler came in with **`0be3ee2` — "added correct app-scoped identity key"**, 2026-01-05 (verified). Only loopback requests are routed to it (`HttpRequestInterceptor.cpp` ~`:4394`). | **P1**, item: decide, then delete or fix | Same protocol, same verifier. ⚠️ Changing which key it sends is signing/identity: invariant 3, owner first. |
| **`derived_public_keys_have_no_prompt_and_can_match_across_sites`**: a connected site gets a derived key with no prompt, and two sites can get the same one | ✅ **Yes** (**code**). `get_public_key` gates only when `wants_identity_key`. The comment at `handlers.rs:~292` still wrongly says derived keys are gated elsewhere. `dispatch_scoped_grant` is not called here. | **P3** (the live ticket after the merge) | wallet-toolbox **does** gate this at levels 1 and 2 by default (**web**), so part 1 is conformance, not invention. |
| **`derivation_params_unbound_to_origin`**: key-derivation inputs come from the app, not bound to the origin | ✅ Yes, same code. ⭐ **Confirmed duplicate.** Same source finding (`FINDING-silent-derived-key-tracking.md`), same handler, same gap. | **Merge into the ticket above**, then close | It adds three things the newer ticket lacks, which the merge must carry: **(a)** "site B can ask for site A's keyID"; **(b)** the caller survey (task 3); **(c)** the BRC-43 note that the app declares its own level. The newer ticket adds the wallet-toolbox comparison and the owner's 2026-09-21 decisions. ⇒ **Keep `derived_public_keys…`**, fold (a)–(c) in, and stamp the older one *"closed: duplicate"* with a pointer. |
| **`createSignature_requires_counterparty`**: our `createSignature` rejects a request with no counterparty | ✅ **Yes, and now confirmed a conformance gap** (**code** + **web**). `CreateSignatureRequest.counterparty` and `CreateHmacRequest.counterparty` are non-optional; so is `VerifySignatureRequest.counterparty`, a `String`. The SDK makes all three optional. `encrypt`/`decrypt` are already optional, default self, which is correct. | **P3**, item | ⛔ Signing code, invariant 3. The fix is a **default**, not a change to signing, but it is still owner-approved. ⚠️ The default must be **`anyone` for createSignature** and **`self` for HMAC/verify**. Do not reuse `resolve_counterparty_pubkey`'s `None → self`. |
| **`dapp_reachable_surface_is_a_denylist_not_an_allowlist`**: an approved site can reach 30 internal wallet routes | ✅ **Yes** (**code**). `is_permission_surface` is still a deny-list, still `POST‖DELETE` only (`main.rs:145,164`). | **P2**, item: allow-list in `domain_trust_mw` | The register left "suggest T1" open. **Recommend keeping it in T5**: the harm it ranks first is privacy (addresses, balance, settings), not spending. |
| **`wallet_bridge_plumbing_is_advertised_to_every_site`**: the bridge's plumbing names Hodos on every https page | ✅ **Yes** (**code**). `global->SetValue("hodosBrowser", …)` at `simple_render_process_handler.cpp:925` is still **outside** the internal-page gate at `:943`. `cefMessage` is still on every page (`:1211`). ⭐ **Item 3 is now answered by reading**: a web page's `cefMessage.send` is limited to four names by P0.5's default-deny allow-list (`IpcMessageAllowedFromWebPage`), and `qr_found` is the named residual. | **P2**, items 1 and 2 | Same surface question as the ticket above. Item 3 needs only a RED on the four names, not a security review from scratch. |
| **`wallet_backend_is_shared_across_os_accounts`**: a second OS account's browser uses (and on macOS stops) the first account's wallet | ✅ **Yes** (**code**). Windows `LaunchWalletProcess` still adopts on `IsPortListening(hodos::WalletPort())` (`cef_browser_shell.cpp:3988`). macOS `SpawnWalletServer` adopts on `QuickHealthCheck()` (`:5473`), and `StopServers` sends `/shutdown` whenever `g_walletServerRunning` (`:5890`). The adblock engine has the same shape. | **P4** | The only ticket here with a **measured** event (macOS, 2026-09-24). |
| **`two_next_address_index_sources_can_reuse_addresses`**: two counters for the next address can drift apart and reuse an address | ✅ **Yes** (**code**). `generate_address` uses `wallet.current_index + 1` (`handlers.rs:~10473`). Change and other paths use `get_max_index()` (`handlers.rs:5811`, `certificate_handlers.rs:4573,5878`, `task_consolidate_dust.rs:185`). Self-heal code sits between them (`handlers.rs:~5934`). | ⭐ **Recommend moving it to B5-T1 Money path** | It is address **issuance**, and T1 already owns **`TICKET_rescan_cannot_find_payments_to_generated_addresses.md` — the rescan looks at BIP32 while generated addresses are BRC-42**, whose scan limit is `max(current_index + 20, 100)`. Two tracks editing the same counter is the serialisation hazard §3.6 warns about. The privacy harm (address reuse) is real, but the code belongs to money-path. |
| **`active_user_count_without_identifying_users`** (D9): count users without identifying them | ✅ Nothing exists (**code**). | **P5** | ⭐ The ticket's own proposed design (an HMAC with a daily and monthly rotating ID) is **superseded** by the owner's direction to adopt an industry standard. Brave's scheme sends **no identifier at all**, which is strictly less than the ticket's rotating ID. See P5. |

### New findings (not in any ticket)

**N1 — the certificate path fails open.** `acquire_certificate_issuance` verifies the certifier's
`initialResponse` signature **only if one is present**. If it is missing or unparseable, the code
logs `"No server signature to verify (proceeding anyway)"` and continues (`certificate_handlers.rs`
~`:1292`, ~`:1567`) (**code**).
- This is **trip-wire 2's shape**: a missing check reads as a pass. It is not a poisoning defect,
  because it corrupts no stored state that later work reads.
- ⇒ **P1 item.** The shared verify function must fail closed for both callers.

**N2 — `authfetch.rs :: fetch` drops the port.** It builds the handshake URL from `host_str()`
without the port (**code**). A BRC-103 server on a non-default port would be handshaken at the wrong
address. MessageBox uses 443, so nothing is broken today. **P1 item, small.**

**N3 — AuthFetch handshakes on every request.** `fetch()` performs a full handshake each call and
keeps no session (**code**). That is a performance matter, not a correctness one. **Note only**, per
the kaleidoscope rule: noted and ticketed, not chased.

---

## 5. Candidate phases (in order)

⭐ **Every phase opens by running its RED on today's build.** Nine of the ten tickets are code reading
only. A ticket whose RED does not reproduce closes instead of being fixed.

### B5-T5-P1 — Servers prove who they are (BRC-103/104)

- **Objective.** Our BRC-103 client refuses a server that cannot sign for the identity key it claims,
  on every path that speaks BRC-103. Our own `/.well-known/auth` either conforms or is gone.
- **Tickets.**
  - `brc103_server_identity_unverified`
  - `well_known_auth_returns_a_key_it_cannot_sign_for`
  - N1, the certificate path's fail-open
  - N2, the dropped port
- **Items, smallest first.**
  1. **The shared verifier.** Extract the certificate path's check into one function that **fails
     closed**.
  2. **Handshake verification** in `authfetch.rs`.
  3. **Certificate path**: call the shared verifier instead of its own copy.
  4. **The port fix.**
  5. **Response verification** (BRC-104 §6.9 preimage). What to do with a response that has no
     headers is decided by the owner question in §9.
  6. **`/.well-known/auth` decision.** Delete it, or send the master key. Owner approval first.
- **Unknowns.**
  - **K1:** Does `messagebox.babbage.systems` sign its `initialResponse` and responses in the form
    the SDK expects? If it does not, the fix **breaks PeerPay**. ⇒ Measure against the live server
    before landing.
  - **K2:** Does anything call our `/.well-known/auth`? The ticket's check is to search a week of
    `debug_output` logs for `Babbage auth request received`.
  - **K3:** Do the certifiers we use (e.g. CoolCert) sign `initialResponse`? If one does not, N1's
    fix breaks certificate acquisition for it.
- **Negative control.**
  - Run a local scratch `/.well-known/auth` that asserts a key it does not hold. Also run it with a
    valid key and one corrupted signature byte, and with **no** signature field at all.
  - Today's build must **accept** all three. That is the RED, and it confirms the ticket.
  - After the fix, all three must be refused. A **correct** server must still pass, so a check that
    always fails is caught.
  - **Subject:** the Rust process's `AuthFetchClient::handshake` log line, not the page.

### B5-T5-P2 — An approved site reaches only the dApp surface

- **Objective.** An external origin can call only endpoints on a declared allow-list, on every
  method (GET included). Pages stop naming Hodos through non-standard globals no dApp asks for.
- **Tickets.**
  - `dapp_reachable_surface_is_a_denylist_not_an_allowlist`
  - `wallet_bridge_plumbing_is_advertised_to_every_site`, items 1–3
- **Items.**
  1. **The allow-list** in `domain_trust_mw`, with the deny-list kept underneath it.
  2. **Move `window.hodosBrowser`** inside the internal-page gate.
  3. **Make the `__hodos_*` globals non-enumerable.**
  4. **Confirm by RED** that `cefMessage` from a web page is limited to the four allowed names.
- **Unknowns.**
  - **K4:** The complete legitimate endpoint set. The shim lists four `/wallet/*` routes. Are there
    others, for example BRC-121 `pay402` over IPC?
  - **K5:** Does any real dApp call a `/wallet/*` route directly? Survey the demo sites.
- **Negative control.** Use the ticket's own test.
  - From an approved origin, today's build returns **200** for `/wallet/addresses` and
    `/wallet/settings`, and the wallet dies on `/shutdown`. That is the RED.
  - After the fix these return **403**, with **no handler log line**, while `createAction` still
    works and the **gold pill** still fires.
  - For the globals, `CWI` must still enumerate while the plumbing names do not.

### B5-T5-P3 — Derived keys: the site asks, the user can tell

- **Objective.** A derived public key at security level 1 or 2 needs the same saved per-site grant
  `createSignature` needs, matching wallet-toolbox. Conforming SDK calls without a `counterparty` are
  accepted with the SDK's defaults.
- **Tickets.**
  - `derived_public_keys_have_no_prompt…`, with `derivation_params_unbound_to_origin` merged in
  - `createSignature_requires_counterparty`
- **Items.**
  1. **Part 1:** `dispatch_scoped_grant` on derived `getPublicKey`. 👤 The owner agreed to this on
     2026-09-21.
  2. **Fix the misleading comment.**
  3. **Optional `counterparty`** with SDK defaults. ⛔ Invariant 3 applies.
  4. **Survey real callers.** ⭐ One is already known: Xanadu calls `createSignature` with
     `[0,"xanaverse"]`, keyID `"1"`, counterparty `self` (**doc**: `0.4.0-beta.3/TICKET_level_0_…`).
  5. **Parts 2 and 3 are owner decisions** (§9). This scope recommends **not** building part 3 in
     beta.5.
- **Unknowns.**
  - **K6:** How often do real dApps call derived `getPublicKey` at levels 1 and 2 before any
    signature? That tells us whether this adds prompts or only moves them one call earlier.
  - **K7:** Does level 0 stay open? Today it matches wallet-toolbox, so cross-site matching at level
    0 is **an ecosystem-wide property**, not a Hodos bug. Say so plainly.
- **Negative control.**
  - Approve two scratch origins. Each calls `getPublicKey({protocolID:[2,'login'], keyID:'1',
    counterparty:'self', forSelf:true})`.
  - Today's build returns the **same key with no prompt**. That is the RED, which is also the
    ticket's own test.
  - After the fix, the first call prompts. A site that already holds the grant does not. The
    identity-key path still behaves as before.
  - For the counterparty item: a `createSignature` with no counterparty must verify under
    **`anyone`** and **fail** to verify under `self`. That pins the default, not just "no error".

### B5-T5-P4 — One OS account, one wallet

- **Objective.** Hodos in one OS account never uses or stops another account's wallet or adblock
  backend.
- **Ticket.** `wallet_backend_is_shared_across_os_accounts`.
- **Items.**
  1. **macOS:** never `/shutdown` a backend we did not spawn. This mirrors the Windows guard.
  2. **Both platforms:** never adopt a listener unless it belongs to this OS user. If it belongs to
     another user, say so clearly instead of adopting.
  3. **The per-user channel** (named pipe / Unix socket) and the "headerless means trusted" question.
     ⇒ Recommend **deferring to beta.6**.
- **Unknowns.**
  - **K8:** Which ownership check is reliable on each OS: a socket-owner lookup, or a per-user
    secret echoed on `/health`? This is new Win32/Darwin API use, so working rule 4 applies: cite
    the docs.
  - **K9:** What should a second account see? Its own wallet on another port, or a refusal? The
    port is fixed per build (`PortConfig.h`), so a second wallet needs a different port.
- **Negative control.** Use the ticket's two-account sequence.
  - Today, on macOS, account A's wallet dies the instant B quits. That is the RED, measured once.
  - After the fix, A's wallet process ID survives. The **subject** is **which identity key B's UI
    shows**, not "a wallet answered".
  - ⚠️ Cross-platform C++ means a relay round (root `CLAUDE.md`, 2026-09-12 rule).

### B5-T5-P5 — Count users without identifying them (D9)

- **Objective.** Adopt Brave's usage-ping scheme, cut down to what we need. Publish exactly what it
  sends before it ships. Prove the off switch sends nothing.
- **Ticket.** `active_user_count_without_identifying_users`.
- **What the scheme is** (Brave's, **web**):
  - Once a day, at the first launch or after midnight if the browser stays open, send **one HTTPS
    GET** with no cookies.
  - The browser keeps three dates locally: last ping day, last week number, last month.
  - It sends `daily=true`, plus `weekly=true` if this is the first ping this week, `monthly=true` if
    first this month, and `first=true` on the very first ping ever.
  - It also sends the platform, channel and version.
  - The server counts requests by day and never needs to tell two users apart:
    - **DAU** = the day's requests
    - **WAU** = requests with `weekly=true`, summed over the week
    - **MAU** = requests with `monthly=true`, summed over the month
- **Brave fields recommended to leave out:**
  - `ref`: we have no referral programme.
  - `dtoi` and `woi` (install date and week): see §9. At our size these are close to identifying.
  - Search counts and `adsEnabled`: not our question.
- **What it can and cannot reveal: the honest statement for users.**
  - ✅ **It cannot** tell us who you are, or link today's ping to yesterday's. It sends no ID,
    cookie, account, key, balance, wallet field, URL or browsing data.
  - ⚠️ **It can** tell us, per ping: that some copy of Hodos was used today; whether it is the first
    use this week, month or ever; the OS, channel and version.
  - ⚠️ **The IP is seen at the moment of the request.** Whoever terminates the connection (our host
    or CDN) sees it, just as the daily update check already does. Brave's protection, and ours, is
    **policy, not cryptography**: take the country at the edge, drop the IP before anything is
    written, keep no access logs. We say that in those words, not "anonymous".
  - ⚠️ **The ping's timing** (about 3 s after the first launch of the day) mirrors when you use the
    browser.
  - ⚠️ **Reinstalling** counts as a new user. **Clock errors** move the day flags.
- **Server side.**
  - One HTTPS GET endpoint (on `hodosbrowser.com` or an edge worker) that validates the parameters
    and increments a per-day counter.
  - A country lookup at the edge, suppressing small countries, then the IP is dropped.
  - Access logging off.
  - A published retention period.
  - No database of requests is needed. Counters are enough.
  - ⛔ **Outward-facing and irreversible once data exists.** The unconditional-stop rule 4 applies at
    every step that turns it on.
- **Unknowns.**
  - **K10:** Opt-in or opt-out (owner, §9).
  - **K11:** Where the endpoint lives and who can read it.
  - **K12:** The exact wording of the public page, which should ship **before** the ping.
- **Negative control.**
  - Capture the bytes on the wire with a proxy. The **subject** is the network, not a log line.
  - **(a)** The day changes, so `daily` fires again and `monthly` does not. The month changes, so
    `monthly` fires.
  - **(b)** ⭐ **With the setting off, zero requests reach the endpoint.** Brave shipped a bug where
    the ping kept going with the setting off (`brave-browser#45271`, filed 2025-04-07, **web**).
    That is exactly the check that catches it.
  - **(c)** A field-by-field diff of the request against the published page. An extra field fails.

---

## 6. Integration check (RELEASE_CYCLE §3.3)

### 6.1 The four privacy-perimeter gates

| Gate | Touched by | Risk | Guard |
|---|---|---|---|
| **Identity-key reveal** | P3 edits `get_public_key`, which holds this gate | Moving the `wants_identity_key` branch, or adding a second gate that fires first, could change identity behaviour | The new scoped grant goes **after** the identity-key branch and runs **only** on the derived path. `R-PERIM` must stay green. The P3 negative control asserts the identity path is unchanged. |
| **Key-linkage reveal** | Nothing in this track | — | — |
| **Sensitive cert fields** | P1 touches `certificate_handlers.rs` acquisition, not disclosure | Low: a different function. But a fail-closed verifier could **stop certificate acquisition** from a certifier that does not sign (K3) | Measure K3 before landing. `prove_certificate` is untouched. |
| **Over-cap spends** | P2's allow-list sits in front of `createAction`, `pay402` and `/wallet/broadcast` | Leaving a payment route off the list breaks paying dApps, and the **gold pill** with them | The allow-list is derived from the shim. P2's GREEN requires `createAction` plus the gold pill on a real site. `R-GOLD` runs at the boundary. |

### 6.2 The Rust permission engine (Matrix C)

- **P3** adds a caller of `dispatch_scoped_grant` from `getPublicKey`. **No new branch, no new
  `CallKind`, and the engine crate is unchanged**, since `ProtocolUse` already exists.
- The owner's "silence level 0 only if a higher grant exists" idea **would** change `matrix_c.rs`.
  It reverses the level-0 rule from **`10b2916` — level-0 protocols are open**. That is an owner
  decision, and it should wait until after P3's measurement.
- **P2** changes `domain_trust_mw` (branch 1's middleware), **not** `decide`.
- ⚠️ **Serialisation with B5-T2 (1Sat Ordinals).** T2 plans a **token-spend permission class** and
  BRC-99/165 basket scopes (README "Basket and permission mechanics"). Both touch
  `permission_service/`. ⇒ One track at a time in `request_gate.rs`.

### 6.3 What we touch that we did not write

- **The BRC-103/104 spec and two SDKs that disagree** on session binding. **MessageBox's server
  behaviour** (K1) and **certifiers' behaviour** (K3). If a third-party server does not sign
  correctly, our conformant client **stops working with it**, and that is outside our control.
- **The OS** for P4: socket ownership on Windows (`GetExtendedTcpTable` plus a token SID) and macOS
  (`LOCAL_PEERCRED` / `proc_pidinfo`). This is new API use in this repo, so working rule 4 applies.
- **A CDN or edge host** for P5, and our statement about what it logs.
- **V8 property attributes** (`V8_PROPERTY_ATTRIBUTE_DONTENUM`) for P2 item 3. That is new use here;
  read the CEF header first.

### 6.4 What we would have to un-ship if we are wrong

| If we are wrong about | We un-ship |
|---|---|
| MessageBox signing (K1) | PeerPay receive and send. The fix is reverted, or it ships with a per-host exception. ⛔ Never land P1 item 2 without a measured GREEN against the live MessageBox |
| The allow-list's completeness (K4/K5) | Any dApp that called a route we left off. Mitigation: 403 carries a named code, so the failure is visible |
| Derived-key prompt frequency (K6) | Prompt fatigue on dApps that fetch keys often. The owner accepted the trade on 2026-09-21 |
| The ping | ⛔ **Data already collected cannot be un-collected.** That is why the public page and the negative control come first |

---

## 7. Feasibility inputs (for G5.5)

⚠️ **All hours are the agent's estimates, not measurements.** "Owner-hours" means time at the
keyboard for human rows and decisions.

| Phase | Owner-hours | Unknowns | Human rows | Cross-track |
|---|---|---|---|---|
| **P1** Servers prove who they are | ~1.5 h: one PeerPay send and receive, one certificate acquisition, the well-known decision | K1–K3 | PeerPay end to end on the real MessageBox | None blocking. ⚠️ Invariant 3 decision for the well-known key |
| **P2** Allow-listed surface | ~1 h: a smoke test on 2–3 real dApps plus a BRC-121 paid page | K4, K5 | dApp smoke plus gold pill | **B5-T4 (402 payments)**: `pay402` must be on the list. **B5-T0 (engine)**: the `"Hodos"` `Sec-CH-UA` brand is the same "what names us" question |
| **P3** Derived keys | ~2 h: decisions (parts 2 and 3, level 0), a real-dApp login check | K6, K7 | Login on 2 real BRC-100 sites | **B5-T2**: serialise on `permission_service/` |
| **P4** One account, one wallet | ~2 h: a two-account rig on **both** platforms | K8, K9 | Two OS accounts, Fast User Switching | **macOS agent** owns half. Relay round needed |
| **P5** User count | ~2–3 h: opt-in/out decision, endpoint, public page wording, first-run notice review | K10–K12 | Proxy capture; the notice as seen by a user | **B5-T6 (browser shell)** hosts the settings switch and the first-run notice |

**Total, P1–P5:** about 8.5–9.5 owner-hours and **12 unknowns**. Recommended cut order within the
track if G5.5 says it is too big:
- Defer **P4 item 3** (per-user channel) first. It is already recommended.
- Then defer **P3 parts 2 and 3**.
- ⛔ **P5 cannot be cut.** It is D9, an owner decision.
- ⛔ **P1 should not be cut.** It is the owner's stated focus.

**Moving out of this track:** `two_next_address_index_sources…` → **B5-T1 Money path** (§4).

---

## 8. Prior-art rows, ready for `development-docs/PRIOR_ART.md`

| Date | Question | Source(s) read | What we learned | Verdict | Landed in |
|---|---|---|---|---|---|
| 2026-09-25 | What must a BRC-103 client verify about the server, and do the reference SDKs agree? | BRC-103/104 at `fb14783`; `@bsv/sdk` 2.8.7 `Peer.ts` (`40dad06`), `AuthFetch.ts`, `SimplifiedFetchTransport.ts`; go-sdk v1.6.0 `auth/peer.go` (`f810a16`), `authhttp.go` | ⭐ Both SDKs verify the `initialResponse` over `[ourNonce, theirNonce]`, `[2,'auth message signature']`, keyID `"<ours> <theirs>"`, counterparty = the **claimed** key, **before** trusting it. TS also verifies every response and, since 2.7.1 (`134e4ec`), binds later messages to the verified key. ⚠️ **Inference: Go does not bind.** BRC-104 has **no explicit client MUST** for responses, so response verification is SDK conformance, not spec text. The TS fallback to plain HTTP **refuses** a plain response carrying `x-bsv-*` headers | 🟢 | B5-T5 scope, P1 |
| 2026-09-25 | Does the reference wallet prompt for a derived `getPublicKey`, and does it bind keys to the origin? | `@bsv/wallet-toolbox` 2.14.2 `WalletPermissionsManager.ts` (`fc6ebef`), `Wallet.ts`; BRC-43, BRC-100, BRC index | ⭐ It **prompts at levels 1 and 2** (`seekPermissionsForPublicKeyRevelation`, default on), is **open at level 0**, and **binds nothing to the origin**: `_originator` is unused in derivation. **No BRC scopes keys to origins.** ⇒ Our part 1 is conformance. Part 3 (origin in the keyID) would be a **Hodos-only divergence** that changes keys users already rely on | 🟢 | B5-T5 scope, P3 |
| 2026-09-25 | Is `counterparty` optional on `createSignature`, and what is the default? | `@bsv/sdk` 2.8.7 `Wallet.interfaces.ts`, `ProtoWallet.ts` | Optional. **`createSignature` defaults to `anyone`**; HMAC, encrypt, decrypt, getPublicKey and verify default to `self`. ⚠️ wallet-toolbox's permission layer records `self` for signing while ProtoWallet signs as `anyone` (**inference**, a toolbox inconsistency) | 🟢 | `TICKET_createSignature_requires_counterparty.md` |
| 2026-09-25 | How do privacy browsers count active users without an ID? | Brave wiki "Brave's Capture of Usage Data" (edited 2026-03-02); `brave-core` `brave_stats_updater_params.cc`, `brave_stats_updater.cc`; Brave P3A wiki + STAR post (2022-07-19); Firefox `pings.yaml` (`usage-reporting`); Tor reproducible-metrics; Omaha `ServerProtocolV3.md`; Plausible data policy | ⭐ **Brave's usage ping sends no identifier.** The device computes daily, weekly, monthly and first flags from local dates, and the server counts flagged requests. The IP is stripped at the CDN, which means it is seen first. **P3A/STAR is for feature questions** and needs heavy infrastructure. **Firefox's "minimal" ping still sends a persistent `profile_id`.** Tor estimates from request counts. Omaha counts by server-issued date. ⚠️ Brave shipped a bug where the ping continued with the setting off (#45271) | 🟢 | B5-T5 P5 |

---

## 9. Open questions for the owner, each with a recommendation

| # | Question | Recommendation |
|---|---|---|
| Q1 | **A BRC-103 server that returns a response with no auth headers**: refuse, or accept as plain HTTP? | **Follow `@bsv/sdk` exactly.** Refuse a response that carries `x-bsv-*` headers without a valid signature. Allow plain HTTP only for a host that never spoke BRC-103. For MessageBox, which always speaks it, refuse. |
| Q2 | **`/.well-known/auth`**: delete it, or make it send the master key (what every SDK does)? | **Delete it**, if K2 shows no caller in a week of logs. Otherwise send the master key. ⛔ Do not keep the "app-scoped" key: it cannot be signed for. Either way this is invariant 3, so it is your call. |
| Q3 | **Merge `derivation_params_unbound_to_origin` into `derived_public_keys_have_no_prompt…`?** | **Yes.** Keep the newer ticket, fold in the older one's three unique points (§4), close the older one as a duplicate, and correct the "identity key always prompts" line. |
| Q4 | **Part 3: should the wallet add the site's address to the keyID itself?** | **Not in beta.5.** No BRC and no reference wallet does it. It would change keys for sites that already work, logins included. Carry it in the BRC draft (`originator-scoped-authentication-keys`) and adopt it only if the ecosystem does. |
| Q5 | **Part 2: log every derived key with the requesting site (a new `derived_key_cache` column)?** | **Defer.** It is a schema change (invariant 2), and its only use is a warning that part 1 mostly makes unnecessary. Revisit after P3's K6 measurement. |
| Q6 | **Level 0: stay open (matching wallet-toolbox), or silence it only when the site holds a higher grant?** | **Stay open in beta.5**, and state the residual plainly in the scope and the user docs: any two sites can agree on a level-0 key, in every BRC-100 wallet. Your idea reverses `10b2916` and needs measurement first. |
| Q7 | **`createSignature` without a counterparty: accept, with default `anyone`?** | **Yes.** It is conformance with the SDK. ⛔ Signing code, so it needs your approval. The negative control pins the default. |
| Q8 | **User count: opt-in or opt-out?** | **Opt-out, as Brave does, with three conditions.** (1) A first-run notice. (2) The public "every byte we send" page ships **before** the ping. (3) The off switch is proven by the zero-requests negative control. Opt-in at "low tens" of users produces a number too small to mean anything. ⚠️ This is the one place the product's privacy promise is at stake, so it is your call, not a default. |
| Q9 | **User count: send install week (`woi`), which gives retention cohorts?** | **No, not at our size.** One install week plus platform plus version could narrow to a handful of people. Revisit when weekly installs are in the hundreds. |
| Q10 | **Move `two_next_address_index_sources…` to B5-T1?** | **Yes.** It edits the same counter as T1's rescan ticket. |
| Q11 | **Keep `dapp_reachable_surface…` in T5, or move it to T1 as its ticket suggests?** | **Keep it in T5**, beside the bridge-plumbing ticket. Both are "what an approved site can reach", and the top harm is privacy. |
| Q12 | **Per-user wallet channel (named pipe / Unix socket).** | **Defer to beta.6.** Ship the floor (never adopt or stop another account's backend) in beta.5. |
