# 🟡 `noSend` outputs are flagged "phantom" and unspendable seconds after creation

**Found:** 2026-09-30, beta.6 G2 offline test (`../../0.4.0-beta.6/G2_OFFLINE_VALIDATOR_RESULT.md`).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned · **Filed by:** Claude (Opus 5.5)

> **Measured** (dev DB, read-only): the outputs of `noSend` tx `2aa435e5…`, including the change listed
> in `noSendChange`, had `spendable=0` and `spending_description='phantom: parent tx not on chain'`
> **3 seconds** after creation. Which task wrote it was not traced. Not run: a real `sendWith` chain.

## What happens

BRC-100 `noSend` exists so a dApp can build a chain of transactions and broadcast them together
(`options.noSendChange` feeds the next `createAction`, then `sendWith` broadcasts the batch). Our
wallet marks a `noSend` transaction's outputs as phantoms almost at once, because the parent is not
on chain, which is true by design for `noSend`.

## Why it matters

A dApp chaining `noSend` transactions may find its `noSendChange` refused as unspendable, so the
batch cannot be built. This is the flow ordinals and token dApps use.

## Fix

Find the writer (phantom check / UTXO validation task) and exempt outputs whose own transaction is
`nosend` until that transaction is broadcast or times out. Money path (R-NODOUBLE): the exemption must
not make a truly dead output spendable.

## Test and negative control

GREEN: `createAction{noSend}` → `createAction{noSendChange, noSend}` → `createAction{sendWith}` on a
scratch wallet completes. RED: today's code ⇒ the second call cannot use the change (predicted, not run).
