# B5-T5-P2 — An approved site reaches only the dApp surface · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (copied 2026-09-28 from `../../../0.4.0-beta.3/PHASE_CONTRACT_TEMPLATE.md`, read-only).
> Written at G3 by the T5 track agent (relaunch). ⛔ Documents only — no code, no schema, no issue.

**Track:** B5-T5 Identity & privacy · **Tickets:** `../../tickets/TICKET_dapp_reachable_surface_is_a_denylist_not_an_allowlist.md`, `../../tickets/TICKET_wallet_bridge_plumbing_is_advertised_to_every_site.md` (items 1–3), `../../tickets/TICKET_loopback_host_form_wallet_routing.md` (rows **W4 / W6 / W7 / W8** only — assigned to T5 at G2), `../../tickets/TICKET_deleting_a_site_in_the_advanced_wallet_does_not_reach_the_browser.md` (T5 in the register, unphased in the scope — placed here, see §12 Q4) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), T5 track agent, repo head `0777b26` on `0.4.0` · **Platforms:** both (Rust middleware is shared; the render-process and IPC changes are shared C++ with no platform split, but macOS rebuilds and re-runs the page probe) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** **T5 Q11** (the denylist ticket stays in T5, beside the bridge-plumbing ticket) · G2-owed item 1 (loopback leftovers W4/W6/W7/W8 → T5) · T5 SCOPE §6.1 (the allow-list is derived from the shim, and P2's GREEN requires `createAction` + the gold pill on a real site). No invariant-2/3 change in this phase: no schema, no signing.

---

## 1. Goal

A site the user approved can call only the wallet methods a dApp legitimately uses — never the wallet's internal routes (addresses, balance, settings, shutdown, unlock) — and an ordinary web page stops naming Hodos through globals no dApp asks for; revoking a site in the advanced wallet actually makes the browser ask again.

## 2. Done means

**Part A — the surface (tickets 1, 2, 4):**
- [ ] `domain_trust_mw` refuses, on **every** verb, any path not on a declared external allow-list, for a request carrying `X-Requesting-Domain`; the refusal happens before `next.call` (no handler log line). `/wallet/addresses`, `/wallet/settings` (GET), `/wallet/balance`, `/shutdown`, `/wallet/unlock` from an approved origin ⇒ 403 with a named code (`P2-A1`).
- [ ] The allow-list is **derived**, not invented: the 28 BRC-100 routes + `/getVersion` + discovery (`/health`, `/.well-known/auth`) + the shim's `/wallet/*` routes (`address-to-script`, `bsv-price`, `encrypt-bie1`, `decrypt-bie1`, `yours-legacy-addresses`) + the C++-stamped `/wallet/pay402` + the self-revoke `DELETE /domain/permissions?domain=<own>`. Each entry cites its caller in a comment. K4 answered before landing (`P2-A0b`).
- [ ] `is_permission_surface` stays underneath, unchanged, and still returns `permission_table_is_first_party_only` for its paths (`P2-A3`).
- [ ] `createAction` + the **gold pill**, and one BRC-121 paid retry, still work on real sites (`P2-A2`).
- [ ] `window.hodosBrowser` is not defined on an external page; `CWI` / `yours` / `panda` still are (`P2-A5`).
- [ ] `__hodos_walletCall`, `__hodos_walletResponse`, `__hodos_walletResponseChunk` and `cefMessage` are non-enumerable on external pages and still callable (`P2-A6`). The write-up says **enumeration is defeated, name-probing is not** (ticket's own wording).
- [ ] A web page's `cefMessage.send` reaches only the four allowed names — confirmed by RED, not by reading (`P2-A7`).
- [ ] Deleting a site under advanced wallet → Approved sites makes the next visit **prompt**; a web page in a tab still cannot send `domain_permission_invalidate` (`P2-A8`).

**Part B — loopback leftovers (W4 / W6 / W7 / W8):**
- [ ] **W6:** an IPC or request whose origin cannot be resolved is treated as **external-and-unapproved**, never as internal (`IsInternalOrigin("")` no longer returns `true` on any decision path), and the Rust wallet refuses a request whose `Host` is not a loopback literal on its own port (DNS-rebinding half, ticket §6.3) (`P2-A9`).
- [ ] **W4:** page-supplied headers can never set a trust header Rust reads; the shape (keep today's exact-strip denylist vs a forward allow-list) is the owner's call — §12 Q2 — because the code's own P0.5 comment argues *against* the ticket's allow-list (`P2-A10`).
- [ ] **W7:** every remaining whole-URL `find("localhost")` / `find("127.0.0.1")` gate is either migrated to the parsed-authority predicates in `PortConfig.h` or listed with a reason it is not trust-bearing. ⭐ The privacy-bearing ones found today (adblock skip, cookie-block skip, farbling-seed skip, scriptlet pre-cache skip) are migrated (`P2-A11`).
- [ ] **W8:** `hodos::IsWalletHostPort` and `hodos::LegacyWalletGateMatch` (the W3 shadow) are deleted once W3's shadow log has been read and shows no unexplained `new=no old=yes` line (`P2-A12`).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-INTEXT` ⭐ | Internal never prompts, external always gates | W6 changes the one function (`IsInternalOrigin`) that decides internal vs external, and `runIpcCallDirect` relies on "internal ⇒ header-free". Getting W6 wrong in one direction breaks the wallet UI (it starts 403ing/prompting itself); in the other it grants a web page internal trust. Run at the boundary, **both** halves |
| `R-GOLD` | Gold pill on every auto-approved payment | The allow-list sits in front of `createAction` and `/wallet/pay402`; leaving either off breaks payment **and** the pill. `P2-A2` is a real-money row |
| `R-PERIM` | Four perimeter gates per `matrix_c.rs` | Not edited, but every perimeter route (`/getPublicKey`, key linkage, `/proveCertificate`, payment routes) must be on the allow-list or the gate becomes an unconditional 403 — which *looks* safe and silently breaks the perimeter's Prompt path. T1 + one T2 identity-key prompt |
| `R-PEERPAY-DELIVERY` Half 1 | PeerPay delivers or never leaves | `/wallet/peerpay/*` goes **off** the external list — correct (no dApp calls it), but the wallet UI must still reach it header-free. Half 1 at the boundary |
| `R-CLOSE` | Overlay close/lifecycle | `domain_permission_invalidate` arm edited (`P2-A8`); `overlay_close` must stay in the internal-only set |
| Invariant 1 | Keys never in JS | `/wallet/reveal-mnemonic` stays refused (deny-list **and** allow-list) |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. Rows marked ⏳ are trust-boundary rows on the money path (they decide whether a web page can reach fund-moving or spend-gating routes): their RED is designed by a second agent (`../../../RELEASE_CYCLE.md` §4.2).

**Step 0 — reproduce on today's build first (measurement).** From an **approved** scratch https origin on the `HODOS_DEV=1` build, `window.__hodos_walletCall('x','/wallet/addresses',{},'GET')`, the same for `/wallet/settings` (GET), `/wallet/balance`, and last `/shutdown`. Expected today: 200 + address list, 200 + settings, 200 + balance, wallet process gone. If any refuses, the ticket is wrong and is corrected before code (ticket "RED — run this first").

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P2-A0` | Step 0 recorded: the four calls' statuses on today's build + whether the wallet pid survived `/shutdown` | This row **is** the RED baseline for `P2-A1` (recorded first, per `REGRESSION_ADDITIONS.md` "run it once before") | `debug_output-<pid>.log` handler lines (`📋 GET /wallet/addresses called` etc.) + `R-INTEXT trust: … requesting_domain=<scratch host>` in the wallet log — proves the request reached Rust **as external**. `/shutdown`: wallet pid via `Get-CimInstance` path-matched (never by name — root `CLAUDE.md`) | T2 | ⬜ |
| `P2-A0b` | K4/K5 answered: the list of every path an external caller legitimately reaches, from (i) `CWIShimScript.h` (`METHODS` + every `/wallet/` literal), (ii) every C++ site that stamps `X-Requesting-Domain` (`runIpcEngineCascade`, `AsyncWalletResourceHandler::startAsyncHTTPRequest`, the three `resume*Response` re-issues, the BRC-121 `/wallet/pay402` call), (iii) 2–3 demo dApps' network logs. Any path in (iii) not in (i)/(ii) ⇒ §12 | Measurement row. Its control: the derivation script run against a **seeded** copy of the shim with one extra `/wallet/zzz` literal must list `/wallet/zzz` (shows the extractor can see a new route) | The script's output checked into the phase folder beside this contract; the dApp list names the sites | T0/T1 | ⬜ |
| `P2-A1` | From an approved origin, every path **not** on the allow-list ⇒ 403 `{"error":"not_on_dapp_surface","endpoint":…}` on GET, POST and DELETE, and the handler does not run; the wallet pid survives `/shutdown` | (i) Remove the allow-list check ⇒ `GET /wallet/addresses` from the approved scratch origin ⇒ `📋 GET /wallet/addresses called` (Step 0 shape) ⇒ red · (ii) POST/DELETE-only allow-list (the deny-list's verb shape) ⇒ `GET /wallet/settings` runs ⇒ red on the GET column · (iii) raw-path-only compare ⇒ `/wallet/%61ddresses` runs ⇒ red · (iv) the transport the allow-list cannot see: from an https page, a `text/plain` POST and a `no-cors` GET straight at `127.0.0.1:<port>` for paths `HttpRequestInterceptor::isWalletEndpoint` does not intercept (`/shutdown`, `/admin/prepare-unpublish`, `/domain/permissions/all`) ⇒ they arrive header-free, which `domain_trust_mw` treats as first-party ⇒ GREEN only if each ends at CORS (`block_on_origin_mismatch` 400 in the actix Logger line) or never reaches its handler; a `no-cors` GET carries no `Origin`, so it reaches the handler — see §4a · Right reason: handler line absent **and** the refusing layer's own line present; record whether Chromium's Local Network Access blocked the request in the renderer (then the red measured Chromium, not us) · Residue: a red on `/shutdown` stops the **dev** wallet on `31401` only — restart with `dev-wallet.ps1`; never target `31301` — designed by controls-F (Opus), 2026-09-28 | Absence of the handler's own log line **and** presence of the middleware's refusal line naming the path + origin — ⛔ not the status the page sees (a CORS 403 and a gate 403 look identical to the caller, ticket "SUBJECT") | T1 (predicate unit over the full route table: every route classified, test fails on an unclassified route) + T2 | ⬜ |
| `P2-A2` 👤 | Real sites: a `createAction` under caps completes silently **and the gold pill fires on the originating tab**; one BRC-121 paid page pays and renders; one identity-key prompt (perimeter) still appears on a site without the V17 toggle | (i) Allow-list derived from the shim only (omit C++-stamped `/wallet/pay402`) ⇒ BRC-121 page ⇒ `not_on_dapp_surface` for `/wallet/pay402` in the Rust log, no payment, no pill ⇒ red — proves the list measured is the derived one · (ii) omit `/getPublicKey` ⇒ the identity-key call gets 403 instead of `Prompt{identity_key_reveal}` in `audit.rs` output ⇒ red (the "looks safer" failure) · (iii) allow-list that refuses everything ⇒ `createAction` 403, no `OnWalletCallSuccess fired` ⇒ red (pairs with A1) · Right reason: the refusal line names the path; under (i) no `pay402` `transactions` row exists (refused before mint) · Residue: GREEN spends cents on real sites (intended); REDs spend nothing — a `transactions` row under (i) is a finding — designed by controls-F (Opus), 2026-09-28 | `OnWalletCallSuccess fired (cefBrowserId=… -> tabId=…)` + the chain txid; for BRC-121 the `firePaymentSuccessIpc` line; for the prompt the Rust `PermissionDecision` = `Prompt{identity_key_reveal}` in `audit.rs` output | T2 real money, cents | ⬜ |
| `P2-A3` | `is_permission_surface` unchanged; `POST /domain/permissions` and `POST /domain/%70ermissions` from an approved origin still ⇒ 403 `permission_table_is_first_party_only`; self-revoke `DELETE /domain/permissions?domain=<own>` still ⇒ 200 | Build with the **new allow-list check removed**: the deny-list alone still refuses both POSTs (proves the lower layer is independent, not shadowed); build with **both** removed ⇒ the POST reaches `set_domain_permission` (row changes in a scratch DB) | The `🛡️ REFUSED … first-party only` warn line vs the new refusal line — each layer's own line, so it is visible **which** layer refused | T1 + T2 | ⬜ |
| `P2-A4` | `GET /wallet/settings` from an approved origin ⇒ 403 (closed by construction because the allow-list is verb-blind) | Today's build ⇒ 200 with the settings object (`P2-A0`) | `wallet_settings_get`'s log line absent after; present in `P2-A0` | T2 | ⬜ |
| `P2-A5` | On `https://example.com/` (subject asserted `role=tab_<n>`), `Object.getOwnPropertyNames(window)` lists `CWI` and **not** `hodosBrowser`; on `http://127.0.0.1:5137/` (internal) `window.hodosBrowser.platform` is still `"windows"`/`"macos"` | Revert the two-line move ⇒ the external probe lists `hodosBrowser` again; delete the shim injection in a scratch build ⇒ `CWI` disappears (shows the probe can see a provider vanish, not only appear) | `development-docs/0.4.0-beta.3/phase-13-bot-detection/p13_signals.py` `hodosGlobals` with `assert_tab` — ⛔ it once read an **overlay** (ticket "Evidence"). Plus the React header still renders platform chrome (internal half) | T2 | ⬜ |
| `P2-A6` | External page: `__hodos_walletCall`, `__hodos_walletResponse`, `__hodos_walletResponseChunk`, `cefMessage` absent from `Object.getOwnPropertyNames(window)` and `for…in`; `'__hodos_walletCall' in window` still `true` (stated, not hidden); a `CWI.getVersion()` and a large-response call (chunk path) still resolve | Set `enumerable: true` / drop `V8_PROPERTY_ATTRIBUTE_DONTENUM` ⇒ probe lists them; break the name the C++ side calls (rename `__hodos_walletResponse` in a scratch build) ⇒ `getVersion` hangs — proves the delivery check measures delivery | Same probe; the render log line `if (window.__hodos_walletResponse) {` path delivering, and the page promise resolving with a value | T2 | ⬜ |
| `P2-A7` | From `https://example.com/` a page calling `cefMessage.send(n,[])` for `n` ∈ {`overlay_close`, `domain_permission_invalidate`, `wallet_prevent_close`, `settings_set`, `add_domain_permission`} ⇒ each `🛡️ IPC DENIED: '<n>' from external origin` and no arm runs; `wallet_call` passes the gate | Dev build with `IpcMessageAllowedFromWebPage` forced to `true` ⇒ `overlay_close` reaches its arm (log line of that arm) — proves the probe exercises the gate, not a renderer-side failure | Browser-process log (`debug_output-<pid>.log`), **not** the page: the page gets no reply either way | T2 | ⬜ |
| `P2-A8` | Approve scratch site S → delete S in advanced wallet → revisit S ⇒ connect prompt appears; no `IPC DENIED (P0.5-B1): domain_permission_invalidate` line; no `🔁 Stale connect prompt … re-sending` line. **Guard:** a web page self-navigated or in a tab sending `domain_permission_invalidate` is still DENIED | Today's build ⇒ the `IPC DENIED` line + the "re-sending" line (reproduce first — ticket is log reading only, two occurrences). Guard RED: the chosen fix with its origin/role check removed ⇒ a web page's send reaches `invalidateDomainPermissionCache` | The shell's own log for the invalidate arm (`🔐 Invalidated cached permission for: <S>`) and the **user seeing the prompt**; guard: the DENIED line naming the web page's role | T2 (human sees the prompt) | ⬜ |
| `P2-A9` | **W6.** (a) Every `IsInternalOrigin` decision site treats `""` as external; the wallet UI (internal) still completes a send with no modal. (b) Rust refuses a request whose `Host` is not `127.0.0.1:<port>` / `localhost:<port>` / `[::1]:<port>` with 421/403 before any handler | (a) Origin-less shapes on **both** transports, from the approved scratch origin: IPC — `about:blank`, `srcdoc`, sandboxed (opaque), `data:` and `blob:` child frames calling `__hodos_walletCall` (parent reaching `frames[0]` where same-origin); HTTP — a `window.open('')` about:blank popup whose `fetch` hits `/wallet/balance` (the direct-fetch path stamps from the main frame's URL) ⇒ Rust must log the parent's domain or refuse, never `<none:internal>` · Switch: restore `IsInternalOrigin`'s `if (origin.empty()) return true;` ⇒ the popup logs `<none:internal>` ⇒ red · Also a page from an unrelated local server (`python -m http.server 8000`, origin `localhost:8000`) — today `IsInternalOrigin` accepts any port (see §4a) · (b) raw-socket requests (not a browser — Chromium rewrites `Host`) with `Host:` `rebind.example:31401`, `LOCALHOST:31401`, `localhost.:31401`, `127.1:31401`, `0.0.0.0:31401`, `[::ffff:127.0.0.1]:31401`, absent, and the right host with the other port ⇒ refused before any handler; switch = remove the check ⇒ `rebind.example` reaches the handler ⇒ red; two-sided: the shell's re-issue with `127.0.0.1:31401` (dev) and `:31301` (release build) still passes · Right reason: target a read-only route (`/health`, `/wallet/balance`), never `/shutdown` · Residue: none — designed by controls-F (Opus), 2026-09-28 | (a) `R-INTEXT`'s subject — the Rust `R-INTEXT trust:` line, `<none:internal>` only for the wallet UI; (b) the refusal line + absent handler line for a raw-socket request with `Host: rebind.example:31401` | T1 + T2 | ⬜ |
| `P2-A10` | **W4.** A page-supplied `X-Requesting-Domain: ""`, `X-User-Approved`, `X-Payment-*`, `X-Bsv-Price-Available`, `X-Cert-Approved-Fields` never reach Rust from the page on either transport (IPC and direct fetch) — shape per §12 Q2 | Page supplies each trust header in variants — exact, lower-case, UPPER-CASE, duplicated, leading-space value, empty value — on (1) direct fetch and (2) the IPC bridge (if `__hodos_walletCall` accepts caller headers at all, say so; if not, (2) is closed by construction and stated) · Switch (i): drop one name from `kTrustHeaderExact` ⇒ Rust logs the page's value ⇒ red · (ii) case-sensitive compare in the strip ⇒ the lower-case variant reaches Rust ⇒ red · ⚠️ `X-User-Approved` must be forged with a **live** approvalId the page obtained from a real 202 envelope — a random id is refused by Rust anyway and cannot show the strip missing · Right reason: the carrier request is one the approved origin may legitimately make (`/getVersion`, `/createSignature`) so nothing else refuses it first · Residue: none — designed by controls-F (Opus), 2026-09-28 | Rust's received header set logged at debug for the test request (names only, values of trust headers only) — the assertion is what **Rust** received | T1 + T2 | ⬜ |
| `P2-A11` | **W7.** Inventory of every whole-URL `find("localhost"/"127.0.0.1")` gate, each classified trust-bearing / privacy-bearing / cosmetic. The privacy-bearing ones — `AdblockCache.h :: shouldSkipAdblockCheck`, `CookieBlockManager::CanSendCookie` (+ its three siblings), `EphemeralCookieManager`, the farbling-seed skip in `simple_handler.cpp :: OnBeforeBrowse`, the scriptlet pre-cache skip in `simple_render_process_handler.cpp :: OnContextCreated` — use the parsed authority. `https://<tracker>/pixel?r=localhost` is **blocked** when its host is on a filter list | Today's build: the same tracker URL with `?r=localhost` appended is **not** blocked while the bare URL is (predicted from reading — reproduce first); revert one gate ⇒ that case passes through again | The adblock engine's block log / `AdblockCache` per-browser blocked count for that request, and the cookie jar for the cookie case — not "the page looked clean" | T1 (predicate unit) + T2 | ⬜ |
| `P2-A12` | **W8.** W3's shadow log (`🔀 P5 gate disagreement`) read across ≥ 1 week of the owner's dev use: every line explained; then `IsWalletHostPort`, `LegacyWalletGateMatch` and the shadow block are deleted, and `tests/wallet_origin_test.cpp`'s legacy-control assertions are rewritten against a frozen copy **inside the test** (so the controls keep their red) | Before deletion, a `new=no old=yes` URL fed to the test must fail the new predicate's assertion (shows the controls still bite after the rewrite) | `grep -rn "IsWalletHostPort\|LegacyWalletGateMatch" cef-native/{src,include}` = 0; the test binary's output with the frozen legacy copy | T0 + T1 | ⬜ |
| `P2-A13` | Boundary: `R-INTEXT` both halves, `R-GOLD`, `R-PERIM` T1 + one T2, `R-PEERPAY-DELIVERY` Half 1, `R-CLOSE` | Per `REGRESSION_SET.md` | Per `REGRESSION_SET.md` | T1–T2 | ⬜ |

**Two-sided rows:** `P2-A1` (refuse the internal route) ⇄ `P2-A2` (accept the dApp route) — an allow-list that refuses everything passes A1 and fails A2. `P2-A5`/`P2-A6` each carry "provider still present". `P2-A8` pairs "our wallet tab may invalidate" with "a web page may not". `P2-A9` (a) pairs with `R-INTEXT` half (a).

### 4a. Independent control notes (2026-09-28)

- `P2-A1` — ⭐ **The allow-list only sees header-bearing requests.** Paths `HttpRequestInterceptor::isWalletEndpoint` does not intercept (`/shutdown`, `/admin/prepare-unpublish`, `/domain/permissions`, `/domain/permissions/{all,certificate,protocol,basket,counterparty}`) reach Rust **header-free** when a page calls `127.0.0.1:<port>` directly, and `domain_trust_mw` reads header-free as first-party (the T6-P5-A9 finding). Code reading: `block_on_origin_mismatch(true)` answers 400 on an Origin mismatch before the handler, which covers POST/DELETE (browsers always send `Origin` there), but a **`no-cors` GET carries no `Origin`** and reaches the handler. Today's un-intercepted GETs are the `/domain/permissions*` reads (read-only; response opaque to the page). Also: the CORS list admits `http://localhost` and `http://127.0.0.1` on port 80, so a page served by any local port-80 server passes CORS. Suggested: the T1 route classification gains a column "reachable header-free from a page?", and the GREEN asserts every such route is a side-effect-free GET or CORS-blocked. Not measured.
- `P2-A9` — `IsInternalOrigin` also returns `true` for **any** `localhost:<port>` / `127.0.0.1:<port>` origin, so a page from an unrelated local dev server is internal (header-free) — the GREEN speaks only of `""`. Say whether W6 covers it or it is a recorded residual.
- `P2-A9` / `P2-A10` — `extractDomain` reads the **main frame's** URL (out of scope, §6): on the direct-fetch transport an unapproved cross-origin iframe inside an approved page is stamped with the approved domain. A10's "what Rust received" can be right for the wrong frame — name the frame in the SUBJECT.
- `P2-A10` — the `X-User-Approved` forgery only shows a missing strip if it uses a **live** approvalId (now in the RED); a random value is refused by Rust for its own reason.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/main.rs :: domain_trust_mw` | ✅ | Header-free ⇒ `next.call` ungated; header present ⇒ deny-list block (POST‖DELETE only), then `domain_trust_gate`. The allow-list goes **between** the header check and the deny-list block, reusing `raw_path` + `decoded_path` |
| `rust-wallet/src/main.rs :: domain_trust_mw :: is_permission_surface` (inner fn) | ✅ | Four classes of prefixes; kept unchanged (`P2-A3`) |
| `rust-wallet/src/main.rs` — `Cors::default()…block_on_origin_mismatch(true)` | ✅ | Ticket §6.2 is **already fixed** (P0.5-C1). Not edited |
| `rust-wallet/src/main.rs` — route table (111 `.route(` + 3 `resource(`) | ✅ | Source for the allow-list classification test. ⚠️ Found: the shim's `window.yours.broadcast` fallback calls `/wallet/broadcast`, which is **not a routed path** (only `/wallet/broadcast-nosend` and `/wallet/debug/broadcast-nosend`) — the fallback always 404s today. Reported, not fixed here (§6) |
| `rust-wallet/src/permission_service/request_gate.rs :: domain_trust_gate` | ✅ | `if trust == "approved"` ⇒ Proceed. Not edited |
| `rust-wallet/src/handlers.rs :: shutdown`, `get_all_addresses`, `wallet_balance`, `wallet_settings_get`, `wallet_unlock` | ✅ (routes at `main.rs` route table) | The Step 0 subjects |
| `cef-native/include/core/CWIShimScript.h :: WALLET_CALL_BRIDGE_SCRIPT`, `METHODS`, the `/wallet/*` literals | ✅ | `__hodos_walletResponse` / `…Chunk` / `__hodos_walletCall` assigned as plain `window.x = function` — the non-enumerable change is here |
| `cef-native/src/handlers/simple_render_process_handler.cpp :: OnContextCreated` — `global->SetValue("hodosBrowser", …)` and `platform`; `global->SetValue("cefMessage", …, V8_PROPERTY_ATTRIBUTE_READONLY)`; the `isInternalPage \|\| isOverlayBrowser` gate; the external `WALLET_CALL_BRIDGE_SCRIPT` + `CWI_SHIM_SCRIPT` injection | ✅ | `hodosBrowser` is set **before** the gate; `isInternalPage` now uses `hodos::IsInternalFrontendUrl` (the `:5137` substring gate of ticket §13 is fixed). `V8_PROPERTY_ATTRIBUTE_DONTENUM` is new use here — **rule 4: read `cef_v8.h` first and cite it** |
| `cef-native/src/handlers/simple_handler.cpp :: IpcMessageAllowedFromWebPage`, `OnProcessMessageReceived` (C2 gate, P0.5-B1 gate, the `domain_permission_invalidate` tab-role deny and its arm) | ✅ | Four names: `wallet_call`, `cosmetic_class_id_query`, `find_result_js`, `qr_found` |
| `cef-native/include/core/IpcAuth.h :: IsTabRole`, `IsApprovalOverlayRole`, `IsGrantApproveMessage` | ✅ | The advanced wallet is a tab ⇒ denied today |
| `cef-native/src/core/HttpRequestInterceptor.cpp :: IsInternalOrigin` | ✅ | `if (origin.empty()) return true;` — **still open**, the W6 hinge (`REGRESSION_SET.md` R-INTEXT ⚠️) |
| `cef-native/src/core/HttpRequestInterceptor.cpp :: runIpcCallDirect`, `runIpcEngineCascade`, `HandleIpcWalletCall` | ✅ | Internal path stamps no header only when `IsInternalOrigin(origin)`; W6 must keep the wallet UI internal |
| `cef-native/src/core/HttpRequestInterceptor.cpp :: AsyncWalletResourceHandler::startAsyncHTTPRequest` — `kTrustHeaderExact` strip | ✅ | Comment: *"⛔ THIS IS A DENYLIST, AND IT MUST STAY ONE"* — BRC-31 `x-authrite-*` / `x-bsv-*` forwarded deliberately. Conflicts with W4 as filed ⇒ §12 Q2 |
| `cef-native/src/core/HttpRequestInterceptor.cpp :: HttpRequestInterceptor::isWalletEndpoint` | ✅ | Admits any path containing `/wallet/` on the direct-fetch transport — so the **Rust** allow-list is the single control for both transports |
| `cef-native/include/core/PortConfig.h :: IsWalletHostPort`, `LegacyWalletGateMatch`, `IsLoopbackHost`, `IsWalletOrigin`, `IsOurWalletOrigin`, `IsInternalFrontendUrl` | ✅ | `IsWalletHostPort`'s only non-test caller is `LegacyWalletGateMatch`, whose only caller is the W3 shadow block in `simple_handler.cpp :: GetResourceRequestHandler` (W8 is small) |
| `cef-native/include/core/AdblockCache.h :: shouldSkipAdblockCheck`; `cef-native/src/core/CookieBlockManager.cpp :: CanSendCookie` (+ 3 sibling sites); `cef-native/src/core/EphemeralCookieManager.cpp`; `simple_handler.cpp` farbling-seed block in `OnBeforeBrowse`; `simple_handler.cpp` PaidContentCache skip; `simple_render_process_handler.cpp :: OnContextCreated` scriptlet skip; `TabManager.cpp` / `TabManager_mac.mm` NTP checks | ✅ | W7 inventory. ⚠️ **Predicted (reading only):** a URL whose **query** contains `localhost` or `127.0.0.1` skips adblock, third-party-cookie blocking and the farbling seed — privacy fail-open |

Not touched: `hodos_permission_engine` (P2 changes middleware, not `decide`), the schema, any signing code.

## 6. Out of scope

- BRC-179 manifest / fingerprint (ticket: separate decision, implemented by nobody).
- Rate-limiting `POST /wallet/unlock` — own defect; after this phase it is unreachable from web content, which removes the external half.
- Removing `/shutdown` / `/admin/prepare-unpublish` as routes.
- The `window.yours.broadcast` → unrouted `/wallet/broadcast` fallback (found above) — a new ticket at close (kaleidoscope-close rule), not fixed here.
- The per-user wallet channel / "headerless means trusted" for **non-browser** callers — P4 / beta.7 (T5 Q12).
- `extractDomain()` discarding the scheme and reading the main-frame URL (loopback ticket §6.4) — noted, not in W4/W6/W7/W8.
- Tempted by: deleting the dead `HttpRequestInterceptor.cpp :: extractProtocolScope` (no caller since 2.6-G) — **reported** in the phase report, not deleted (rule 3).

## 7. Rollback

Part A and Part B land as separate commits (allow-list · globals · revoke · W6 · W4 · W7 · W8); each `git revert`s alone. No schema, no data.

## 8. Pre-mortem (adversarial review — before)

| Story: it shipped and failed because… | Row that catches it |
|---|---|
| The allow-list missed a route a real dApp calls (e.g. `/wallet/pay402`, stamped by C++ not the shim) — paying sites break and the gold pill never fires | `P2-A0b` derivation includes C++ stamp sites; `P2-A2` real money |
| A perimeter route was left off, so the identity-key gate became an unconditional 403 — looks "safer", silently breaks sign-in | `P2-A2` identity prompt half; `R-PERIM` T2 |
| The allow-list compared the raw path only, so `%2F`/`%70` spellings slipped past (the 2026-08-20 bypass) | `P2-A1` T1 unit runs every route in raw **and** percent-encoded form |
| W6 made `""` external, and the wallet UI's own calls (header-free by design) started 202-ing — every send prompts | `P2-A9` (a) + `R-INTEXT` half (a) |
| W6's Host check refused the C++ re-issue whose `Host` is `127.0.0.1:31401` in dev — every dApp breaks on dev only | `P2-A9` (b) runs on **both** ports; the dev build is the one tested |
| Non-enumerable `__hodos_walletResponse` was defined in a way C++'s `window.__hodos_walletResponse(...)` lookup no longer resolves (e.g. moved into a closure) — every wallet call hangs | `P2-A6` delivery half incl. the chunk path |
| The revoke fix let **any** internal-origin tab invalidate, re-opening P0.5-B1's self-navigation hole | `P2-A8` guard half |
| W7 migration narrowed a gate so internal traffic got ad-blocked / cookie-blocked and the wallet UI broke | `P2-A11` + `R-INTEXT` half (a) + minimal site basket |
| The test drove an overlay, not a tab (the 0.4.0 CDP trap) | `P2-A5`/`A6` SUBJECT: `assert_tab` + `role=tab_<n>` |

### 8a. Prior art to read at kickoff (added 2026-09-29, from the owner's morning report)

⛔ **A reading assignment, not a scope change.** Read the diffs before `P2-A6` and the non-enumerable change are designed; record in `../../../PRIOR_ART.md` what we follow and what we do not (rule 5).

**`bsv-blockchain/bsv-browser` `af3fdbd...baf14a0`** — the BSV mobile browser hardening its injected `window.CWI`, the same surface as our `CWIShimScript.h`. Range verified 2026-09-29: 11 commits, 2026-09-27 → 29; the webview ones are in `utils/webview/{cwiProvider,documentStartScript,messageSizeCeiling,walletEnvelope,walletResponseScript}.ts`, each with a test file. Only commit messages have been read so far.

| Their commit | What it says | Where it lands here |
|---|---|---|
| `de88db6` | The provider accepted any message whose id matched; it now trusts only bridge-delivered replies and uses **crypto-random** request ids | ⭐ Our shim uses **sequential** ids (`String(nextId++)`), and `window.__hodos_walletResponse` is a plain writable global that any script in the page can call with a guessed id. ⚠️ **Their hole was cross-window message events. Ours is probably narrower:** a cross-origin frame cannot call a function on our window, and a script already in the page can replace `CWI` anyway. Decide at kickoff whether (a) random ids and (b) a response function the page cannot overwrite or call belong beside `P2-A6`'s non-enumerable change — making it non-enumerable does not stop anyone calling it. Record the answer either way |
| `ae13035`, `baf14a0` | An oversize request was dropped silently and the page waited forever. It now gets an error envelope, and oversize responses are refused rather than injected | Our chunk path (`__hodos_walletResponseChunk`, 30 s stall timer) covers large **responses**. Check what an oversize **request** from the page does today: an error, or a hang |
| `b7e3b69`, `dddf13e` | `getVersion` answered in the page (the SDK allows one second for discovery), but **only in frames where replies can land**. Answering it in a cross-origin iframe let discovery pass and the first real call hang | Our shim injects only in the main frame (ticket gating cascade), so the iframe trap should not apply. Confirm it, and note whether we have the one-second discovery problem at all |
| `18c7b54`, `67a9e8c` | Canonical BRC-100 envelopes: `status: 'success'` (not `'ok'`), integer error codes 1–255, `WERR_*` names mapped to their assigned codes so the SDK keeps the message | Belongs mainly to **B5-T1-P2 — a failure says it failed** (its §8a). Here, only as a check that the allow-list's refusal body does not add a third error shape |
| `1fc9ddf` | A string response was treated as pre-serialised JSON and embedded as script; that path is now separate so a string is always serialised, never executed | Our C++ delivers through `escapeJsonForJs` (`JsStringEscape.h`, the F6 fix). Worth confirming every delivery site goes through it |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | Dev wallet on `31401`; page probe via CDP on a **tab** role |
| macOS | `P2-A1`, `A2` (one createAction + pill), `A5`, `A6`, `A7`, `A8`, `A9`, `A11` | Same C++ files, no `#ifdef` split expected. ⚠️ Shared C++ touched (`simple_render_process_handler.cpp`, `simple_handler.cpp`, `HttpRequestInterceptor.cpp`, `PortConfig.h`, `AdblockCache.h`, `CookieBlockManager.cpp`, `EphemeralCookieManager.cpp`, `TabManager_mac.mm` if touched) ⇒ **relay round** naming them (root `CLAUDE.md` 2026-09-12 rule). The mac `WALLET_CALL` path and overlay roles must be re-checked, not assumed |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1.5 h**: real-site smoke on 2–3 dApps incl. one BRC-121 paid page and the gold pill (`P2-A2`, ~50 min, cents), the revoke-and-revisit check (`P2-A8`, ~10 min), reading the §12 answers (~20 min), Part B's shadow-log read-out (~10 min) |
| Human-bound rows | `P2-A2` (real money + seeing the pill), `P2-A8` (seeing the prompt return) |
| Unknowns (K) — uncertainty, not difficulty | **K4** the complete legitimate endpoint set · **K5** whether any real dApp calls a `/wallet/*` route directly · **K19 (new)** which internal callers today reach `IsInternalOrigin` with an empty origin (W6's blast radius — measured by a debug log before the flip) · **K20 (new)** whether any real site depends on the tracker-with-`localhost`-in-query fail-open (expected none) |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | **B5-T0 Build 1** | Browser-level evidence (`P2-A2`, `A5`–`A8`, `A11`) is gathered once, on the new engine |
| shares | **B5-T0-P4** (`"Hodos"` Sec-CH-UA brand) | Same "what names us" family, opposite direction. ⭐ T0-P4's local **echo page** can record `P2-A5`/`A6`'s globals in the same load — cheap reuse, not a dependency |
| needs | **B5-T4-P1** | `/wallet/pay402` is on the list; T4 may add `/wallet/*` routes stamped with `X-Requesting-Domain` — any new one must be added here or it 403s (the point of the allow-list) |
| gives | **B5-T5-P1** | `/.well-known/auth` must be on the list for P1's SDK `Peer` test only if the client is a page; P1's Node client is header-free, unaffected |
| gives | **B5-T5-P4** | P4's "headerless means trusted" residual is the non-browser half of this phase's surface |
| serialize | **B5-T2-P2**, **B5-T5-P3** | Neither edits `main.rs :: domain_trust_mw`; no conflict. ⚠️ If T2 adds a route a dApp calls, it joins the allow-list |
| gives | **B5-T6** (browser shell) | `R-CLOSE`: the `domain_permission_invalidate` arm edit |

## 12. Open questions for the owner

1. **Refusal code for off-list routes.** Recommendation: a new code `not_on_dapp_surface` (the deny-list keeps `permission_table_is_first_party_only` for its own paths, so existing evidence rows stay valid). Yes / no?
2. ⚠️ **W4 as filed contradicts a later decision in the code.** The loopback ticket (2026-08-18) says *forward-allowlist, not deny-list*; `startAsyncHTTPRequest`'s P0.5 finding-5 comment (later, measured) says *"⛔ THIS IS A DENYLIST, AND IT MUST STAY ONE"* because BRC-31 `x-authrite-*` / `x-bsv-*` must be forwarded and an allow-list would silently break dApp auth. Two readings: **(a)** keep the exact-strip denylist and add the missing assertion that it covers both transports (small); **(b)** a forward allow-list of **prefixes** (`content-type`, `accept*`, `x-authrite-*`, `x-bsv-*`) **minus** the exact trust names. Recommendation: **(a)** — follow the measured code over the older ticket, and close W4 with `P2-A10` proving the strip. Which?
3. **W6's DNS-rebinding Host check** is a Rust change the loopback ticket places "with W6" (§6.3). Chromium's Local Network Access prompt bounds it today, but that control is not ours and has moved once. Recommendation: include it. Yes / no?
4. **Placement of the revoke ticket** (`deleting_a_site…`): T5 in the register but in no candidate phase. Placed here as consent surface. Recommended fix: **the Rust side tells the shell** (the wallet already knows about the delete; the shell's `invalidateDomainPermissionCache` is then driven by the wallet's response to the internal `DELETE /domain/permissions`, or by the advanced-wallet page going through the wallet_call bridge whose origin C++ re-derives) — **not** an exception for internal-origin tabs, which re-opens P0.5-B1. Agree, or move it to T6-P7 (prompt surfaces)?
5. **Phase size.** Part B (W4/W6/W7/W8) is independent of Part A and carries its own risk (W6 flips the internal/external predicate). Recommendation: run it as **sub-phase P2.1** with its own boundary `R-INTEXT` run, same contract file. Yes / no?
6. No evidence that a G2 decision is wrong.

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
