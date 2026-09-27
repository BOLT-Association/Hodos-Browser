# 🚨❔ When a 402 server refuses after a payment, the wallet releases it without asking the chain — its coins may already be spent

**Found:** 2026-09-25, while re-verifying `TICKET_brc121_remint_on_retry.md` (code reading by a review agent).
**Status:** 📌 PROPOSED (G2) · **Track:** B5-T4 402 payments · **Filed by:** Claude, at the owner's request

> ⚠️ **Method note.** **Code reading only — a hypothesis, not a finding.** Nothing was run and no wallet row was
> checked. Root `CLAUDE.md` working rule 7: this *would* be trip-wire 1 (a money-path row in a state no code expects)
> **if** it happens — so the first step is the cheap ground-truth check below, **before** anyone treats it as live.

---

## What may happen

When the paid retry gets a refusal from the server (`status > 0`, including a 5xx after retries), the shell calls
`release_nosend`. That marks the payment transaction failed and **restores its inputs as spendable**. `release_nosend`
consults only the wallet's own record — never the chain. If the origin had already **broadcast** the payment before a
gateway in front of it returned 5xx, the wallet would put coins that are **already spent on chain** back into its
spendable set.

## Why it matters

Balance overstated; a later send selects a spent coin and fails ("Missing inputs"). The WS1 reconcile path heals that on
the next failed broadcast, which limits the damage — but it is a wallet row that disagrees with the chain.

## Step 1 — the cheap ground-truth check (do this first)

List transactions released by `release_nosend` (failed `nosend` payment rows) in a real wallet that has used BRC-121,
and ask WhatsOnChain whether each txid exists on chain. **Zero found ⇒ theoretical; keep as a hardening item.**
**Any found ⇒ poisoning; stop and tell the owner** (rule 7).

## Proposed fix (if confirmed, or as hardening)

Before restoring inputs, check the chain for the payment txid (the reconcile primitives in `reconcile.rs` already do
this for sends); if it is on chain, record it as paid instead of releasing it.
