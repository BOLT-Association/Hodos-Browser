# ⛓️ A confirmed transaction is never re-checked, so a chain reorg leaves the wallet trusting a block that no longer exists

**Found:** 2026-09-25, by the Wallet-Hardening pre-archive review (source doc named below, now in `archived-docs/Wallet-Hardening/`).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned — suggested: money-path safety · **Filed by:** Claude, at the owner's request (cleanup before archiving)

> ⚠️ **Method note.** **Code reading** (review agent, plus a listing of `rust-wallet/src/monitor/` on 2026-09-25): no task named or doing reprove/reorg exists; `verify_tsc_proof_against_block` runs only when a proof is first acquired. Not measured — no reorg was simulated.

---

## What happens

Once a transaction is marked completed with a merkle proof, nothing checks again that its block is
still on the main chain. Source: `archived-docs/Wallet-Hardening/FOLLOWUP_REORG_HANDLING.md` — planned,
never built.

## Why it matters

After a reorg that orphans the block, the wallet keeps treating the inputs as spent and the change as
confirmed. If the transaction does not re-confirm, the balance is wrong and "phantom change" can be
selected for spending. **Medium severity, low probability** on BSV.

## Proposed fix

A monitor task that re-verifies proofs over a bounded recent window (`verify_tsc_proof_against_block`)
and, on failure, replaces the proof or routes the transaction through the existing failure path — the
pattern `wallet-toolbox` uses (`reproveProven`). ⭐ Read that prior art first (working rule 5).
**Scope note:** the backup track covers reorg only for the **backup chain** (its H17).
