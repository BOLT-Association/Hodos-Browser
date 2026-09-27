# HOTFIX — permission prompts stop appearing in the public `v0.4.0-beta.4` (Windows)

**Opened:** 2026-09-25, 👤 owner report, on the **installed public `v0.4.0-beta.4`** (Windows).
**Status:** 🟡 **EMERGENCY AVERTED 2026-09-25 ~16:15 (owner).** With a **single window** the prompts work; the
failure is the **two-window Z-order** defect (Lead C) — *buggy, not broken, not a security issue*, but it will feel
broken to a user. ⇒ **To become beta.5 tickets** after the owner walks through what he sees with the modal in
general (next session). Lead B still to be checked.
**Was:** 🔴 OPEN — investigate and fix today. ⛔ **Nothing is decided about the version number or
the release yet** (see §6).
**Folder name is date + symptom on purpose** — not a version number, so it cannot collide with a
planning folder the way `0.4.0-beta.4/` did. beta.5 planning is paused: `../0.4.0-beta.6/README.md`
§"▶️ RESUME HERE".

---

## 1. What the owner saw

1. Building a demo site (`archie.demo`) in another session: the **permission pop-up "wasn't working
   correctly"** and "it lets some stuff through" *(owner's words — not yet pinned down; ask)*.
2. On **zanaadu.com**: **right-click → Manage Site Permissions would not open.**
3. In the **advanced wallet → Approved sites**, deleted zanaadu.com, went back, refreshed: **no prompt
   appears, and the wallet does not respond** — it does *not* bypass the gate, but the user is never
   asked.
4. 👤 It **worked in the dev browser** shortly before publishing. 👤 **macOS does not show the issue**
   (owner, same day — how it was tested on macOS is not yet recorded).

## 2. Evidence already collected (measured, read-only, 2026-09-25 ~16:00)

- **Production wallet is healthy:** `GET http://127.0.0.1:31301/health` → `{"status":"ok"}`. Not a
  dead backend.
- Logs copied to `evidence/` (⛔ **not committed** — they can hold browsing data; they are on the
  Windows machine only): `debug_output-53788.log` (the running shell, **started 2026-09-24 12:08**, i.e.
  ~28 h uptime), `cef_debug.log`, `audit-53788.log`, `wallet_tail_2026-09-25.log`.

**Lead A — every prompt that failed went through the keep-alive reuse path.** The notification overlay
was created **once** (`🔔 Notification overlay browser created (first time)` at 2026-09-24 12:08:13),
then every later prompt reused it:
```
15:24:28 Creating notification overlay (type: permission_request, domain: 127.0.0.1) → Reusing existing notification overlay (keep-alive, JS injection)
15:41:37 … domain_approval, archie.demo:4321 → Reusing existing …
15:44:52 … edit_permissions, archie.demo      → Reusing existing …
15:55:14 … manifest_connect_bundle, zanaadu.com → Reusing existing …   (owner: nothing appeared)
15:57:03 … edit_permissions, zanaadu.com       → Reusing existing …   (owner: right-click did not open)
```
The shell *believes* it showed them. ⇒ Hypothesis: the keep-alive overlay (HWND or its page) is dead
or hidden after some event / long uptime, and the JS injection lands nowhere. The dev browser is always
freshly launched — which would explain "worked in dev".

**Lead B — deleting a site in the advanced wallet does not reach the browser's approval cache.** Both
times the owner deleted a site:
```
15:45:09 🛡️ IPC DENIED (P0.5-B1): domain_permission_invalidate from tab role 'tab_16'
15:54:13 🛡️ IPC DENIED (P0.5-B1): domain_permission_invalidate from tab role 'tab_21'
```
…and immediately after, for zanaadu.com:
```
15:54:17 🔁 Stale connect prompt for already-approved zanaadu.com /getVersion — re-sending the call instead of asking again
15:54:17 ⏳ manifest_connect_bundle for zanaadu.com queued behind the prompt on screen
15:55:14 ⏭️ Showing next queued prompt … (5 more waiting from this site)
```
⇒ The advanced wallet runs **as a tab**, and the beta.3 Phase 0.5 IPC gate (P0.5-B1) rejects this
message from tab roles — so the browser still thinks the site is approved, and the "stale connect"
re-send (a **beta.4** fix: *"connect prompts arriving after approval were orphaned"*) re-sends instead
of asking. Requests then park on a prompt the user never sees.

**Lead C — 👤 owner found it, 2026-09-25 ~16:10: the prompt window IS there, but hidden BEHIND.** With
**two browser windows** open (one torn away from the other — the beta.3 tear-away work), when the prompt
opens, one window drops away and the owner sees the *other* window; the prompt is behind it. ⇒ a
**Z-order / owner-window** defect: the notification overlay is positioned or activated against the wrong
window (likely the primary, or the window it was created with), not the window the site is in. This is
the same family as the beta.3 tear-away bugs *("open an overlay and the browser disappears; I see the
other one")*. It **replaces Lead A as the likely cause** of "the prompt never shows" — the keep-alive
overlay was never dead, just behind. Owner is re-testing with a **single window**.

⚠️ Leads B and C are **hypotheses from logs + one owner observation**, not yet reproduced. Lead B (the
invalidate message rejected from the advanced-wallet tab) is independent of C and still needs checking.

## 3. First steps for the fix session

1. **Reproduce on the dev build** — fresh launch, then: (a) delete a site from the advanced wallet and
   watch for `IPC DENIED … domain_permission_invalidate`; (b) trigger several prompts in one long session
   and see whether the keep-alive overlay stops appearing.
2. `git log` the prompt-window path and the permission cache since the last known-good build — who
   changed what (Windows vs macOS commits), especially the beta.4 "stale connect prompt re-show" change,
   `CreateNotificationOverlay` keep-alive, and the P0.5-B1 IPC role gate.
3. Only then design the fix — with a **negative control** for each lead (root `CLAUDE.md` Testing).

## 4. Load-bearing safeguards to audit (root `CLAUDE.md` phase kickoff)

The gold pill · right-click Manage Site Permissions (**currently broken — this bug**) · the "Always
notify" toggle · the four privacy-perimeter gates · per-session counters.

## 5. Owner's quick test before leaving (optional, ~2 min)

Fully quit Hodos (every window), relaunch, go to zanaadu.com, trigger the connect; then right-click →
Manage Site Permissions. **Does the prompt / editor appear now?** One line back is enough — the new
run writes its own log.
- Appears after restart ⇒ Lead A is state/uptime-dependent.
- Still missing ⇒ systematic; likely Lead B keeps it broken, since the browser still believes the site is approved.

## 6. Open — owner decisions, NOT made yet

- **Version for the fix release.** The next number is `v0.4.0-beta.5` — which is also the name of the
  planning folder. Options: ship the fix as beta.5 and rename planning to a **version-neutral codename**
  (so this never happens again), or rename it beta.6. Recommendation: codename.
- Whether the **engine security refresh** (`../0.4.0-beta.6/track-0-engine/SCOPE.md`) rides with this fix
  or follows it.
