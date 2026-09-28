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

## T1 (P1 Fable, P2–P7 Opus) — committed
- P1 reqwest · P2 a failure says it failed (+sendWith, auto-unlock other phrase, dead cert builder, wallet_cleanup) · P3 reservations + money index · P4 ingest truth · P5 classifier + Unknown display · P6 late payments to generated addresses · P7 records true over time
- Error shape: TS SDK 2.8.7 decodes WERR_REVIEW_ACTIONS only on HTTP 400+isError+code:5 (+ sendWithResults array, txids in request, status enum). Go SDK treats any non-200 as generic — compatible, not "agree"
- Awaiting independent RED: P1 A3,A4,A9 · P2 A1,A2,A3,A4,A6,A7,A10,A11 · P3 A1,A2a–f,A3–A12,A14,A15 · P4 A1,A2,A4,A5,A6,A7,A9,A10 · P5 A1,A2,A3,A4,A5a–c,A6,A7,A9,A10 · P6 A1–A8,A10–A15 · P7 A1,A2,A5–A8
- Owner-h ≈10.5–11.5 + macOS. K=6 of 7
- wallet_cleanup: two lenient readings stacked; reachable by an approved dApp (not first-party-only). Read-only DB check: 0 `ghost-cleanup` rows dev + prod ⇒ not escalated; fix in P2 A10/A11
- Owner Qs: P2 Q1 retries (rec 2) · Q2 keep `sending` after transient (rec yes) · Q3 honour sendWith (rec yes) · Q4 createAction 500 → same envelope (rec yes w/ OK) · Q5 auto-unlock identity check (inv 3) · P3 Q1 build index on today's rule, switch to change=1 in P5 (sequencing) · P3 Q2 on-chain second gate · P4 Q1 1,024 B inline cap + keep parents · P4 Q2 which explorer is "banana blocks" · P5 Q1 Unknown wording, Q2 keep 1-sat floor · P6 Q1 BIP-32 sweep user-confirmed, Q2 no-schema restore trigger, Q3 external sweep adds service fee (pays none today) · P7 Q1 144-block window, Q2 shedding confirmed + fee
- Consequences carried: wallet_activity uses o.change=0 for received ⇒ P5-A11; "six selectors + send_max" = 2 selectors × 6 routes (S1–S6)
- Edge to T5-P2: first-party-only list lacks /wallet/cleanup, /wallet/rescan, /wallet/release-nosend. Schema at V25; T1-P3 then T5-P3 renumber
- Doc fixes: reqwest sites 23 not 35; SCOPE §7 P3 selector wording

## T2 (P1–P3 Opus before the stop, P4 Opus) — committed
- P1 classify & file · P2 token-spend permission (per action) · P3 receive & transfer · P4 hold & show (extends /wallet/tokens; amt from chain script; exact u128 sums; no inline media — View opens a normal tab; no BSV-21 send)
- Awaiting independent RED: P1 A1–A7,A11,A12 · P2 A1–A8 · P3 A3–A12 · P4 A2,A3,A6,A12
- Owner-h ≈4.0. K=4 phases
- Owner Qs: P1 Q1 BRC-162 binary hold only (rec) · Q2 money = exact 25-byte P2PKH template (into T1-P5) · P2 Q1 asset baskets 1sat/bsv21/bsv20/opns + unfiled 1-sat · Q2 build createSignature binding in P2 (rec yes) · Q3 relinquishOutput on asset: prompt each time (rec) · P3 Q1 new signing call site from customInstructions triple (inv 3; rec approve w/ pubkey-matches-script) · Q2 addresses only · Q3 write-ahead in existing rows vs new table (inv 2, at kickoff) · Q4 transfer pays 1,000-sat fee · P4 Q1 no inline media · Q2 held tokens shown in T1's line only · Q3 BSV-21 "not verified" only · Q4 metadata in parent_transactions
- Reported: BRC-162 (PR #273) merged 2026-09-28, no 1-sat requirement ⇒ "held 1-sat coin" wording should read "held at any satoshi value"; money rule must be exact 25-byte match. createSignature + relinquishOutput are spend paths the scope missed.
- ⚠️ GET /wallet/tokens (and likely /wallet/balance, /wallet/activity) readable by approved websites (domain_trust_gate; first-party-only covers POST/DELETE) — bypasses BRC-165 view prompt; P4-A0 measures; T5-P2 allow-list decides. list_token_outputs DB error → 200 [] (trip-wire 2 shape, display path) — P4-A8
- Doc fixes: README "Basket and permission mechanics" still says BRC-147 MUST (should be 165); WATCH_fungibles.md stale (DO NOT BUILD, log stops 2026-08-29); decision-8 watch wording; T1-P5 cite P4-A6 + extend A10 to held token rows

## T3a (P0/P1 Fable, rest Opus) — committed
- P0 prove + measure · P1 round-trip harness · P2 split: P2.1 carry what matters · P2.2 write path cannot hurt money · P2.3 restore classifies & reports · P3 re-fetchable bytes · P4 freeze format + crash safety (reserves 16-byte device_id + extension)
- Other tracks citing "T3a-P2": restore/BS-C2/report/high-water ⇒ P2.3; lock/baseline/BRC-177/size cap ⇒ P2.2
- Awaiting independent RED: P0 A7–A10 · P1 A1,A2,A4 · P2.1 A2–A4 · P2.2 A1–A5,A8–A11 · P2.3 A1–A7,A10 · P3 A1–A6 · P4 A1–A9
- Owner-h ≈4.75. K=3
- get_backup_hash: CONFIRMED in code (error ⇒ Ok(None) ⇒ full backup broadcast; siblings get_last_backup_at, discarded set_* results, wallet_recover_onchain 6b). Logs: 0 error lines prod (09-05→09-28) + dev. Latent, not poisoning. Fix P2.2-A3; T3b-P6-A2 RED → "revert P2.2-A3"
- ⚠️ BS-C2 worse: backup_found:false (also on indexer error) ⇒ WalletPanelPage doRecoverWallet silently falls back to coins-only /wallet/recover ⇒ degraded wallet; HYPOTHESIS its next backup supersedes/spends the good on-chain backup. P2.3-A2 reproduces first. Decision 6 tier ① "must work first" not true today
- mark_failed on inconclusive oracle restores inputs on a timer (breaks BRC-177); 0 log hits
- Owner Qs: P2.1 Q1 excluded-tables list (carry peerpay_outbox) · P2.2 Q1 mark_failed owner (rec T3a-P2.2; conflicts T4-P1 "unchanged") · Q2 size cap from P0 numbers · P2.3 Q1 never auto-offer coins-only scan when backup can't be checked · P3 Q1 don't strip scripts ≤1 KB · P4 Q1 intent table (inv 2) · Q2 header as GCM AAD (inv 3) · Q3 device_id 16 bytes
- Doc fixes: T3b-P6 §0/A2 owner text; T3b-P5 §11 + others "T3a-P2" → P2.2/P2.3; T4-P1 §5 mark_failed note; REGRESSION_ADDITIONS R-RESTORE runnable from T3a-P1, green at P2.3; stale TaskValidateUtxos comment (T1-P6)
