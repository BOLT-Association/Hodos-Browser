# 🔎 The wallet cannot find a late payment to an old generated address — the rescan looks at the wrong kind of address, and after a restore the address is gone from the database

**Found:** 2026-09-25, 👤 owner request for a *"scan my old addresses"* button in the Tools tab; the code
check behind it found the two gaps below.
**Status:** 📌 PROPOSED (G2) · **Track:** B5-T1 Money path (the scan) · the button is card 2 of
`../track-6-browser-shell/TOOLS_TAB_claim_a_payment.md` (B5-T6) · **Filed by:** Claude, at the owner's request

> ⚠️ **Method note.** **Code reading only**, 2026-09-25 — nothing was run. Not verified: whether any
> other path (the on-chain backup's own restore, `TaskSyncPending`) already recovers BRC-42 receive
> addresses after a restore.

---

## Background — how "legacy" receiving works

Besides PeerPay (delivered through MessageBox), a user can press **generate new address** and hand it
out. Nothing tells the wallet a payment is coming, so it watches that address **on chain**: the row's
`pending_utxo_check` flag is set, and after **90 days** the flag is cleared
(`database/address_repo.rs :: clear_stale_pending_addresses`, `monitor/task_sync_pending.rs`,
`PENDING_TIMEOUT_HOURS = 2160`). ⭐ **The row itself is never deleted** — the wallet just stops looking.
👤 Owner: keep the 90-day window; it is fine. The gap is only the rare late payment.

## What happens

**Gap 1 — the existing rescan scans the wrong derivation.** `POST /wallet/rescan`
(`handlers.rs :: wallet_rescan`) exists for exactly this — its own comment: *"Useful when a user
believes coins were sent to an old address."* It re-derives from the recovery phrase starting at index
0, up to `max(current_index + 20, 100)` with a gap limit of 20. **But it derives BIP32 addresses only**
(`recovery.rs :: recover_wallet_from_mnemonic`: *"BIP32 only — BRC-42 disabled"*), while
**`handlers.rs :: generate_address` hands out BRC-42 addresses** (invoice `2-receive address-{index}`).
⇒ A button wired to today's rescan would report **"nothing found"** for the payment the user is
looking for. That is a verdict where the truthful answer is "I did not look there" (root `CLAUDE.md`
rule 7, trip-wire 2).
⚠️ Why BRC-42 scanning was disabled, per the code comment: it *"creates duplicate/phantom UTXOs when
backup change outputs exist at derived addresses."* Re-enabling it must solve that, not bypass it.

**Gap 2 — after a restore from the on-chain backup, the address is gone.** The backup deliberately
**drops "operationally dead" addresses** — used, no spendable output, not pending, older than 30 days
(`backup.rs`, address time-tiered strip). That is precisely the old, used, emptied address a late
payment would land on. ⇒ After a restore, a scan of **what is in the database** cannot see it.
✅ The backup **does** carry the wallet's `current_index`, so the wallet still knows how far up the
sequence it went.

## ⭐ Scope decided by the owner, 2026-09-25

**Build BOTH scan functions, and test both** — a **BIP32** sequential scan and a **BRC-42
self-counterparty** sequential scan — even where a caller does not use one yet. Each is a
separately tested function; the Tools-tab button and the post-restore scan are callers of them.

⛔ **The limit that must be stated wherever this is used: BRC-42 can only be scanned sequentially
when the counterparty is SELF.** A BRC-42 key is derived from *our key + the counterparty's key + an
invoice number*. For addresses **we generate for the user** (`2-receive address-{index}`,
counterparty = self) every input is known, so index 0, 1, 2… can be walked. For a key derived with
**another party** — a PeerPay/BRC-29 payment someone sent us with their own key and random
prefix/suffix — the inputs are not guessable, so **no scan can find it**. Those arrive through
MessageBox, or by hand through Tools-tab card 1 (*Claim a payment*).

🚨 **Owner, 2026-09-25 — the phantom-coin cause is the most important part of this ticket.** BRC-42
scanning was switched off because it *created phantom coins where backup change outputs sit*. That
cause is **fixed at its root, not bypassed** — take the time: research it properly, reproduce it,
and write negative controls that go red when the fix is removed. A money-path row in a state no code
expects is trip-wire 1 of root `CLAUDE.md` rule 7.

⇒ **Scanning is not a general BRC-42 recovery tool.** It recovers exactly one thing: payments to
addresses the wallet generated for the user and the user handed to someone. The UI and docs must
not claim more.

## Proposed design (for the track's research to confirm)

1. **The scan re-derives by index, not by database contents.** For each index from 0 up to the
   **high-water mark**, derive the BRC-42 receive address (`2-receive address-{i}`) and ask the chain.
   Because the high-water mark is known, **no large gap limit is needed** — the owner's concern about
   users generating many never-used addresses disappears. A small margin beyond it (e.g. 20) covers an
   index lost to a crash.
2. **High-water mark = the larger of `wallets.current_index` and `MAX(addresses.index)`** — the two can
   disagree today (`TICKET_two_next_address_index_sources_can_reuse_addresses.md`).
3. **Also keep the BIP32 legacy scan** for addresses from before BRC-42.
4. 👤 **Mark a restored wallet.** Record that the wallet was restored, and at what index. Every index
   at or below it is known to need a **derivation scan**, because its address row may have been
   stripped. Offer (or run) that scan once after a restore, not only on a button press.
5. **Fix the phantom-UTXO reason BRC-42 scanning was disabled** before re-enabling it — likely by
   excluding outputs the backup chain owns.
6. Inherits `TICKET_bulk_utxo_sync_truncates_at_20_per_address.md`: a scan that finds an address with
   more than 20 coins must not silently stop at 20.

## Test and negative control

| | |
|---|---|
| **GREEN** | Pay an old BRC-42 generated address (older than 90 days, or stripped by a backup/restore) → the Tools-tab scan finds it and credits it once |
| **RED** | Today's `/wallet/rescan` against the same setup finds **nothing** — run it first; that is also the proof of Gap 1 |
| **SUBJECT** | The coin appears in `/wallet/balance` and in `outputs`, with the correct derivation, and no duplicate/phantom row for a backup change output |

## Also noticed (reported, not fixed — working rule 3)

`frontend/src/components/wallet/CLAUDE.md` says `SettingsTab` has a **Wallet Rescan** section calling
`/wallet/rescan`; **no `.tsx` file calls it today**. Only the `.wd-rescan-*` CSS remains. The layer doc
is stale, and the backend endpoint has no screen.
