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

## controls-A — T1 P1, P2, P3, P4 (33/33 cells)
- 👤 **P3 design question before kickoff:** the two money selectors do not share a predicate (`get_spendable_confirmed_by_user` confirmed-only vs `get_spendable_by_user`); one index with one membership rule cannot give S3/S5/S6 their confirmed-only set — say how those routes join back to `outputs.confirmed`.
- **GREENs that can pass with the feature absent:** P1-A9 (second visit served from `PaidContentCache` — require a cache MISS + the pay402 log line) · P1-A2 (truncated bulk body falls back to single-address fetch and rebuilds the same set) · P4-A6 (`TaskPurge` only deletes parents in `proven_txs` — seed it) · P4-A10 (need ≥20 consecutive Errs + a coin beyond)
- Wrong/under-specified SUBJECT: P2-A10 (helper maps Unknown→Ok(false) internally — GREEN must name the three-way reader) · P2-A11 (depends on ARC GP answering 404) · P1-A3 (no recorded "before") · P1-A4 (`peerpay_received` dedups on message_id — count per txid) · P2-A2 (name the transport: IPC vs HTTP) · P4-A11/A12
- Proposed rows: P2-X1 sendWith on phase-1 signAction · P3-X1 kill between claim and sign (backstop becomes TaskFailAbandoned) · P3-X2 old sweeper ignores P3 rows (rollback) · P4-X1 offloaded empty `locking_script` reaching a sighash path · P4-X2 `upsert_received_utxo_with_confirmed` hard-codes `type='P2PKH'`
- Residue: small real spends on scratch wallets; trip-wire-1-shaped scratch rows declared in-cell; ⚠️ P2-A7 macOS Keychain item outlives the profile (manual delete); P1-A9/P1-A4 leave rows on the **dev** wallet. ⚠️ Hosts-file fault injection warned against — it would blind the installed production wallet; use a fake `IndexerProvider` (needs a test constructor on `WalletServices`).
- Many controls need a **scratch build** (one switch compiled flipped) — budget build time.
- No poisoning evidence.
