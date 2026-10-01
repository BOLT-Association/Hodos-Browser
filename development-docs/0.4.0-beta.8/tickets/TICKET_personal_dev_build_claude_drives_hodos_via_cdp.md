# 🔍 Personal dev build: Claude drives Hodos over CDP and pays through our wallet

**Found:** 2026-09-27, owner direction during the BRC-181 / agent-policy discussion.
**Status:** ⬜ UNASSIGNED · **Track:** beta.6 intake — a build variant and experiment, not a defect ·
**Filed by:** owner + Claude

> ⚠️ **Method note.** The capability claims are readings: Hodos is CEF/Chromium so it exposes the
> Chrome DevTools Protocol (CDP), and `TICKET_cdp_port_open_in_release.md` records the port as
> currently open. Nothing below has been tried against a running Hodos. Attribution settled 2026-09-27:
> the x402 client skill IS John Calhoun's — "Copyright (c) 2026 Calgooon" in the LICENSE of both
> copies; the owner was right. Two diverged copies exist (`Hodos-Browser/.claude/skills/x402`
> and the b-open-io plugin cache; `lib/payment.py` differs). Pick one lineage before building.

---

## What this is

A **personal build for us only — never production** — where:

1. **The CDP port stays open on purpose** (possibly renamed/relabeled so it is obviously the dev
   surface). Production builds close it — this ticket is the deliberate twin of
   `TICKET_cdp_port_open_in_release.md`, which stays a real security finding for release builds.
2. **The x402 payment path runs through our own wallet** instead of the MetaNet Client — this is
   task `O-013` in the owner's plan, landed here. Practically: implement or adapt the standard
   localhost substrate (see BRC-185, Wallet Substrate Discovery, in draft) so existing x402 client
   tooling works against Hodos unchanged.
3. **Claude attaches over CDP** and can browse as the owner (X, LinkedIn, dApps), run the demo
   sites end-to-end, and make paid reads — with every spend going through the wallet's own policy
   engine.

## Why it matters

- It turns the owner's own browser into the **live test bench for agent spend policy**: the
  auto-approve engine as the inner wall, a BRC-181-style isolated capped agent account as the
  outer wall (`TICKET_evaluate_brc188_ump_account_recovery.md`'s sibling decision,
  `NOTE_2026-09-26_brc181_outer_wall.md`). We learn what is good and bad by using it, months
  before any shipped assistant.
- It replaces per-lookup external costs (paid X reads ran ~29k sats each through a third-party
  agent this week) with our own wallet and policies.
- It is the smallest real step toward the Edwin/assistant integration: eyes via CDP, hands via
  the wallet API, no assistant UI to build.

## Known gaps to design for

- **No tab, no gold pill.** A headless/terminal caller gives the pill nowhere to flash. Agent
  spends need their own visibility surface (wallet-side activity feed or notification). Same
  point as our registry comment #280's operator-visibility theme.
- **Prompts need the browser UI.** The decision engine runs in Rust, but the ask-the-user path
  renders in the browser. Fine here (the browser is running by definition), but policy for the
  agent should be reject-fast or capped, not prompt-dependent.
- **Dev carve-outs bundle naturally into this build:** localhost/`file://` provider injection
  (currently impossible — see the two Plan-inbox captures of 2026-09-25) and local HTML file
  opening, which the owner hits weekly (has to open Claude-built HTML in Chrome because Hodos
  won't open local files).

## Proposed next step (months out, by owner's call)

At beta.6 telescope: half a day to (1) confirm CDP attach against a dev build, (2) scope the
substrate work for the wallet port, (3) define the agent account + policy shape to test. Then
decide how much lands in beta.6 vs later.
