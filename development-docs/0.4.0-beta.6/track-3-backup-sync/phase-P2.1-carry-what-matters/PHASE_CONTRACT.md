# B5-T3a-P2.1 — A restored wallet keeps the permissions, payment records and settings the user had · PHASE CONTRACT

**Track:** B5-T3a Backup you can trust (single device) · **Tickets:** closes `../research/ONCHAIN_BACKUP_REVIEW.md` **BS-H2, BS-M4, BS-M5, BS-L2, BS-L3** (the finding register; no ticket file) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 agent for T3a (resumed after the Fable agent stopped) · **Platforms:** both (Rust payload + import; no UI) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** 2a (the money index is **derived** — the H2 manifest classifies it *re-derived*, never *travels*), 3, 5 (the on-chain payload never claims BRC-38 — adding tables here is our format, not interop), 11 via T5-P3 (`derived_key_cache` and its new requester child table are **excluded**; the detector restarts empty after restore), `SCOPE.md` §0, §4 (P2 list), §6a.

> **Why T3a-P2 is three contracts, not one.** SCOPE §5 P2 bundles ~18 items that touch three different
> things: **what the payload carries** (this contract), **the write path that spends money to broadcast it**
> (P2.2), and **the restore path** (P2.3). Each has a different subject, a different rollback, and a different
> T1 dependency — one rollback paragraph could not cover all three in two lines (template §7). The split is
> **presentation**: nothing is cut. Other tracks' "T3a-P2" citations map as: *restore classifies before
> rebuilding / BS-C2 / restore report / high-water mark / T1-P6 review* → **P2.3**; *BS-C1 lock / baseline
> (`get_backup_hash`) / BRC-177 / size cap / backoff / BS-SYNC-1* → **P2.2**; *missing tables and columns* →
> **P2.1**. Default order P2.1 → P2.2 → P2.3; only P2.3 is on T3b-P5's critical path, and it does not
> hard-depend on P2.1 or P2.2 (§11).

---

## 1. Goal

A user who restores from the twelve words gets back the site permissions (including a cautious "Always notify" choice), payment records and settings they had — not defaults — and every column the wallet stores has a written reason for travelling or not.

## 2. Done means

