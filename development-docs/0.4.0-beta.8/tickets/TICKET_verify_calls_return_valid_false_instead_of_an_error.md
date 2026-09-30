# 🟢 `verifySignature` / `verifyHmac` answer `{valid:false}` where the SDK now expects an error

**Found:** 2026-09-30, beta.6 G2 offline test (`../../0.4.0-beta.6/G2_OFFLINE_VALIDATOR_RESULT.md`, row B4; advisory TSA-040).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned · **Filed by:** Claude (Opus 5.5)

> **Measured:** our `{"valid":false}` fails `@bsv/sdk` 2.8.11 `validateWalletResult` (`valid: expected true`).

## What happens

For a bad signature or HMAC we return 200 `{valid:false}`. The hardened SDK treats any `valid` other
than `true` as an invalid result and throws. The SDK's own `ProtoWallet` throws on a bad signature too,
so the dApp already saw a throw. Only the error's type and message differ.

## Why it matters

Low. It becomes worth doing together with `TICKET_wallet_error_bodies_are_not_the_brc100_envelope.md`,
so that the throw is the typed error wallet-toolbox raises.

## Fix

Match wallet-toolbox's error for an invalid signature or HMAC (read it first). Check our own callers
of these endpoints for `valid === false` handling before changing the shape.

## Test and negative control

GREEN: a wrong-data verify through `WalletClient` throws the toolbox's error type. RED: today's
`{valid:false}` ⇒ `Invalid verifySignature result valid` (observed).
