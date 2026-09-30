# B5-T2-P3 — You can receive an ordinal and send it on purpose, and it survives the trip · PHASE CONTRACT

**Track:** B5-T2 1Sat Ordinals · **Tickets:** `../../tickets/TICKET_token_outputs_lost_if_crash_between_broadcast_and_record.md` (item) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 track agent · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`. Read them before filling this in.
**G2 decisions carried:** **8** (ordinals: receive + deliberate transfer; ⛔ **no BSV-21 transfer**) · **2 / 7** (a received ordinal is never money; unresolved ⇒ held and shown) · T2 **Q3** (BRC-150: **verify if present, do not produce**) · **Q5** (indexers for display/discovery only — never a spend input) · **Q7** (patterns only).

---

## 1. Goal

A user can receive an ordinal from a 1Sat or Yours wallet, see whether its origin is proven or only claimed, and send it to an address on purpose — with the ordinal never used to pay fees, never landing in a change output, and never lost to a crash.

## 2. Done means

- [ ] **Receive (BRC-147 import flow):** `internalizeAction` with the sender's BEEF + basket insertion `1sat` ⇒ filed per P1; the tip→origin walk (BRC-158/159/160 checks, SDK `internalizeOutpointBeef` as **pattern**) runs over the supplied BEEF; the row is marked **verified** when the walk reaches a valid first `ord` envelope, **claimed** otherwise. A BRC-150 `provenance` object is verified **if present**; none is produced.
- [ ] **Receive at a plain address** (an ordinal paid to a wallet address, e.g. a Yours-era `ordAddress` from `YOURS_LEGACY_ORD_RECEIVE_PROTOCOL`): found by sync, filed or held by P1's recognisers once T1-P4's real scripts exist; origin **claimed** until walked.
- [ ] **Transfer from the wallet panel** (internal call, no engine prompt — `R-INTEXT`; its own confirm screen names the ordinal and the destination address): one `createAction` whose **input 0 is the ordinal** and **output 0 is the 1-sat P2PKH to the recipient**; fees and the 1,000-sat service fee come from `default` coins; change to `default`. The ordinal's satoshi lands in output 0 (sat-ordering), never in change.
- [ ] The ordinal input is signed with its real key: from the row's derivation fields, or — for a row received by basket insertion, which stores no derivation — from the CI's `protocolID/keyID/counterparty` triple via the existing BRC-42 derivation. ⚠️ **Invariant 3: a new signing call site — owner approval before the code** (§12 Q1).
- [ ] **Crash contract** (the ticket), per token type, written into this contract before code and proven: *self-derivable* (a key the wallet can re-derive from its own addresses) ⇒ re-discoverable by sync; *CI-only* (key known only from the sender's CI) ⇒ **write-ahead** before the step that can lose it, and cleanup is **affirmative** (chain says the tx does not exist — `check_tx_exists_on_chain` returns *not found*), never on absence or error.
- [ ] Measurement rows for T3 (`HARNESS_DELTA.md` §1.3) recorded with numbers for every real ordinal received: `customInstructions` bytes, BEEF bytes, BEEF ancestry depth, and `beefB64` bytes — **stated honestly as zero/absent** where the sender emitted none.
- [ ] The receiving wallet (1Sat desktop/CLI or Yours 5.1.0) shows the transferred ordinal **with its origin** — the receiver's verdict (`R-BEEFOUT`'s rule).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-NOSPEND` | Nothing automatic spends an unclassified output | The transfer builder selects fee coins; the ordinal must be the **named** input, never a selected one; other 1-sat rows must never be selected as fee coins |
| `R-BEEFOUT` | A BEEF we hand over stands on its own | The transfer's BEEF goes to a receiver who must SPV-validate it; `build_beef_for_txid` caps at 50 ancestors and truncates silently |
| `R-TOKENPERM` | Pay grants never authorize token spends | The wallet-UI transfer must not open a route by which a dApp reaches the same builder silently |
| `R-INTEXT` | Internal never prompts, external always gates | The panel transfer is internal; a dApp-driven transfer of the same ordinal must still hit P2's prompt |
| `R-ONE-CLICK-ONE-SPEND` | One confirm signs exactly what was shown | The confirm screen's ordinal + destination must equal the built tx |
| `R-RESTORE` | Fail-closed survives recovery | CI-only ordinals are exactly what restore cannot re-derive — T3 edge |
| Service fee (CLAUDE.md) | Every outgoing tx carries the 1,000-sat fee output | Output order must stay *request → fee → change*, with the ordinal output first |

