# B6-P5 — An upgraded dApp's `listActions`, `listCertificates` and deferred-signing `createAction` stop throwing · PHASE CONTRACT

**Release:** `v0.4.0-beta.6` · **Source:** `../G2_OFFLINE_VALIDATOR_RESULT.md` B1, B1b, B2, B3 (ts-stack advisory TSA-272 and the new `@bsv/sdk` result validators) · **Status:** ✅ DONE (T1 + live, no broadcast)
**Opened:** 2026-09-30 · **Author:** Claude (Opus 5.5) · **Platforms:** shared Rust, both
**Standard:** `../../0.4.0-beta.3/HARNESS.md` + `../../0.4.0-beta.7/HARNESS_DELTA.md`.
**Owner decisions carried (2026-09-30):** fix B1 (BRC-100 statuses and input/output shapes), B2 (`totalCertificates`), B3 (honour `signAndProcess:false`); measure `createAction` with dApp-supplied inputs before closing; re-record and re-validate. **Deferred, not here:** B4 `{valid:false}` (beta.8), typed error envelope, `verifySignature` `self`, `noSend` phantom flag.

---

## 1. Goal

A dApp on the hardened `@bsv/sdk` (2.8.11) gets Hodos's `listActions`, `listCertificates` and two-phase `createAction` results without its own SDK throwing.

## 2. Done means

- [x] **B1** `listActions`: `status` is the stored BRC-100 word (`completed`, `unproven`, `nosend`, …), not `to_action_status`'s legacy word; inputs are `{sourceOutpoint, sourceSatoshis, sourceLockingScript?, unlockingScript?, inputDescription, sequenceNumber}` and outputs `{outputIndex, satoshis, lockingScript?, spendable, tags, outputDescription, basket, customInstructions?}`, filled **only from stored facts** (unlocking script and sequence from the stored raw tx; locking script, spendability, basket, tags from `outputs`); a description BRC-100 cannot carry (absent, or < 5 chars) is `""`. A DB error is an error, never "not ours".
- [x] **B2** `listCertificates` returns `totalCertificates`.
- [x] **B3** `createAction` with `signAndProcess:false` returns `signableTransaction { tx: AtomicBEEF (dApp inputBEEF + our ancestry + the unsigned tx), reference }` and signs at `signAction(reference)`. The `createAction` reference is base64 (12 random bytes, as wallet-toolbox mints it): the validator requires it, and `action-<uuid>` is not base64 — so the **existing** two-phase path (dApp input without an unlocking script) would have thrown too. The signable BEEF (dApp `inputBEEF` + our ancestry + the unsigned tx, topologically sorted) is built **before** the reservation is committed, with a 30 s timeout, so a failure is released by the guard; the reservation is then re-keyed from its placeholder to the txid (as the pre-change flow did), so an abandoned deferred action is released by `TaskFailAbandoned`. `sendWith` with `signAndProcess:false` is refused (400), not silently dropped.
- [x] Re-recorded against the rebuilt dev wallet: every B1–B3 failure gone; `createAction` with a dApp-supplied input validates end to end.

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| Internal UI | Our own pages are unaffected | No frontend or C++ code calls `/listActions` (grep); the browser only routes it |
| `R-NODOUBLE` | A deferred action's reservation is released if never signed | ⚠️ My first B3 draft broke this (review HIGH-1: the early return skipped the placeholder→txid re-key, and `TaskFailAbandoned` releases by txid only). Fixed and **measured**: T1 `p5_b3_abandoned_…` (real `task_fail_abandoned::run`; txid-held released, placeholder-held control not) + live (below) |

## 4. Evidence table

| ID | 🟢 GREEN | 🔴 RED | 🎯 SUBJECT | Tier | Result |
|---|---|---|---|---|---|
| `P5-B1` | All 8 statuses returned as stored; input/output shapes and their sources (sequence `0xFFFFFFFE` and unlocking script from the raw tx; our output `spendable`/basket/tags/customInstructions; a payee output `spendable:false`, `basket:""`, `tags:[]`); absent and 3-char descriptions ⇒ `""` | `negative_controls.py`: **B1a** legacy words · **B1b** description passed through (null) · **B1c** sequence invented · **B1d** spendable not read · **B1e** old input shape | `brc100_action_json` (what `list_actions` calls) over a real temp wallet DB | T1 | 🟢🔴 |
| `P5-B2` | `totalCertificates: 3` | **B2** rename removed | `ListCertificatesResponse` serialisation | T1 | 🟢🔴 |
| `P5-G2` | Live, rebuilt dev wallet, same recorders (`record.mjs` + `record2.mjs`), `validate.mjs`: **25 pass / 2 fail**; the 2 are `verifyHmac` / `verifySignature` `{valid:false}` (B4, deferred) | Same recorders on the pre-fix binary, same day (`recording_before_p5.json`): **21 pass / 6 fail** — `createAction` (B3), `listActions` ×2 (B1), `listCertificates` (B2), + the same 2 B4 | `@bsv/sdk` 2.8.11 `validateWalletResult` over real responses | T3 (no broadcast) | 🟢🔴 |
| `P5-B3` | **Abandoned deferred action, live (final binary):** `createAction{signAndProcess:false}` never signed ⇒ its 2 inputs held under the txid, 0 under any `pending-` placeholder; released by `TaskFailAbandoned` on its own (row 1003 `failed`, 0 held, 19:58:27). **Signed deferred action:** row 1007 re-keyed to the signed txid, `nosend`, input held under the signed txid. Live `record3.mjs`: wallet funds a throwaway dApp key (noSend) → `createAction` spending it with no unlocking script ⇒ `signableTransaction` (base64 reference) → the dApp signs its input with the SDK → `signAction` with `spends`: **3/3 validate**, and both transactions' scripts verify in the SDK interpreter (`tx.verify('scripts only')`) | B3's RED is `P5-G2`'s: the pre-fix `signAndProcess:false` call returned `txid` + a bare unsigned raw tx and failed "expected a complete, exactly framed BEEF" | Real responses + SDK script interpreter | T3 (no broadcast) | 🟢🔴 |

