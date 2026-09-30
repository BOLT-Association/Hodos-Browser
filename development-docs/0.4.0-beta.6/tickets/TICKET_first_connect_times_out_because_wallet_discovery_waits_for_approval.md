# 🟡 A site's first connect times out: we hold its `getVersion` discovery call behind the connect prompt, and the site's connector gives up before the user approves

**Found:** 2026-09-30, on `themetoken.dev` (a new dApp) in the dev build, with the owner at the keyboard. The installed build also failed to connect there.
**Status:** ⬜ UNASSIGNED · **Track:** unassigned. ⭐ Suggested: **B5-T5-P2 — what a dApp can reach** (its §8a already records the other BSV browser answering `getVersion` in-page because "the SDK allows one second for discovery"). · **Filed by:** Claude (Opus 5.5), with the owner

> ⚠️ **Method note.** Measurement: dev shell log, dev wallet log, and the page's own network and console via CDP. Repro by the owner (below). **Not verified:** the connector's actual timeout value, and which library the site uses (it probes like 1Sat's connector; not confirmed).

---

## What happens

1. On **Connect**, the page probes several wallets at once: `POST http://localhost:3321/getVersion`, `POST https://localhost:2121/getVersion`, `http://localhost:3301` (refused) and an iframe to `https://1satwallet.com/wallet/cwi`. It does **not** use our injected `window.CWI`.
2. Our interceptor treats the first wallet call from an unapproved domain as the trigger for the connect modal and **holds** it. The shell log shows `Drained 3 pending request(s) for themetoken.dev after approval (3 resumed)` about **6 s** after the click, when the owner approved. The wallet then answered all three `getVersion` calls with 200.
3. By then the site's connector had given up. The page spins and makes no further wallet call.
4. On the **second** attempt, with the domain approved, `getVersion` is answered at once. The page proceeds (`isAuthenticated`, `getPublicKey`, `listOutputs`, each prompting separately because the site sends no manifest) and reaches its "Select theme" screen.

**Repro (owner, 2026-09-30):** remove the site's permission ⇒ Connect ⇒ approve ⇒ spins forever; refresh ⇒ Connect ⇒ connects. Theory confirmed by this repro.

## Why it matters

Every new site the user meets fails its first connect, and the user concludes Hodos "doesn't work with" the site. This is likely also why the installed build never connected to `themetoken.dev`.

## The fix is a policy question — review before changing

👤 Owner: answering `getVersion` without a prompt is probably right, since it reveals nothing about the user. But today's rule is effectively "**the first request that reaches the wallet from an unapproved domain opens the modal**, whatever it is". Changing that needs a **review of what triggers the modal**, call by call:
- which calls are safe to answer before consent (`getVersion`? `getNetwork`? `getHeight`? `isAuthenticated`?), and what each one leaks;
- whether answering discovery **without** a prompt means the modal then opens on the first *real* call instead, and whether that changes what the user is asked;
- the privacy perimeter (identity-key reveal, key linkage, sensitive cert fields, over-cap spends) must not move;
- prior art (rule 5): how the other BSV browser, MetaNet Client and the SDK's own discovery treat pre-consent calls.

Owner will also ask dApp developers to allow time for the wallet to respond. That helps, but it does not remove the need for the fix.

## Negative control for any fix

With the fix reverted, the owner's repro above must spin again on the first connect. With it applied, a **first** connect to a site with no stored permission reaches the site's connected state, and the connect modal still appears before any call that reveals identity or spends.
