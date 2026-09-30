# 🟡 `verifySignature` rejects `counterparty: "self"`

**Found:** 2026-09-30, beta.6 G2 offline test (`../../0.4.0-beta.6/G2_OFFLINE_VALIDATOR_RESULT.md`).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned · **Filed by:** Claude (Opus 5.5)

> **Measured** on the dev wallet: `POST /verifySignature {protocolID:[2,"g2 probe test"], keyID:"1",
> counterparty:"self", forSelf:true, data, signature}` ⇒ `400 {"error":"Invalid counterparty public key
> hex: Invalid character 's' at position 0"}`. The same call with the identity key as hex ⇒ 200.

## What happens

BRC-100 allows `"self"` and `"anyone"` as counterparty on every key-deriving call. `createSignature`
accepts `"self"`; `verifySignature` parses the value as hex. A dApp that signs and verifies with
`counterparty:"self"` gets an error on the verify.

⚠️ Related, not the same: `../../0.4.0-beta.7/tickets/TICKET_createSignature_requires_counterparty.md`
(assigned to beta.7 **B5-T5 Identity & privacy**). Whoever takes that one should look at this one too:
same area, same "check against the spec first".
Check `verifyHmac`, `encrypt`, `decrypt` and `revealSpecificKeyLinkage` for the same parse.

## Fix

Route `verifySignature`'s counterparty through the same `self` / `anyone` / hex resolution as
`createSignature`. Signing and derivation code: invariant 3, ask first.

## Test and negative control

GREEN: sign with `"self"`, then verify with `"self"` ⇒ `valid:true`, and verify with wrong data ⇒ not
valid. RED: today's code ⇒ the 400 above (observed).
