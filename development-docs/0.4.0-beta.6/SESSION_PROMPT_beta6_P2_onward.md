# Session prompt: v0.4.0-beta.6, phases P2 → P5 (written 2026-09-30)

You are continuing **`v0.4.0-beta.6`**. Kickoff is done and **P1 (MessageBox handshake) is done and pushed**. Your job is **P2 → P5, in order**, each to the same standard P1 met. The owner has made every scope decision; do not reopen them.

## Read first, in this order

1. Root `CLAUDE.md`: working rules 1–8, invariants 2/3/13, the negative-control rule.
2. `development-docs/0.4.0-beta.6/README.md`: §RESUME HERE, the "Kickoff" section (owner decisions) and the phase table.
3. `development-docs/0.4.0-beta.6/phase-P1-messagebox-handshake/PHASE_CONTRACT.md`: **the worked example** of the standard (evidence table, REDs, controls runner, adversarial review, sign-off). Copy its shape.
4. `development-docs/0.4.0-beta.6/phase-P2-abort-action/PHASE_CONTRACT.md`: written, REDs **not yet designed**, **no code yet**. Start here.
5. `development-docs/0.4.0-beta.6/G2_OFFLINE_VALIDATOR_RESULT.md`: the source of P5's scope.
6. `development-docs/0.4.0-beta.6/ADVISORY_TRIAGE.md` §1 G3/G4 rows: the source of P3 and P4.

## The phases

| Phase | Scope (owner-approved 2026-09-30) |
|---|---|
| **P2** `abortAction` | TSA-260 re-lock hang; abort only from `unsigned`/`unprocessed`/`nonfinal`, and `nosend` only after the chain says the txid is absent (fail closed); refuse `sending`/`unproven`/`completed`/`failed`/incoming; remove the `PENDING_TRANSACTIONS` entry; lookup by reference or txid; no swallowed errors. Prior art: `reference/go-wallet-toolbox/pkg/storage/internal/actions/abort.go` (already read; our `nosend` chain check is deliberately stricter, see contract §8). **Origin binding is deferred** (beta.8 ticket) |
| **P3** amounts / cap / broadcast | `createAction` negative or overflowing output amounts (satoshis are `i64`, unchecked `+=`); the spending-cap sum fooled to ~1 sat (fixture ≥ 2 sats so the old floor cannot pass it); broadcast accepting an **empty** returned txid, or a different txid whenever a merkle path is attached (`handlers.rs :: broadcast_transaction`), and weak ARC statuses counted as on chain. TSA-148/-069/-141, -192/-255/-250 |
| **P4** recipient-name spoofing | `identity_resolver.rs` shows a name in Send without verifying the certificate signature, the certifier or that the subject is the key asked about. TSA-077/-283, 5vmp F8d |
| **P5** G2 fixes | **B1** `listActions`: BRC-100 status words (not `action_storage.rs :: to_action_status`'s legacy ones) and BRC-100 input/output shapes; **B2** `listCertificates` → `totalCertificates` (one serde rename; `proveCertificate` got the same fix in `e4b73fe`); **B3** honour `signAndProcess:false` (return `signableTransaction`). Measure `createAction` with dApp-supplied inputs before closing. Re-record with `g2-validator/record*.mjs` and re-run `validate.mjs`: the GREEN is 0 failures, the RED is today's 6 |

⛔ **Deferred, not yours:** typed error envelope, `valid:false`, `verifySignature` `self`, `noSend` phantom flag, `abortAction` origin binding, MessageBox fees (all beta.8 intake); certificate-issuance handshake verification (beta.7 T5-P1).
❌ **Not a bug:** the `/createHmac` leading-zero "divergence" was retracted; our endpoints already strip (`crypto/CLAUDE.md` Rule 2).

## How P1 was done — do the same

- **Contract first**, from `../0.4.0-beta.7/PHASE_CONTRACT_TEMPLATE.md`, every row with GREEN / RED / SUBJECT.
- **Money and crypto rows: REDs designed by a second agent** (background `general-purpose`, read-only), before the tests are final.
- **Reference vectors come from the reference implementation**, never from our code. `@bsv/sdk` 2.8.11 is pinned in `development-docs/0.4.0-beta.6/g2-validator/` (`npm i` there; `node_modules` is git-ignored). Example generator: `g2-validator/gen_authfetch_vectors.mjs`.
- **Negative controls are run by a script**, not by hand: `phase-P1-messagebox-handshake/negative_controls.py` restores every file, applies one mutation, checks **exactly which tests failed and why**, then restores. Copy it for each phase.
- **Adversarial review after**, by a second agent, on the evidence. Close what it finds, then record it in the sign-off.
- `cargo test --workspace` (check the counts: 20 targets, 0 failed) + `pwsh scripts/preflight.ps1 -Full` (the default run is INCOMPLETE) + `-NegativeControl`.
- Commit code and docs separately; `git fetch` first; push to `origin 0.4.0`.

## ⛔ Traps hit in P1

- **Ground-truth every subagent "may also".** P1 reported a crypto divergence that the code had already fixed, because an agent's unchecked "may have the same problem" was passed on. Read the code before telling the owner anything.
- **Python on Windows:** `Path.write_text` / text-mode writes turn LF into **CRLF**. Read and write source files as bytes (`read_bytes`/`write_bytes`). In heredocs, never put `\n` escapes inside nested triple quotes; use `chr(92)`.
- **The dev browser is often running.** To rebuild only the wallet, stop the dev wallet by **exe path** (the `Where-Object { $_.ExecutablePath -like '*Hodos-Browser\rust-wallet\target\release\*' }` form), never by name; the owner's installed wallet shares the image name. Restart it **detached**: `Start-Process powershell -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','C:\Users\archb\Hodos-Browser\dev-wallet.ps1'`.
- The wallet log `wallet_rCURRENT.log` **rotates on restart**; count lines only after the restart.
- Recording against the dev wallet can trigger its automatic on-chain backup (~13k sats fee). Declare it as residue.

## Standing facts

- Dev wallet `127.0.0.1:31401`, DB `%APPDATA%\HodosBrowserDev\wallet\wallet.db` (open read-only: `file:...?mode=ro`).
- **Fixture:** PeerPay `8a24596f…`, outbox row `exhausted`. Kept for beta.7 B5-T6-P5 "Claim a payment". **Never touch it.**
- 🍎 **macOS:** all beta.6 work is shared Rust; no macOS code expected. If any `cef-native/**` C++ changes, add a round to `development-docs/0.4.0-beta.7/MAC_RELAY_BETA5.md` naming the files. The Mac app must be rebuilt and smoke-tested on the same engine before promote.
- Security advisories: check `gh api repos/bsv-blockchain/ts-stack/security-advisories` at the start; five known as of 2026-09-30.
- Hand the owner a short summary at each phase boundary. Ask before starting the next only if a decision is actually needed.
