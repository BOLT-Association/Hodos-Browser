# G3 — resume notes (usage limit hit 2026-09-28)

Owner decision 2026-09-28: remaining tracks run on **Opus** (Fable stopped for usage). Relaunch per-track agents with the brief in `SESSION_PROMPT_G3_phase_contracts.md`, telling each to KEEP the contracts already in its `phase-P*` folders and write only the missing phases. Then: independent negative-control agents (Opus) for every RED cell marked `⏳ independent control`, the integration pass, README G3 certify, MAC relay round.

## Contract status
- ✅ T0 Build 2 — P3, P4, P5-lite (`b735c69`)
- ✅ T4 — P1, P2 (`0280986`)
- 🟡 T1 — P1 only · owed P2–P7
- 🟡 T2 — P1, P2, P3 · owed P4
- 🟡 T3a — P0, P1 · owed P2 (agent planned P2 parent + P2.1 sub-phase), P3, P4
- ⬜ T3b — none · owed P5–P8
- 🟡 T5 — P1 · owed P2–P5
- 🟡 T6 — P1, P2 · owed P3–P7, P9, P10
- Partial files (`9f0c7dc`) are unreviewed; their agents never filed a report.

## Finished agents' reports (orchestrator notes)
## T0 Build 2 (Opus) — committed b735c69
- P3 adblock pull (b3), P4 Hodos brand, P5-lite verify (L0–L13; build/pin/publish live in L0–L4)
- Build 1 engine = negative control for P3/P4
- Owner-h ≈2.9 (P3 0.4, P4 0.25, P5-lite 2.25). K=2 (P3, P4). Human: L0 fork push ⛔, L3 asset upload ⛔, L10 basket/DPI, L11 R-GOLD, P3-A11 visual
- Edges: Build1 gates browser-level evidence for T1 R-GOLD, T4 paid retry, T5 prompts/CWI, T6 overlays/DPI/Exit/update, T2/T3 UI rows. Build2 gates G9 RC regression; land before final human sittings. P4 echo page can record T6-P9/T5-P2 globals
- Owner Qs: P3-A6 fail ⇒ allow one change in simple_render_process_handler.cpp? (rec yes); CSS half → ticket; scratch filter rule in dev profile for P3-A4; brand visible to real sites during testing after P4-A0 (rec yes); upstream past .255 ⇒ merge (rec yes)
- Evidence b3 "app unchanged" may not hold (s_contextRanUrl/s_injectedUrl) — P3-A6 predicted fail, stop gate
- Shared-doc fixes: SCOPE §5 P4 literal brand order wrong (Chromium shuffles); cef_patch_drift_audit.sh blind to P3; NEXT_CHROMIUM_BUILD.md stale (Q2=b3); PRIOR_ART.md duplicate Sec-CH-UA row; HODOS_MIN_PATCHES raise = own commit (rule 6)
- Q2→b3, Q3→merge (pushes human-bound), Q5 already applied

## T4 (Opus) — committed
- P1 never lose track (A1–A10), P2 431 path. Owner-h ≈2.5 (P1 2.0 incl A5 real-money 1.5h, not shareable w/ T1; P2 0.5). K=3
- Awaiting independent RED: P1 A2a–f, A3a–f, A5, A6, A7, A8, A9b, A10; P2 A1a–c
- check_tx_exists_on_chain: ONE definition, 3 callers (internalize_action, wallet_cleanup, monitor PeerPay) — CLAUDE.md "duplicated copy" wrong. Unknown→Ok(false). T4-P1 strict 3-way for own sites; T1 reviews lenient callers.
- ⚠️ wallet_cleanup `.unwrap_or(false)` ⇒ unreachable chain marks outputs spent 'ghost-cleanup'; /wallet/cleanup registered, no UI caller found. Trip-wire-2 SHAPE, code reading only ⇒ T1 edge + owner mention
- T1-P3 edge: release_unbroadcast_transaction → restore_by_spending_description (T1-P3 replaces); T1-P3 control: coin P1 kept as paid not reselectable
- /wallet/release-nosend not on internal-only list — P1 kickoff item
- Owner Qs: Q-P1-1 strict reading (deviates from "reuse" wording), Q-P1-2 5xx re-check, Q-P1-3 survive restart (existing columns), Q-P1-4 unknown time bound ~10 min
- Q8: the unread #2890 comment is our own; deferral stands. Q7 verified.
- Doc fixes: X402_INTEGRATION §4 intro/§10 row 7 stale; body-transport ticket UNASSIGNED + recheck condition; track README goal line mentions x402 adapter; relay: HttpRequestInterceptor.cpp + simple_handler.cpp