## 4. Evidence table

⛔ Transfer, signing and crash rows are money/crypto rows: RED by a second agent (`../../../RELEASE_CYCLE.md` §4.2).
**Subjects named before the run:** one real ordinal received from 1Sat desktop/CLI **or** Yours 5.1.0 into a Hodos scratch profile (inbound txid:vout `________`, origin `________`); the destination wallet and address `________`.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P3-A1` | Received ordinal (basket insertion with BEEF) ⇒ filed `1sat`, marked **verified** | Hand the same output with a BEEF whose origin tx has **no** `ord` envelope (or a forged `origin:` tag) ⇒ must be marked **claimed**, not verified (must be seen) | The stored row's verified/claimed flag + the walk's log naming the origin outpoint it reached | T1 + T2 | ⬜ |
| `P3-A2` | A BRC-150 `provenance` object, if present and valid, verifies; if absent, nothing fails | Corrupt one byte of a hand-built `beefB64` ⇒ verification fails, row stays **claimed** (must be seen) | Verifier result for that object | T1 | ⬜ |
| `P3-A3` | Transfer built from the panel: input 0 = the ordinal outpoint; output 0 = 1 sat to the recipient; next = 1,000-sat fee; last = change; no other 1-sat input present | Three switches, read from the serialised (`noSend`) tx · ① append the ordinal after the wallet-selected inputs in `handlers.rs :: create_action_internal` ⇒ input 0 ≠ the ordinal (seen) · ② emit change before the request output ⇒ output 0 is change (seen) · ③ revert fee selection to value-only (`output_repo.rs :: get_spendable_by_user` without T1's index) with a second held **≥2-sat** inscription in the wallet and default coins sized so the selector must take a small coin ⇒ that inscription is a fee input (seen; a 1-sat second row is refused by `is_token_reserved_value` and would mask this) · Right reason: outpoints and output scripts decoded from the bytes; the builder is the panel's production entry, not a test-built request · Residue: none (`noSend`; abort the pending tx and release its reservations) — designed by controls-D (Opus), 2026-09-28 | The **serialised transaction's** input outpoints and output values/scripts (`R-DUST` SUBJECT rule — never a broadcast result or balance) | T1 + T2 | ⬜ |
| `P3-A4` | Sat-ordering: the ordinal's sat index (sum of values of inputs before it = 0) falls in output 0 | Two switches · ① apply `randomizeOutputs` (today parsed and logged, never applied) with a fixed seed that moves output 0 ⇒ the ordinal's computed sat offset lands in fee or change (seen) · ② put one wallet input before the ordinal ⇒ offset = that input's value, which falls in output 1 or later (seen) · Also assert the panel builder passes `randomizeOutputs: false` explicitly, as `certificate_handlers.rs` does for PushDrop — BRC-100's default is true, so the pin must not rest on the feature being unimplemented (§4a) · Right reason: the offset function is first run on a hand-built tx with a known answer · Residue: none — designed by controls-D (Opus), 2026-09-28 | Computed from the serialised tx; ⚠️ `randomizeOutputs` is logged but **never applied** in `create_action_internal` today — this row pins that; if randomisation is ever implemented the row must go red | T1 | ⬜ |
| `P3-A5` | Ordinal input signs and the tx is accepted by the network (scratch, real sats) | ⚠️ For this input the sighash scriptCode is the **full** locking script (P2PKH + inscription), not 25 B · Fixtures: the received ordinal **and** one whose script exceeds T1-P4's 1,024 B inline cap (the offload path — T1 finding P4-X1) · Switch: sign with `outputs.locking_script` as stored pre-P4 (fabricated 25 B), or the empty offloaded value ⇒ the interpreter rejects locally and ARC rejects on submit (seen) — verify locally first, do not rely on the network alone · Right reason: local verification against the real script fetched by txid; a fee/policy rejection is not a red · Residue: GREEN spends real sats and moves the ordinal to the owner's second wallet (declared); a RED that reaches ARC leaves the ordinal reserved behind a failed tx (trip-wire-1 shape) — release it or delete the scratch profile — designed by controls-D (Opus), 2026-09-28 | Broadcast result **plus** WhatsOnChain showing the tx; the input's unlocking script verifies against the row's real locking script | T2 | ⬜ |
| `P3-A6` | Basket-insertion row (no derivation fields) signs using its CI triple (§12 Q1 approved) | Two switches, both needed · ① derive with decision 12's default counterparty (`self`/`anyone`) instead of the triple's verbatim one ⇒ derived pubkey hash ≠ the script's and the pre-sign comparison refuses (seen as refusal) · ② with ① still in, remove the pre-sign comparison ⇒ signing proceeds and the interpreter rejects the signature (seen) — proves the comparison is the guard that fired · Plus a forged-CI fixture (keyID altered) ⇒ refused before signing · Right reason: derived pubkey hash and script hash logged side by side before any signature exists · Residue: none — designed by controls-D (Opus), 2026-09-28 | The derived pubkey equals the pubkey hash in the row's real locking script — compared **before** signing | T1 | ⬜ |
| `P3-A7` | Receiver (1Sat/Yours) shows the ordinal **with its origin** | Control on the **receiver**: send a 1-sat output from an ordinary coin (no inscription ancestry) to the same receiving address ⇒ the receiving wallet must show it as a plain sat with no origin, not as an ordinal (seen) — proves the receiver checks origin rather than listing whatever arrives · Right reason: both txids named; the receiver's own indexer lookup for each; an output the receiver's indexer has not seen yet is INCOMPLETE, not red · Residue: 1 sat + fees in the owner's second wallet — declared — designed by controls-D (Opus), 2026-09-28 | The **receiving wallet's** display / its indexer's origin lookup — not our log | T2 + T3 | ⬜ |
| `P3-A8` | The BEEF we hand over validates at the receiver and states whether `MAX_BEEF_ANCESTORS` was reached | ⚠️ For an address-only transfer (§12 Q2) no BEEF is handed to the receiver: it learns the tx from chain, and ARC gets EF (R-BEEFOUT's carve-out) — the receiver-verdict half may have no subject (§4a) · Where a BEEF is handed over: ① build it via the ARC-tuned path that stops at a BUMP, or strip one ancestor to a txid-only stub ⇒ `beef.rs :: validate_beef_ancestry` fails and the receiver rejects (seen) · ② scratch build with `beef_helpers.rs :: MAX_BEEF_ANCESTORS` = 1 on an ordinal with ≥2 unproven ancestors ⇒ `build_beef_for_txid` returns Ok with a truncated BEEF; the result must say 'cap reached' and the receiver rejects (seen) · Right reason: hash of the exact bytes recorded with both verdicts · Residue: none — designed by controls-D (Opus), 2026-09-28 | `validate_beef_ancestry` on the exact bytes sent **and** the receiver's verdict (`R-BEEFOUT`) | T1 + T2 | ⬜ |
| `P3-A9` | **Crash, send path:** kill the wallet between the pre-broadcast insert and broadcast (scratch) ⇒ after restart the ordinal row, its tags and CI still exist; the pending tx is resolved **only** by an affirmative chain answer | Crash points via a scratch-build env switch (`process::abort()`): (a) after the pre-broadcast insert, before broadcast; (b) after broadcast, before the record update — the tx **is** on chain · Restart with the fake indexer provider returning 5xx / timeout · Switch: resolve with `check_tx_exists_on_chain`'s `Err` read as 'not found' ⇒ at (b) the tx is marked failed and the ordinal restored spendable though the chain has spent it (seen, trip-wire 1) · Correct build: pending until the chain answers; then (a) releases the ordinal with tags + CI intact and (b) marks it spent · Right reason: the resolving log line names the lookup's result kind (found / not found / error) · Residue: (b) really transfers the ordinal (declared); the RED leaves a spendable row for a spent outpoint in the scratch DB — delete the profile, never reuse it — designed by controls-D (Opus), 2026-09-28 | The `outputs` row (outpoint, basket, CI) after restart + the log line of the chain lookup that resolved it | T2 | ⬜ |
| `P3-A10` | **Crash, receive path (CI-only key):** kill between `internalizeAction`'s ownership check and its insert ⇒ after restart the ordinal is recoverable (write-ahead) | Kill point: scratch-build `process::abort()` after `internalize_action`'s ownership pass, before the insert, with the sender's tx already broadcast · Switch: remove the write-ahead record ⇒ after restart no row exists and nothing re-discovers it — sync cannot see a counterparty-derived key (seen) · Second switch: write the record **after** the insert ⇒ same loss (seen) · Right reason: record presence queried before restart; after recovery the row's outpoint, basket `1sat` and CI bytes equal the original request's · Residue: the RED leaves a real ordinal on chain that the scratch wallet does not know — recoverable by resubmitting the saved `internalizeAction` body; declared — designed by controls-D (Opus), 2026-09-28 | Write-ahead record + the recovered row | T2 | ⬜ |
| `P3-A11` | With P1's recognisers reverted, the fee-coin selection for the same transfer must **not** be able to reach the ordinal (named input only) — and with the P1 classifier + T1 guard reverted the ordinal **is** reached as a fee input (`R-NOSPEND` GREEN c and its RED) | ⚠️ Two masks: a `1sat`-filed row is excluded by the basket filter even with the recogniser reverted, and a 1-sat row by `is_token_reserved_value` even with P1 + T1 reverted — with the natural fixture the RED never fires (§4a) · Fixture: a ≥2-sat (546) inscription received at a plain address via the sync route (lands default, no sender basket) · Switch: revert P1's recogniser **and** T1's index to value-only ⇒ it appears in the fee selection set (seen); with only one reverted it must not · Size default coins so the selector must take a small coin, and state the `dust_threshold_sats` used · Right reason: selection set printed by outpoint · Residue: none (`noSend`) — designed by controls-D (Opus), 2026-09-28 | The selection set's outpoints | T1 | ⬜ |
| `P3-A12` | A dApp asking the same builder (external origin, same outpoint) still gets P2's per-action prompt | Two switches · ① revert P2's token branch ⇒ the external request goes `SilentWithinCaps` (seen) · ② if P3 adds a panel-transfer endpoint, call it from an approved origin with `X-Requesting-Domain` stamped ⇒ refused as first-party-only; remove it from `main.rs`'s first-party list ⇒ 200 and a built tx (seen) — see `P3-X1` · Right reason: decision reason + the stamped header in the Rust log; a header-free request is treated as internal and proves nothing here · Residue: none (`noSend`) — designed by controls-D (Opus), 2026-09-28 | `PermissionDecision` for the external request | T2 | ⬜ |
| `P3-A13` | The confirm screen names the ordinal (name/thumbnail + origin outpoint, verified/claimed) and the full destination address, before any signing | Remove the destination from the confirm payload ⇒ the reviewer marks FAIL (told a field may be missing, not which) | The rendered wallet-panel confirm screen, read by a person | T3 | ⬜ |
| `P3-M1` | **Measurement:** per received ordinal — CI bytes, BEEF bytes, ancestry depth, `beefB64` bytes (0 if absent) | A row with no number is INCOMPLETE, not green (`HARNESS_DELTA.md` §1.3) | The inbound `internalizeAction` body and the stored row | — | ⬜ |

**Two-sided rows:** `P3-A1` (verified) ↔ its RED (claimed); `P3-A11` halves; `P3-A12` ↔ the panel transfer staying unprompted.

### 4a. Independent control notes (2026-09-28)

*By controls-D (Opus). Findings only — the author's GREEN/SUBJECT cells are untouched. Most serious first.*

- `P3-A11` — **the RED as written cannot fire.** A `1sat`-filed ordinal is excluded by the basket filter with P1's recogniser reverted, and a 1-sat ordinal is excluded by `is_token_reserved_value` with P1 + T1 reverted. Fix: fixture = a ≥2-sat inscription received at a plain address (default basket), and say so in the GREEN.
- `P3-A4` — the pin rests on `randomizeOutputs` being unimplemented; BRC-100's default for it is true. Fix: GREEN also asserts the panel builder passes `randomizeOutputs: false`, as `certificate_handlers.rs` does for PushDrop.
- `P3-A8` — **subject may not exist.** With address-only destinations (§12 Q2) nothing hands the receiver a BEEF; ARC receives EF, which R-BEEFOUT's carve-out excludes. Either name the BEEF handoff (MessageBox / identity-key send — out of scope) or mark the receiver half N/A this release rather than green.
- `P3-A5` — fixture set must include an inscription larger than the 1,024 B inline cap: signing needs the full script as scriptCode, and the offloaded row is exactly where a 25-byte or empty `locking_script` reaches the sighash (T1 finding P4-X1).
- `P3-A9` — the ticket's loss window is **after** broadcast and before record; the GREEN names only 'between insert and broadcast'. Add kill point (b) with the indexer faulted.
- `P3-X1` (missing row — a new first-party spend endpoint) — GREEN: if P3 adds an endpoint for the panel transfer, a call with `X-Requesting-Domain` present is refused by `main.rs`'s first-party-only list · RED: remove it from the list ⇒ 200 and a tx built · SUBJECT: response code + no new `transactions` row. Same family as T1-P5 `P5-X1`; and per controls-C's T6-P5-A9 note, confirm the path is in `HttpRequestInterceptor::isWalletEndpoint` so a page cannot reach it header-free as 'internal'.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/handlers.rs :: internalize_action` | ✅ | Ownership pass (`our_output_indices`) then insert; basket-insertion rows get no derivation fields |
| `rust-wallet/src/handlers.rs :: create_action_internal` | ✅ | User inputs added **first**, wallet inputs after; basket outputs inserted before broadcast (send-path ordering already right); `randomizeOutputs` parsed but not applied |
| `rust-wallet/src/handlers.rs :: sign_action` | ✅ | User inputs: `spends` or row derivation fields; no CI-triple path today |
| `rust-wallet/src/handlers.rs :: select_utxos_with_preference` | ✅ | Fee coins; default basket only (T1 adds `change=1`) |
| `rust-wallet/src/handlers.rs :: check_tx_exists_on_chain` | ✅ | Affirmative-cleanup oracle; `Err` must never become a verdict (rule 7, trip-wire 2) |
| `rust-wallet/src/handlers.rs :: YOURS_LEGACY_ORD_RECEIVE_PROTOCOL`, `derive_yours_legacy_addresses_core` | ✅ | Yours-era ord receive addresses (BRC-42 self-derived) |
| `rust-wallet/src/handlers.rs :: HODOS_SERVICE_FEE_SATS` | ✅ | 1,000 sats; applies to this transfer |
| `rust-wallet/src/beef.rs :: validate_beef_ancestry`; `rust-wallet/src/beef_helpers.rs :: build_beef_for_txid`, `MAX_BEEF_ANCESTORS` | ✅ | Cap 50; `break` + `warn!` + `Ok(())` on cap (R-BEEFOUT note) |
| `rust-wallet/src/crypto/brc42.rs :: derive_child_private_key` (used by `handlers.rs :: derive_child_pubkey_to_address`) | ✅ | Reused as-is; **no** change to derivation math |
| `frontend/src/components/wallet/TokensTab.tsx` (via `frontend/src/pages/WalletOverlayRoot.tsx`) | ✅ | Existing tokens list; the transfer entry point is added here (P4 extends the same component) |
| 1Sat SDK `packages/actions/src/ordinals/index.ts`, `utils/ordinalRemittance.ts`, `utils/internalizeOutpointBeef.ts` at `52cfe69` | ✅ (no commits since 2026-09-20) | Transfer shape, CI shape, origin-walk checks — pattern only |
| BRC-147 §Transfer (input spending the tip, output `satoshis: 1`), §Security; BRC-150; BRC-159/160 | ✅ fetched 2026-09-28 | |

