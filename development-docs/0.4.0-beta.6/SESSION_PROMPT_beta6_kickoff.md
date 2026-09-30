# Session prompt: v0.4.0-beta.6 kickoff (written 2026-09-30)

You are picking up **`v0.4.0-beta.6`**: the verified engine security refresh + the BRC-103 handshake fix + selected fixes from the ts-stack security advisories. **No code has been written for it yet.**

## Read first, in this order

1. Root `CLAUDE.md`: the working rules (especially 1, 2, 4, 5, 6, 7), invariants 2/3/13, the phase kickoff workflow and the negative-control rule.
2. `development-docs/0.4.0-beta.6/README.md` §RESUME HERE, the front page of this release.
3. `development-docs/0.4.0-beta.6/ADVISORY_TRIAGE.md`: 438 findings merged into themes G1–G7, the scope table (§2) and the ground-truth checks already run (§3). Per-finding detail is in `ADVISORY_TRIAGE_TABLES.md`; look up rows there, don't read it whole.
4. `development-docs/0.4.0-beta.7/tickets/TICKET_messagebox_rejects_our_auth_handshake_since_2026_09_23.md`: the measured root cause.

## 👤 Owner decisions already made (2026-09-30), not to be reopened

- `v0.4.0-beta.5` is tagged and drafted but **never promoted**. Never move or reuse that tag. This release is `v0.4.0-beta.6` (build 40006) and reuses the **already-verified engine assets** `cef-binaries-*-150.0.48-g7d50c1c`: **no engine rebuild**.
- **The nonce fix is approved**, including that it touches crypto (invariant 3): send the **exact `@bsv/sdk` `createNonce` format**, 16 random bytes + a 32-byte HMAC under protocol `[2,'server hmac']`, 48 bytes base64. Read the SDK source before writing it (rule 4). Only the handshake `initialNonce` changes; the per-request nonce and request id stay 32 bytes (measured against the server's validator).
- **Scope = `ADVISORY_TRIAGE.md` §2 as recommended:**
  - **In:** **G1** (nonce, server signature/identity verification, the live MessageBox paging shape, recipient fees); **G2** (offline SDK-validator test first, then fix any confirmed break); **G3** (the `abortAction` re-lock and its siblings; negative/overflowing amounts and the cap-sum bypass; broadcast accepted for an empty/different txid and weak statuses); **G4** recipient-name spoofing in Send (`identity_resolver.rs`).
  - **Deferred to beta.7**, into the tracks that already own them: unverified certificates and certifier SSRF (T5-P1); overlay/SHIP (T5); wallet HTTP surface (T5-P2); BRC-121 redirects/cache (T4); merkle-proof path verification, the 6 h double-spend verdict, and partial `sendWith` (T1).
- The folders were renumbered: `0.4.0-beta.6/` = this release, `0.4.0-beta.7/` = the planned asset-layer release (its G5 waits for this one), `0.4.0-beta.8/` = intake.

## Your first jobs, in order

1. **G2's offline test.** Record real Hodos responses from the **dev** wallet (`createAction`/`signAction` including the 1000-sat service-fee output; `listActions`; `listOutputs` with tags and entire transactions; an error response; `verifySignature` false). Run them through the hardened ts-stack validators (commit `b3155fa2` or later; `@bsv/sdk` 2.8.11+ is on npm). This is a measurement, not a fix, and needs no money. Report what fails and why. ⭐ The service-fee output under the new "bound-action"/template checks (TSA-082/-278/-401) is the question that matters most.
2. **Phase kickoff** for the in-scope items (CLAUDE.md "Phase kickoff workflow"). Phase contracts from `development-docs/0.4.0-beta.7/PHASE_CONTRACT_TEMPLATE.md`, **each with its negative control**. Harness: `0.4.0-beta.3/HARNESS.md` + `0.4.0-beta.7/HARNESS_DELTA.md`. Suggested order: G1 → G3 abort → G3 amounts/broadcast → G4 → G2 fixes. Hand the owner a tight summary and **wait for confirmation before the first commit** (kickoff step 6).
3. Implementation, each fix with its control (examples: the nonce fix must turn MessageBox's `ERR_AUTH_MALFORMED` into a successful `listMessages` **and** one delivered PeerPay notice, and reverting it must bring the 400 back; the abort fix needs a unit test with a timeout that **hangs on today's code**).

## Things to know

- Dev environment: always `HODOS_DEV=1` via the launchers. Stop dev processes only with `scripts/stop-dev.ps1` (path-matched; the owner's installed wallet shares the image names). Long-running dev processes must be started **detached** (e.g. `Start-Process`), not as Claude Code background tasks, which are killed after 2 hours.
- 🍎 **macOS:** the fixes are shared Rust (`rust-wallet/`), so no macOS-specific code is expected. But the release builds and ships **both** platforms, and the Mac app must be rebuilt and smoke-tested on the same engine before promote. Relay: newest round first in `development-docs/0.4.0-beta.7/MAC_RELAY_BETA5.md` (round W-30a explains the renumbering). If any `cef-native/**` C++ changes, name the files in a new relay round.
- **Test fixture:** the owner's 30-cent self-PeerPay, txid `8a24596f7d5862c41c3fb3529df9fec4ab97a71dee2333571206144afb4ebd4e`, dev wallet. It was never notified and is kept for **B5-T6-P5 "Claim a payment"**. Don't clean it up. After the nonce fix, check whether its queued outbox notice gets delivered.
- Security advisories are now watched by the morning report (Marston Enterprises skill, local). New ts-stack advisories published during this work may change scope: check `gh api repos/bsv-blockchain/ts-stack/security-advisories` at kickoff.
- A BRCs comment is owed (owner's voice): BRC-103 §5.3 specifies a "random 256-bit" nonce, but the reference server now rejects 256 bits. Draft it for the owner; do not post it.
