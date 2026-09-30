# Development Docs

This folder contains feature research, design exploration, and implementation guides. These are **working plans**, not commitments or a roadmap.

⛔ **The root holds only living, cross-release documents** — `README.md`, `RELEASE_CYCLE.md`,
`SCOPING_PROCESS.md`, `PRIOR_ART.md`, `MACOS_CATCHUP_PLAYBOOK.md`, `KNOWLEDGE_AND_MEMORY.md`. Release-specific work lives in a
release folder (`0.4.0-beta.7/` is current — planned as beta.5 until the 2026-09-27 rename; intake for the next one is `0.4.0-beta.8/`); finished work lives in `archived-docs/`.

## Where things moved (2026-09-25)

| Was | Now |
|---|---|
| `Final-MVP-Sprint/` | `archived-docs/Final-MVP-Sprint/` — backup lessons in `0.4.0-beta.7/track-3-backup-sync/BACKUP_HISTORY_OVERVIEW.md` |
| `Wallet-Hardening/` | `archived-docs/Wallet-Hardening/`, except three docs the backup plan cites → `0.4.0-beta.7/track-3-backup-sync/research/` |
| `Dolphin Milk + Edwin Integration/` | `Future-Features/Dolphin Milk + Edwin Integration/` |
| `0.4.0-beta.7/track-3-opns-naming/` | `Future-Features/Decentralized-Naming/OPNS_TRACK_SCOPE_beta5_deferred.md` — OpNS is out of beta.5 |
| 16 open tickets in `0.4.0-beta.3/` | `0.4.0-beta.7/tickets/` |

## Where things moved (2026-09-24)

| Was (root) | Now |
|---|---|
| `0.4.0-beta.4/` (the next-release plan) | **`0.4.0-beta.7/`** — the beta.4 version was spent by a hotfix; see that README |
| `X402_INTEGRATION.md`, `ONCHAIN_BACKUP_SYSTEM.md` | `0.4.0-beta.7/` — inputs to this cycle's tracks |
| six loose `TICKET_*.md` (BRC-121 remint, debug-log filtering, knowledge architecture, screenshots in history, profile-lock message, reservation ownership) | `0.4.0-beta.7/tickets/` |
| `TICKET_farbling_constant_seed_shipped.md` | `archived-docs/tickets/` — **closed with evidence** first |
| `QR_SCAN_*.md` ×3, `AUDIT_SCOPE.md`, `FUTURE_AUTO_APPROVE_ENGINE_ARCHITECTURE.md` | `archived-docs/` |
| `MACOS_SPRINT_HANDOVER_20260501.md` | `archived-docs/Mac-port/` — superseded by the playbook |
| `CEF_BINARIES_README.md` | `DevOps-CICD/` |

## Where things moved (2026-08-15)

The `Sigma-BRC121-Sprint` folder was retired. Phases 0–2.6 are done and the whole folder is archived
at **`archived-docs/Sigma-BRC121-Sprint/`** (name kept so old cross-references still resolve). Its
`README.md` carries a stale phase table — trust that folder's `CHECKLIST.md` and the git log instead.

Two pieces were split out and are live:

| Topic | Now lives at |
|---|---|
| 1Sat Ordinals + BSV21 | **`development-docs/0.4.0-beta.7/track-2-1sat-ordinals/`** — now beta.5, Track 2 (moved 2026-08-29 as "beta.4 sprint 2"; renamed 2026-09-24). Read `BSV-Tokens/` first; the trust ratings are in that README. |
| Demo example sites | **`demos/README.md`** (repo root) — runnable code, not docs |
| Demo *videos* | **`Marston Enterprises/Hodos/Marketing/Videos/README.md`** — outside this repo. No video files in this repo, ever. |

Phase 1.6 (indexer resilience) is tracked in a separate sprint; the copy in the archive is superseded.

## Purpose

Documents here capture thinking at a point in time. They may be:
- Early research into a problem space
- Design options being weighed
- Implementation guides ready for coding
- Archived plans that were superseded or abandoned

Research, design, and planning are iterative. A document may move back and forth between phases as understanding evolves. Implementation is the phase where code, testing, debugging, and optimization occur—it begins only when a plan is ready.

## Status Convention

Each document should have a status block at the top:

```
Status: Research / Exploration
```

| Status | Meaning |
|--------|---------|
| **Research / Exploration** | Gathering information, understanding the problem |
| **Design / Planning** | Defining approach, weighing trade-offs |
| **Ready for Implementation** | Plan is complete and consistent with project architecture |
| **Implemented** | Code exists; document is reference material |
| **Archived / Superseded** | No longer current; kept for historical context |

Status reflects current thinking, not priority or guarantees.

## Before Implementation

Before moving a document to "Ready for Implementation," sanity-check the plan against:
- `CLAUDE.md` — invariants and safety rules
- `PROJECT_OVERVIEW.md` — architecture and security model
- `THE_WHY.md` — design philosophy and trade-offs

If the plan conflicts with these, revise the plan or surface the conflict for discussion.

## How Claude Should Use These Docs

- Treat documents as context and guidance, not mandates
- Do not assume everything described must be implemented
- Prefer the most recent document marked "Ready for Implementation"
- If a plan conflicts with current architecture or invariants, surface the conflict before writing code
- When implementing, follow the plan's approach but adapt to what the code actually requires
