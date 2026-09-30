# 🔴 Any local caller that omits the requesting-domain header is trusted as the wallet UI

**Found:** 2026-09-30, by reading the code at `0503e39` on `0.4.0`, during the website accuracy audit
(`Marston Enterprises/Hodos/Website/AUDIT_2026-09-30.md`, owner checklist item 1). The known half
has been on record since 2026-08-18 (`TICKET_loopback_host_form_wallet_routing.md` §1).
**Status:** 📌 **ASSIGNED 2026-09-30** (👤 owner approved the move from beta.8 intake) · **Track:** **B5-T5 Identity & privacy, phase P2 — an approved site reaches only the dApp surface** (Part B, row `W9`, evidence `P2-A14`; the floor answers `P2-A9`) · **Filed by:** Claude (Opus 5.5), Marston-side session

> 📝 **Renumbering note (2026-09-30).** Written before the folders were renumbered: every "beta.6
> T5-P2 / T5-P4" in the original text meant **this** release, beta.7. The references below are updated.
> **Why beta.7 and not beta.8:** a header-free `POST /transaction/send {sendMax:true}` from any local
> program or any `localhost:<port>` page drains the wallet with no prompt, no cap and no gold pill
> (code reading); `../../0.4.0-beta.6/ADVISORY_TRIAGE.md` names it TSA-322 (P1) and TSA-066 (the
> certifier-URL P0 candidate) relies on it; and the fix sits in the two functions T5-P2 already
> changes (`IsInternalOrigin`, the header-free branch of `domain_trust_mw`).

> ⚠️ **Method note.** Everything below is **code reading**. Nothing was run: no request was sent to a
> live wallet, because the obvious RED moves real funds. Not verified: whether a page on
> `http://localhost:<port>` inside a Hodos tab actually reaches `AsyncWalletResourceHandler::Open` with
> `requestDomain_` set to its own origin (the reading says yes; §How exposed, row 2).

This ticket is the beta.7 home that `../track-5-identity-privacy/phase-P4-one-os-account-one-wallet/PHASE_CONTRACT.md`
row `P4-A6` refers to ("the beta.7 per-user-channel ticket"). That ticket did not exist until now.
Its fix now lands in **B5-T5-P2** (this release), not a later one; the per-user named pipe / socket
itself stays deferred (T5 Q12).

---

## What happens

The Rust wallet decides first-party versus third-party with one test: is the `X-Requesting-Domain`
header present?

- `rust-wallet/src/main.rs :: domain_trust_mw`: no header ⇒ `next.call`, with no gate.
- `rust-wallet/src/handlers.rs :: check_domain_approved`: `None => return Ok(None), // Internal request`.
- `dispatch_payment` returns `Proceed` when the header is absent (comment in `handlers.rs :: send_transaction`).

The header is stamped by the C++ shell, for traffic the shell intercepts and classifies as external.
Anything that reaches `127.0.0.1:<wallet port>` without passing through that stamp is therefore
**trusted as the wallet UI**. Two callers get there:

1. **A non-browser local process** (any program, any OS account on the machine). It sends no `Origin`
   header, so actix-cors treats it as a non-CORS request and `block_on_origin_mismatch(true)` never
   fires. It sends no `X-Requesting-Domain`, so every gate is skipped.
2. **A page served from `localhost:<any port>` or `127.0.0.1:<any port>` in a Hodos tab.**
   `cef-native/src/core/HttpRequestInterceptor.cpp :: IsInternalOrigin` matches `localhost` and
   `127.0.0.1` with **any** port suffix (and `""`), and both `AsyncWalletResourceHandler::Open` and
   `HandleIpcWalletCall` send such callers down the header-free path (`runIpcCallDirect`).

## Why it matters

`POST /transaction/send` with `{"toAddress": "<any P2PKH>", "sendMax": true}` sweeps the balance, and
on the header-free path `dispatch_payment` returns `Proceed`: no modal, no spending cap, no gold pill.
The PIN guards `/wallet/reveal-mnemonic` only. It does not guard spending.

So the bar for draining a Hodos wallet from the same machine is one HTTP request. Without the port,
the attacker needs the PIN to decrypt the recovery phrase at rest. The port lowers the bar from "steal
the PIN" to "send one request".

## How exposed are we — answer this first

