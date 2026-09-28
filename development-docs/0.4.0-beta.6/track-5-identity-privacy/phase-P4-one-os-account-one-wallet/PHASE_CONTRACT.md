# B5-T5-P4 — One OS account, one wallet · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (copied 2026-09-28 from `../../../0.4.0-beta.3/PHASE_CONTRACT_TEMPLATE.md`, read-only).
> Written at G3 by the T5 track agent (relaunch). ⛔ Documents only — no code, no schema, no issue.

**Track:** B5-T5 Identity & privacy · **Tickets:** `../../tickets/TICKET_wallet_backend_is_shared_across_os_accounts.md` · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), T5 track agent, repo head `0777b26` on `0.4.0` · **Platforms:** both — half of it is macOS-only code (`StopServers`), and the macOS session owns those rows · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** **T5 Q12** — the per-user channel (named pipe / Unix socket) is **deferred to beta.7**; this phase ships the **floor** (never adopt or stop another account's backend). 👤 Owner 2026-09-24 (in the ticket): **profiles inside one OS account keep sharing one wallet** — the fix must stop cross-account sharing and keep same-account sharing.
**Not here:** `TICKET_auto_unlock_accepts_another_wallets_mnemonic.md` is **B5-T1-P2** in the register (T1 SCOPE: "Item in P2"), not T5 — checked 2026-09-28.

---

## 1. Goal

Hodos running in one OS account never shows, operates or shuts down the wallet (or ad-block engine) of another account on the same computer; a second account is told plainly what is happening instead of silently using someone else's wallet.

## 2. Done means

- [ ] Step 0 re-run on **today's** build, both platforms (macOS measured once 2026-09-24 on beta.29; Windows never) — `P4-A0`.
- [ ] **macOS:** `StopServers` sends `/shutdown` only to a backend this process spawned (`g_wallet_server_pid > 0` / `g_adblock_server_pid > 0`), mirroring Windows `StopWalletServer`'s handle guard. Account A's `hodos-wallet` pid survives account B quitting (`P4-A1`).
- [ ] **Both:** before adopting a listener on the wallet or ad-block port, the shell proves the listener belongs to **this OS user**; if not, it does **not** adopt, does not send it anything but the proof request, and tells the user in plain words that Hodos is running in another account on this computer (K9 wording) — `P4-A2`, `P4-A3`.
- [ ] Same-account adoption still works: a second profile, and the dev workflow (`dev-wallet.ps1` / `dev-wallet.sh` started first), still adopt their own account's wallet (`P4-A4`).
- [ ] The backend supervisor (`BackendSupervisorLoop` on Windows, `StopBackendSupervisor` on macOS) never relaunches onto, or reports healthy, a port owned by another account (`P4-A5`).
- [ ] The residual is written down, not hidden: a **non-browser** local process in any account that talks to `127.0.0.1:<port>` directly is still treated as the wallet UI (headerless ⇒ trusted, `domain_trust_mw`) — closed only by the deferred per-user channel (`P4-A6`, docs).
- [ ] `R-XACCOUNT` proposed for `../../REGRESSION_ADDITIONS.md` in its **own** commit after this phase closes (working rule 6 — not in the change that implements it).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-INTEXT` | Internal never prompts | The ownership proof is a new request to `/health` (or a new endpoint) before adoption; it must not change how the wallet UI's calls are classified. Half (a) at the boundary |
| `R-UPDATE` | Staged update applies N−1 → N | The 2026-09-24 event happened **during a Sparkle install** (the test account's browser quit to update). The quit path is what this phase edits on macOS. `R-UPDATE`'s real N−1 → N run on macOS at RC includes a second account |
| Wallet-graceful-exit (`../../../DevOps-CICD/WALLET_GRACEFUL_EXIT_SPEC.md`) | Our own wallet still gets `/shutdown` and flushes the WAL | Over-tightening the macOS guard would leave **our own** wallet running after quit (the Windows 2026-09-01 "no wallet" shape, inverted). `P4-A1` two-sided |
| Dev/prod isolation (root `CLAUDE.md`) | Dev builds refuse to run without `HODOS_DEV=1`; dev and prod use different ports | The ownership check must key on the **resolved** port (`hodos::WalletPort()`), never a literal |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. `P4-A2` and `P4-A5` decide whether one account's browser can operate — and therefore spend from — another account's wallet: money rows, RED by a second agent (`../../../RELEASE_CYCLE.md` §4.2). The others are availability/UX rows with self-designed REDs.

**Rig.** One machine, two standard OS accounts A and B, Fast User Switching (Windows: `MultipleSessionEnabled`; macOS: default). Each account has its **own** installed or dev build and its own wallet (different recovery phrases, so the identity key tells them apart). ⚠️ Dev builds in two accounts share the dev port `31401`; release builds share `31301` — test each pairing that users can reach (release/release is the user case).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P4-A0` | **Step 0, today's build, both platforms:** A running; B launches Hodos ⇒ record (i) whether B spawned its own `hodos-wallet`, (ii) which identity key B's wallet UI shows, (iii) on B quitting, whether A's wallet pid survives | This row **is** the RED baseline for `P4-A1`/`A2` (expected: B adopts A's wallet; B's UI shows **A's** identity key; macOS: A's wallet dies when B quits — ticket §1; Windows: A's survives — code reading) | `ps -o user,pid,command` / `Get-CimInstance Win32_Process` (owner + `ExecutablePath`, path-matched), and the **identity key shown in B's wallet panel** — ⛔ "a wallet answered" re-creates the bug (ticket SUBJECT) | T2 👤 | ⬜ |
| `P4-A1` | **macOS:** B quits ⇒ A's `hodos-wallet` and `hodos-adblock` pids alive, A's `/health` answers, A's dApps still see a wallet; **and** in a single-account run, quitting still stops **our own** wallet gracefully (WAL flushed, pid gone within the cap) | Remove the new spawned-pid guard ⇒ A's wallet dies the second B quits (the 2026-09-24 event); remove the `/shutdown` send entirely ⇒ our own wallet outlives the browser (proves the two-sided half can fail) | A's pids by owner (`ps -o user`), A's `curl /health`; our own exit: the wallet log's graceful-shutdown line, not "the process is gone" (a SIGTERM also makes it gone) | T2 👤 (macOS session) | ⬜ |
| `P4-A2` | **Both:** with A running, B's shell **does not adopt** A's listener; B shows the K9 message; B's wallet UI shows **no** wallet (or B's own, per §12 Q1) — never A's identity key | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | B's UI (identity key or the message), B's shell log line naming the refusal and the reason (`owner mismatch` / `proof failed` / `proof unreachable`), and A's wallet log showing **no** request from B other than the proof | T2 👤 | ⬜ |
| `P4-A3` | The ownership proof **fails closed**: a listener that does not answer the proof, answers wrongly, or cannot be queried (access denied, timeout) ⇒ treated as **foreign**, never as ours | Force the proof query to return an error in a dev build ⇒ today's shape would adopt; the fixed shape must refuse. Also: a non-Hodos listener on the port (`python -m http.server <port>`) ⇒ refused, not adopted | The shell log's refusal reason per case — each of the three failure causes seen once. ⚠️ Rule 7 trip-wire 2: an **error must never become the verdict "ours"** | T1 (unit over the proof decision) + T2 | ⬜ |
| `P4-A4` | Same account: profile 2 adopts profile 1's wallet (owner policy); a dev wallet started by `dev-wallet.ps1` is adopted by the dev browser | Break the proof so it always says "foreign" ⇒ profile 2 shows the other-account message — proves this row can see over-refusal | The identity key in profile 2's wallet panel = profile 1's; the dev browser log `Wallet server already running … adopting (owner verified)` | T2 | ⬜ |
| `P4-A5` | The supervisor never adopts or relaunches onto a port held by another account: with A running, B's supervisor logs the foreign owner and does not spin its relaunch attempts against A's listener | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | B's shell log over ≥ 3 supervisor periods; A's wallet pid unchanged | T2 | ⬜ |
| `P4-A6` | Residual documented: the ticket's §3 (headerless ⇒ trusted for any local process) is recorded in the user-facing security notes and in the beta.7 per-user-channel ticket as the remaining half; no claim anywhere that P4 closes it | Grep the release notes / security page draft for a claim like "other accounts cannot reach your wallet" ⇒ must be 0 | The two docs' text | T0 | ⬜ |
| `P4-A7` | Boundary: `R-INTEXT` (a), wallet graceful exit on both platforms, minimal site basket, `R-UPDATE` T1 | Per `REGRESSION_SET.md` | Per `REGRESSION_SET.md` | T1–T2 | ⬜ |