## 6. Out of scope

- ⛔ BSV-21 transfer (decision 8). ⛔ Producing BRC-150 packages (Q3).
- Sending an ordinal to an **identity key** (needs a delivery channel and a receiver-side contract) — §12 Q2.
- Marketplace listings (OrdLock), OpNS registration, minting.
- A standing grant for dApp-driven transfers (P2 decides prompts).
- Fixing the BEEF ancestor cap's silent truncation (`R-BEEFOUT` records it; this phase only reports whether the cap was hit).

## 7. Rollback

Revert the transfer-builder entry + UI commit and the receive-walk commit. The crash write-ahead record, if it adds a table, is a schema change (invariant 2 — see §12 Q3) and has its own down-migration; if it reuses `transactions`/`outputs` rows, rollback is code-only.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| Wallet input placed before the ordinal input ⇒ ordinal sat lands in the recipient's output at the wrong offset or in change ⇒ origin chain ends | `P3-A3`, `P3-A4` |
| Someone later implements `randomizeOutputs` ⇒ ordinal output moves | `P3-A4` pins it |
| Walk marks **verified** because tags say so | `P3-A1` RED |
| Signing path derives from the CI triple of a **forged** CI, producing a key that does not lock the output (harmless) — or the wrong counterparty default (`self` vs `anyone`, decision 12's trap) | `P3-A6` compares derived pubkey to the real script before signing |
| Crash cleanup deletes the row because the tx "isn't found" during an indexer outage | `P3-A9` — `Err` is not *not found* |
| We only check our own `validate_beef_ancestry`; receiver rejects | `P3-A7`, `P3-A8` receiver's verdict |
| Test uses an ordinal we minted ourselves, not a real peer's | Subject rule: inbound from 1Sat/Yours |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | Scratch profile; real sats |
| macOS | `P3-A1`, `P3-A3`, `P3-A5`, `P3-A13` | Same Rust; the confirm screen is React in the wallet overlay — check it renders and takes input in the macOS overlay. No C++ planned |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~2.0 h** — receive a real ordinal from Yours 5.1.0 or 1Sat desktop (~1.0 h, second wallet + visual check); deliberate transfer back out + confirm the receiver shows the origin (~1.0 h, real money, receiver's verdict) |
| Human-bound rows | `P3-A7` (receiver's display), `P3-A13` (confirm screen), the funding of `P3-A5` |
| Unknowns (K) — uncertainty, not difficulty | **K = 1 phase.** (a) Do dApps send ordinals to Hodos at a plain address or to a counterparty-derived key? The second is invisible to sync and is the crash/backup hazard; (b) whether the CI triple of real received ordinals is complete enough to re-derive the key |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| T2-P1 → T2-P3 | filing | Needed first |
| T2-P2 → T2-P3 | prompt | Needed first (`P3-A12`) |
| T1-P4/P5 → T2-P3 | real scripts, classifier | Plain-address receive path needs them |
| T1 (`created`-status reconcile ticket) | crash contract | Cited; not blocking |
| T2-P3 → T3a-P0/P2 | **measurement rows** `P3-M1` | ⚠️ Tell T3: BRC-150 `beefB64` rows will be ~zero (nobody emits them); the real workload is the internalize BEEF / cached parent txs |
| T2-P3 → T3a-P2.3 / T3b | restore of CI-only ordinals | Keys known only from CI must survive backup + restore (CI is in the backup payload; the write-ahead record must be too) — `R-RESTORE` |
| T2-P3 ↔ T5-P3 | counterparty defaults | CI-triple signing must use the triple's counterparty verbatim; never decision 12's default |

## 12. Open questions for the owner

1. **Invariant 3:** signing an ordinal whose key is described only by the sender's CI triple (`protocolID/keyID/counterparty`) is a **new signing call site** over the existing BRC-42 derivation (math unchanged). ⭐ Recommend approve — without it, ordinals received through the standard BRC-147 import flow cannot be sent from the panel. Guard: the derived pubkey must match the real locking script before signing.
2. **Destination = address only this release?** The scope said "address or identity key". Sending to an identity key needs a delivery channel and an agreed receiver contract. ⭐ Recommend address only; identity key to the next release.
3. **Crash write-ahead record:** reuse existing `transactions`/`outputs` states (no schema), or a new child table (invariant 2)? ⭐ Recommend reuse if the receive path allows it; decide at phase kickoff with the code open.
4. Confirm: a deliberate ordinal transfer pays the **1,000-sat service fee** like every outgoing transaction (CLAUDE.md). Carried as-is unless you say otherwise.

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