## T3b (Opus, restarted) — P5–P8 committed
- P5 export/import + phrase tiers (OWNS decision 5 round-trip, P5-A4) · P6 deltas · P7 two devices R&D (answers in R4_ANSWERS.md; build = P7b) · P8 publish BRC
- P5 can start early: Step 0 now; build needs T1-P3, T1-P5, T3a-P1; A4 closes after T3a-P2; re-run at T3a-P4 freeze
- Awaiting independent RED: P5 A1,A4,A5,A6,A9,A11,A12,A13,A14 · P6 A1,A4,A5,A6,A7,A8,A9 · P7 R1a,R1b · P8 A1,A2
- Owner-h ≈5.5 (P5 2.25, P6 0.5, P7 1.5, P8 1.25). K=2 design-invalidating: root convention (P5); conflict mechanism / chain-as-sync (P7)
- Owner Qs: P5 Q1 foreign file = adopt root (inv 2+3) vs sweep (rec sweep) · P5 Q2 always scan all tiers (HandCash evidence; refines decision 6) · P5 Q3 Centbee confirm + Centi path? · P6 Q1 side table for deltas (schema) · P6 Q2 padding default no · P7 R0 numeric bar · P7 Q2 P7b build in beta.6? (rec answers-only) · P8 submit after ship
- Found: P5-A8 hypothesis — `2-receive address-{i}` invoice ⇒ BRC-29 wallet can't spend our receive outputs from an export. ⚠️ `do_onchain_backup` → `get_backup_hash().unwrap_or(None)` + DB error ⇒ None ⇒ full backup rebroadcast (trip-wire 2 shape, code reading; suggest fix in T3a-P2)
- Doc fixes: T3a P0/P1 relative paths one `../` too many; SCOPE §2.3 HandCash summary incomplete (+PRIOR_ART row); SCOPE §7.1 2h sitting → P7b; IMPLEMENTATION_PLAN App. A item 3 superseded by decision 5

## T5 (P1 Fable, P2–P5 Opus) — committed
- P1 servers prove who they are · P2 dApp-reachable surface (Part B loopback W4/W6/W7/W8 → rec P2.1) · P3 derived keys parts 1+2 + optional counterparty · P4 one OS account one wallet (auto-unlock-other-mnemonic stays T1-P2) · P5 usage ping
- Awaiting independent RED: P1 A1,A2,A3,A3b,A5 · P2 A1,A2,A9,A10 · P3 A2,A3,A4,A5,A7,A8,A10 · P4 A2,A5 · P5 none
- Owner-h ≈10.5 (P1 1.5, P2 1.5, P3 2, P4 2.5, P5 3). K=5 phases; new K18–K23. K11 proposal: edge fn hodosbrowser.com/u/1, daily counters only, <10/day countries → other, no logs, 25-month retention, owner-only. K12 draft in P5 §12 Q2
- Edges: T5-P3 migration after T1-P3 (V26+); T5-P3 & T2-P2 both edit create_signature — T5 first; T6-P7 wording must cover "share a key"; P5 builds switch+notice, T6 reviews (T6 scope lacks it); T6 fast-relaunch ↔ T5-P4 serialize; /wallet/pay402 on P2 allow-list; relay: P2+P4 touch shared C++ (P4 macOS StopServers)
- Owner Qs: W4 keep the denylist (P0.5 measured) vs ticket's allowlist · hex-encoded `anyone` bypass of part 1 + hex counterparty silent at L1–2 (differs from wallet-toolbox) — P3 §12 Q2 · child table keyed on derived_pubkey, no cascade, parent upsert (P3 Q1) · include verifyHmac default self (P3 Q3) · P2 refusal code, split P2.1, P4 K8/K9, P5 `first` on upgrade
- Found: derived_key_cache write-only (nothing reads it; rust-wallet/src/CLAUDE.md stale); W7 `localhost` in query skips adblock/cookie blocking/seed (fail-open, P2-A11); shim window.yours.broadcast → unrouted /wallet/broadcast (404) — ticket at close; dead code extractProtocolScope (report only); IsInternalOrigin("") still true
