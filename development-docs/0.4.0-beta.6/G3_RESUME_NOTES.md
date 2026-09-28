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