| If | Then |
|---|---|
| Malware runs as the **same OS user** | Row 1 applies. That user's session is usually compromised anyway (keylogging, screen capture), but today it needs no PIN and no user interaction to spend |
| A **different OS account** on the same machine runs anything | Row 1 applies, and DPAPI/Keychain would otherwise have stopped it. This is the cross-account half that B5-T5-P4 deliberately leaves open (G2 decision **T5 Q12**: per-user channel deferred) |
| The user opens a **local dev server** (`http://localhost:3000`, an npm tool, a Jupyter page) in Hodos | Row 2 applies *(reading)*: any script on that page is treated as the wallet UI. B5-T5-P2 records this as open question `P2-A9` ("say whether W6 covers it or it is a recorded residual") |
| The Hodos wallet is not running | Nothing is exposed |

**Prior art** *(inference, not verified for this ticket)*: the BRC-100 local HTTP substrate
(`localhost:3321`, MetaNet Client) is unauthenticated by convention, which is likely how "headerless
means trusted" became the design. The convention assumes the wallet prompts the user for every
spend. Hodos's header-free path skips the prompt, so the convention's safety assumption does not
carry over.

## What already protects us, and how that shapes the fix

- **Web pages on any other origin are blocked.** CORS `block_on_origin_mismatch(true)` stops a
  cross-origin POST before any handler runs (measured 2026-08-19, per the comment in `main.rs`).
  `Origin` is a forbidden header, so a page cannot forge it.
- The port binds to `127.0.0.1` only, so nothing off the machine reaches it.
- The deny-list in `domain_trust_mw` blocks `/wallet/reveal-mnemonic`, `/wallet/settings` and
  `/wallet/debug/*` for **header-bearing** callers only.
- **B5-T5-P2 (W6)** makes an empty origin external and adds a `Host` check against DNS rebinding.
- **B5-T5-P4** stops one account's shell from adopting another account's wallet backend.

So the gap is narrow: **proof that a header-free caller really is our own UI.** Everything else exists.

## Proposed fix

**Both halves ship in beta.7, B5-T5-P2** (👤 owner, 2026-09-30):

**The floor (answers `P2-A9`):**
- `IsInternalOrigin` accepts only the first-party UI origin (`127.0.0.1:5137` / the release
  equivalent), never `localhost:<any port>`. This closes row 2.

**The system (P2 row `W9`, evidence `P2-A14`):**
- The shell generates a random per-launch secret, hands it to the wallet at spawn (an environment
  variable or an inherited handle, **not** argv, which other users can read), and stamps it on every
  first-party call. Rust treats a request as internal **only** if the secret matches. Header-free
  without the secret ⇒ external and unapproved ⇒ gated or refused.
- ~~Or move first-party traffic to the per-user channel (named pipe / Unix socket with an owner ACL)
  that T5 Q12 deferred.~~ Not chosen for beta.7: the secret is the smaller change inside P2's
  existing functions. The per-user channel stays deferred (T5 Q12).
- Either way: a header-free request without proof gets the same gate as an unknown site, never `Proceed`.

**Deliberately out of scope:** authenticating BRC-100 dApp traffic itself (that is the domain-trust
engine's job, and it works); changing the `3321`/`2121` compatibility re-pointing; the PIN model.

## Test and negative control

⛔ Per `../../0.4.0-beta.3/HARNESS.md`: **a fix is not done until the check has been *seen* to fail.**

| | |
|---|---|
| **GREEN** | From a plain local process (`curl`, no Origin, no header, no secret), `POST /transaction/send {sendMax:true}` ⇒ refused or gated (403/202), handler does not run. The wallet UI's own send still completes with no modal |
| **RED** | Remove the secret check (or restore `IsInternalOrigin`'s any-port match) ⇒ the same request reaches the handler. ⚠️ Use a **regtest/scratch wallet** or a bogus destination the handler rejects *after* the gate (log line proves the gate was passed). Never run the RED against a funded mainnet wallet |
| **SUBJECT** | Wallet log shows the request's path and the gate decision; process is the release build's wallet on its release port |
| **Tier** | T2 (live local wallet, no network) |

**Standing invariant?** Yes. Propose for `REGRESSION_ADDITIONS.md`: *"a header-free request from
outside the shell never reaches a fund-moving handler."* It belongs beside `R-INTEXT`.

## Links

- `TICKET_loopback_host_form_wallet_routing.md` §1: "a missed match is not ungated, it is trusted"
- `TICKET_wallet_backend_is_shared_across_os_accounts.md`
- `../track-5-identity-privacy/phase-P2-dapp-reachable-surface/PHASE_CONTRACT.md` (`P2-A9`, W6)
- `../track-5-identity-privacy/phase-P4-one-os-account-one-wallet/PHASE_CONTRACT.md` (`P4-A6`)
- `../track-5-identity-privacy/SCOPE.md` (Q12)
- `../G3_CONTROLS_FINDINGS.md` (T5-P2-A1, T6-P5-A9)
