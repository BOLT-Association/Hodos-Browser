# 🟡 `verifyLegacyMessage(message, signature, identityKey)` ignores `identityKey`

**Found:** 2026-09-30, reading code at `aaf44aa` on `0.4.0`, during the website accuracy audit
(`Marston Enterprises/Hodos/Website/AUDIT_2026-09-30.md`, checklist item 19, doc row D-B168).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned · **Filed by:** Claude (Opus 5.5), Marston-side session, at the owner's request

> ⚠️ **Method note.** **Code reading** only. Not verified: which key the call actually verifies against
> with `counterparty: 'anyone', forSelf: false`. That needs one run with a known signer.

---

## What happens

`cef-native/include/core/CWIShimScript.h`, the `verifyLegacyMessage` helper on the `window.yours`
legacy surface, takes `identityKey` as its third argument and never uses it:

```js
return canonical.verifySignature({
    data: data, signature: sigBytes,
    protocolID: YOURS_LEGACY_V1.SIG_PROTOCOL, keyID: YOURS_LEGACY_V1.KEY_ID,
    counterparty: YOURS_LEGACY_V1.COUNTERPARTY_ANYONE,
    forSelf: false
})
```

The design doc for the shim (`archived-docs/Sigma-BRC121-Sprint/phase-0.2-window-yours-shim-design/SHIM_TRANSLATION_SPEC.md` §R1)
describes it as a helper to verify **another user's** signature under the `yours-legacy-v1` convention.

## Why it matters

A site calls `verifyLegacyMessage(msg, sig, aliceKey)` expecting "did Alice sign this?". The answer it
gets does not depend on `aliceKey`, so a valid signature by someone else can come back `true` against Alice's
name, or Alice's real signature can come back `false`. Either is a wrong answer to an identity question.
`WALLET_API_MAP.md` also lists it as ungated.

## How exposed are we — answer this first

| If | Then |
|---|---|
| No site calls `window.yours.verifyLegacyMessage` | Correctness debt only. The usage count is unknown |
| A site uses it to authenticate a user | It authenticates the wrong party. Severity rises to 🟠 |

## Proposed fix

Pass `identityKey` as the `counterparty` (the signer), per the BRC-100 `verifySignature` semantics for
verifying a signature someone else made, and add a test with two keys. If the `yours-legacy-v1`
convention says otherwise, fix the convention doc and the helper together.

**Deliberately out of scope:** the other legacy shim methods.

## Test and negative control

| | |
|---|---|
| **GREEN** | A signature made by key A verifies `true` with `identityKey = A` and `false` with `identityKey = B` |
| **RED** | Today's code ⇒ the result is identical for A and B (proves the argument is ignored) |
| **SUBJECT** | Shim call from a scratch https page in a Hodos tab, with the wallet log showing the `/verifySignature` body |
| **Tier** | T2 |

## Links

- `development-docs/architecture/WALLET_API_MAP.md` (row for `verifyLegacyMessage`)
- BRC-33 (PeerServ Message Relay Interface) note from the same audit: the local relay endpoints stay local by design, alongside the federated MessageBox. That is a docs item, not part of this ticket
