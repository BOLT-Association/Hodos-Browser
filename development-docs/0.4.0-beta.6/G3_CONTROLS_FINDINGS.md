# G3 — independent negative-control round: findings

Brief: `G3_CONTROLS_BRIEF.md`. Designers ran on **Fable**; most contracts were authored by **Opus** (model comparison → `AAR_NOTES.md`).
Detail per row lives in each contract's `§4a. Independent control notes`. This file is the index, most serious first per group.

## controls-B — T1 P5, P6, P7 (31/31 cells + P6-A16 containment)
- **P6-A6 GREEN passes with the feature absent** — today's `wallet_rescan` already scans to `max(current_index+20, 100)`; NC-6 does not go red. Fix: fixture above 100 + a bounded half.
- **P5-A5a can pass on the wrong guard** (`is_p2pkh_script` len==25 refuses real inscriptions by itself) — needs a pre-P4-shaped Unknown row.
- **P7-A5 SUBJECT unreachable** — `cache_helpers.rs :: verify_tsc_proof_against_block` uses a hard-coded WhatsOnChain URL, not `WalletServices`; no fake header can drive it. 👤 owner: route through `WalletServices::get_block_header` or accept no T1 control.
- P6-A7 red not a red after P5 (default `change=0`) · P5-A4 equation omits Token class · 👤 P6-A16 "compiled out" vs `HODOS_DEV` (runtime) contradictory — choose env- or feature-gate · P6-A1 second phantom route (TaskReviewStatus) not exercised · P5-A9 indexer-down vs on-chain restore contradiction · P5-A6 route I6 no seam-bypass red · 👤 P5-A2 the `change=1` stamp is forgeable by any write path — write-side T0 gate? (own commit, rule 6) · smaller: P6-A14/A15/A10/A13, P7-A1/A8/A9
- Proposed rows: P5-X1 "Treat as money" endpoint first-party-only · P5-X2 migration + treat-as-money ⇒ zero gold-pill emits, counters unchanged · P6-A2b stale-indexer split
- 👤 P5-A10 GREEN must extend to held token-shaped rows (T2-P4 edge)
- Residue: real spends on scratch profiles (P6-A13, A15, A16, P7-A6, A7); impossible-state fixtures in-memory/scratch (P5-A5a, P6-A2, P7-A8)
- No poisoning evidence.
