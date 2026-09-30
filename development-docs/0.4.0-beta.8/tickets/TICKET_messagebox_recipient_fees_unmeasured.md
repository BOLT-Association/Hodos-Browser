# 🔬 MessageBox may now charge delivery fees our client never pays (research)

**Found:** 2026-09-30, beta.6 advisory triage (`../../0.4.0-beta.6/ADVISORY_TRIAGE.md`, G1, TSA-047).
**Status:** ⬜ UNASSIGNED · **Track:** unassigned (research) · **Filed by:** Claude (Opus 5.5)

> **Code reading** of the advisory only. Nothing measured: MessageBox refused our handshake until the
> beta.6 nonce fix, so no send has reached its fee logic.

## Question

The hardened MessageBox enforces recipient-set fees on `sendMessage`. Our `messagebox.rs ::
send_message` attaches no payment. After beta.6's nonce fix, does a PeerPay notice to a recipient who
set a fee get refused? If so, who pays: added to the sender's payment, or a separate prompt?

👤 Owner, 2026-09-30: not urgent; a future research item or release cycle.

## First step

After the beta.6 nonce fix, send one notice to a fee-free recipient and one to a recipient with a fee
set (a throwaway key on MessageBox), and record the responses.
