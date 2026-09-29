# G4 — Admin & Logistics checklist *(RELEASE_CYCLE §3.4 — nothing we depend on expires inside the cycle)*

**Prepared:** 2026-09-28 from the repo and the installed build. 👤 = needs the owner. The cycle runs ~1–3 months ⇒
anything expiring before **~2027-01-01** is a G4 finding.
⭐ **This is also the standing register of what expires** (owner, 2026-09-28: *keep a list so we are not surprised
when we go to sign something*). At G12 it moves to `../DevOps-CICD/` as a permanent doc.

## 1. Credentials and expiry

| Credential | Used by | Expires? | Where to look | State |
|---|---|---|---|---|
| **Windows code signing** — Azure Trusted Signing (`AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`) | `release.yml` | Signing certificates are short-lived + timestamped — not a risk. The **Trusted Signing account/profile "Hodos signing"**: created 2026-03-23, status Current, **no expiry shown** (owner, 2026-09-29) ✅. ⚠️ Still owed: the **app registration's client secret** — a different screen (see §5) | 👤 client secret date owed |
| **Apple Developer ID certificates** | macOS signing | Two: company expires **2031-08-13**, personal **2031-03-25** (owner, 2026-09-29) — ✅ far past the cycle. 👤 confirm which one `MACOS_CERT_BASE64` holds (likely the company one) | ✅ |
| **Apple Developer Program membership** | signing + notarization | Renews **2027-03-23** — ✅ past this cycle. ⭐ Recommend **auto-renew ON**: a lapsed membership stops notarization for every macOS release | 👤 auto-renew |
| **App Store Connect API key** (`APPLE_API_KEY_ID`, `APPLE_API_ISSUER`, `APPLE_API_KEY_P8_BASE64`) | notarization | No (until revoked) | — | ✅ |
| **Sparkle EdDSA** (`SPARKLE_EDDSA_PRIVATE_KEY`) · **WinSparkle DSA** (`WINSPARKLE_DSA_PRIVATE_KEY`) | update-feed signatures | No | ⛔ never rotate casually — every installed build trusts the current public key | ✅ |
| **`WEBSITE_DEPLOY_TOKEN`** | website/appcast deploy at promote | 👤 set to **never expire** after the 2026-09-24 mid-promote expiry | Trade-off: no surprise expiry, but a leaked token never dies ⇒ rotate it deliberately, e.g. at each G12 | ✅ recorded |
| **`GITHUB_TOKEN`** | workflows | per run, automatic | — | ✅ |
| **TAAL ARC API key** | `rust-wallet/src/services/providers/arc_taal.rs :: API_KEY` — **hard-coded in the binary**; fallback broadcaster behind GorillaPool | ⚠️ The file's own header says it is **"rotated monthly between builds"** ⇒ it may expire **inside every cycle** | 👤 TAAL console → key expiry. The code already reports a dead key distinctly ("TAAL ARC authentication failed — hardcoded API key expired or invalid") | 👤 see §4 |
| **WhatsOnChain** | primary indexer | no key in the code (free tier) | grep found none | ✅ |
| **Domain `hodosbrowser.com`** + hosting | website, appcast, T5-P5 usage ping | Yes (registration) | 👤 registrar account | 👤 date owed |

GitHub stores the secrets above but never shows an expiry — the dates live at each issuer, which is why this table exists.

## 2. Environments — the physical things the phases need

| Need | For | State |
|---|---|---|
| Engine build machine | T0 engine build 2 | ✅ this machine (`C:\cef\cef150` present); macOS on the MacBook Pro |
| An older installed Hodos to update from | T6-P1/P4 update tests, G10 | ✅ the installed build at `%LOCALAPPDATA%\HodosBrowser\` |
| Two **OS user accounts** on one computer (not browser profiles) | T5-P4: the wallet listens on one fixed port, so today user B's Hodos can reach user A's running wallet | 👤 create a second Windows user when that phase runs; same on the Mac. (Per-profile wallets inside the browser = a possible future feature, not this phase) |
| Two monitors at different scaling | T6-P9 row D2 — **Windows only** (macOS overlays size in points, so the Mac has no such row) | 👤 does this Windows machine have a second screen? A laptop screen + an external monitor at different scaling counts. If not, D2 is recorded "not run" |
| Your Chrome profile | T6-P6 import | ✅ Chrome is installed here |
| An export file from a **BRC-100 reference wallet** (Metanet Desktop / BSV Desktop — Babbage or BSVA) or published BRC-38/39 test vectors | T3b-P5 row A15: a wallet exported elsewhere imports into Hodos, and ours into theirs | 👤 owner 2026-09-29: **not HandCash** (not a BRC-100 wallet). If no vectors exist when the phase opens, the owner coordinates with Babbage/BSVA |
| Funded scratch wallets | money rows T1–T4, T6-P5 | ✅ |

## 3. CI and pins

| Item | State |
|---|---|
| Runner images | Pinned (`ubuntu-24.04`, `windows-2022`, `macos-15`) except `cef-fork-watch.yml` → `ubuntu-latest` (a watcher job, not a build — low risk, noted) |
| CEF engine | Branch 7871, supported to ~Apr 2027 — ✅ past this cycle (owner confirmed) |
| vcpkg baseline, macOS deployment target | ⬜ read at G4 close |

## 4. The TAAL key — owner: "only needed for rate limits we almost never exceed; test without it"

Probe 2026-09-29 (read-only, `GET https://arc.taal.com/v1/policy`): **200 without a key**, 200 with the shipped key.
That proves reads work keyless — **not** that a broadcast does. The deciding test is one real broadcast through TAAL
with no key (a few sats, scratch wallet). Proposed as a row in **B5-T1-P1** (the HTTP-client phase, which already
touches every provider). If keyless broadcast works: remove the key and its monthly rotation; TAAL stays fallback.

## 5. Where to look up each date

| Date | Link | Steps |
|---|---|---|
| Azure client secret | https://portal.azure.com/#view/Microsoft_AAD_IAM/ActiveDirectoryMenuBlade/~/RegisteredApps | "All applications" → open the app whose **Application (client) ID** matches the `AZURE_CLIENT_ID` secret → **Certificates & secrets** → **Client secrets** → "Expires" column |
| Azure Trusted Signing identity | https://portal.azure.com → search "Trusted Signing Accounts" | open the account → **Identity validation** → status / expiry |
| Apple Developer ID certificate | https://developer.apple.com/account/resources/certificates/list | the **Developer ID Application** row → "Expiration" |
| Apple Developer Program membership | https://developer.apple.com/account | **Membership details** → "Expiration date" |
| Domain `hodosbrowser.com` | 👤 your registrar (which one?) | domain list → expiry / auto-renew |
