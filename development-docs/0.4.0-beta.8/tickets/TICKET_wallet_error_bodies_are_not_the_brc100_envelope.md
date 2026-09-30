# 🟠 Wallet error bodies are not the BRC-100 envelope, so dApps lose typed errors

**Found:** 2026-09-30, beta.6 G2 offline test (`../../0.4.0-beta.6/G2_OFFLINE_VALIDATOR_RESULT.md`).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned · **Filed by:** Claude (Opus 5.5)

> ⚠️ **Method note.** **Measured:** our 400 bodies are `{"error":"…"}` or `{"status":"error","code":…,"description":…}`.
> **Code reading** of `@bsv/sdk` 2.8.11 and 2.0.13 `substrates/HTTPWalletJSON.js`: a typed error is
> rebuilt only from a 400 whose body has `isError`. Not run through a real dApp.

## What happens

On the `3321` / `2121` bridge ports (`HTTPWalletJSON` substrate), every Hodos error reaches the dApp as
a generic `HTTPWalletJSON <call> failed with HTTP status 400`. The SDK cannot rebuild
`WERR_INSUFFICIENT_FUNDS`, `WERR_REVIEW_ACTIONS`, `WERR_INVALID_PARAMETER`, and so on. Through our
`window.CWI` shim the dApp gets an `Error('[Hodos] <method> failed: …')` with `.code`, which is also
not a `WalletError`.

**Not new.** SDK 2.0.13 had the same `isError` requirement; 2.8.11 dropped even the message text.

## Why it matters

A dApp that handles "insufficient funds" (offer a top-up) or `WERR_REVIEW_ACTIONS` (the `sendWith`
batch review flow) cannot tell those apart from any other failure.

## Fix

Emit the BRC-100 / wallet-toolbox error envelope (`isError: true`, `name` = `WERR_*`, `message`, plus
the per-code fields the SDK's `deserializeWalletError` requires) on every wallet error path. Have the
shim throw the same shape. Read `WalletError.js` / `deserializeWalletError` first (rule 4).

## Test and negative control

GREEN: an insufficient-funds `createAction` through `HTTPWalletJSON` throws `WERR_INSUFFICIENT_FUNDS`.
RED: today's body ⇒ generic `HTTP status 400` (already observed in the G2 reading).
