# DRAFT — comment for bitcoin-sv/BRCs on BRC-103 (not posted)

> 👤 For the owner to post, edit or drop. Written 2026-09-30 by Claude (Opus 5.5) from the B6-P1 measurements.
> Suggested venue: an issue (or a PR against `peer-to-peer/0103.md`) in the BRCs repository.

---

**BRC-103 §5.3 says the initial nonce is "random 256-bit"; the reference server now rejects that**

BRC-103 describes the `initialNonce` in `initialRequest` as a random 256-bit value. Since the ts-stack
hardening in GHSA-qp3j-h5xf-p2p7 (commit `b3155fa2`, deployed to `messagebox.babbage.systems` on
2026-09-23), `auth-express-middleware` refuses a 32-byte `initialNonce` with `400 ERR_AUTH_MALFORMED`. It
accepts only the 48-byte format that `@bsv/sdk` `createNonce` produces: 16 random bytes followed by a
32-byte HMAC under protocol `[2, 'server hmac']` (keyID = the UTF-8 decoding of the 16 bytes,
counterparty self), base64-encoded.

We measured it against the live server: the same `initialRequest` succeeds with a 48-byte nonce and
fails with a 32-byte one. Per-request `nonce` and `requestId` are still accepted at 32 bytes.

Any client written from the spec rather than the SDK stopped working on that date, with no notice
except the advisory. Our browser's PeerPay notifications failed for a week before we traced it.

Could the spec either:
1. describe the SDK nonce format (length, HMAC construction, keyID derivation) as **required** for
   `initialNonce`, or
2. say the server may impose a format, and name the one the reference implementation uses?

Two details that matter for anyone re-implementing it, both measured against `@bsv/sdk` 2.8.11:
- the keyID is WHATWG `TextDecoder` output, so invalid UTF-8 becomes U+FFFD and a leading BOM is dropped;
- the HMAC key is the shared secret's x-coordinate in **minimal** big-endian form, so when it starts with
  `00` (about 1 in 256) the key is 31 bytes, not 32.

Happy to contribute test vectors.
