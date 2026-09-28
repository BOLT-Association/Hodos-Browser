# B5-T3a-P3 — Token media stays off the chain backup and comes back byte-for-byte on restore · PHASE CONTRACT

**Track:** B5-T3a Backup you can trust (single device) · **Tickets:** none closed directly; delivers plan Phase 3 / H13 and SCOPE §7.3 guarantee (3) (no nulled `transaction_id` on carried outputs) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 agent for T3a (resumed) · **Platforms:** both (Rust payload + restore refetch) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` (§1.1 — SUBJECT names the output) + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** 2 / 7 (a rehydrated token must classify as a token, never money; an output whose bytes cannot be re-fetched is **held and shown**), 5 (the on-chain payload is ours — integrity here is for **our** importer and T3b-P5's comparator, not a BRC-38 claim), 8 (T2 files `1sat` / `bsv21`), `SCOPE.md` §0, §5 P3 (reason corrected: **media in held scripts**, not BRC-150 ancestry — nobody produces `beefB64`).

---

## 0. What changed since SCOPE was written — the phase is narrower than planned

T1-P4 (*ingest truth*, contract re-read 2026-09-28) stores a locking script **inline only up to a cap** (recommended 1,024 B, its §12 Q1); above it `outputs.locking_script` is empty and `script_length` / `script_offset` point into the raw transaction, whose parent row is **exempted from purge**. `backup.rs :: collect_payload` copies `outputs.locking_script` as it finds it, so **after T1-P4 the large media scripts already leave the payload**. ⇒ P3 no longer has to *invent* the strip. It has to make the **return trip** true: every carried output whose script was not carried comes back **byte-identical**, from bytes proven to be the right transaction, and the restored token is **spendable** (T2-P3 transfers it). Whether the ≤ 1 KB held-token scripts are also worth stripping is decided from P0-E2/E6, not assumed (§12 Q1).

---

## 1. Goal

A restored wallet holds every ordinal and token it had — with the exact bytes that lock it — while the on-chain backup carries only what cannot be fetched again.

## 2. Done means

- [ ] **Rehydrate covers every carried output with no carried script**, not only outputs whose transaction row survived the strip — today `handlers.rs :: refetch_stripped_data` builds its fetch list from `payload.transactions` + `payload.proven_txs` only, so a held token whose transaction row was dropped is never re-fetched
- [ ] **Fetched bytes are proven before use:** every raw transaction fetched on restore hashes to the requested txid (`reconcile.rs :: verify_raw_txid`) before any script is extracted or any parent cached; a mismatch is an error for that outpoint, never a silent write
- [ ] **Byte-match (H13):** every rehydrated `locking_script` equals the original byte-for-byte, and its length equals the carried `script_length`
- [ ] **Referential integrity:** no carried output has a nulled `transaction_id` — `backup.rs :: compress_for_onchain`'s "Null out orphan FK references" is replaced by carrying a minimal transaction stub (txid, status) for every carried output's transaction, including `backup-%` change (SCOPE §7.3 guarantee 3; also what T3b-P5-A4's comparator needs)
- [ ] The restored token is **spendable where T2 allows it**: the parent raw transaction is cached in `parent_transactions` so a BEEF can be built (T2-P3's transfer), and it is in its basket (`1sat` / `bsv21`), never in `money_utxos`
- [ ] An output whose bytes cannot be re-fetched (indexer down, 404) is **held and listed** in P2.3's report — never dropped, never guessed
- [ ] Payload bytes before/after measured on both P0 wallets (fixture C with real T2-P1 ordinals, not synthetic)

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-RESTORE` a + b | unclassifiable ⇒ not spendable, not lost | An output with no bytes after restore must be held + shown; a rehydrate that drops it on 404 fails (b) |
| `R-NOSPEND` / `R-DUST` | a 1-sat output is never incidentally spent | A rehydrated inscription that loses its script classifies as nothing — it must still be held, and never enter the index |
| `R-TOKENPERM` | token spends prompt per action (BRC-165) | Untouched, but T2-P3 transfers what this phase restores — the permission path is T2's, the bytes are ours |
| Working rule 7, trip-wire 3 | durable state | A wrong rehydrated script is written to the DB as fact; signing against it later fails or, worse, succeeds on the wrong output — the txid proof prevents it |
| Invariant 2 / 3 | no schema / crypto change | None: payload fields and restore logic only |

## 4. Evidence table

