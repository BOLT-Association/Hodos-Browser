# G3 — integration pass over the cross-track edges

**Written:** 2026-09-28 by the orchestrating session, after all 41 phase contracts existed.
**Input:** the seven edges `SESSION_PROMPT_G3_phase_contracts.md` requires closed, plus the edges the per-track
agents found. Per-track reports are summarised in `G3_RESUME_NOTES.md`.

⭐ **"Closed" here means:** both contracts name the edge, agree on who gives what, and a row (or a
serialization rule for G5) holds it. It does not mean the work is done.

## 1. The seven required edges

| # | Edge | Closed by | State |
|---|---|---|---|
| 1 | **T1 → T3a** — the money index is derived ⇒ omitted from backups and rebuilt; old backups carry `change=0` ⇒ restore classifies first. T1 lands before T3 format work | T1-P3 §11 + `P3-A12` (restore from an old backup rebuilds and can spend); T1-P5 §11; T3a-**P2.3** `A3` (old backup classified, then index rebuilt); T3a-P0 `A8` (index equals its defining query) | ✅ closed. Order: T1-P3 + T1-P5 before T3a-P2.3 and T3a-P4 |
| 2 | **T1-P6 ↔ T3a restore path** (`reconcile_backup_tx`, address strip) — T1 writes, T3 reviews | T1-P6 §11 (writes the semantics, counter write-back, scan-after-restore hook); T3a-P2.3 owns only BS-M8 parsing bounds and reviews T1's change (`P2.3-A11`) | ✅ closed. Whichever lands second rebases and re-runs |
| 3 | **T1-P4 → T2-P1** — the classifier needs real scripts (≥ ~256 B) | T1-P4 stores real scripts, inline cap recommended **1,024 B** (toolbox `maxOutputScript`; existing `settings.max_output_script`, no schema) + an explicit *not observed* state; T2-P1 `A6`/`A7` blocked on it | ✅ closed. Cap value is owner question T1-P4 Q1 |
| 4 | **T5-P3 → T3** — `derived_key_cache` not backed up; detector restarts empty after restore | T5-P3 §11; T3a-P2.1 classifies the new requester table as **excluded**; T3a-P2.3 restore report says the detector starts empty; T3b-P5 import preview says the same | ✅ closed. ⚠️ T5 found `derived_key_cache` is **write-only** (nothing reads it) — T3 SCOPE §6a's "keep the PushDrop lookup working" has nothing to protect |
| 5 | **T6-P5 claim endpoint** — T6 builds it, T1's money-path harness rows apply | T6-P5 rows `A1–A11` (money, independent controls pending); claim routes through T1-P5's classifier (route I5) and T1-P3's index; T1-P6 supplies card 2 | ✅ closed. T6-P5 needs T1-P3 + T1-P5; card 2 needs T1-P6 |
| 6 | **T0 Build 1 before any browser-level evidence** | T0 report names the gated rows: T1 `R-GOLD` + real-money sittings, T4 paid retry, T5 prompts/CWI, T6 overlays/DPI/Exit/update, T2/T3 UI rows. Rust/frontend builds are not gated. **New:** Build 2 gates the G9 release-candidate regression; land it before the final human sittings | ✅ closed. Scheduling rule for G5 |
| 7 | **T4-P1 reuses `check_tx_exists_on_chain`; `Err` never a verdict** | T4-P1 takes a **strict three-way** reading for its own two call sites (owner Q-P1-1 — deviates from "reuse" as literally written); T1-P2 fixes the lenient `wallet_cleanup` caller (`A10`/`A11`) and builds the check so T4-P1 adopts it rather than writing a second | ✅ closed. ⚠️ Correction: it is **one** function with three callers (`internalize_action`, `wallet_cleanup`, monitor PeerPay) — root `CLAUDE.md`'s "duplicated copy" is wrong |

## 2. Edges the agents found — closed in this pass

