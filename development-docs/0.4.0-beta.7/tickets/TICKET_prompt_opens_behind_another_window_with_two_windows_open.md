# 🪟 With two browser windows open, a permission prompt opens BEHIND the other window — the site looks frozen

**Found:** 2026-09-25, 👤 owner, on the **installed public `v0.4.0-beta.4`** (Windows), while building a demo site.
**Status:** 📌 PROPOSED (G2) · **Track:** B5-T6 Browser shell · **Filed by:** Claude, from the owner's report
**Origin:** `../../HOTFIX_2026-09-25_prompts_not_showing/README.md` (Lead C) — first reported as an emergency, downgraded
the same day: **buggy, not broken, not a security issue — but it will feel broken to a user.**

> ⚠️ **Method note.** One owner observation + the shell log (`debug_output-53788.log`, kept locally, not committed).
> Not yet reproduced on a dev build. macOS: owner saw no issue there — how it was tested is not recorded.

---

## What happens

With **two windows** open (one torn away from the other — the beta.3 tear-away work), when a site asks for permission
(connect prompt, domain approval) or the user right-clicks → **Manage Site Permissions**, the prompt window is created
but **ends up behind the other browser window**: one window seems to drop away and the user sees the *other* window.
The shell log shows the prompt *was* shown (`🔔 Reusing existing notification overlay (keep-alive, JS injection)`).
With **one window**, the same prompts work (owner, 2026-09-25).

Likely mechanism (unverified): the keep-alive notification overlay is owned by / positioned against the wrong window —
the primary, or whichever window it was first created with — not the window the requesting tab lives in. Same family as
the beta.3 tear-away bugs (*"open an overlay and the browser disappears; I see the other one"*) and
`TICKET_window_scoped_work_uses_process_globals.md`.

## Why it matters

The site waits on a prompt the user cannot see, so the page looks hung and the wallet "not responding" — the user
concludes the wallet is broken. Requests pile up behind it (`5 more waiting from this site`).

## To capture before fixing (ask the owner, or reproduce)

1. Which window was in front — the original or the torn-away one — and which window held the requesting tab?
2. Does clicking/Alt-Tabbing reveal the prompt behind, or is it off-screen / zero-size?
3. Every prompt type, or only some (connect bundle, domain approval, edit-permissions)?
4. macOS: repeat with two windows, one torn away.

## Test and negative control (sketch)

| | |
|---|---|
| **GREEN** | Two windows; a tab in window B triggers a prompt → the prompt is **topmost over window B** and has activation |
| **RED** | Revert the fix → same setup, prompt behind (reproduce this first — it is also the confirmation) |
| **SUBJECT** | The overlay HWND's owner and Z-order read from Win32 (`GetWindow(GW_OWNER)`, `IsWindowVisible`, Z-order relative to window B), not the shell's own "shown" log line |
