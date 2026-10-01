# 🟡 Weak ARC statuses count as "on the network", and the code base draws that line four different ways

**Found:** 2026-09-30, beta.6 P3 (triage TSA-250), code reading by the author and a second agent
**Status:** 📌 ASSIGNED · **Track:** B5-T1 Money path (👤 owner, 2026-09-30: deferred from beta.6 P3) · **Filed by:** Claude (Opus 5.5)

> ⚠️ **Method note.** Everything below is **code reading**. Not measured: how often ARC actually answers `QUEUED`/`RECEIVED`/`STORED` for a payment we receive, and whether such payments ever fail to propagate.

---

## What happens

ARC's lifecycle is `QUEUED → RECEIVED → STORED → ANNOUNCED_TO_NETWORK → REQUESTED_BY_NETWORK → SENT_TO_NETWORK → ACCEPTED_BY_NETWORK → SEEN_ON_NETWORK → MINED`. The first four mean "ARC has it", not "a peer has it".

- `services/providers/arc_gorillapool.rs :: arc_response_to_tx_status` maps all of them to `TxState::InMempool`, so `handlers.rs :: check_tx_exists_on_chain` says **true**. `internalize_action` and `TaskCheckPeerPay` then accept an incoming payment as on-chain and skip their own broadcast.
- `arc_gorillapool.rs :: interpret_broadcast_response` treats `QUEUED`/`RECEIVED`/`STORED` as a **successful broadcast** (only `ANNOUNCED` advances the chain). So making the money-in check stricter alone changes nothing: the wallet rebroadcasts and accepts the same weak answer. beta.6 P3 tried exactly that and withdrew it.
- The line is already drawn three different ways:
  - `monitor/task_check_for_proofs.rs` calls `STORED`/`QUEUED`/`RECEIVED` "strong mempool signals";
  - `monitor/task_send_waiting.rs` promotes on `STORED`/`ANNOUNCED`/`QUEUED`/`RECEIVED`;
  - `arc_status.rs :: ArcTxStatus::is_in_mempool` includes `ANNOUNCED` and `STORED`, excludes `QUEUED`/`RECEIVED`.

  The project's own ladder rule says `ANNOUNCED` ≠ success.

## Why it matters

A received payment can be credited to the user while it is only queued at ARC. If it never propagates (ARC drops it, or a conflicting spend wins), the user sees money that is not theirs, and may spend it. That is working rule 7, trip-wire 1, on the money-in path.

## How exposed are we — answer this first

| If | Then |
|---|---|
| ARC rarely answers below `ANNOUNCED` for an already-broadcast incoming tx | Low practical exposure; this is a consistency fix |
| It does, and some never propagate | Real phantom credits; measure before choosing the line |

## Design question (owner)

Pick **one** "accepted by the network" line, put it in `arc_status.rs`, and use it in all four places: broadcast success, money-in acceptance, `task_send_waiting`, `task_check_for_proofs`. Changing broadcast success changes behaviour for **every outgoing payment** (more fall-through to TAAL/mAPI/WoC), so it needs a live smoke.

## Prior art

wallet-toolbox `ARC.ts`: primary broadcast accepts everything except `DOUBLE_SPEND_ATTEMPTED`/`SEEN_IN_ORPHAN_MEMPOOL`; its re-poll requires `SEEN_ON_NETWORK` or `STORED`. Memory note `reference_arc_tx_status_ladder`.
