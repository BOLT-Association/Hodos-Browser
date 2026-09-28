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
| `P4-A1` ⭐ *H15 legacy decodes forever* | Today's headerless token fixture (captured from the current writer before any change) and every later version's fixture decode and restore on the new build | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The fixture bytes (checked in, hash recorded) → `canonical()` of the restored wallet vs the fixture's source | T1 | ⬜ |
| `P4-A2` ⭐ *header is authenticated* | Flipping any header byte (seq, parent_txid, device_id, ext) of a headed token ⇒ decryption fails; the token is skipped as junk, never used | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The decode result per flipped byte position | T1 | ⬜ |
| `P4-A3` ⭐ *H5 crash matrix* | Hard kill at every step boundary of the write path (before reserve, after reserve, after sign, after broadcast/before intent-consume, mid-record, after record) crossed with index lag 0 / 30 s / 5 min: after restart, zero double-spends, zero phantom spendable rows, at most one extra identical re-broadcast | ⏳ independent control — second agent (RELEASE_CYCLE §4.2). SCOPE's suggested control: intent record disabled ⇒ the 2026-04-11 double-spend shape reappears | The mock ARC's received transactions (by txid and inputs) + `outputs` rows by outpoint after each restart | T1 | ⬜ |
| `P4-A4` *UNKNOWN is not failure* | ARC returns no response (accepted-but-lost) and, separately, 200 with a different txid: no rollback, inputs stay reserved, the cached raw tx is re-broadcast, the record completes when the chain shows the txid | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Input outpoints' state across ticks; the mock's rebroadcast log | T1 | ⬜ |
| `P4-A5` *intent consulted on every trigger* | A live intent from a crashed run blocks a new build from the 3-hour tick and from an event trigger, not only at startup; two live intents resolve oldest-first | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The build log per trigger path + the order intents reach terminal state | T1 | ⬜ |
| `P4-A6` *H7 recency, three branches* | Snapshot-only chains; for every token t with the indexer's newest visible token pinned at t: restore picks the true tip by child-existence; spent-status `Spent` ⇒ refetch history, never restore stale; `ExplicitUnspent` ⇒ proceed; `Unknown` ⇒ child-existence decides | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | `canonical(restored)` vs the true state at the tip, per t; the mock serves real `NoSignal` for P2PKH (T3a-P1-A5) | T1 | ⬜ |
| `P4-A7` *H10 litter* | 50 foreign PushDrop tokens + crafted headers + an outranking dust marker at the backup address: restore picks the real tip, counts the skipped candidates, never aborts | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The chosen tip txid + skip count | T1 | ⬜ |
| `P4-A8` *H6 corruption + mid-chain hole* | Truncated hex, bad GCM tag, valid-GCM-bad-JSON, a missing parent: each a typed error for that candidate; a hole healed from the history set; an unhealable hole ⇒ restore offered only from a complete state, flagged older-than-tip | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Typed error per case; the restore offer's declared state | T1 | ⬜ |
| `P4-A9` *H17 reorg / eviction* | A 1–2 block reorg and a mempool eviction of the tip: re-broadcast the cached raw tx first; rebuild from the last chain-visible ancestor only on rejection; baseline marked dirty | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Mock chain state + the wallet's recorded tip after each event | T1 | ⬜ |
| `P4-A10` *reserved room is really usable* | A test-only v1 token with a non-zero `device_id` and one unknown TLV in `ext` decodes on the v1 reader (fields surfaced, unknown TLV ignored) — so T3b-P6/P7 can use them without a new version | Make the reader reject non-zero `device_id` ⇒ the test goes red (proves the test exercises the reserved field, not a zeroed one) | The decoded header struct | T1 | ⬜ |
| `P4-A11` 👤 *freeze signed, then re-verified* | Owner signs the layout; then H18 live restore and T3b-P5-A4 re-run on the frozen format, both green | Before signing, the sign-off table's commit hash must equal the build that produced the H15 v1 fixture — a mismatch blocks the first broadcast (checked by the release step, not by memory) | The signed row (date, commit) + the two re-run results | T4 (human) | ⬜ |

**Two-sided rows:** A3 (nothing spent twice) ↔ A4 (nothing given up too early); A6 (never stale) ↔ A8 (never bricked by one bad link).

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
