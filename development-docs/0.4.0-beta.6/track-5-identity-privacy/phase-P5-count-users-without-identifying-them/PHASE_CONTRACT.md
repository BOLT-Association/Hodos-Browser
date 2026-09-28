# B5-T5-P5 — Count users without identifying them (D9) · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (copied 2026-09-28 from `../../../0.4.0-beta.3/PHASE_CONTRACT_TEMPLATE.md`, read-only).
> Written at G3 by the T5 track agent (relaunch). ⛔ Documents only — no code, no endpoint, no public page, no issue.

**Track:** B5-T5 Identity & privacy · **Tickets:** `../../tickets/TICKET_active_user_count_without_identifying_users.md` (its own HMAC-rotating-ID design is **superseded** by the owner's direction to adopt the industry standard — T5 SCOPE §4) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), T5 track agent, repo head `0777b26` on `0.4.0` · **Platforms:** both · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** **Decision 13** — a cut-down **Brave usage ping, OPT-OUT**, with three conditions: ① a first-run notice; ② the public **"every byte we send"** page is live **before** the ping ships; ③ a **wire-level** negative control proves the off switch sends **zero** requests (Brave shipped exactly that bug, `brave-browser#45271`). Once a day, one cookieless HTTPS GET: `daily` / `weekly` / `monthly` / `first` flags + OS, channel, version. **No** install date or week (`dtoi` / `woi`), no referral, no ID / account / key / balance / URL. Users are told plainly that the server **sees the IP** at request time and that the protection is **policy** (country at the edge, small countries suppressed, IP dropped before write, no access logs) — never called "anonymous". K11 (endpoint location/readers) and K12 (page wording) are **proposed here** (§12). ⛔ **Switch-on is outward-facing and irreversible once data exists — it returns to the owner** (unconditional stop 4). 👤 Owner direction (ticket, 2026-09-25): adopt the best-privacy existing standard, **do not design our own**.

---

## 1. Goal

We can say how many people used Hodos today, this week and this month, while the browser sends nothing that identifies a person or links one day's report to the next — the user was told before the first report, can read every byte that is sent, and can switch it off knowing it then sends nothing.

## 2. Done means

- [ ] The "every byte we send" page text is written, owner-approved (K12), and — **before any build that can ping ships** — published by the owner (`P5-A0`). ⛔ Publishing is the owner's act.
- [ ] Browser: one HTTPS GET per local calendar day at most, at first launch of the day or after midnight while running; query = `daily=1` [+ `weekly=1` if first this ISO week] [+ `monthly=1` if first this month] [+ `first=1` on the very first ping ever] + `os` + `channel` + `version` — and **nothing else**: no cookies, no `Referer`, no custom headers, no body (`P5-A1`, `P5-A2`, `P5-A5`).
- [ ] The device keeps only three local values — last ping day, last ISO week, last month — plus a "first sent" bit; nothing that can be sent as an identifier (`P5-A5`).
- [ ] Setting (Privacy section) **on by default**; off ⇒ **zero** requests to the endpoint, measured on the wire, including when switched off mid-session and after restart (`P5-A3` — decision 13 ③).
- [ ] First-run notice shown before the first ping can fire, with a working link to the setting and to the public page (`P5-A4` — decision 13 ①).
- [ ] Dev builds (`HODOS_DEV=1`) never contact the production endpoint (`P5-A6`).
- [ ] `simple_handler.h`'s *"we ship no telemetry"* comment and every doc/UI string making the same claim are corrected **in the same commit** as the ping (`P5-A7`).
- [ ] Server side (K11): counters only, no request log, IP dropped before any write, small countries folded — **specified and reviewed**; the switch that makes a released build actually send is ⛔ **the owner's**, after `P5-A0`–`A7` are green (`P5-A8`).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| The privacy promise (G1 mission; `simple_handler.h`) | We never send identifying data | This phase **adds** the first outbound report. Every row below exists because this is the one place the promise is at stake |
| `R-UPDATE` | Auto-update applies and never forces reinstall | The ping must not piggy-back on or alter the Sparkle/WinSparkle appcast request (T5 SCOPE §3: owner declined counting appcast hits; Sparkle's request is not ours to shape). `P5-A1` SUBJECT separates the two requests |
| CEF `TID_FILE_*` threads (memory: all three are **one** thread) | A slow send stalls balance polls | The ping must run on its own thread / async client with a short timeout, never a blocking `SyncHttpClient` call on `TID_FILE_*` — `P5-A9` |
| Dev/prod isolation | Dev never touches production | `P5-A6` |
| Farbling / fingerprint surface | The ping is not a fingerprint | Only coarse `os` (`windows`/`macos`), `channel`, `version`; no screen, locale, timezone, hardware |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. No money, schema or crypto rows in this phase, so REDs are designed here; the **owner-decided** control (decision 13 ③) is cited as such. ⭐ Every SUBJECT is **the network**, captured by a proxy between the browser and the endpoint — not a log line (T5 SCOPE §5 P5).

**Rig.** A dev endpoint (a local HTTPS stub or the K11 host's staging path) behind a capturing proxy (mitmproxy or equivalent) with the dev build pointed at it by a dev-only override; the system clock or an injected "today" provider to cross day/week/month boundaries.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P5-A0` 👤 | The public page's field list (K12) is final and owner-approved; the build's request is generated from the **same** field list (one source); the page is live before any ping-capable build is released | Release checklist item fails if the page URL returns non-200 at promote time (checked by curl in the RC checklist; a deliberately wrong URL shows the check can fail) | The live page's text vs the request builder's field table; the RC checklist line | T0 + human | ⬜ |
| `P5-A1` | Field-by-field: the captured request equals the published list exactly — method `GET`, path, the flag params, `os`, `channel`, `version`; headers limited to what the HTTP client cannot avoid (`Host`, `User-Agent`, `Accept`, `Connection`) | Add one extra query field (or a custom header) in a dev build ⇒ the diff script reports it and fails (decision 13: "an extra field fails") | The proxy's raw request dump, diffed by script against the page's field table — not the browser's own log | T2 | ⬜ |
| `P5-A2` | Day/week/month logic: same day ⇒ no second request; next day same week ⇒ `daily` only; first day of a new ISO week ⇒ `daily`+`weekly`; first day of a new month ⇒ `+monthly`; the very first ping ⇒ `+first`, and never again | Freeze the injected "today" and restart twice ⇒ exactly one request total (shows the de-dup can be seen); remove the month comparison ⇒ `monthly` fires daily and the case table fails | Proxy capture per simulated date, one row per date in the result table | T1 (pure date logic, table-driven) + T2 | ⬜ |
| `P5-A3` | Off switch ⇒ **zero** requests reach the endpoint: (i) off before launch, over a simulated day change; (ii) switched off **mid-session** before midnight; (iii) off, restart, day change | 👤 **Owner-decided control (decision 13 ③), stated as its instrument check:** the same capture with the setting **on** sees ≥ 1 request on the same schedule — proving the proxy and endpoint can see a ping, so the zero in the GREEN means "not sent", not "not captured". Brave `#45271` is the failure this catches | Proxy capture across the whole window, **filtered on host only** (not path — a path filter would miss a mis-routed ping) | T2 | ⬜ |
| `P5-A4` 👤 | Fresh profile: the first-run notice appears **before** any ping can fire; it names what is sent, links the setting and the public page; dismissing it does not change the setting | Make the ping scheduler ignore the "notice shown" state in a dev build ⇒ the proxy sees a request timestamped before the notice's shown-event | Proxy timestamp vs the notice's shown-event log line; the owner reads the notice (visual judgement) | T2 + T3 | ⬜ |
| `P5-A5` | No identifier: two fresh profiles on one machine, and the same profile on two consecutive days, produce requests that differ **only** in flag values — no cookie, no stable token, no timestamp beyond the server's own receipt | Point the request at a context **with** the cookie jar in a dev build and pre-seed a cookie for the endpoint host ⇒ the capture shows `Cookie:` — proves the check can see a cookie | Byte-diff of the captured requests (headers + URL) across profiles/days | T2 | ⬜ |
| `P5-A6` | `HODOS_DEV=1` builds never resolve or contact the production endpoint host | Remove the dev guard ⇒ the capture (or a DNS log) shows the production host | DNS / proxy capture on the dev machine over one simulated day change | T2 | ⬜ |
| `P5-A7` | *"we ship no telemetry"* (`simple_handler.h`) and any same claim in `PrivacySettings.tsx`, the privacy docs and marketing copy in the repo are corrected in the ping's commit | Grep for `no telemetry` / `we don't collect` ⇒ > 0 before, 0 after (or each remaining hit is true and says so) | The grep output over `cef-native/`, `frontend/src/`, `development-docs/` user-facing docs | T0 | ⬜ |
| `P5-A8` 👤 | Server spec reviewed: counters keyed `(date, flag, os, channel, version, country-or-other)`; countries under the fold threshold ⇒ `other`; no request rows; access logging off; IP never written; retention published. **Switch-on is not part of this row** | A staging request's IP searched for in everything the endpoint writes (counters, logs, error reports) ⇒ must be absent; the same search run after deliberately enabling access logs on staging must **find** it (shows the search can find an IP) | The endpoint host's stored data, read by the owner (K11 readers) | T2 human | ⬜ |
| `P5-A9` | The ping never blocks a browser thread: endpoint black-holed ⇒ balance polling and page loads unaffected; the ping gives up within its timeout and retries next launch/day, never in a loop | Put the send on `TID_FILE_USER_BLOCKING` with a 30 s timeout in a dev build ⇒ the wallet balance poll stalls (measured shape from the 2026-09-14 file-thread incident) | Browser log timing of balance polls during a black-holed ping; proxy shows at most one attempt per day | T2 | ⬜ |
| `P5-A10` | Boundary: `R-UPDATE` T1 (appcast request unchanged — its capture shows no ping params), minimal site basket | Per `REGRESSION_SET.md` | The appcast request in the same capture as `P5-A1` | T1–T2 | ⬜ |

**Two-sided rows:** `P5-A3`'s GREEN (off ⇒ zero) is only meaningful beside its on-side instrument check. `P5-A2` pairs "fires when due" with "does not fire twice".

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `cef-native/include/handlers/simple_handler.h` — comment *"Logs locally only — we ship no telemetry and this does not change that."* | ✅ | Becomes false; changes in the same commit (`P5-A7`) |
| `cef-native/include/core/SettingsManager.h :: PrivacySettings` | ✅ | Home of the new `usagePing` bool (default `true` — opt-out). Persistence via `SettingsManager`; ⚠️ the three local dates are **not** settings the user edits — stored beside them, never sent as values |
| `cef-native/include/core/SettingsManager.h` — `global_update_mode_absent_at_load_` ("first-run collapse flag") | ✅ | The only existing first-run notion; the notice needs its own "shown" bit — reuse the load-time detection pattern, not the flag |
| `frontend/src/components/settings/PrivacySettings.tsx` | ✅ (file exists) | The switch lives here (native `<input>`, CEF input rules) |
| `cef-native/include/core/SyncHttpClient.h :: SyncHttpClient::Get` | ✅ | Cross-platform client (invariant 9). ⚠️ Blocking — must run on a dedicated thread (`P5-A9`) and on a request context **without** cookies |
| `cef-native/src/core/AutoUpdater.cpp :: AutoUpdater::Initialize` (`win_sparkle_set_update_check_interval(86400)`), `AutoUpdater_mac.mm`; `cef_browser_shell.cpp` `appcastUrl = "https://hodosbrowser.com/appcast.xml"`; `Info.plist` `SUFeedURL` | ✅ | ⛔ Not edited; the ping is a **separate** request on its own path. Honesty note carried to the page: the update check already shows `hodosbrowser.com` the IP and version once a day |
| `cef-native/include/core/PortConfig.h :: IsDevEnv` (dev detection) | ✅ (used by the ports helpers) | Dev guard for `P5-A6` |

## 6. Out of scope

- Brave P3A / STAR (feature questions, heavy infrastructure — T5 SCOPE §2.1).
- `dtoi` / `woi` retention cohorts (decision 13 — near-identifying at our size); `ref`; search/ads counts.
- Counting appcast hits (owner declined 2026-09-17).
- The ticket's HMAC rotating-ID scheme (superseded).
- Any wallet data, ever.
- Turning it on in a released build — ⛔ the owner's act (§12).

## 7. Rollback

One commit for the client (setting + scheduler + notice + comment fix); reverting it removes the request entirely. Server data, once collected, **cannot** be un-collected — which is why `P5-A0`–`A8` precede switch-on and switch-on is the owner's.

## 8. Pre-mortem (adversarial review — before)

| Story: it shipped and failed because… | Row that catches it |
|---|---|
| The off switch stopped the scheduler but a pending timer still fired once (Brave `#45271`) | `P5-A3` (ii) mid-session |
| The request went through the default request context and carried a cookie the endpoint's host had set on a visit to `hodosbrowser.com` — a stable identifier by accident | `P5-A5` |
| Upgrading users had no local dates, so every existing install sent `first=1` on upgrade — "new users" spike that is really old users | §12 Q3 decides; `P5-A2` table includes an upgrade case |
| The ping ran on `TID_FILE_*` and a black-holed endpoint froze balance polls for 30 s at every launch | `P5-A9` |
| The page and the code drifted — the page listed five fields, the code sent six | `P5-A0` one source + `P5-A1` diff |
| A dev machine flooded production counts | `P5-A6` |
| The notice was shown after the first ping on a slow first launch | `P5-A4` timestamp ordering |
| The endpoint host kept default access logs, so the IP **was** written | `P5-A8` |
| A small country + OS + version combination identified one person | `P5-A8` fold threshold (K11) |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | |
| macOS | `P5-A1`–`A6`, `A9` | Same C++ (SyncHttpClient = libcurl on macOS). New C++ ⇒ **relay round** naming the scheduler file(s); `os=macos` value checked in the capture |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~3 h**: K11 hosting decision + staging access (~45 min), K12 page wording review + publish (~45 min), first-run notice review (`P5-A4`, ~20 min), server spec review / IP search (`P5-A8`, ~40 min), the switch-on decision itself (~15 min, after everything is green) |
| Human-bound rows | `P5-A0` (publish), `P5-A4` (visual), `P5-A8` (server data access), switch-on |
| Unknowns (K) — uncertainty, not difficulty | **K11** where the endpoint lives and who can read it (proposal §12 Q1) · **K12** page wording (draft §12 Q2) · **K23 (new)** what `hodosbrowser.com` is hosted on today and whether it can run code / disable access logs — decides whether K11's proposal needs a separate edge host |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | **B5-T6** (browser shell) | T5 SCOPE §7 says T6 "hosts the settings switch and the first-run notice", but ⚠️ **T6's SCOPE has no such item** (grep 2026-09-28). Recommendation: this phase builds both (small: one toggle in `PrivacySettings.tsx`, one notice) and T6 reviews for overlay/first-run conflicts — §12 Q4 |
| needs | **B5-T0 Build 1** | Browser-level captures on the new engine |
| gives | release comms (G5) | The public page is outward-facing — it joins the comms plan and the release notes |
| shares | `R-UPDATE` | The appcast request is captured beside the ping to prove they stay separate |

## 12. Open questions for the owner

1. **K11 — endpoint location and readers (proposal).** `GET https://hodosbrowser.com/u/1?daily=1&…` served by a small edge function on the **same host** as the appcast (so no new host learns anything the update check does not already see), storing **only** daily counters keyed `(date, flag, os, channel, version, country)`; countries with fewer than **10** requests that day fold into `other`; no request log; access logs off at the host; IP never passed to storage; counters kept **25 months** and then deleted; **readers: the owner only** (named on the page). ⚠️ If `hodosbrowser.com`'s host cannot run code or turn access logs off (K23), the fallback is a subdomain on an edge-function host — which is **not** the "Worker in front of the appcast" the owner declined (that counted appcast hits; this is a separate endpoint). Approve, or name the host?
2. **K12 — page wording (draft, to be refined with the owner):**
   > **What Hodos sends, and nothing else.** Once a day, when you first open Hodos, it sends one request to `hodosbrowser.com`. Here is every piece of it: `daily=1`; `weekly=1` if it is the first this week; `monthly=1` if it is the first this month; `first=1` the very first time only; your operating system (`windows` or `macos`); your update channel; the Hodos version. **It contains no ID, no cookie, no account, no wallet data, no keys, no balance, and no website you visited.** We cannot link today's request to yesterday's. **It is not anonymous:** like any web request — including the daily update check Hodos already makes — our server sees your IP address at the moment it arrives. We use it only to look up your country, then drop it before anything is stored, and we keep no access logs. Countries with very few users are grouped as "other". Only [owner] can read the counts, which are deleted after 25 months. **Switch it off** in Settings → Privacy; Hodos then sends nothing at all.
   Approve / edit?
3. **Upgrade and `first`.** Existing installs have no local ping dates. (a) send `first=1` on their first ping after upgrade (overcounts new users once); (b) send `first=1` only when the profile was created **after** this version (no install date is sent — the bit is decided locally). Recommendation: **(b)**. Yes / no?
4. **Who builds the switch and notice** — this phase (recommended; T6 reviews) or a new T6 item?
5. ⛔ **Switch-on.** After every row is green, the released build's ping stays **disabled by a build flag** until you say "on". Confirming that is the procedure.
6. No evidence that a G2 decision is wrong.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
