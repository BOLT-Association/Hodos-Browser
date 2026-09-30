# v0.4.0-beta.6: engine security refresh + BRC-103 compatibility + advisory triage

**Opened:** 2026-09-30 (👤 owner decision, this session). **Status:** 🟡 **TRIAGE**. The advisory triage is running; no code has been written yet.

## ▶️ RESUME HERE

1. **Read the triage:** `ADVISORY_TRIAGE.md` (this folder; written when the eight triage agents finish). 👤 The owner selects which findings ship in this release.
2. **Phase contracts** for the selected items only (`../0.4.0-beta.7/PHASE_CONTRACT_TEMPLATE.md`). The harness is inherited from `../0.4.0-beta.3/HARNESS.md` + `../0.4.0-beta.7/HARNESS_DELTA.md`.
3. **The nonce fix** (already approved by the owner, below).
4. **Rebuild the app** on the **same verified engine asset** (`cef-binaries-*-150.0.48-g7d50c1c`, see the engine plan). Validation run, then the full step-5 checks, tag **`v0.4.0-beta.6`**, draft, promote.

## Why this release exists, and why it is not beta.5

- The engine security refresh (Chromium `.255`, V8 fix) was built, verified on both platforms, tagged **`v0.4.0-beta.5`** and built as a **signed draft** on 2026-09-30. Record: `../0.4.0-beta.7/track-0-engine/SECURITY_RELEASE_PLAN.md` steps 1–6.
- The same day we found that **MessageBox has refused every Hodos BRC-103 handshake since 2026-09-23**, so PeerPay notices are not delivered and incoming PeerPay is not received, for every Hodos user. Root cause measured: our handshake `initialNonce` is 32 bytes; the hardened server (ts-stack advisory **GHSA-qp3j-h5xf-p2p7**) requires the SDK's 48-byte format. Ticket: `../0.4.0-beta.7/tickets/TICKET_messagebox_rejects_our_auth_handshake_since_2026_09_23.md`.
- 👤 **Owner, 2026-09-30:** **do not promote beta.5.** Two back-to-back updates are a worse user experience than a day's wait, and the Chromium exposure for our few users is low. Fix the handshake properly, triage the whole advisory first, then ship **one** release.
- ⛔ **The `v0.4.0-beta.5` tag and its draft stay as they are**: unpromoted, never moved, never reused. Build number 40005 is spent. This release is `v0.4.0-beta.6` (build 40006).

## Scope

| Item | State |
|---|---|
| Engine: CEF `150.0.48` / Chromium `150.0.7871.255` / V8 `15.0.245.40`, fork `g7d50c1c` | ✅ built and verified. Carry the assets over unchanged; no engine rebuild |
| **BRC-103 handshake nonce:** send the SDK's exact format, 16 random bytes + a 32-byte HMAC under `[2,'server hmac']` (`@bsv/sdk` `createNonce`) = 48 bytes | 👤 **Approved 2026-09-30** (crypto change, invariant 3). Read the SDK source before writing it (rule 4). Negative control: the old code must get `ERR_AUTH_MALFORMED` from MessageBox again |
| Advisory findings (GHSA-qp3j: **391 findings**, plus GHSA-2qqx, -5vmp, -622h, -36f9 and go-sdk GHSA-rh54) | ⏳ triage running. Two axes per finding: **(A)** can a hardened server now reject our client, **(B)** do we have the same weakness |
| BRCs comment: BRC-103 §5.3 says a "random 256-bit" nonce, but the reference server now rejects 256 bits | ⬜ draft for the owner to post |
| Morning report watches security advisories | ✅ added to the skill 2026-09-30 (Marston Enterprises, local) |

## Renumbering, 2026-09-30

| Was | Now |
|---|---|
| `0.4.0-beta.6/` (the planned asset-layer release, G4 certified, G5 paused) | **`0.4.0-beta.7/`**, ships as `v0.4.0-beta.7` |
| `0.4.0-beta.7/` (next-release intake) | **`0.4.0-beta.8/`** |
| (new) | **`0.4.0-beta.6/`**: this folder |

A pure move plus a path-only sweep. Phase ids keep their **`B5-`** prefix and the Mac relay keeps its name (`../0.4.0-beta.7/MAC_RELAY_BETA5.md`), as before. The closed `0.4.0-beta.3/` folder was not touched.