| Edge | Resolution |
|---|---|
| **T3a-P2 was split** into P2.1 (carry what matters), P2.2 (write path cannot hurt money), P2.3 (restore classifies & reports) | Every cross-track "T3a-P2" citation repointed: restore / classify / report / BS-C2 ⇒ **P2.3**; lock / baseline / BRC-177 ⇒ **P2.2** (T1-P5, T1-P6, T2-P1/P3/P4, T3b-P5, T3b-P6) |
| **`get_backup_hash` error ⇒ full backup** — T3b-P6 said "owned here" | Fix moves earlier to **T3a-P2.2 `A3`** (confirmed in code, 0 log hits prod + dev). T3b-P6 row updated: its RED becomes "revert P2.2-A3" |
| **`mark_failed` on an inconclusive oracle restores inputs** (T3a-P2.2) vs T4-P1 "unchanged by P1" | T4-P1 §5 annotated; ownership is owner question T3a-P2.2 Q1 (recommend T3a-P2.2, T4 + T1 review) |
| **T4-P1 ↔ T1-P3** — `release_unbroadcast_transaction` → `restore_by_spending_description`, which T1-P3 replaces | `release_unbroadcast_transaction` stays the only release call; T1-P3 `A7` (a coin T4-P1 kept as paid is never selectable) and `A8` |
| **T5-P3 ↔ T2-P2** — both edit `create_signature` | Serialize: **T5-P3 first** (both contracts say so). T2-P3 signing from the `customInstructions` triple uses its counterparty verbatim, never decision 12's default |
| **T5-P3 ↔ T1-P3** — both add a migration (schema at V25) | **T1-P3 first**; T5-P3 takes the next free version |
| **T5-P4 ↔ T6-P3 item F** — same `cef_browser_shell.cpp :: LaunchWalletProcess` adopt branch | Serialize: **T5-P4 first** |
| **T5-P5 usage ping switch + first-run notice** — T6's scope lacked it | **T5-P5 builds, T6 reviews** (a T6-P6 sign-off item, same Settings page) |
| **T5-P3 → T6-P7** — the protocol prompt now also fires for key fetches | T6-P7 `A7` covers "share a key" wording |
| **GET endpoints readable by approved sites** — `/wallet/tokens` (T2-P4), `/wallet/balance`, `/wallet/activity`; and `/wallet/cleanup`, `/wallet/rescan`, `/wallet/release-nosend` not first-party-only (T1, T4) | All go to **T5-P2's allow-list** decision. T2-P4 `A0` measures `/wallet/tokens` first; T4-P1 kickoff checks `release-nosend`. Code reading only — no row claims a defect yet |
| **T2-P4 ↔ T1-P5** — one place per outpoint | T2-P4 `A6`; T1-P5 `A10` ("Treat as money") must also refuse held token-shaped rows — carried to the independent-control round for T1-P5 |
| **T3a-P4 reserves `device_id` (16 B) + an extension field** | Gives T3b-P7's R4-1 room without a new format version; T3b-P6 builds deltas on P4's header (`kind=1` reserved) |
| **T0-P4 echo page → T6-P9 window globals** | ⛔ **Does not apply** — P9's globals are C++ process globals no page can see (T6-P9 §11). The echo page **can** record T5-P2's JS bridge globals |
| **T4-P1 "possibly paid" list → T6 Tools tab** | Not in beta.6 (T6-P5 Q1; beta.7 ticket recommended) |

## 3. Serialization the G5 map must carry (summary)

T0 Build 1 → all browser-level evidence · T1-P3 → T1-P5 → T3a-P2.3 → T3a-P4 · T1-P4 → T2-P1 · T1-P3 → T5-P3 (migration) · T5-P3 → T2-P2 (`create_signature`) · T5-P4 → T6-P3-F · T6-P3-Z → T6-P7 → T6-P10 (permission-type ids) · T6-P2 → T6-P4/P10 · T6-P3-F + T6-P1 → T6-P4 · T3b-P5 in parallel with T3a-P2..P4 (build needs T1-P3, T1-P5, T3a-P1; its round-trip `A4` closes after T3a-P2.3) · T0 Build 2 → final human sittings + G9.

## 4. Still open after this pass — not edges, recorded so they are not lost

- **Independent negative controls:** **208 RED cells in 29 contracts** read `⏳ independent control — second agent`. G3 is **not certified** until a second agent has designed them (RELEASE_CYCLE §4.2). Planned as a Fable run (owner's end-of-day slot).
- **Owner questions** in every contract's §12 — indexed per track in `G3_RESUME_NOTES.md`.
- **Shared-doc fixes the agents recommended** (T6 SCOPE names, T4 `X402_INTEGRATION.md` stale text, `WATCH_fungibles.md`, `NEXT_CHROMIUM_BUILD.md` Q2=b3, `PRIOR_ART.md` duplicate row, layer `CLAUDE.md` rows, root `CLAUDE.md`'s `check_tx_exists_on_chain` line) — listed per track in `G3_RESUME_NOTES.md`; not applied in this pass except the one G3 owed by decision 8 (README "Basket and permission mechanics": BRC-147 → **BRC-165**).
