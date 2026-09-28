# B5-T3a-P1 — Seed-only restore → spend is tested in CI before anything changes · PHASE CONTRACT

**Track:** B5-T3a Backup you can trust (single device) · **Tickets:** none directly; the harness is the instrument every later T3 row runs on · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Fable 5.1), G3 agent for T3a · **Platforms:** both (Rust `cargo test`; runs in `scripts/preflight.ps1` leg `T1a`) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** 2a (the money index is derived; restore rebuilds it — the comparator must include it), 3, 5 (⭐ the required *export file ⇄ on-chain restore* round-trip test — this phase builds its **instrument**, see §2), 7 (restore never fails outright on unidentified coins — the harness must be able to assert that), `SCOPE.md` §0, `IMPLEMENTATION_PLAN.md` Phase 1 / H1 / H2 (survives per SCOPE §5.9), retrospective constraint 13 (*never ship a recovery-affecting change without a round-trip test*).

---

## 1. Goal

Any change to what the backup carries, how it is written, or how it is restored is caught by a test that restores a wallet from the twelve words alone against a mock chain and **spends** from it — before that change reaches a user.

## 2. Done means

- [ ] `rust-wallet/tests/backup/` exists and runs under `cargo test` (preflight `T1a`): today `rust-wallet/tests/` holds **zero** backup tests (listing re-checked 2026-09-28 — `tier3`…`tier12`, `beef_crypto_cert_test`, `sdk_interop_test`, `sighash_transaction_test`, `diagnostic_test`; `grep -rl backup tests/` = nothing)
- [ ] **H1** — fresh DB → seed only → restore against the mock chain → build and sign a spend of (a) a `2-receive address` output and (b) a BRC-42 counterparty (PeerPay-shaped) output → mock ARC accepts — is green **or red with filed findings** on today's shipped format (a red H1 on today's code is a finding, not a blocker to landing the test)
- [ ] **H2** — a checked-in coverage manifest classifying every live `table.column` as *travels / re-derived (with its correctness oracle) / excluded (tier, reason)*; the test enumerates the live schema via `PRAGMA table_info` and fails on any unclassified column, so a migration cannot land without a backup decision
- [ ] ⭐ **The canonical wallet comparator** (`canonical(db) → bytes`: sorted, id-remapped, volatile columns excluded per the H2 manifest, **money-index rows included** once T1-P3 lands) exists as a library function every later row reuses — decision 5's "identical wallet" is defined **here, once**
- [ ] The **mock chain** implements exactly the endpoints the backup/restore path consumes, with scriptable faults (index lag, 404, 429, truncated hex, ARC 200-with-other-txid, `DOUBLE_SPEND_ATTEMPTED` for both, accepted-but-lost) and the **real** P2PKH spent-status semantics (`NoSignal` — R1-01), per plan §5
- [ ] ⛔ The backup path's two literal `https://api.whatsonchain.com/…` call sites (`handlers.rs :: fetch_onchain_backup` and `do_onchain_backup` Step 5c / `adopt_onchain_backup`) are routed through `services::WalletServices` (the `IndexerProvider` seam every other indexer call already uses) so the mock can be injected — **the minimum code change that makes the harness possible**, and the closing of H16's "zero uncontracted call sites" for this path
- [ ] Fixture wallets A (300 tx / 100 UTXOs), B (mixed + 500 text-token rows); C (image ordinals) is **stubbed** with synthetic 1-sat PushDrop rows until T2-P1 produces real ones (edge, §11)
- [ ] `R-RESTORE` (`REGRESSION_ADDITIONS.md`) becomes **runnable** at this phase (its note says "not until track 4" — that is the old track number; see report)

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-INTEXT` | internal never prompts | Routing `fetch_onchain_backup` through `WalletServices` touches no HTTP handler gate; but a refactor near `do_onchain_backup` must not add an `X-Requesting-Domain` path — boundary run |
| `R-DUST` / `R-NOSPEND` | no incidental spend of a 1-sat output | H1's spend rows use the **production** selectors; a harness that builds its spend with a private selector proves nothing about the guard — SUBJECT below |
| `R-RESTORE` | fail-closed survives recovery | This is the phase that makes it runnable; both halves must be **seen** red on the pre-T1 format (silently spendable) before the T1 guard lands |
| `R-PEERPAY-DELIVERY` half 1 | backup funding smallest-sufficient | The Step 5c refactor must not touch Step 6 selection; the existing `a3_backup_funding_is_smallest_sufficient` test is the control |
| Invariant 8 (CEF) | — | Not touched: Rust only |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. Money, schema and crypto rows carry the second-agent placeholder.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P1-A1` (H1 restore) | Fixture A: backup on the mock → wipe every local file → `wallet_recover_onchain` with the mnemonic only ⇒ success; refetch `errors == 0`; `canonical(restored) == canonical(original)` **minus exactly the H2 exclusion list** (every diff line names a manifest row) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The **restored** DB (a fresh file, new process), diffed with the comparator; never the pre-wipe DB, never the response JSON's counts | T1 (mock) | ⬜ |
| `P1-A2` (H1 spend) | From the restored wallet: build + sign (a) a `2-receive address` coin and (b) a BRC-42 counterparty coin through the **production** `create_action_internal` path (no_send) ⇒ full script verification against the mock's recorded prev-out scripts passes; mock ARC accepts | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The signed transaction's inputs and the verifier's verdict; the selector must be `select_utxos_with_preference` / `select_all_spendable` (production), named in the run | T1 | ⬜ |
| `P1-A3` (H2 gate) | The manifest classifies every column `PRAGMA table_info` returns across all live tables (36 after migrate — `database/CLAUDE.md`), on both axes (equivalence, correctness); `cargo test` fails on an unclassified column naming `table.column` | Remove one column from the manifest (e.g. `outputs.confirmed`) ⇒ H2 **red naming `outputs.confirmed`**. Second RED: add a column to a scratch migration in the test DB without a manifest row ⇒ red naming it | The test's own failure message; the schema enumerated from the **migrated** test DB (`WalletDatabase::migrate`, gate `current_version < 25`), not a hand-typed list | T1 | ⬜ |
| `P1-A4` ⭐ *the comparator* (decision 5's instrument) | `canonical()` is deterministic (two calls on one DB ⇒ identical bytes), id-remap-invariant (renumber `outputId`/`transactionId` consistently ⇒ identical bytes), and **sees money**: flip one `outputs.change` bit (or, after T1-P3, delete one index row) ⇒ different bytes | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The bytes; the three properties each asserted by its own test. ⚠️ The volatile-column exclusion list is the H2 manifest's, not a second list |
| `P1-A5` *mock chain fidelity* | For each consumed endpoint, the mock's response shape equals P0-A11's recorded fixture byte-for-byte in the fields the code reads; `check_outpoint_spent` on a plain P2PKH outpoint returns `Unknown` (both providers `NoSignal`), exactly as the live probe found | Change the mock to return `200 {spent:true}` for P2PKH ⇒ the fidelity test goes red citing R1-01. Second: rename `tx_hash` in the unspent fixture ⇒ red | The fixture files from P0-A11 diffed against the mock's output; `reconcile.rs :: SpentStatus::Unknown` observed | T1 | ⬜ |
| `P1-A6` *fault injection works* | Each scriptable fault, when armed, is **observable** by a probe test: lag hides a marker for N ticks; 429 carries `Retry-After`; truncated hex fails `hex::decode`; ARC returns 200 with a different txid; accepted-but-lost returns no response | Arm each fault with its probe **disabled** in the mock (fault flag on, injection code stubbed) ⇒ the probe test must go red — a fault the harness cannot see is a fault H5/H6 will never exercise | The probe's observation per fault, listed | T1 | ⬜ |
| `P1-A7` *the seam* | `fetch_onchain_backup`, `adopt_onchain_backup` and Step 5c reach the chain only through `WalletServices`; with the mock provider injected, a full `cargo test` run makes **zero** outbound connections | Restore the literal URL at one site ⇒ the harness's "no network" assertion (a `reqwest` builder that refuses non-loopback, or a firewall probe) goes red at that site | The **network layer** (connection attempts), not a grep. ⚠️ A grep gate for `api.whatsonchain.com` in this path is worth adding — **in its own commit after the fix, with `-NegativeControl`** (working rule 6), never inside this change | T1 | ⬜ |
| `P1-A8` *R-RESTORE runnable* | `R-RESTORE` GREEN a/b are expressible against fixture B's held-token rows through the restored DB: (a) the token row is not selectable, (b) it is present and shown. On the **pre-T1** format the expected result is recorded honestly (a: RED — silently spendable today) | Force everything `Spendable` ⇒ (a) fails; force everything `Unknown` ⇒ (b) must still show — each half is the other's control (the row's own design) | The restored `outputs` rows by outpoint, plus the selector's candidate set; **never** the balance total | T1 | ⬜ — RED expected until T1-P5; do not round up |
| `P1-A9` *harness self-check* | Deleting `backup.rs :: import_entities`'s outputs loop makes A1 red at the diff, **not** earlier (a control that goes red upstream tested nothing — `HARNESS.md` §2) | This row *is* the control for A1's instrument: the failure must be *inside* the comparator, naming `outputs` | The failing assertion's location | T1 | ⬜ |

**Two-sided rows:** A1 (nothing lost) and A2 (nothing invented or unspendable) — a restore that inflates the balance (the E1-58cf9a3 class) passes neither; A8's two halves.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/handlers.rs :: fetch_onchain_backup` | ✅ | Builds a raw `reqwest::Client` and hits literal WoC URLs (`unspent/all`, `tx/{txid}/hex`); picks `max_by_key(height, 0 ⇒ i64::MAX)`; shared by `wallet_backup_onchain_verify` and `wallet_recover_onchain` — the refactor must keep all three callers |
| `rust-wallet/src/handlers.rs :: adopt_onchain_backup`, `do_onchain_backup` Step 5c | ✅ | Same literal-URL pattern (`unspent/all`, `tx/hash/{txid}` JSON — the JSON endpoint truncates large `scriptPubKey.hex`, which is why `adopt_onchain_backup` uses `/hex`; keep that) |
| `rust-wallet/src/services/mod.rs :: WalletServices`, `services/provider.rs :: IndexerProvider` | ✅ (roster in `src/CLAUDE.md`) | The seam. `fetch_utxos` chain is WoC → GorillaPool Ordinals; a backup-marker lookup must not silently fall through to the ordinals API (which cannot see plain P2PKH) — pin the provider for these calls |
| `rust-wallet/src/backup.rs :: import_to_db_with_ids`, `import_entities` | ✅ | One DB transaction; the pre-import `DELETE`s in `wallet_recover_onchain` are **outside** it (BS-H4 — P2.3, not here) |
| `rust-wallet/src/reconcile.rs :: SpentStatus`, `check_outpoint_spent` | ✅ | `NoSignal` semantics the mock must reproduce |
| `rust-wallet/src/database/connection.rs :: WalletDatabase::migrate` | ✅ | The harness creates test DBs through it (V25) |
| `rust-wallet/Cargo.toml` | ✅ | **No `[dev-dependencies]` HTTP-mock crate today**; the mock is an in-process `IndexerProvider`, not an HTTP server — no new network dependency needed |
| `scripts/preflight.ps1` `Invoke-CargoTest -Id 'T1a'` | ✅ | `cargo test --manifest-path rust-wallet/Cargo.toml` — a `tests/backup/` suite is picked up with no instrument change |

## 6. Out of scope

Fixing anything H1/H2 find (P2). Deltas (H3), two devices (H4), padding (H12), cost accounting beyond E5 (P0), the live H18 restore (P2.3 / P4). A BRC-38 exporter (T3b-P5). Changing the format, the strip rules or the payload shape. Replacing `reqwest` (T1-P1). An HTTP-level mock server (an in-process provider is the seam; if a later phase needs HTTP fidelity, that is its own item).

## 7. Rollback

One commit for the `WalletServices` routing (revertible: the three call sites go back to their literal clients); one commit for `tests/backup/` + fixtures + manifest (deletable). No schema, no format.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| H1 went green because the "wipe" left the old DB file and the restore refused (`409`) while the test read the surviving DB | A1's SUBJECT: a fresh file in a new process; the harness asserts the data dir is empty before restore |
| The spend row used a harness-private coin selector, so R-NOSPEND-shaped bugs stay invisible | A2 names the production selector |
| The manifest was generated **from** the payload structs, so it can never disagree with them | A3's second RED: a column added to the DB without a manifest row must go red — the manifest is checked against `PRAGMA`, not against `BackupPayload` |
| The mock returned "spent" for P2PKH and every P4 recency test later passed for the wrong reason (R1-01 again) | A5 |
| The refactor onto `WalletServices` let backup-marker lookups fall through to the GorillaPool ordinals provider, which returns nothing for plain P2PKH — "no backup found" on a healthy wallet | §5 note; A7's provider pinning; P2.3-A1 (indexer error ≠ no backup) |
| A red H1 on today's format was "fixed in the test" (assertion loosened) instead of filed | Invariant 13: a failing test pointing at production code stops and asks; the contract's §2 says a red H1 is a **finding** |
| The comparator excludes `change` as "volatile", so the money index never enters the diff and decision 5's test is vacuous | A4's "sees money" property |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | all (`cargo test`) | Rust-only; the harness is platform-neutral. 🍎 One thing differs: `create_wallet_from_existing_mnemonic` stores the mnemonic through DPAPI on Windows and Keychain on macOS — A1 exercises that write on each platform; record both |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **0 h** (agent-run; mock only). The owner reviews the H2 manifest's *excluded* rows at G6 — reading, not testing |
| Human-bound rows | none |
| Unknowns (K) — uncertainty, not difficulty | **No.** Hard-but-understood. The one open input (WoC history shape) comes from P0-A11 and is needed by P4, not here |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | **P0-A11** | recorded endpoint fixtures for A5 (the mock's ground truth) |
| needs (soft) | **T2-P1** | real ordinal rows for fixture C; until then synthetic 1-sat PushDrop rows, declared as such in the fixture |
| needs (soft) | **T1-P3** | the money-index table, so `canonical()` can include it (A4's "sees money" runs on `change` until then) |
| gives | **every later T3a/T3b row** | the harness, the mock, the comparator, the manifest |
| gives | **T3b-P5** | ⭐ the comparator is the instrument for decision 5's *export file ⇄ on-chain restore* test; **T3b-P5 owns running it** (it is the first phase where a strict `.brc39` file exists) — see the report |
| gives | **T1-P6** (Q6 edge) | a harness in which `reconcile_backup_tx` changes can be tested against the March phantom (T1 NC-1) — T1 writes, T3 reviews |
| gives | `REGRESSION_ADDITIONS.md` `R-RESTORE` | runnable from this phase (A8) |

## 12. Open questions for the owner

None. (Decision to route the two WoC call sites through `WalletServices` is agent-level: minimal code, reuse-first, and it is the only way a mock can reach that path; named here so it is visible in the diff.)

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed (A1/A2/A4: RED designed by the second agent, name recorded)
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded (R-RESTORE now runnable; expected state recorded honestly)
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../../RELEASE_CYCLE.md` §4.1a
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