**Controls run:** `negative_controls.py` (this folder): **6/6 RED OK** on the final code, green after restore (2026-09-30). Final binary re-recorded: like-for-like **25 pass / 2 fail** (the 2 = deferred B4), `record3` 3/3 + both txs' scripts verify.

**Not falsifiable in T1 (declared):** the placeholder→txid re-key line itself (it sits inside `create_action_internal`'s full selection + signing setup); its effect is measured live above, and its mechanism by `p5_b3_abandoned_…`. Author-designed: B1/B2 change a response shape, not money or crypto. B3 is on the signing path; its RED is the observed pre-fix recording, and the adversarial review covered it specifically.

**Residue (working rule 7, declared):** dev wallet only, nothing broadcast by the probes. Rows 994, 996, 998, 1001 (`g2probe`) are `failed` with **no input still held** and their outputs disabled (checked read-only after the run). ⚠️ As predicted, the wallet's own `TaskBackup` then broadcast a routine on-chain backup, tx 1002 `e3a8dcf1…` (on chain: WhatsOnChain 200), the indirect cost of recording.

## 5. Blast radius (verified 2026-09-30)

| Cited code | Note |
|---|---|
| `rust-wallet/src/handlers.rs :: list_actions`, new `brc100_action_json`, `brc100_description` | B1 |
| `handlers/certificate_handlers.rs :: ListCertificatesResponse` | B2 (its two `discover*` siblings already had the rename) |
| `handlers.rs :: create_action_internal` (signAndProcess:false branch; reference minting) | B3. `internalize_action`'s own `action-<uuid>` reference is never returned to a dApp and is unchanged |
| `beef_helpers.rs :: build_beef_for_txid`, `beef.rs :: Beef` (`sort_topologically`) | Reused unchanged |
| Pre-existing, found by the review, **filed not fixed** (beta.8 intake) | `TICKET_commission_rows_deleted_when_an_action_is_signed.md` — **measured: 3 commission rows across 636 outgoing transactions**; `update_txid` deletes the fee record on every sign. Nothing reads the table back, so accounting loss, not poisoning. `TICKET_live_reservation_placeholders_fails_open_on_poisoned_lock.md` |

## 6. Out of scope

B4 `{valid:false}`, typed error envelope, `verifySignature` `self`, `noSend` phantom flag: beta.8. `listActions` defaults `includeInputs`/`includeOutputs` to **true** (BRC-100 says false); unchanged.

## 7. Rollback

One commit; `git revert`. No schema, no data. References minted after the change are base64; old rows keep theirs.

## 9. Platforms

Shared Rust; T1 via `cargo test`.

## 12. Open questions for the owner

None.

---

## Sign-off

| Item | Result | Date | By |
|---|---|---|---|
| preflight `-Full` | PASS — all checks ran | 2026-10-01 | Claude (Opus 5.5) |
| preflight `-NegativeControl` | PASS — every gate seen to fail | 2026-10-01 | Claude (Opus 5.5) |
| `cargo test --workspace` | 0 failed across 20 targets (binary 634, lib 482). The first run was stopped by the host for low memory (2026-09-30), re-run on the owner's go-ahead | 2026-10-01 | Claude (Opus 5.5) |
| P5 negative controls | 6/6 RED OK (final code) | 2026-09-30 | Claude (Opus 5.5), author-designed |
| adversarial review | Second agent (Claude Opus 5.5). **Closed:** HIGH-1 abandoned deferred action stranded its inputs (re-key restored; T1 + live); HIGH-2 BEEF failure after the commit stranded inputs with no reference (BEEF built before the commit); MEDIUM unsorted BEEF (sorted); MEDIUM no timeout under `create_action_lock` (30 s); MEDIUM description length in chars vs the SDK's bytes (bytes); `sendWith`+deferred silently dropped (refused). **Filed:** commission deletion, fail-open placeholder set. **Recorded:** `internalize_action` still mints `action-<uuid>` (returned by `listActions` as `referenceNumber`, a field the validator does not check); a deferred action signed later skips single-phase extras (overlay submit, backup nudge), as the existing two-phase path does | 2026-09-30 | second agent + author |