**Two-sided rows:** `P4-A1` (spare A's wallet ⇄ still stop our own). `P4-A2` (refuse the foreign listener) ⇄ `P4-A4` (adopt our own) — a proof that always refuses passes A2 and fails A4.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `cef-native/cef_browser_shell.cpp :: LaunchWalletProcess` | ✅ | `if (IsPortListening(hodos::WalletPort())) { LOG_INFO("Wallet server already running (dev mode) - skipping launch"); g_walletServerRunning = true; return; }` — not gated on `IsDevEnv()`, any listener |
| `cef-native/cef_browser_shell.cpp :: LaunchAdblockProcess` | ✅ | Same adopt branch on `IsPortListening(hodos::AdblockPort())` |
| `cef-native/cef_browser_shell.cpp :: StopWalletServer`, `StopAdblockServer` | ✅ | Graceful `/shutdown` only inside `if (g_walletServerProcess.hProcess)` — the guard macOS lacks |
| `cef-native/cef_browser_shell.cpp :: BackendSupervisorLoop`, `StartBackendSupervisor`, `StopBackendSupervisor` | ✅ | Not-launched ⇒ `walletDead = !IsPortListening(…)`; `walletOwned = g_walletProcessLaunched` — an adopted foreign wallet is "not owned" so it is not relaunched, but its death flips `g_walletServerRunning` and it is re-adopted if another listener appears |
| `cef-native/cef_browser_shell_mac.mm :: QuickHealthCheck`, `SpawnWalletServer`, `SpawnAdblockServer`, `StopServers`, `SendShutdownRequest` | ✅ | `SpawnWalletServer`: `if (QuickHealthCheck()) { … g_walletServerRunning = true; return; }`; `StopServers`: `if (g_walletServerRunning) SendShutdownRequest(hodos::WalletPort());` — the bug. Spawned pids are `g_wallet_server_pid` / `g_adblock_server_pid` (`posix_spawn`) — the guard keys on them |
| `rust-wallet/src/handlers.rs :: shutdown`, `health`; `rust-wallet/src/main.rs :: domain_trust_mw` (header-free ⇒ ungated) | ✅ | `/shutdown` has no check. If the proof is a per-user secret echoed on `/health` (§12 Q2 option b), `health` gains a nonce parameter — Rust change, both binaries |
| `adblock-engine/src/main.rs` route `/health` → `adblock-engine/src/handlers.rs :: health` | ✅ | Needs the same proof if option (b) |
| `cef-native/include/core/PortConfig.h :: WalletPort`, `AdblockPort` | ✅ | Resolved port; never a literal |

**New OS API (working rule 4 — docs read and cited in the kickoff before the diff is accepted), only if §12 Q2 picks option (a):** Windows `GetExtendedTcpTable(TCP_TABLE_OWNER_PID_LISTENER)` → pid → `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` → `OpenProcessToken` → `GetTokenInformation(TokenUser)` → `EqualSid`; macOS `proc_listpids` / `proc_pidfdinfo(PROC_PIDFDSOCKETINFO)` or `LOCAL_PEERCRED` (Unix sockets only). ⚠️ Cross-user process queries may be **denied** for a standard user — that must read as "foreign" (`P4-A3`).

## 6. Out of scope

- The per-user channel (named pipe ACL'd to the SID / Unix socket in the user's own Application Support) and making "headerless ⇒ trusted" hold only for callers that prove they are us — **beta.7** (T5 Q12).
- Per-profile wallets (owner: keep one wallet per account for now).
- Same-account malware.
- The dApp-facing HTTP surface — B5-T5-P2.
- Auto-unlock accepting another wallet's phrase — B5-T1-P2.
- Fast relaunch attaching to a dying wallet — B5-T6 (`TICKET_fast_relaunch_attaches_to_dying_wallet_or_fails_port_bind.md`); ⚠️ same adopt branch — serialize (§11).

## 7. Rollback

Two commits: (1) macOS `StopServers` guard (one condition per backend), (2) the ownership proof at both adopt sites + supervisor + the message. Each reverts alone; no schema, no data.

## 8. Pre-mortem (adversarial review — before)

| Story: it shipped and failed because… | Row that catches it |
|---|---|
| The macOS guard keyed on `g_walletServerRunning` alone again, so our own wallet was left running after quit (inverse bug) or the foreign one still got `/shutdown` | `P4-A1` two-sided |
| The ownership query was denied for another user's process and the code treated "couldn't tell" as "ours" — adopts exactly as before, test green on a single-account rig | `P4-A3` (rule 7 trip-wire 2) + `P4-A2` on a real two-account rig |
| The per-user-secret file lived in a shared or dev-shared directory, so both accounts read the same secret | `P4-A2` with two real accounts (not two profiles); SUBJECT = identity key |
| The test used two **profiles** in one account and called it cross-account | `P4-A0`/`A2` rig definition: two OS accounts, `ps -o user` shows two users |
| The dev workflow broke — the developer's separately started wallet is now refused | `P4-A4` dev half |
| B, refused, retried the proof every supervisor period and spammed A's wallet log | `P4-A5` |
| The fix shipped on Windows and the macOS half was assumed from reading | §9: macOS rows are the macOS session's, run on a Mac |
| The adopt branch was edited by T6's fast-relaunch fix at the same time and one change undid the other | §11 serialize |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | `P4-A0`, `A2`–`A7` | Fast User Switching; `StopWalletServer` already guarded — expected RED for A1's shape does **not** exist here (code reading; `P4-A0` confirms) |
| macOS | `P4-A0`–`A7` (all) | `StopServers` is macOS-only code (`cef_browser_shell_mac.mm`) — **the macOS session owns `P4-A1`**. ⚠️ Shared C++ touched on both sides ⇒ **relay round** naming `cef_browser_shell.cpp`, `cef_browser_shell_mac.mm` (flagged as a `*_mac.*` file if edited from Windows) and any Rust `/health` change |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~2.5 h**: Windows two-account rig set-up + `P4-A0`/`A2`/`A4` (~1.25 h); macOS two-account run with the Mac session (~1 h, the owner's second account); reading K9 wording (~15 min) |
| Human-bound rows | `P4-A0`, `P4-A1`, `P4-A2` (two OS accounts, switching between them — the agent environment cannot drive a second interactive session) |
| Unknowns (K) — uncertainty, not difficulty | **K8** which ownership proof is reliable on each OS (socket owner lookup vs per-user secret) · **K9** what a second account should see (refusal vs its own wallet on another port) · **K22 (new)** whether a standard user can query another session's process owner at all on Windows 11 / macOS 26 without elevation (decides option (a)'s viability) |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| serialize | **B5-T6** — fast-relaunch ticket (`TICKET_fast_relaunch_attaches_to_dying_wallet_or_fails_port_bind.md`) | Same adopt branch in `LaunchWalletProcess` / `SpawnWalletServer`. One edits, the other rebases and re-runs `P4-A4` |
| gives | **B5-T5-P2** | P2's surface is the browser half; this phase's residual (`P4-A6`) is the non-browser half |
| needs | macOS session | `P4-A1` is macOS code; the relay round carries this contract |
| gives | beta.7 intake | Per-user channel ticket carries `P4-A6`'s residual + prior art to read (Docker Desktop, 1Password agent, VS Code CLI↔server, MetaNet Client on `3321` under two accounts) |

## 12. Open questions for the owner

1. **K9 — what does account B see?** (a) a clear refusal: *"Hodos is already running in another account on this computer. Your wallet will be available when that account quits Hodos."* — no wallet in B until then; (b) B runs its **own** wallet on a different port. (b) needs a per-user port, which `PortConfig.h` does not have and every internal caller assumes — a larger change and the natural home of the deferred per-user channel. Recommendation: **(a) this release.** Which?
2. **K8 — the proof.** (a) Ask the OS who owns the listening socket (new Win32/Darwin API, may be denied cross-user — K22); (b) a per-user secret: the wallet writes a random token into the user's own data directory at start, and `/health?nonce=<n>` answers `HMAC(token, n)`; the shell reads the same file and compares — no new OS API, same on both platforms, and it is the prior-art shape (Jupyter's token). Recommendation: **(b)**, because it fails closed by construction and needs no elevation. ⚠️ It adds a Rust change to both backends' `/health`. Which?
3. No evidence that a G2 decision is wrong.

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
