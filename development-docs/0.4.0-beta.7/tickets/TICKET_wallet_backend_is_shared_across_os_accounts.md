# 🔐 A second OS account's browser uses — and on macOS shuts down — the first account's wallet

**Found:** 2026-09-24, macOS, during the post-promotion self-update test (relay 2026-09-24a). A standard
test account (`hodostest`) ran `0.3.0-beta.29` while the owner's installed beta.2 was running in his own
account. Owner's suspicion: *"it still has the wallet … it's using the same database"*.
**Status:** ⬜ UNASSIGNED · **Track:** unassigned · **Filed by:** Mac session, owner-directed

> ⚠️ **Method note.** One event is **measured** (the owner's wallet stopped when the test account's
> browser quit — §1). The *mechanism* is **code reading** of the macOS and Windows shells and the Rust
> middleware; the log is consistent with it but I did not capture the `/shutdown` request itself.
> ⛔ **Not verified:** that a headerless request from another account can actually **sign or spend**
> (§3) — no such request was sent against a real wallet. Also not run on Windows at all.

---

## What happens

⭐ **Loopback ports are machine-wide on macOS and Windows.** Every OS account sees the same
`127.0.0.1:31301`, and only one process can own it.

1. **Adopt.** At startup the shell asks "is a wallet already on the port?" and, if so, **uses it instead
   of starting its own** — without checking *whose* it is.
   - macOS `cef_browser_shell_mac.mm :: SpawnWalletServer` — `if (QuickHealthCheck()) { LOG_INFO("Wallet server already running (dev mode) - skipping launch"); g_walletServerRunning = true; return; }`
   - Windows `cef_browser_shell.cpp :: LaunchWalletProcess` — same, keyed on `IsPortListening(hodos::WalletPort())` (any listener, not even a health reply)
   - Adblock (`31302`) has the same branch on both platforms.
   The log line says "dev mode", but the branch is **not** gated on `IsDevEnv()` — it runs in release builds.
2. **Shut down (macOS only).** `cef_browser_shell_mac.mm :: StopServers` sends `POST /shutdown` to the
   port whenever `g_walletServerRunning` — which step 1 set to `true` for a wallet it did **not** start.
   Windows `StopWalletServer` only sends it when it holds `g_walletServerProcess.hProcess`, so an adopted
   wallet is left running there (code reading).
3. **`/shutdown` has no check** (`rust-wallet/src/main.rs` route table → `handlers::shutdown`, behind only
   `cors` and `domain_trust_mw`, and a headerless request passes the latter — §3).

## §1 — Measured, 2026-09-24 (macOS 26.6)

| time | event (unified log, `processImagePath`) |
|---|---|
| 06:55:36 | test account launches `/Users/hodostest/Applications/HodosBrowser.app` (beta.29), pid 9325 |
| 06:55–07:09 | **no** `hodos-wallet` process from the test account's bundle appears — consistent with adopting |
| 07:09:13 | pid 9325 quits (the moment Sparkle installed beta.4) |
| **07:09:13** | the owner's wallet **pid 56792** and adblock **pid 56793** (running since 2026-09-08) log their last line — same second |
| after | owner's browser pid 56785 still up; **nothing listening on 31301 or 31302** — dApps would see no wallet |

The owner's wallet **data** was not touched — a stopped process, not a changed DB. Recovered by
restarting his browser.

## Why it matters

- **Cross-account wallet use.** On any Mac/PC where two accounts run Hodos, the second account's browser
  shows and operates the **first account's** wallet. The OS account boundary — the one isolation users get
  for free — is crossed.
- **Cross-account denial of service (macOS).** Quitting Hodos in one account switches off the other
  account's wallet silently, the 2026-09-01 "no wallet for hours" shape
  (`../../0.4.0-beta.3/TICKET_wallet_backend_death_is_silent_and_unrecovered.md`), caused from outside.
- **Testing.** Every multi-account test rig on one machine is contaminated by this — including the one
  that found it.

## How exposed are we — answer this first

| If | Then |
|---|---|
| one account per machine (the common case) | not reachable through this path |
| two accounts both running Hodos (family Mac, shared office PC, Fast User Switching) | second account uses the first's wallet; on macOS quitting it kills the first's |
| §3 holds (headerless = fully trusted) **and** the wallet is auto-unlocked | a local program in **any** account can call signing endpoints with no prompt — ⛔ **unverified**, see method note |

### §3 — the wider fact under it (code reading)