- [ ] Every `table.column` in the migrated schema (V25 today — `database/connection.rs :: WalletDatabase::migrate`; more after T1-P3 / T5-P3 land) is classified in T3a-P1's H2 manifest, and P1-A3 is **green** — the manifest is the list of what this phase fixes, not a second list
- [ ] Carried that are dropped today: `domain_permissions` columns beyond the seven `backup.rs :: BackupDomainPermission` holds (`max_tx_per_session`, `identity_key_disclosure_allowed`, the rest per A1 §2 items 23–26); the V18 scoped-grant child tables `domain_protocol_permissions` / `domain_basket_permissions` / `domain_counterparty_permissions`; `settings` defaults + `sender_display_name`; `outputs.confirmed` (BS-L3 — restored rows land `confirmed=1` today); `transactions.price_usd_cents/recipient/recipient_name`; `peerpay_received` (BRC-42 counterparty data, not re-derivable)
- [ ] Decided and recorded in the manifest with a reason (owner reads the *excluded* list — §12 Q1): `peerpay_outbox` (BS-M5), `peerpay_pending_verification`, `transaction_inputs` / `transaction_outputs` (BS-M4), `domain_manifest_snapshots`, `messages`, `relay_messages`, `monitor_events`, `permission_audit_log`, `engine_shadow_log`, `bsv_price_cache`, `derived_key_cache` (+ T5-P3's child table), T1-P3's `money_utxos` (**re-derived**, decision 2a)
- [ ] **BS-L2:** decode checks `payload.version`; a payload newer than the binary understands is a **typed error naming the version**, never a silent field drop. Older payloads keep decoding (`#[serde(default)]` on every new field — the pattern `domain_permissions` already uses)
- [ ] The payload grows by a **measured** amount on both P0 wallets (bytes before/after, recorded beside P0-E2), and the new size stays under P2.2's cap

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| Load-bearing safeguard — `DomainPermissionForm` "Always notify" (root `CLAUDE.md` kickoff step 4) | zeroed limits force a prompt on every payment | Only **two of its three** zeroed limits travel today (`per_tx_limit_cents`, `per_session_limit_cents`; `max_tx_per_session` does not). Whether the restored site still prompts then rests on the per-tx limit alone — **not measured**; A2 measures it on today's code first and records the answer. Mis-mapping a column while fixing it would turn a cautious site into a silent one |
| Privacy perimeter — identity-key reveal | silent only when `identity_key_disclosure_allowed=1` or a session opt-in | The column is dropped today (restore ⇒ global default ON); carrying the wrong value flips a user's explicit OFF |
| `R-RESTORE` | fail-closed survives recovery | `outputs.confirmed` and `change` feed selection preference; a mapping slip changes which coins are picked first |
| Invariant 2 | no schema change without asking | **None here** — the payload is a serde struct, not a table. The manifest is a test fixture |
| Decision 5 | on-chain never claims BRC-38 | New `Backup*` structs stay ours; no BRC-38 field names are implied |

## 4. Evidence table

⛔ Money, schema and crypto rows: RED designed by a second agent (`../../../RELEASE_CYCLE.md` §4.2).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P2.1-A1` *manifest complete* | P1-A3 (H2 gate) green on the migrated schema with this phase's manifest; every *excluded* row names a reason and a tier | Delete one new manifest row (e.g. `domain_permissions.max_tx_per_session`) ⇒ H2 red **naming that column** — reuses P1-A3's mechanism, so this row proves the manifest edit, not the gate | P1-A3's own failure text on the migrated test DB | T1 | ⬜ |
| `P2.1-A2` ⭐ *cautious site survives restore* | Fixture site with "Always notify" (per-tx, per-session, max-tx-per-session all `0`) and `identity_key_disclosure_allowed=0`: after seed-only restore on the mock, the Rust permission engine (`hodos_permission_engine :: decide` via `permission_service`) returns **Prompt** for a 1-cent payment and for an identity-key request from that site | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The **restored** DB's `domain_permissions` row and the engine's decision for a request from that domain — the decision, not the column value | T1 | ⬜ |
| `P2.1-A3` *scoped grants survive* | A protocol, a basket and a counterparty grant on the fixture site: after restore each is present with its FK to the restored `domain_permissions` row, and `is_protocol_granted` matches on **both** `[level, namespace]` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The restored child rows joined to their parent by FK; the grant check's answer | T1 | ⬜ |
| `P2.1-A4` *payment records survive* | `peerpay_received` rows and `outputs.confirmed` / `transactions.price_usd_cents` / `recipient` values in the restored DB equal the source row for row (P1's `canonical()` diff shows no line for these columns) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | `canonical(restored)` vs `canonical(source)`; the diff lines for these columns specifically | T1 | ⬜ |
| `P2.1-A5` *excluded is really excluded* | `derived_key_cache`, T5-P3's requester table and `money_utxos` are **absent** from the decoded payload; after restore `money_utxos` is rebuilt (P2.3) and `derived_key_cache` is empty | Put `derived_key_cache` rows into the payload by hand ⇒ the "absent" assertion goes red. Second: skip the rebuild ⇒ `money_utxos` empty ⇒ red (P2.3-A3 owns the rebuild — this row only proves the absence assertion can fail) | The decoded payload JSON's top-level keys; the restored tables' row counts | T1 | ⬜ |
| `P2.1-A6` *BS-L2 version check* | A payload with `version` = current + 1 decodes to a typed `UnsupportedPayloadVersion(n)`; a payload without the new fields (captured from today's build — H15's first fixture) still decodes and restores | Remove the version check ⇒ the newer payload decodes with fields silently dropped and the typed-error assertion goes red | `backup.rs :: deserialize_from_onchain`'s return value on the two fixtures | T1 | ⬜ |
| `P2.1-A7` *measured growth* | Payload bytes before/after on both P0 wallet copies, compressed, recorded; each under P2.2's cap | Run the measurement against a payload builder with the new tables stubbed empty ⇒ growth reads 0 and the row must be marked **not measured**, not green | `backup.rs :: compress_for_onchain` on the copy with the fixed `reference_timestamp` P0-A2 used | T1 | ⬜ |

**Two-sided rows:** A2 (the cautious choice returns) and A5 (what must not travel does not) — carrying *everything* passes A2 and fails A5.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/backup.rs :: BackupPayload`, `BackupDomainPermission` | ✅ | 23 top-level fields; `domain_permissions` / `cert_field_permissions` already `#[serde(default)]` — the pattern for every addition. `BackupDomainPermission` holds 7 fields |
| `rust-wallet/src/backup.rs :: BackupOutput` | ✅ | Carries `change` and `spendable`; **no `confirmed`** (BS-L3) |
| `rust-wallet/src/backup.rs :: collect_payload`, `import_entities` | ✅ | Every new table lands here, FK order respected; `domain_permissions` / `cert_field_permissions` inserted `INSERT OR IGNORE` |
| `rust-wallet/src/backup.rs :: deserialize_from_onchain` | ✅ | `serde_json::from_slice::<BackupPayload>` — no version check, no `deny_unknown_fields` (BS-L2) |
| `rust-wallet/src/database/migrations.rs` | ✅ | Tables `peerpay_received`, `peerpay_outbox`, `domain_protocol_permissions`, `transaction_inputs` exist; 39 `CREATE TABLE` names in total |
| `rust-wallet/src/handlers.rs :: wallet_import` / `backup.rs :: encrypt_backup` (file backup) | ✅ | Shares `collect_payload` — the `.hodos-wallet` file gains the same tables for free; T3b-P5's `.brc39` is a separate exporter |

## 6. Out of scope

The write path and its money (P2.2). The restore flow, report and classification (P2.3). Stripping token scripts (P3). The envelope/header (P4). Any schema change. BRC-38 mapping of these tables (T3b-P5). Carrying `derived_key_cache` "to be safe" — excluded by decision 11's design.

## 7. Rollback

One revert commit: the new `Backup*` fields go; payloads written meanwhile still decode on the old binary because every addition is `#[serde(default)]` and unknown JSON keys are ignored today. ⚠️ Once A6 ships, a *newer* payload on an *older* binary errors — which is the point.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| The permission columns were carried but mapped by position and `per_session` landed in `max_tx_per_session` — a cautious site became a silent one | A2 checks the **engine's decision**, not the column |
| `peerpay_received` was carried and restore re-credited payments already in `outputs`, doubling a balance | A4 diffs the whole canonical wallet; R-RESTORE's balance leg in P2.3 |
| T1-P3's `money_utxos` got "travels" in the manifest because it looked like data, and restore trusted a stale index | A5 + decision 2a; P2.3-A3 |
| A newer payload restored on an older binary and dropped the permission tables silently | A6 |
| The manifest was written from `BackupPayload`, so it agreed with itself | P1-A3's second RED (a column added without a manifest row) |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | all | Rust `cargo test`; no platform code |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~0.25 h** — read the manifest's *excluded* list (§12 Q1) |
| Human-bound rows | none |
| Unknowns (K) | **No.** Known fixes on a known struct |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | **T3a-P1** | H2 gate, `canonical()`, mock chain |
| needs (soft) | **T1-P3** | `money_utxos` exists so the manifest can classify it *re-derived* |
| needs (soft) | **T5-P3** | the requester child table's name, to classify it *excluded* |
| gives | **T3a-P2.3** | the permission / payment tables the restore report can count |
| gives | **T3b-P5** | the manifest is the list the BRC-38 exporter maps from (which of our tables have no BRC-38 home) |

## 12. Open questions for the owner

1. **The excluded list** (§2 third item). Recommendation: carry `peerpay_outbox` (an in-flight BRC-29 delivery lost on restore is a *recipient-side* loss — BS-M5) and exclude the rest as logs/caches. Yours to confirm at G6.
2. No evidence that a G2 decision is wrong.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed (A2–A4: RED designed by the second agent, name recorded)
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
