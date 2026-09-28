# B5-T1-P1 — the wallet's TLS validator has no open advisories · PHASE CONTRACT

**Track:** B5-T1 Money path · **Tickets:** none (owner decision 2026-09-15; outline in `../reqwest-tls-bump/README.md`) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Fable 5.1), G3 track agent · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** none of the 14 directly; T1 Q9 (**`reqwest` 0.12, not 0.13**) taken per `../SCOPE.md` §11 recommendation. Root `CLAUDE.md` invariant 3: transport only — nothing the wallet signs, derives or broadcasts changes.

---

## 1. Goal

Every outbound HTTPS call the wallet makes validates the server's certificate with a library that has **no open RustSec advisory**, and the wallet sends, receives and signs exactly what it did before.

## 2. Done means

- [ ] `cargo audit` on `rust-wallet/Cargo.lock` lists **none** of RUSTSEC-2026-0098, -0099, -0104 (`rustls-webpki`) or -0258 (`h2`).
- [ ] `rust-wallet/Cargo.lock` resolves `reqwest 0.12.x`, `rustls 0.23.x`, `rustls-webpki 0.103.≥13`, `hyper 1.x`, `h2 0.4.x` (today, measured 2026-09-28: `reqwest 0.11.27`, `rustls 0.21.12`, `rustls-webpki 0.101.7`, `hyper 0.14.32`, `h2 0.3.27`, `webpki-roots 0.25.4`, `ring 0.17.14`).
- [ ] The crypto provider is **`ring`** and the root store is **`webpki-roots`** (bundled Mozilla roots — what 0.11 `rustls-tls` used), both chosen explicitly in `Cargo.toml` features and written down in this contract with the reason.
- [ ] Every `CallClass` timeout (`services/call_class.rs :: CallClass::timeout` — 8 s / 15 s / 30 s / 240 s) is unchanged **and re-measured** against an unroutable host after the bump (0.12 changed how connect and total timeouts compose — `../reqwest-tls-bump/README.md` "Risks").
- [ ] The number of `reqwest::Client::builder()/new()` construction sites does not grow: **76 sites in 23 files** (measured 2026-09-28 by grep — ⚠️ the scope doc said 35 files; the file count is 23).
- [ ] `adblock-engine` (`reqwest 0.11`, default features ⇒ schannel on Windows; carries only the `h2` advisory) is bumped **only** if the wallet bump was uneventful; otherwise it is its own item and this contract says which.
- [ ] `../../../DevOps-CICD/DEPENDENCY_VERIFICATION.md` review table updated.

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-INTEXT` | internal never prompts, external always gates | Not touched in logic, but every wallet-to-indexer call now rides a new transport; a regression in header handling would appear as a changed decision, not a changed log line. Run the natural A/B (86-vs-1 shape) at the boundary |
| `R-PERIM` | four perimeter gates | T1 only (`cargo test`) — engine untouched |
| `R-DUST` | no incidental 1-sat spend | T1 only — 16 tests must still pass; the fetcher's `is_token_reserved_value` is in the file that changes most |
| `R-PEERPAY-DELIVERY` half 1 | a PeerPay delivers or never leaves | `messagebox.rs` + `authfetch.rs` are the most bespoke `reqwest` users (custom headers `x-bsv-auth-*`, response header reads). Half 1's T2 dev-wallet run is free and directly on the changed code |
| memory: ARC txStatus ladder | ANNOUNCED ≠ success | `services/providers/arc_*.rs` parse ARC bodies; a changed `Response::json` error path would change a verdict. `P1-A3` |
| memory: caches must not self-poison | a failed fetch must not write | `price_cache.rs` / `fee_rate_cache.rs` error classification must survive the `reqwest::Error` type changes. `P1-A5` |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. A green result is reported with its red half or not at all.
⛔ Money, schema and crypto rows: the RED (negative control) is **designed by someone other than the assertion's author** — a second agent (`../../../RELEASE_CYCLE.md` §4.2). Record who designed it.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P1-A1` | `cargo audit` (installed: `cargo-audit 0.22.2`, measured 2026-09-28) on the **new** `Cargo.lock` reports none of the four ids | Run the same command on the **old** lock (`git show HEAD~:rust-wallet/Cargo.lock`) ⇒ all four ids listed. Both runs recorded, same day, same advisory-db commit | The advisory-db commit hash printed by `cargo audit`, and the four ids by name — not "0 vulnerabilities" alone (a stale advisory db reports zero for the wrong reason) | T0 | ⬜ |
| `P1-A2` | Balance and UTXO set parity: `GET /wallet/balance` and the row set from `utxo_fetcher :: fetch_all_utxos` on the dev wallet's addresses are **identical** before and after the bump (same outpoints, same `confirmed` flags) | Point `WOC_BULK_CONFIRMED` at a host that returns a truncated body ⇒ parity check reports a diff. Proves the comparison sees a difference when one exists | The outpoint list diff (`txid:vout`, `confirmed`) from two runs against the **dev** wallet (`31401`), not the balance number alone | T2 | ⬜ |
| `P1-A3` | One real small send (dev wallet → its own address) broadcasts through the Services chain and the ARC status ladder reaches the same terminal state as before (`arc_status.rs` vocabulary: ANNOUNCED → SEEN_ON_NETWORK → MINED), with `transactions.status` following the same transitions | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The `transactions` row's status history and WhatsOnChain for the txid — never the HTTP 200 alone | T2 (money, cents) | ⬜ |
| `P1-A4` | BRC-103 AuthFetch handshake + MessageBox poll round-trips: `TaskCheckPeerPay` logs a successful authenticated list call; a PeerPay of a few hundred sats to a second dev wallet is delivered and credited once (`R-PEERPAY-DELIVERY` half 1's dev run, plus one real delivery) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Recipient's `outputs` + `peerpay_received` rows; the sender's `peerpay_outbox` empty; the `x-bsv-auth-*` request headers as **sent** (log at debug), because header normalisation is the risk | T2 (money, cents) | ⬜ |
| `P1-A5` | Bad-certificate hosts are **refused** after the bump: `expired.badssl.com`, `wrong.host.badssl.com`, `self-signed.badssl.com`, `untrusted-root.badssl.com` each fail with a TLS error **from the validator** (the `rustls` error text, not a DNS or timeout error), and `https://api.whatsonchain.com/v1/bsv/main/chain/info` succeeds | Build a test-only client with `.danger_accept_invalid_certs(true)` (a `#[cfg(test)]` helper, never shipped) ⇒ the same four hosts **succeed**. Also record the **pre-bump** result for the four hosts (all refused on 0.11 too — this row does not discriminate the bump; it proves the door is still locked, `../reqwest-tls-bump/README.md` 0.5) | The error string contains the validator's own reason (`CertificateExpired`, `NotValidForName`, `UnknownIssuer`) — a control that goes red on DNS tested nothing (`HARNESS.md` §2) | T1 (ignored network test, run by hand) | ⬜ |
| `P1-A6` | Price fallback chain: with the primary (WhatsOnChain) forced to fail, `price_cache.rs` falls through to CoinGecko then MEXC and returns a price; with **all three** forced to fail it returns the last good cached price (V21 `bsv_price_cache`) and never writes a poisoned value | Remove the fallthrough on the primary's `Err` ⇒ `price_unavailable` with the primary down. ⚠️ Instrument: if no injection seam exists for the primary URL, add a `#[cfg(test)]` base-URL override in `price_cache.rs` — a test seam, not a feature | The `bsv_price_cache` row before/after and the returned price, not a log line saying "fallback" | T1 | ⬜ |
| `P1-A7` | Timeouts hold: for each `CallClass` a request to an unroutable address (`10.255.255.1`) fails at the class's total timeout ± 1 s, on both platforms | Set one class to `Duration::from_secs(1)` in a scratch build ⇒ that class fails at ~1 s and the others do not. The existing `classes_have_distinct_timeouts` test stays green either way, so it is **not** the control | Wall-clock measured around the call, per class, per platform | T2 | ⬜ |
| `P1-A8` | Construction-site count: grep `reqwest::Client::(builder|new)` in `rust-wallet/src` = **76 sites / 23 files** or fewer after the bump | Add one throwaway `reqwest::Client::new()` ⇒ 77. (A count that cannot rise is not a gate) | The grep command and its output, pasted | T0 | ⬜ |
| `P1-A9` | One BRC-121 paid retry (`pay_402`, custom headers) completes end-to-end on the dev rig and the gold pill fires (`R-GOLD`'s paid-retry route, still owed since beta.3) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The five retry headers as sent, the 200 on the retry, the tab that shows the pill (`Tab::id`, not `CefBrowser::GetIdentifier()`) | T2/T3 (money) | ⬜ |

**Two-sided rows:** `P1-A5` (bad hosts refused) and its positive half (a good host succeeds) are each other's control — a client that refuses everything passes the first and fails the second.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/Cargo.toml` — `reqwest = { version = "0.11", features = ["json", "rustls-tls"], default-features = false }` | ✅ | The one line that changes. ⚠️ Read the 0.12.28 feature list before choosing the TLS feature name; `cargo outdated` says `rustls-tls` is obsolete **in 0.13.5** — do not infer it is unchanged in 0.12 (rule 4) |
| `adblock-engine/Cargo.toml` — `reqwest = { version = "0.11", features = ["json"] }` | ✅ | Default features ⇒ native TLS (schannel); only the `h2` advisory applies |
| `rust-wallet/Cargo.lock` — versions listed in §2 | ✅ measured | crates.io 2026-09-28: `reqwest` max `0.13.5`, latest 0.12 line `0.12.28`; `rustls-webpki` latest 0.103 line `0.103.15`; `rustls 0.23.45`; `hyper 1.11.1`; `h2 0.4.19` |
| `services/call_class.rs :: CallClass::timeout` | ✅ | 8 / 15 / 30 / 240 s. Single source of truth for builder timeouts |
| `authfetch.rs :: AuthFetch` (`http_client: reqwest::Client`, `.header("x-bsv-auth-*")` ×6, `reqwest::Url::parse`) | ✅ | Most bespoke use; first place an API rename lands |
| `utxo_fetcher.rs :: fetch_utxos_bulk / fetch_bulk_chunk / fetch_all_utxos` | ✅ | The money-path reader; 8 construction sites in this file |
| `handlers.rs` (20 sites), `handlers/certificate_handlers.rs` (12), `overlay/mod.rs` (5), `services/providers/*.rs`, `price_cache.rs`, `fee_rate_cache.rs`, `paymail.rs`, `messagebox.rs` via `authfetch`, `monitor/task_*.rs` | ✅ | The 76-site surface. ⛔ The phase enumerates every site with its headers-read / status-branched / timeout, as 0.1 in the outline asks — this table is the starting list, not the deliverable |
| `cef-native/include/core/WalletService.h :: kWalletBroadcastTimeoutMs` = 30 000, `kBridgeCallTimeoutMs` = 45 000 | ✅ | Not changed here; `P1-A7` must show the Rust `IndexerBulk` 30 s still fits under the C++ 45 s bridge deadline |

## 6. Out of scope

- Any behaviour edit beyond what the API forces. A rename fix is in scope; "while we are here" cleanup of the 76 sites is not (working rule 3). Consolidating construction sites is a ticket, not this phase.
- `reqwest 0.13` — one more API break for no advisory benefit we know of (Q9). Re-check only if 0.12 is found end-of-life at phase open.
- `aws-lc-rs` — needs a C toolchain and NASM on Windows; `ring` unless a measured reason appears.
- A platform/native root store (`rustls-native-certs`) — would change **which** certificates are trusted on a user's machine behind a corporate proxy or AV MITM root. Keep `webpki-roots`; a switch is its own decision with its own negative control.
- Certificate pinning for ARC/WhatsOnChain — no reference SDK does it (`../reqwest-tls-bump/README.md` note 4; record in `PRIOR_ART.md` when checked).

## 7. Rollback

Revert `rust-wallet/Cargo.toml` and `rust-wallet/Cargo.lock` (and `adblock-engine/*` if bumped) in one commit; the rename fixes revert with them. No data, no schema, no wire change.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Row that catches it |
|---|---|
| The bump compiled, `cargo audit` was run against a stale advisory database and printed zero; the four advisories were still open | `P1-A1` requires the advisory-db commit and the ids by name, and the old-lock RED |
| A `CallClass` timeout silently became connect-only; the 30 s bulk sync now runs 60 s and the C++ bridge (45 s) reports "timed out" on successful sends — the exact 2026-09-14 defect, reintroduced | `P1-A7` |
| AuthFetch's header names were lowercased/normalised differently and MessageBox rejected the signature; PeerPay polling silently stopped (rule 7 trip-wire 2: a swallowed error) | `P1-A4` |
| The default root store changed to native certs and a user behind a corporate proxy could no longer reach any indexer; on the build host everything worked | §2 root-store choice written down; `P1-A5` records the store used. ⚠️ A user-machine row is **not** in this table — if native certs are ever chosen, add one |
| `price_cache` treated a new `reqwest::Error` variant as "price 0" and wrote it | `P1-A6` (cache never writes a failed fetch) |
| The badssl rows went "red" because DNS failed in the agent sandbox, not because the validator refused | `P1-A5` SUBJECT: the validator's own error text |
| Gold pill stopped firing on the paid retry because the 402 client's response headers were read case-sensitively | `P1-A9` |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | A1–A9 | Primary |
| macOS | A1, A5, A7, plus `cargo build --release` + `cargo test` | The wallet is one Rust binary with one `Cargo.lock`; macOS must **compile** it (`ring` needs no NASM on macOS) and re-measure A7 because the TCP stack differs. A3/A4/A9 are money rows run once, on Windows. Relay round names this contract |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1 h** — A3 (one send), A4 (one PeerPay), A9 (one paid page), a glance at the price shown. Batched into one sitting with P2's rows |
| Human-bound rows | A3, A4, A9 (real money, cents); A9 also needs eyes on the pill |
| Unknowns (K) — uncertainty, not difficulty | **0.** The API break at `authfetch.rs` is work, not uncertainty. K stays 0 unless the 0.12 feature list turns out not to offer `webpki-roots` + `ring` together (then the root-store decision reopens) |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| gives → all T1 phases | P2–P7 | Lands **first**; every later T1 row measures on the new transport |
| gives → T4-P1 | 402 chain check | `check_tx_exists_on_chain` rides this transport; T4 must not measure before P1 lands |
| gives → T5-P1 | server identity (BRC-103) | `authfetch.rs` is edited here for renames only; T5 changes its semantics later — do not overlap in one commit |
| needs ← T0 Build 1 | engine | None: the wallet's TLS is not libcef's BoringSSL (`DEPENDENCY_VERIFICATION.md`) |

## 12. Open questions for the owner

None. Q9 (0.12) taken per the scope. If the 0.12 feature set forces a root-store change, that returns here as a question before the bump lands.

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