A request with **no** `X-Requesting-Domain` is treated as the wallet's own UI:
- `rust-wallet/src/main.rs :: domain_trust_mw` — `None => return Ok(next.call(req).await?…)` ("Internal call → no gate")
- `rust-wallet/src/permission_service/request_gate.rs` — `None => return GateOutcome::Proceed` (three gates)
- `cors` restrains browser pages only; a non-browser client ignores it
- the wallet auto-unlocks at startup (Keychain on macOS, DPAPI on Windows — `main.rs`, "🔓 DPAPI auto-unlock succeeded"), so no PIN stands in the way

⭐ This does **not** undo the beta.3 work on the *browser* path: dApp traffic is stamped by the C++ layer
and gated. It is the **non-browser** path — any local process talking to `127.0.0.1:31301` directly — that
looks identical to the wallet UI. Same-account processes are roughly the normal desktop limit (they can
read your files too); **other-account** processes are not.

## What already protects us, and how that shapes the fix

- Windows already refuses to `/shutdown` a wallet it did not spawn — the macOS half is a one-condition port.
- Profiles inside one account share one wallet **by design** (👤 owner, 2026-09-24: keep it that way for
  now), so the fix must keep same-account sharing and stop cross-account sharing.

## Proposed fix (not sized — not scoped)

**Floor** (small, both platforms): never adopt a wallet on the port unless it belongs to **this OS user**,
and never `/shutdown` one we did not start (macOS: mirror the Windows handle guard).
- Owner check options: ask the OS which user owns the listening socket's peer (macOS `LOCAL_PEERCRED` /
  `proc_pidinfo`; Windows `GetExtendedTcpTable` owning PID → token SID); or a per-user secret written in the
  user's own app-data dir that the wallet echoes on `/health`.
- If the port is held by **another** user: fail loudly ("another account on this computer is running
  Hodos") rather than adopt — or use a per-user port (next item).

**System** (a design decision, owner): move browser↔wallet off a shared TCP port onto a **per-user channel**
— a Unix domain socket in the user's own `~/Library/Application Support/HodosBrowser/`, a named pipe ACL'd
to the user's SID on Windows. Other accounts cannot open it by construction; profiles in the same account
share it, matching the owner's policy. ⚠️ The dApp-facing surface still needs HTTP on loopback — this is
about the **internal** channel and about making "headerless = trusted" true only for callers that can prove
they are us (e.g. a per-user token), which is the §3 question.

Prior art to read before designing (CLAUDE.md working rule 5): how Docker Desktop, 1Password's local agent
and VS Code's CLI↔server IPC scope a local daemon per user; MetaNet Client's fixed port `3321` has the same
shape and is worth checking for how it behaves under two accounts.

**Deliberately out of scope:** same-account malware (needs a different threat model); the dApp HTTP
surface (`TICKET_dapp_reachable_surface_is_a_denylist_not_an_allowlist.md`); per-profile wallets.

## Test and negative control

| | |
|---|---|
| **GREEN** | account B launches Hodos while account A's Hodos runs ⇒ B starts **its own** wallet (or refuses clearly); B's wallet UI shows **B's** wallet; quitting B leaves A's `hodos-wallet` pid alive and A's `/health` answering |
| **RED** | today's build, same two-account sequence ⇒ no `hodos-wallet` from B's bundle, and A's wallet pid dies the second B quits (macOS). Reproduced once, 2026-09-24, §1 |
| **SUBJECT** | owner of each `hodos-wallet` pid (`ps -o user`), its bundle path (`processImagePath`), and **which wallet B's UI shows** (identity key) — a green that only checks "a wallet answered" re-creates this bug |
| **Tier** | T2 (two real accounts, one machine) — needs Fast User Switching (`MultipleSessionEnabled`) |

**Standing invariant?** Yes — propose `R-XACCOUNT` for `../REGRESSION_ADDITIONS.md`: *a Hodos in one OS
account never uses or stops another account's backend.*

## Links

- Relay: `../../0.4.0-beta.3/MAC_RELAY_BETA3.md`, round 2026-09-24a
- `../../0.4.0-beta.3/TICKET_wallet_backend_death_is_silent_and_unrecovered.md` — the "no wallet" symptom
- `TICKET_dapp_reachable_surface_is_a_denylist_not_an_allowlist.md` — the dApp-facing half of the surface
- `../../DevOps-CICD/WALLET_GRACEFUL_EXIT_SPEC.md` — `/shutdown` semantics
