# 🔁 Deleting a site in the advanced wallet does not reach the browser — it keeps treating the site as approved

**Found:** 2026-09-25, from the shell log during the beta.4 "prompts don't appear" investigation (Lead B).
**Status:** 📌 PROPOSED (G2) · **Track:** B5-T5 Identity & privacy (permission correctness) · **Filed by:** Claude
**Origin:** `../../HOTFIX_2026-09-25_prompts_not_showing/README.md` (Lead B)

> ⚠️ **Method note.** **Log reading only**, two occurrences; not reproduced. The wallet (Rust) side did remove the
> permission — nothing here bypassed the gate; the browser (C++) side kept a stale "approved" belief.

---

## What happens

Both times the owner deleted a site under **advanced wallet → Approved sites**, the shell refused the message that
tells it to forget that site:
```
15:45:09 🛡️ IPC DENIED (P0.5-B1): domain_permission_invalidate from tab role 'tab_16'
15:54:13 🛡️ IPC DENIED (P0.5-B1): domain_permission_invalidate from tab role 'tab_21'
```
The advanced wallet runs **as a tab**, and beta.3 Phase 0.5's IPC role gate (P0.5-B1) rejects this message from tab
roles. Immediately after, the browser still believed zanaadu.com was approved and the beta.4 "stale connect" fix
**re-sent the call instead of asking**:
```
15:54:17 🔁 Stale connect prompt for already-approved zanaadu.com /getVersion — re-sending the call instead of asking again
```

## Why it matters

A user who revokes a site in the wallet UI expects the next visit to ask again. Here it neither asks nor proceeds
cleanly. It is a **consent-surface** defect: the revoke looks like it worked but the browser does not act on it.

## Design question (do not guess — `CLAUDE.md` working rule 1)

P0.5-B1 exists so that **web pages** in tabs cannot send privileged messages. The advanced wallet is **our own UI** in a
tab. The fix must let *that* page invalidate without reopening the hole for web content — e.g. identify it by its
internal origin in C++, or have the **Rust** side (which already knows about the delete) tell the shell. Read how
P0.5-B1 identifies roles first.

## Test and negative control (sketch)

| | |
|---|---|
| **GREEN** | Approve a site → delete it in the advanced wallet → revisit: **the connect prompt appears**; no `IPC DENIED` line |
| **RED** | Current build: the `IPC DENIED` line and the "already-approved … re-sending" line (reproduce first) |
| **SUBJECT** | The shell's approval cache for that domain, read from its log, and the prompt's appearance to the user |
| **Guard** | A **web page** in a tab sending `domain_permission_invalidate` must still be DENIED — the negative control for the security gate |
