# Knowledge & memory — where facts live, and how both machines keep them in step

**Created:** 2026-09-25, 👤 owner decision during beta.5 planning. **Version: v1.**
**Applies to:** every agent on every machine (🪟 Windows, 🍎 macOS) and every human on the project.
**Supersedes:** the research ticket `0.4.0-beta.6/tickets/TICKET_knowledge_and_memory_architecture.md`
(its questions are answered or carried as open items in §6).

> ⭐ **The one rule.** *If another person or agent would ever need it, it goes in the repo.* An agent's
> private memory holds only how that agent works with the owner, and traps specific to its machine.

---

## 1. Why this exists

Each agent has a **private memory** on its own machine (`~/.claude/…/memory/`). It is not versioned,
the other machine's agent **cannot see it**, and the owner effectively cannot either. The macOS relay
exists precisely because memory does not cross machines — yet project facts kept landing in private
memory, where only one agent could use them. 📏 On 2026-09-08 the Windows `MEMORY.md` hit **24.2 KB
against a 24.4 KB read limit** and had to be compacted mid-session; much of it was project knowledge.

## 2. The stores — what each one is for

| Store | Versioned | Both machines see it | Holds | Does NOT hold |
|---|---|---|---|---|
| **Root `CLAUDE.md`** | ✅ | ✅ | Shape, contracts, invariants, working rules, pointers | Inventories, phase status |
| **Per-directory `CLAUDE.md`** | ✅ | ✅ | Inventory and durable code facts for that layer (rosters, gotchas, "why this code is shaped this way") | Release plans |
| **`development-docs/RELEASE_CYCLE.md`** | ✅ | ✅ | The process and its gates | Release-specific content |
| **This file** | ✅ | ✅ | Where knowledge goes and how it is shared | — |
| **Release folder** (`development-docs/0.4.0-beta.N/`) | ✅ | ✅ | Tickets, phase contracts, evidence, decisions (`README.md` planning notes), lessons (`AAR_NOTES.md`), the relay | Permanent rules — promote those to a `CLAUDE.md` |
| **`development-docs/PRIOR_ART.md`** | ✅ | ✅ | What we looked at outside the project, and whether it was worth it | — |
| **Relay** (`MAC_RELAY_<release>.md`) | ✅ | ✅ | **Messages between machines**, one round at a time: measurements, asks, "this doc changed" | The facts themselves — a round points at where the fact now lives |
| **GitHub issues** | on GitHub | ✅ | **Tracking only**, one per phase (`RELEASE_CYCLE.md` §4.1a) | Evidence or detail — the markdown is the truth |
| **Agent private memory** | ❌ | ⛔ **no** | How to work with the owner (preferences, corrections); traps specific to *this* machine's environment | Anything the other agent or a human would need |

## 3. The five-second test — where does a new fact go?

1. **Would the other machine's agent, a future developer, or the owner need it?** → the repo. Then:
   - a rule that should always hold → **root `CLAUDE.md`**
   - a fact about specific code → that directory's **`CLAUDE.md`**
   - a decision, finding or lesson from this release → the **release folder** (`README.md` planning
     notes, a ticket, a phase contract, `AAR_NOTES.md`)
   - the other machine must *act* on it → **also** a relay round pointing at it (§4)
2. **Only about how I work with the owner, or a trap on this machine?** → private memory.
3. **Unsure?** → the repo. A fact in the repo that turns out private costs nothing; a project fact in
   private memory is invisible to half the team.

⛔ **Never delete on migration.** Memory entries often carry the *why* as well as the *what* — move
both.

## 4. Keeping both machines in step — the sharing protocol

Shared documents change on both machines. Git carries the text; **the relay carries the attention.**

| When | Do |
|---|---|
| You change a **shared rule or process doc** (any `CLAUDE.md`, `RELEASE_CYCLE.md`, this file, a harness doc) in a way the other side must act on | Add a relay round naming **the file, what changed, and what to do**. ⛔ A silent edit to a shared rule is the failure this protocol exists to prevent |
| You receive such a round | Pull, read the named change, **adopt it**, and answer in your next round: adopted / adopted with a platform difference (say which) / **proposed improvement** |
| You learn something the other side needs | Write it into the repo first (§3), then send a round that points at it. The round is the notification, not the storage |
| You find your private memory holds project knowledge | Move it into the repo (§3) and leave a one-line pointer in memory |

Relay conventions are `RELEASE_CYCLE.md` §3.5: newest round first, platform-prefixed ids
(`W-25a`, `M-25a`), measurements not conclusions, **"do NOT inherit this"** for platform-specific
results, one driver queuing work for the owner at a time.

## 5. Reviewing and improving this document

⭐ **Both machines own it.** Each agent reviews it against how it actually works and proposes changes
through the relay — what works well, what is missing, what is wrong for its platform. The owner
approves changes; each change bumps the version and gets a line in §7, like `RELEASE_CYCLE.md`.

## 6. Open items *(carried from the research ticket)*

| # | Item | Owner / when |
|---|---|---|
| 1 | **Migrate project knowledge out of each machine's private memory** into the repo, per §3. Windows first (its `MEMORY.md` is the larger); macOS reviews its own | Each agent, during beta.5 — not in one big bang; migrate as entries are touched |
| 2 | **What keeps docs current.** Root `CLAUDE.md` invariants #11/#12 are advisory with no gate. Candidate: a phase sign-off line *"every ticket this phase touched has a current status"* — 📏 on 2026-09-25, 17 of beta.3's 51 tickets read "open" but were fixed. ⚠️ Working rule 6: a gate is not authored in the change it measures | Proposal for the beta.5 AAR |
| 3 | **Ticket queryability** — the research ticket proposed a generated ticket index. The beta.5 `tickets/README.md` register, with its Track column, now does this by hand; generate it only if hand upkeep fails | Watch |
| 4 | GitHub issues — the ticket said *provisionally no*; **superseded** by the owner's 2026-09-25 decision (`RELEASE_CYCLE.md` v5 §4.1a: tracking layer, one per phase) | Decided |

## 7. Changelog

- **v1 — 2026-09-25.** Created from the owner's decision to adopt the knowledge-and-memory research
  ticket as a shared policy for both machines, announced to macOS in `0.4.0-beta.6/MAC_RELAY_BETA5.md`
  round `W-25a`.