⛔ Money and token-state rows: RED designed by a second agent (`../../../RELEASE_CYCLE.md` §4.2).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P3-A1` ⭐ *H13 byte-match* | Fixture C (real T2-P1 ordinals incl. one image inscription over the cap and one BSV-21 output): backup → wipe → seed-only restore on the mock ⇒ every rehydrated script byte-identical to the original, length = `script_length` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2). SCOPE's suggested control, for the second agent to adopt or replace: corrupt one byte of one rehydrated transaction in the mock | Per outpoint (txid:vout, sats, basket): original script bytes vs restored bytes | T1 | ⬜ |
| `P3-A2` ⭐ *proven bytes only* | The mock serves a **different** valid transaction for one requested txid ⇒ that outpoint is held and reported; nothing from the wrong transaction is written (no script, no `parent_transactions` row) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The outpoint's restored row + `parent_transactions` for that txid + the report entry | T1 | ⬜ |
| `P3-A3` *every missing script is fetched* | A held token whose transaction row the strip dropped is rehydrated (its txid is in the fetch list) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The fetch list (mock request log) and that outpoint's restored script | T1 | ⬜ |
| `P3-A4` ⭐ *restored token is a token and can move* | After restore, the image ordinal is in basket `1sat`, not in `money_utxos`, and T2-P3's transfer builds and signs a BEEF for it (mock ARC accepts) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | That outpoint's basket, index membership, and the signed transfer's input 0 | T1 | ⬜ — needs T2-P3 |
| `P3-A5` *unfetchable ⇒ held + shown* | Indexer down during rehydrate: the over-cap ordinal is held, listed in the restore report, not selectable; restore completes | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The outpoint in the restored DB + the report's Unknown list (P2.3-A5 is the same control on the classify side — keep both) | T1 | ⬜ |
| `P3-A6` *referential integrity* | The decoded payload has **zero** outputs with `transaction_id = null`; every `spent_by` resolves; P1's `canonical()` diff shows no `transaction_id` lines | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The decoded payload JSON (not the DB); count of null FKs | T1 | ⬜ |
| `P3-A7` *measured* | Payload bytes on both P0 wallet copies before/after (T1-P4's cap alone, then this phase), compressed, recorded beside P0-E2/E6 | Stub the new stubs out ⇒ the "after" equals "before" and the row must read **not measured** | `compress_for_onchain` with P0-A2's fixed `reference_timestamp` | T1 | ⬜ |

**Two-sided rows:** A1 (bytes come back) ↔ A2 (wrong bytes never come back); A4 (a token restores as a token) ↔ A5 (an unfetchable one is held, not lost).

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/backup.rs :: collect_payload` | ✅ | Carries `locking_script` for **spendable** outputs, strips it for spent ones; carries `script_length` / `script_offset` |
| `rust-wallet/src/backup.rs :: compress_for_onchain` ("Null out orphan FK references") | ✅ | Nulls `output.transaction_id` / `spent_by` / `basket_id` when the referenced row was stripped |
| `rust-wallet/src/handlers.rs :: refetch_stripped_data` | ✅ | Fetch list = `payload.transactions` ∪ `payload.proven_txs` txids; writes `outputs.locking_script` for rows with an empty script via `extract_output_script`; no txid proof visible at this call site |
| `rust-wallet/src/cache_helpers.rs :: fetch_parent_transaction_from_api` | ✅ | Delegates to `services.get_raw_tx(txid)`; whether `WalletServices` verifies the txid is **to check at kickoff** |
| `rust-wallet/src/reconcile.rs :: verify_raw_txid` | ✅ | Exists, tested (upper/lower case, wrong txid, malformed); used once in `handlers.rs` today |
| T1-P4 accessor for over-cap scripts, T2-P1 baskets, T2-P3 transfer | ⬜ not yet built | Re-verify at kickoff |

## 6. Out of scope

Choosing the inline cap (T1-P4 §12 Q1). Classification rules (T1-P5 / T2-P1). Transfer and its permission (T2-P2/P3). The envelope/header (P4). BRC-150 `beefB64` ancestry (nobody produces it — T2 SCOPE). Stripping sub-cap token scripts unless §12 Q1 says so.

## 7. Rollback

Revert the phase's commits: the refetch list and proof check revert together; the stub-carrying change reverts alone (payloads written meanwhile still decode — the stubs are ordinary transaction rows).

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| Rehydrate trusted an indexer that returned another transaction and wrote a wrong script, and the token later became unspendable | A2 |
| A token whose transaction row was stripped was never re-fetched and restored with an empty script — shown, but untransferable | A3 + A4 |
| The rehydrated script matched, but the parent was not cached, so T2-P3 could not build a BEEF | A4 |
| The fixture used synthetic PushDrop rows and passed; real inscriptions (envelope, over-cap) failed | §2 last item + A1's SUBJECT (real T2-P1 fixture) |
| Stubs for `backup-%` transactions made restore re-classify backup change as a user payment | P2.3-A3 re-run on this phase's payload; T1-P6's backup-change identification |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | all | Rust `cargo test`; no platform code |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~0.25 h** — §12 Q1 from P0's numbers |
| Human-bound rows | none (A4's live transfer, if wanted, rides T2-P3's sitting) |
| Unknowns (K) | **No.** Hard-but-understood; the one input (the cap) is T1-P4's decision |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | **T1-P4** | real scripts, the inline cap, the over-cap accessor, purge exemption — ⚠️ **blocking**: a rehydrate that compares against a fabricated script proves nothing |
| needs | **T2-P1** | real ordinal / BSV-21 rows for fixture C; baskets |
| needs | **T2-P3** | the transfer path A4 signs through |
| needs | **T3a-P2.3** | the restore report and classify-then-rebuild this phase's held outputs flow through |
| gives | **T3b-P5** | referential integrity the export ⇄ restore comparator relies on (P5-A4) |
| gives | **T1-P6** | guarantee (3): outpoint + transaction link kept for de-duplication |

## 12. Open questions for the owner

1. **Strip the small token scripts too?** After T1-P4, only scripts ≤ the cap remain inline. Recommendation: **no**, unless P0-E6 shows they are a material share of the payload — a 1 KB script saved is one fetch added to every restore.
2. No evidence that a G2 decision is wrong. The scope narrowing in §0 follows from T1-P4's contract, not a decision change.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed (A1–A6: RED designed by the second agent, name recorded)
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
