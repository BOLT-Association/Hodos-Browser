# B5-T3a-P4 — The backup format is frozen before it goes on chain, and a crash or a slow indexer can never restore the wrong backup · PHASE CONTRACT

**Track:** B5-T3a Backup you can trust (single device) · **Tickets:** closes `../research/ONCHAIN_BACKUP_REVIEW.md` **BS-H1, BS-H3 (re-verify first), BS-H5, BS-H6, BS-M2, BS-M7**; `../research/FIX_B_CRASH_SAFETY_SHUTDOWN_PLAN.md` (plan D7); retires `../research/FOLLOWUP_RECORD_BEFORE_BROADCAST_TOKENS.md`'s "correct as-is" (its own trigger fired — T2 adds non-self-derivable outputs) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 agent for T3a (resumed) · **Platforms:** both (Rust) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** **5** (the on-chain format is ours, never claims BRC-38; ⛔ **frozen before the first mainnet broadcast of a new token version** — from then on it must decrypt forever, plan G11), **4** (provisional: one active device — the header **reserves** the room T3b-P7's R4-1 answer needs, it does not decide it), 4(a) (deltas ride this header — T3b-P6), 2a (restore still rebuilds the index — P2.3's path, unchanged), `SCOPE.md` §0, §5 P4, §9 Q6 (*format-first for what is irreversible*). Plan D4/D5 (KDF and backup address) **stand unchanged** — owner-settled.

---

## 0. Why this is the irreversible phase

Today's on-chain token is headerless: `nonce(12) ‖ AES-256-GCM(gzip(JSON)) ‖ tag(16)` (`backup.rs :: encrypt_compressed` / `deserialize_from_onchain`, re-read 2026-09-28) — no magic, no version byte, no sequence, no parent link. The `version` field lives **inside** the encrypted JSON. Every token already on mainnet is in that shape and must keep decoding (H15). The first token broadcast with a header is a format this wallet — and anyone implementing T3b-P8's BRC — supports **forever**. So the header is designed, tested and **signed by the owner** before that broadcast, and every field a known next step needs is reserved now rather than forcing a second version.

---

## 1. Goal

Whatever crashes, lags or litter the chain throws at it, a restore finds the newest real backup and a backup write either completes or leaves nothing it cannot finish — on a token format the owner has frozen and that later deltas and a second device can extend without a new version.

## 2. Done means

- [ ] **Header (plan Phase 4), bound as AES-GCM additional authenticated data:** `magic ‖ format_version ‖ kind (0 = snapshot; 1 reserved for T3b-P6 deltas) ‖ seq ‖ parent_txid ‖ device_id (reserved: all-zero = unassigned) ‖ ext_len ‖ ext (TLV, empty in v1)`. Nothing in the header influences tip selection before the tag verifies (plan D6 rule 2, *decrypt before trust*). `prev_payload_sha256` (plan D3): decided and recorded here, in or out
- [ ] **Legacy decode kept:** the headerless shape is detected by attempting the headed decode first and falling back to the legacy decode on authentication failure — never by a magic-byte match alone (a random nonce can begin with any bytes)
- [ ] **H15:** one checked-in binary fixture per token version ever broadcast to mainnet (starting with today's headerless shape, captured before this phase changes the writer) decodes on every later build
- [ ] **Intent record (plan D7, Fix B)** persisted **before** broadcast: planned token outpoint, `seq`, parent txid, payload hash, **every** reserved input (funding included), and the complete signed raw transaction; stored in a table **excluded from the payload by construction** (a new table — invariant 2 ask, §12 Q1); consulted by the reconcile decision table on **every** trigger (startup, 3-hour, event) — a live intent blocks a new build; multiple intents processed oldest-first to a terminal state
- [ ] **Three-valued broadcast outcome:** OK / REJECTED(permanent reason) / UNKNOWN. UNKNOWN ⇒ no rollback, no input restore, no rebuild — re-broadcast the cached identical raw transaction until the chain decides (BS-H6: a *suspected* double-spend never triggers rollback on its own; BS-M2: a crash before the baseline write re-uses the intent, never spends again)
- [ ] **Discovery (plan D6):** bootstrap from the backup address's **full history** (P0-A11's contract), every candidate decrypted, junk skipped and counted (never abort on one bad candidate); tip = the decryptable marker no other decryptable marker names as parent (child-existence); the spent-status oracle only corroborates — `Unknown` decides from child-existence alone (BS-H1, BS-H5, BS-M7). Pre-header tokens walk by input 0
- [ ] **BS-H3 re-verified at kickoff:** `restore_pending_placeholders` no longer exists (the reservation sweep replaced it — `output_repo.rs` doc comment); either a crash-between-broadcast-and-record test proves the phantom cannot recur, or it is fixed here via the intent record
- [ ] ⛔ **Freeze gate (human, irreversible):** before the first mainnet broadcast of the headed token, the owner signs the header layout (date + commit in the sign-off table); T3b-P5-A4's round-trip and H18 are re-run on the frozen format

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| Plan G11 / trip-wire 3 | every mainnet token decrypts forever | The writer changes shape; a reader that drops the legacy path strands every existing user's backup |
| Invariant 3 | no crypto change without asking | Binding the header as GCM AAD changes the encryption construction (not the KDF, not the key) — **owner ask** (§12 Q2) |
| Invariant 2 | no schema change without asking | The intent table — **owner ask** (§12 Q1) |
| Working rule 7, trip-wire 2 | no verdict where an error is owed | UNKNOWN broadcast outcomes and `Unknown` spent status are exactly where the old code guessed |
| `R-RESTORE` | fail-closed survives recovery | Discovery changes which payload is restored; P2.3's classify-then-rebuild runs on whatever is chosen |
| Service fee | backups carry none | Intent-driven re-broadcast rebuilds nothing, so no fee path is added |

## 4. Evidence table

⛔ Money, schema and crypto rows: RED designed by a second agent (`../../../RELEASE_CYCLE.md` §4.2).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P4-A1` ⭐ *H15 legacy decodes forever* | Today's headerless token fixture (captured from the current writer before any change) and every later version's fixture decode and restore on the new build | **Switches:** (1) delete the legacy fallback ⇒ the headerless fixture fails to decode ⇒ red · (2) ⭐ replace *try headed, fall back on authentication failure* with magic-byte dispatch ⇒ a legacy fixture whose 12-byte nonce **begins with the magic** is routed to the headed path and fails ⇒ red; today's writer draws a random nonce, so this fixture is built by encrypting with a chosen nonce under `derive_onchain_backup_key` (marked *synthetic, legacy shape* in the fixture manifest) · (3) provenance: the captured fixture's commit predates the phase's first writer change (`git log --follow`), its SHA-256 matches the recorded hash, and it decodes under the pre-phase `deserialize_from_onchain` — a fixture regenerated by the new writer passes both ways · **Red for the right reason:** decode error per fixture naming the path taken (headed/legacy); `canonical()` diff vs the fixture's source DB · **Residue:** none — designed by controls-E (Opus), 2026-09-28 | The fixture bytes (checked in, hash recorded) → `canonical()` of the restored wallet vs the fixture's source | T1 | ⬜ |
| `P4-A2` ⭐ *header is authenticated* | Flipping any header byte (seq, parent_txid, device_id, ext) of a headed token ⇒ decryption fails; the token is skipped as junk, never used | **Switches:** (1) build with the header **not** bound (empty AAD) ⇒ flips of `seq`, `parent_txid`, `device_id`, `ext` decrypt successfully ⇒ red per position · (2) ⭐ flip **every** header byte position in a loop, and assert the error *kind*: flips of `magic`/`format_version`/`kind`/`ext_len` fail by parsing whether or not AAD exists, so a green on those positions proves nothing about AAD · (3) header transplant: header of valid token A + nonce/ciphertext of valid token B (same key) ⇒ must fail — catches AAD bound to a constant or to a subset of fields · (4) the legacy fallback must not rescue a flipped token (it fails authentication too) — assert the final result is *junk, skipped* and the skip counter increments, and the candidate is absent from tip selection · **Red for the right reason:** per position: error kind = authentication-tag failure for the AAD fields · **Residue:** none — designed by controls-E (Opus), 2026-09-28 | The decode result per flipped byte position | T1 | ⬜ |
| `P4-A3` ⭐ *H5 crash matrix* | Hard kill at every step boundary of the write path (before reserve, after reserve, after sign, after broadcast/before intent-consume, mid-record, after record) crossed with index lag 0 / 30 s / 5 min: after restart, zero double-spends, zero phantom spendable rows, at most one extra identical re-broadcast | **Switches:** (1) SCOPE's control adopted: intent record disabled ⇒ at the *after broadcast / before record* kill point, restart rebuilds and spends the same inputs in a second tx ⇒ the mock ARC shows two distinct txids sharing an input ⇒ red (the 2026-04-11 shape) · (2) ⭐ each kill point must be **proven reached** (the test hook writes a marker file, the process exit code is the kill code) — a kill point never hit gives a clean restart and a free green · (3) define double-spend at the mock as *two different txids spending one outpoint*; identical re-broadcasts count separately against *at most one* · (4) phantom check is the trip-wire-1 query: `spendable=1` rows whose outpoint the mock shows spent, and `spendable=0 AND spent_by IS NULL` rows with no live reservation · (5) lag axis: at 5 min the restart's view of the chain must be the lagged one — assert the mock returned *unknown* for the broadcast txid at restart time · **Red for the right reason:** the mock's per-outpoint spend map across the restart + the two queries · **Residue:** scratch data dirs per cell; mock chain holds the extra txs — declared, mock only — designed by controls-E (Opus), 2026-09-28 | The mock ARC's received transactions (by txid and inputs) + `outputs` rows by outpoint after each restart | T1 | ⬜ |
| `P4-A4` *UNKNOWN is not failure* | ARC returns no response (accepted-but-lost) and, separately, 200 with a different txid: no rollback, inputs stay reserved, the cached raw tx is re-broadcast, the record completes when the chain shows the txid | **Switches:** (1) map UNKNOWN to failure (today's shape: `broadcast_transaction` Err ⇒ `rollback_backup` ⇒ `restore_by_spending_description`) ⇒ inputs are released, the next tick builds a different tx over them ⇒ mock shows a conflicting second txid ⇒ red · (2) *200 with a different txid*: record the returned txid instead of our own ⇒ the wallet's tip names a tx the chain never has ⇒ red · (3) ⭐ two-sided (missing from GREEN): a genuine REJECTED(permanent, e.g. script failure) must roll back and release inputs — an implementation mapping every failure to UNKNOWN passes this row and locks funds forever; include that case in the same test · (4) *record completes when the chain shows the txid*: completion must be driven by the mock's tx-status endpoint, never by the ARC response — flip the mock's status to *never seen* ⇒ record stays open · **Red for the right reason:** input outpoints' state per tick; the mock's rebroadcast log (same raw bytes each time) · **Residue:** none beyond the mock — designed by controls-E (Opus), 2026-09-28 | Input outpoints' state across ticks; the mock's rebroadcast log | T1 | ⬜ |
| `P4-A5` *intent consulted on every trigger* | A live intent from a crashed run blocks a new build from the 3-hour tick and from an event trigger, not only at startup; two live intents resolve oldest-first | **Switches:** (1) consult the intent only on the startup path ⇒ the 3-hour tick and the event trigger each build a second backup over the live intent ⇒ mock shows a second backup tx ⇒ red · (2) ⭐ vacuity: `do_onchain_backup` Step 2 skips when the payload hash is unchanged, so *no build* is also what an unconsulted intent produces on an idle wallet — mutate wallet state between the crash and each trigger so a build is otherwise due, and assert the log shows the intent-block reason, not the hash-skip reason · (3) oldest-first: reverse the order ⇒ order assertion red; the two intents must conflict (the second's parent is the first's token) so order is observable · (4) prove each trigger ran: one log line per trigger path reaching the decision point · **Red for the right reason:** per-trigger log (reason), the mock's tx list, and the terminal-state order with timestamps · **Residue:** scratch profile with crashed-run intents — declared — designed by controls-E (Opus), 2026-09-28 | The build log per trigger path + the order intents reach terminal state | T1 | ⬜ |
| `P4-A6` *H7 recency, three branches* | Snapshot-only chains; for every token t with the indexer's newest visible token pinned at t: restore picks the true tip by child-existence; spent-status `Spent` ⇒ refetch history, never restore stale; `ExplicitUnspent` ⇒ proceed; `Unknown` ⇒ child-existence decides | **Switches:** (1) revert tip pick to `max_by_key(height, 0 ⇒ i64::MAX)` over the unspent-only query (BS-H5/M7) ⇒ with the indexer pinned at t < tip, restore picks a stale token ⇒ `canonical()` diff vs the true tip ⇒ red · (2) branch by branch: make `Spent` proceed (no history refetch) ⇒ stale restore at t < tip; make `Unknown` abort or treat as unspent-and-final ⇒ a healthy tip fails or a stale one wins ⇒ red · (3) ⭐ vacuity: at t = tip every implementation passes — the red is only meaningful at t < tip, so the test asserts the pinned t for each red · (4) the oracle is the mock's real `NoSignal` for P2PKH (T3a-P1-A5), never a stub that returns `Spent`/`Unspent` for convenience · **Red for the right reason:** chosen tip txid vs true tip per t, and which branch fired (log) · **Residue:** none — designed by controls-E (Opus), 2026-09-28 | `canonical(restored)` vs the true state at the tip, per t; the mock serves real `NoSignal` for P2PKH (T3a-P1-A5) | T1 | ⬜ |
| `P4-A7` *H10 litter* | 50 foreign PushDrop tokens + crafted headers + an outranking dust marker at the backup address: restore picks the real tip, counts the skipped candidates, never aborts | **Switches:** (1) select the tip from header fields **before** decrypt ⇒ a crafted header with high `seq` wins ⇒ red · (2) abort on the first bad candidate (`?` on decode) ⇒ restore errors out ⇒ red · (3) the outranking dust marker: revert to height-based pick ⇒ the dust marker wins ⇒ red · (4) ⭐ count vacuity: assert the skip count **equals** the planted litter count (50 + crafted + dust) and that the mock's history response for the backup address contained them — a discovery that never fetched the litter also skips zero and picks the tip · **Red for the right reason:** chosen tip txid, skip count vs planted count, per-candidate reason · **Residue:** none (mock) — designed by controls-E (Opus), 2026-09-28 | The chosen tip txid + skip count | T1 | ⬜ |
| `P4-A8` *H6 corruption + mid-chain hole* | Truncated hex, bad GCM tag, valid-GCM-bad-JSON, a missing parent: each a typed error for that candidate; a hole healed from the history set; an unhealable hole ⇒ restore offered only from a complete state, flagged older-than-tip | **Switches:** (1) collapse the typed errors into one string ⇒ the per-case type assertion red · (2) ⭐ fall-back scope: if the headed path falls back to legacy on **any** error, *valid-GCM-bad-JSON* surfaces as the legacy path's authentication error — assert that case's error kind is the JSON/decompress error · (3) disable healing from the history set ⇒ a healable hole restores the pre-hole state or fails ⇒ red · (4) remove *complete state only* ⇒ an unhealable hole yields a restore offer built from a partial walk, not flagged older-than-tip ⇒ red · (5) two-sided with A6: the same fixture with no hole restores the true tip · **Red for the right reason:** error kind per case; the offer's declared state and flag · **Residue:** none — designed by controls-E (Opus), 2026-09-28 | Typed error per case; the restore offer's declared state | T1 | ⬜ |
| `P4-A9` *H17 reorg / eviction* | A 1–2 block reorg and a mempool eviction of the tip: re-broadcast the cached raw tx first; rebuild from the last chain-visible ancestor only on rejection; baseline marked dirty | **Switches:** (1) today's reflex: on eviction/reorg rebuild at once ⇒ a new tx spends the same inputs while the cached raw tx is still valid ⇒ mock shows a conflicting txid ⇒ red · (2) never rebuild even on rejection ⇒ after the mock permanently rejects the re-broadcast, no backup lands for N ticks ⇒ red (two-sided) · (3) baseline not marked dirty ⇒ after the reorg drops the tip, the next tick sees an unchanged hash and skips ⇒ chain has no current backup ⇒ red · (4) the rebuild parent must be the last **chain-visible** ancestor — assert the new token's parent txid · **Red for the right reason:** mock chain state after each event + the wallet's recorded tip txid + re-broadcast bytes identical to the cached raw tx · **Residue:** mock only — designed by controls-E (Opus), 2026-09-28 | Mock chain state + the wallet's recorded tip after each event | T1 | ⬜ |
| `P4-A10` *reserved room is really usable* | A test-only v1 token with a non-zero `device_id` and one unknown TLV in `ext` decodes on the v1 reader (fields surfaced, unknown TLV ignored) — so T3b-P6/P7 can use them without a new version | Make the reader reject non-zero `device_id` ⇒ the test goes red (proves the test exercises the reserved field, not a zeroed one) | The decoded header struct | T1 | ⬜ |
| `P4-A11` 👤 *freeze signed, then re-verified* | Owner signs the layout; then H18 live restore and T3b-P5-A4 re-run on the frozen format, both green | Before signing, the sign-off table's commit hash must equal the build that produced the H15 v1 fixture — a mismatch blocks the first broadcast (checked by the release step, not by memory) | The signed row (date, commit) + the two re-run results | T4 (human) | ⬜ |

**Two-sided rows:** A3 (nothing spent twice) ↔ A4 (nothing given up too early); A6 (never stale) ↔ A8 (never bricked by one bad link).

### 4a. Independent control notes (2026-09-28)

*controls-E (Opus), second agent per `../../../RELEASE_CYCLE.md` §4.2. Findings on GREEN/SUBJECT cells I did not rewrite. One line each: row — problem — suggested fix. Code facts are from reading, not from a run.*

- P4-A2 — *decryption fails* is satisfied by a parser for `magic`/`format_version`/`kind`/`ext_len` flips, AAD or not. GREEN should require the authentication-tag failure specifically for the AAD-only fields, and include a header-transplant case (header of token A on ciphertext of token B).
- P4-A4 — one-sided: an implementation that maps every failure to UNKNOWN passes it and holds inputs forever. Add REJECTED(permanent) ⇒ rollback + inputs released to the GREEN (A3 ↔ A4 pairing covers *too early*, nothing covers *never*). Note `rollback_backup` today discards every step's result (`let _ =`) — a failed release reads as released.
- P4-A5 — `do_onchain_backup` Step 2 skips on an unchanged payload hash, so *no second build* is also the idle-wallet outcome with no intent check at all; the GREEN needs a state change between crash and trigger and the intent-block reason in the log.
- P4-A1 / A8 — the legacy fallback must trigger on **authentication failure only**; falling back on any headed error turns A8's *valid-GCM-bad-JSON* into an authentication error from the legacy path and loses the typed error. State it in §2's legacy rule.
- P4-A1 — the magic-prefixed-nonce legacy fixture (pre-mortem row 2) cannot be *captured* from today's random-nonce writer; it must be constructed with a chosen nonce under the unchanged KDF and marked synthetic in the fixture manifest, beside the captured ones.
- P4-A3 — *zero phantom spendable rows* should name the query (trip-wire 1: `spendable=1` rows spent on the mock; `spendable=0 AND spent_by IS NULL` with no live reservation), and every kill point must be proven reached.
- P4-X1 (proposed) — §3 *Service fee: backups carry none* has no row. GREEN: every tx the intent path broadcasts or re-broadcasts has no output to `HODOS_FEE_ADDRESS` · RED: route the re-broadcast build through `create_action_internal` ⇒ a 1,000-sat fee output appears ⇒ red · SUBJECT: the mock ARC's received txs, outputs by script. (Same gap in T3b-P6 — see P6-X1.)

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/backup.rs :: encrypt_compressed`, `deserialize_from_onchain`, `derive_onchain_backup_key`, `serialize_for_onchain` | ✅ | Headerless; KDF `SHA-256(master ‖ "hodos-wallet-backup-v1")` (D4, unchanged) |
| `rust-wallet/src/handlers.rs :: do_onchain_backup` Steps 5b–13 | ✅ | Broadcast-then-record; suspected-double-spend relabel then `rollback_backup` (BS-H6) |
| `rust-wallet/src/handlers.rs :: rollback_backup`, `adopt_onchain_backup`, `fetch_onchain_backup` | ✅ | `max_by_key(height, 0 ⇒ i64::MAX)` tip pick (BS-H5); unspent-only query (BS-M7); literal WoC URLs (routed through `WalletServices` by T3a-P1) |
| `rust-wallet/src/handlers.rs :: broadcast_transaction` | ✅ | Two-valued today (Ok / Err) — the third value is added at the backup call site |
| `rust-wallet/src/arc_status.rs :: is_double_spend_error`, `SUSPECTED_DOUBLE_SPEND_PREFIX` | ✅ | |
| `rust-wallet/src/reconcile.rs :: check_outpoint_spent` | ✅ | `Unknown` for P2PKH on both providers (plan R1-01) — corroboration only |
| `rust-wallet/src/database/output_repo.rs` (reservation sweep doc comment: *"The previous `restore_pending_placeholders()` was a single…"*) | ✅ | BS-H3's function is gone; re-verify the phantom path |
| `BackupIntent` / intent table | ✅ absent (grep 2026-09-28: no match in `rust-wallet/src`) | New — invariant 2 ask |

## 6. Out of scope

Deltas and their envelope rules (T3b-P6 — rides this header's `kind=1`). Assigning device ids or any active-device mechanism (T3b-P7 / P7b — this phase only reserves the field). Chunking (plan: not in v1). Size cap and backoff (P2.2). Changing the KDF or backup address (D4/D5, owner-settled). Publishing the format (T3b-P8). Moving backup change off the user's receive-address namespace (SCOPE §7.3 (4) — derivation change, invariant 3; not asked here).

## 7. Rollback

**Before the first headed mainnet broadcast:** revert the phase's commits — the writer returns to headerless; the reader keeps both paths (never revert the legacy reader). **After:** the headed **reader** stays forever (H15); only the writer can be reverted. The intent table stays (empty) if the writer reverts. This is why A11 gates the first broadcast.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| The header shipped unauthenticated and a junk token with a high `seq` won tip selection | A2 + A7 |
| Legacy detection keyed on "first byte ≠ magic", and one old token whose nonce began with the magic failed to restore | §2 legacy rule; A1 over many legacy fixtures |
| The intent record was read only at startup; a 3-hour tick built a second backup over a live intent | A5 |
| UNKNOWN was treated as REJECTED and the 2026-04-11 double-spend came back | A3 + A4 |
| The history endpoint turned out to be capped/paginated (P0-A11) and discovery missed the tip on a long-lived wallet | K (§10) — P0-A11 before build; A6 on a fixture longer than the cap |
| P7 later needed per-device `seq` and forced a v2 two months after the freeze | A10 (reserved `device_id` + TLV really usable) |
| The freeze was signed on one commit and a different build broadcast first | A11's RED |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | all `cargo test` rows; A11's H18 re-run once on macOS | Rust-only; the H15 fixtures are platform-neutral bytes |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~1.25 h** — §12 Q1 + Q2 (schema + crypto asks), 0.5 h · freeze sign-off, 0.25 h · H18 re-run on the frozen format, 0.5 h |
| Human-bound rows | A11 (irreversible outward event + live restore) |
| Unknowns (K) | **Yes, 1:** whether WhatsOnChain's address **history** is complete and pageable enough for D6 (P0-A11). If not, discovery needs a second history provider (plan D12) — design-level, found before the build if P0-A11 runs first |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | **T3a-P0** A11 | the address-history contract D6 bootstraps from |
| needs | **T3a-P1** | mock with lag, ARC faults (UNKNOWN, other-txid), kill points |
| needs | **T3a-P2.2** | three-valued baseline, lock, backoff — the write path this extends |
| needs | **T3a-P2.3** | classify-then-rebuild on whatever discovery picks |
| needs | **T1-P3** | reservation model the intent record's input list uses (one ownership mechanism — SCOPE §3.2) |
| gives | **T3b-P6** | the header deltas ride (`kind=1`), `seq`, `parent_txid`, intent record, discovery |
| gives | **T3b-P7** | ⭐ reserved `device_id` + `ext` TLV, so R4-1's answer (e.g. an active-device marker in the tip) needs no new version — or P7 records that it does |
| gives | **T3b-P5** | the frozen on-chain format P5-A4 re-runs against |
| gives | **T3b-P8** | the frozen header, D6, D7 the BRC text describes |

## 12. Open questions for the owner

1. **Schema (invariant 2):** a new intent table (Fix B), excluded from backups by construction. Recommendation: yes — it is the only way a crash between broadcast and record cannot double-spend (A3), and a table (not a `settings` column) is needed for the multi-intent rule.
2. **Crypto construction (invariant 3):** bind the plaintext header as AES-GCM additional authenticated data. Key, KDF and address unchanged. Recommendation: yes — without it the header is attacker-editable and D6's decrypt-before-trust has nothing to trust.
3. **`device_id` width** (reserved now, assigned by T3b-P7b): recommendation 16 bytes, zero = unassigned.
4. No evidence that a G2 decision is wrong.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed (A1–A9: RED designed by the second agent, name recorded)
- [ ] 👤 **Freeze signed before the first headed mainnet broadcast** — date + commit: ______
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
