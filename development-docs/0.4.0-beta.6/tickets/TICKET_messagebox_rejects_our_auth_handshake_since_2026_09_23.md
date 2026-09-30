# 🚨 MessageBox rejects every AuthFetch handshake with `400 ERR_AUTH_MALFORMED` since 2026-09-23 — PeerPay notices are not delivered and incoming PeerPay is not received

**Found:** 2026-09-30, while diagnosing an unrelated dApp connect, from the owner's orange-dot notice (*"payment sent, recipient not notified"*) on a PeerPay the owner sent to our own identity key at 07:23.
**Status:** ⬜ UNASSIGNED · **Track:** unassigned. ⭐ Suggested: **B5-T5-P1 — servers prove who they are**, which rewrites `AuthFetchClient::handshake` and already lists `R-PEERPAY-DELIVERY` as at risk. 👤 Urgency (hotfix vs beta.6) is the owner's call. · **Filed by:** Claude (Opus 5.5), with the owner

> ⚠️ **Method note.** Everything below is **measurement from logs** (dev and installed wallet logs, read-only) except where marked *code reading*. **Not verified:** the cause. The MessageBox server's current handshake requirements were not read and no request was replayed. The outbox row for the stuck notice was not opened.

---

## What happens

Every BRC-103 handshake our AuthFetch client makes to `messagebox.babbage.systems` is refused:

```
AuthFetch error: Handshake failed: server returned 400 (expected 200):
{"status":"error","code":"ERR_AUTH_MALFORMED","description":"The authentication request is malformed."}
```

- **Receive side:** `monitor/task_check_peerpay.rs` logs this on every poll (about once a minute), so incoming PeerPay payments are never listed or auto-accepted.
- **Send side:** `handlers.rs` (PeerPay send, after broadcast) logs `MessageBox delivery failed, queuing for retry` and writes the notice to the outbox (`PeerPayRepository :: insert_outbox`, *code reading*). The payment itself is broadcast and accepted, so **money moves and the recipient is not told**.

**When it started (measured):** first `ERR_AUTH_MALFORMED` at **2026-09-23 01:49:24** (dev wallet) and **01:50:01** (installed wallet). The last success-looking poll was 01:47:16. Both have failed continuously since; the most recent is 2026-09-30 10:54. Two different builds breaking within a minute of each other points to a **server-side change at MessageBox**, not our code. Earlier failures (2026-09-20) were network errors, a different signature.

## Why it matters

- Every PeerPay sent since 2026-09-23 reached the chain but the recipient's wallet was never told. A recipient without a claim flow cannot find it: PeerPay keys use a random prefix and suffix, so no scan can (`track-1-money-path/SCOPE.md` §5.4).
- Every PeerPay **to** a Hodos user since then has not been picked up.
- ⚠️ **It is not the size problem.** The handshake fails before any payload is sent, the inbox poll carries no payment at all, and the notice that failed was a 4,003-byte BEEF. The body-transport work (`TICKET_brc121_client_has_no_body_transport_for_large_beef.md`) and the large-parent controls (`TICKET_wallet_cannot_shed_large_parents.md`) do not touch this path and would not fix it. The owner's orange-dot notice shows our **surfacing** control working.

## How exposed are we — answer this first

| If | Then |
|---|---|
| MessageBox changed what a valid handshake looks like (a new required field, header or version) | Every Hodos install is affected until we match it. Read the current server and `@bsv/sdk` AuthFetch/Peer source first (rule 5), and compare with our `authfetch.rs` |
| The outbox retries forever against a permanent refusal | Noise, not harm. Check whether `ERR_AUTH_MALFORMED` is classed transient (`is_permanent()`, *code reading*: it took the retry branch here) |
| Queued notices go out once auth is fixed | The recipients of past payments are notified late. Confirm the outbox drains, and in order |

## Test fixture — keep this payment

👤 Owner, 2026-09-30: use the accidental payment as the first real test of **B5-T6-P5 — Tools tab "Claim a payment"** rather than repairing it by hand.

| | |
|---|---|
| txid | `8a24596f7d5862c41c3fb3529df9fec4ab97a71dee2333571206144afb4ebd4e` (broadcast OK, GorillaPool mAPI `SEEN_ON_NETWORK`, 2026-09-30 07:23:52) |
| amount | 1,470,588 sats (≈ $0.30 at the time), output 0 |
| recipient | our own master identity key `020b95583e18ac93…` (a self-payment via PeerPay) |
| wallet | the **dev** wallet (`HodosBrowserDev`) |
| where the derivation prefix/suffix should be | the outbox row queued at 07:23:52 (not opened) |

⛔ Do not "clean up" this output or its outbox row before T6-P5 has used it.

## Fix — not scoped

Unscoped. First step is prior art: the current MessageBox server and the `@bsv/sdk` / wallet-toolbox AuthFetch handshake. **Negative control** for any fix: with the fix reverted, the dev wallet's PeerPay poll must log `ERR_AUTH_MALFORMED` again. A green run must show a successful `listMessages` **and** one delivered notice (the owner's recipient wallet shows it), not just the absence of the error.
