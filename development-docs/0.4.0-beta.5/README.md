# beta.5 release — scope and running order

**Opened:** 2026-08-29, from the beta.4 kickoff prompt (the telescope pass; archived to `../../archived-docs/0.4.0-beta.4-planning/`).
**Status:** 🔭 TELESCOPED. Track-level scope is set; **no phase-level design exists yet** and none
should be written here. The microscope pass produces that — see `TELESCOPE.md`.

> **Naming.** This folder produces **`v0.4.0-beta.5`**. It sits beside `0.4.0-beta.3/`, whose cycle
> is closed (AAR written) but whose relay stays the live macOS channel until this release opens its
> own at gate `G5`. ⛔ **Do not edit anything under `0.4.0-beta.3/`.**

> ⚠️ **Why this folder was called beta.4 until 2026-09-24.** It was opened as the beta.4 plan. The
> **beta.4 version number was then spent** by a hotfix: `v0.4.0-beta.4` (promoted 2026-09-24) is the
> beta.3 cycle's code plus two logging fixes, and it superseded the unpublished `v0.4.0-beta.3`.
> ⇒ **The beta.5 release follows the beta.4 release, which was a hotfix. No work in this folder was
> ever in a shipped beta.4.** The folder and its forward-looking references were renamed on
> 2026-09-24 and merged with the two beta.5 files that already existed. Dated provenance lines
> (*"created at the beta.4 telescope pass"*) and mentions of the **shipped** `v0.4.0-beta.4` keep
> the old name on purpose. The two beta.4-era planning files (`RESUME_beta4.md`,
> `SESSION_PROMPT_beta4_kickoff.md`) were folded into this README and archived to
> `../../archived-docs/0.4.0-beta.4-planning/` on 2026-09-25.

## 📝 Planning notes — owner conversation, 2026-09-24 → 25 *(pre-cycle; read before G1)*

⚠️ These are **decisions and directions from the owner, recorded before the cycle's planning gates
run**. Where they conflict with older text further down this file (the track table, "why this
order"), **these win** until the cleanup below rewrites that text.

### Decided

| # | Decision | Detail |
|---|---|---|
| D1 | **Engine: stay on CEF 150** (branch 7871) | Refresh in-branch to the newest 7871 build (upstream is at `chromium-150.0.7871.255`; we ship `.187`) and add the two queued engine fixes (ad-blocker payload pull, `"Hodos"` in `Sec-CH-UA`). Reason: 7871 is CEF's **long-term** branch, patched until ~Apr 2027. The 154 branch (8037) is a normal branch that CEF stops patching ~Oct 5, 2026, so a 154 build would need another bump within weeks to stay patched. See `tickets/TICKET_engine_behind_its_own_cef_branch_and_upstream_stable.md` (option A) |
| D2 | **Engine policy: build on CEF long-term branches only.** | We are too small a team to bump every four weeks. An off-cycle bump needs a reason (a critical security issue, or a platform feature we actually need). Next long-term branch named by CEF: **160**, expected around spring 2027 *(estimate from ~4 weeks per milestone, not a published date)* |
| D3 | **On-chain backup & sync is fixed scope — not cuttable** | The money path goes first, but backup ships in this release |
| D4 | **OpNS is out of this release** | Its track folder moves to `Future-Features/Decentralized-Naming/`. A different naming system may be researched later; it is R&D too large for this cycle |
| D5 | **Every ticket lands in a track** | Each becomes a phase, sub-phase or item inside a track, or is closed (fixed / duplicate / stale) or deferred with a re-check condition. The groupings proposed at G0 become tracks: **identity & privacy** is a real track (incl. BRC-103/104 server identity verification and `/.well-known/auth`); **browser shell** is a candidate track; **instruments & hygiene** are background items, not a track |

### ⭐ Research step one, for every track — the ecosystem is current, our copies are not

- **Re-fetch the current SDKs, libraries and specs before any design. Never trust our copies** — every
  doc in this repo predates recent ecosystem changes.
- **1Sat Ordinals:** the 1Sat TypeScript SDK was updated days before 2026-09-25. `../BSV-Tokens/` is
  kept as background with a pointer from the ordinals track, but **nothing is built on its research
  without re-checking** it against current sources.
- **Backup & sync:** review the BRCs on **wallet database format and export/import** (updated around
  2026-09-18). Goal: **interoperability** — our storage follows the standard format so users can move
  into or out of Hodos cleanly, and sync rests on a format others recognise.
- 👤 **Compatibility is a top product priority.** We build inside a set of standard protocols (BRCs)
  and alongside other apps and wallets we must work with.

### Backup track — its history is consolidated ✅

⭐ **Read `track-4-onchain-backup-sync/BACKUP_HISTORY_OVERVIEW.md` first** (2026-09-25). One agent
mapped every backup doc — the 2026-04-11 double-spend incident, the efficiency plan, the July backup
review, `ONCHAIN_BACKUP_SYSTEM.md`, the track's plan and research — against today's code, and wrote a
single overview: what ships, what was fixed, what contradicts what, and what the research must answer.
👤 **The bar the whole system must clear: stable, no conflicts, efficient.** If we cannot do both
conflict-free and efficient, the design is not ready. ⚠️ The overview's own verdict: **not shown
ready yet** — conflict-free for the backup chain on paper, not for two devices spending the same coin,
and delta sizes have never been measured.

### Cleanup — ✅ done 2026-09-25

| Item | Result |
|---|---|
| `RESUME_beta4.md`, `SESSION_PROMPT_beta4_kickoff.md` | Live content folded into this README (**Research questions owed**); both archived to `../../archived-docs/0.4.0-beta.4-planning/` |
| Open tickets in `../0.4.0-beta.3/` | File-by-file review of all 51: **16 open → moved to `tickets/`**; **17 that read "open" but were fixed → stamped closed with evidence**, left in beta.3. See `tickets/README.md` |
| `Final-MVP-Sprint/` | Backup inputs captured in the overview; five unscheduled leftovers → one ticket; folder archived to `../../archived-docs/Final-MVP-Sprint/`; `CLAUDE.md` pointers updated |
| `Dolphin Milk + Edwin Integration/` | Moved to `../Future-Features/` |
| `Wallet-Hardening/` | Reviewed: 7 open items → 7 tickets; the 3 docs the backup plan still cites → `track-4-onchain-backup-sync/research/`; the rest archived to `../../archived-docs/Wallet-Hardening/` |
| `../BSV-Tokens/` | Kept; the ordinals track now opens with *re-fetch first, BSV-Tokens is background* |
| `track-3-opns-naming/` | Moved to `../Future-Features/Decentralized-Naming/OPNS_TRACK_SCOPE_beta5_deferred.md` (D4) |
| Every move | Pointer sweep across the repo, including every `CLAUDE.md`, code comments and CI files |

**Next:** triage every ticket in `tickets/` into tracks (`../RELEASE_CYCLE.md` §1 step 9, gate G2), after the mission is signed off at G1.

### ✅ Process change — approved 2026-09-25

**Ecosystem currency** is now part of orientation for every cycle: `../RELEASE_CYCLE.md` **v4**, §1
step 2 and §3.1a.

## The naming convention — settled 2026-09-18, applies from beta.4 forward

👤 Owner decision. Four rungs, all named from the field, and **"sprint" is retired as a scope word**
— in Scrum, SAFe and Shape Up it names a *timebox*, never a chunk of work, which is what made
"the beta.4 sprint" and "sprint 2" collide.

| Rung | Name | What it is | Artifact | Example here |
|---|---|---|---|---|
| 1 | **Release** | the version we ship, 1–3 months | `README.md`, `RELEASE_PLAN.md` | `v0.4.0-beta.5` |
| 2 | **Track** | one of the 4–5 big parallel efforts — a feature, or a bug bundle | folder + scope doc | Track 2 — 1Sat Ordinals |
| 3 | **Phase** | a slice of a track finished and **verified on its own** | **phase contract** + evidence table | *(none yet — microscope produces them)* |
| 3.5 | **Sub-phase** | ⭐ **optional, used only when needed.** A phase that grew too big to verify in one contract splits into sub-phases, each with its own | its own phase contract | — |
| 4 | **Item** | one numbered defect or row inside a phase | a row in the evidence table | `A1` |

⭐ **Sub-phase is a pressure valve, not a required rung.** Most phases go straight to items. Reach
for it when a phase has too many large items to carry one honest evidence table — and when you do,
say so in the phase contract rather than letting it happen silently.

⛔ **Never write a bare rung number.** Every reference starts at the release: *"beta.5, Track 2
(1Sat Ordinals), Phase 3"* in prose, **`B5-T2-P3`** as a short id — `B5-T2-P3.1` for a sub-phase. A
sentence that opens with "track 2" does not say which release, and beta.3 is live beside this one.

## Statuses, which are not rungs

**Candidate phase** — a proposal, during a pass, for something that *may* become a phase. It means
exactly what the words mean: we are looking into making it one. It becomes a **Phase** when the
microscope pass confirms it, and the heading over it changes from *Candidate phases* to *Phases*.
⛔ It is not a name for a rung, and nothing is a candidate once it has a contract.

⭐ **"Phase" means rung 3 and nothing else.** The planning stages are **passes** — the *telescope
pass* sets the tracks, the *microscope pass* turns one track into phases. ⛔ Do not write "scoping
phase"; write "telescope pass" or "scoping".

⚠️ **beta.3 is not renamed and will not be.** It ships in days and its vocabulary is internally
consistent; it simply calls rung 3 a phase and has no name for rung 2. Read `0.4.0-beta.3/` as
written. This convention starts here.

---

## What beta.5 is

beta.3 is browser-shell work — overlays, DPI, window identity, logging, the trust boundary.
**beta.5 is the wallet's asset layer.** Four tracks, in a fixed order, that take the wallet from
"cannot tell a token from a coin" to "holds, spends and recovers tokens deliberately".

| # | Track | Folder | One-line goal |
|---|---|---|---|
| **0** | **`reqwest` 0.11 → 0.12+** — the wallet's TLS certificate validator | `track-0-reqwest-tls-bump/` | Every outbound HTTPS call validates the server with a library that has no open advisories, without changing what the wallet sends or signs. 👤 **Added 2026-09-15 by owner decision** from the beta.3 Phase 9 dependency review; sits *ahead of* the settled 1–4 order because it is a money-path dependency change, not an asset-layer feature |
| **0.5** | **UTXO reservation ownership** — `tickets/TICKET_reservation_ownership_converge_on_spent_by.md` | *(folder at its microscope pass)* | Reservations are owned by a transaction row (`spent_by`), not a placeholder string, converging on wallet-toolbox. 👤 Placed here 2026-09-15: the ticket forbids doing it in beta.3 (it reorders `create_action_internal`), and track 1's classification seam touches the same `output_repo` exclusion logic, so it should land **before** the guard, not under it. Owner to confirm at the beta.4 kickoff |
| 1 | **UTXO safety guard** | `track-1-utxo-safety-guard/` | No path — automatic or manual — can spend an output the wallet has not classified as spendable |
| 2 | **1Sat Ordinals** | `track-2-1sat-ordinals/` | Hold, display, receive and deliberately transfer 1Sat ordinals to BRC-147 + BRC-150 |
| 3 | **OpNS unique names** | `../Future-Features/Decentralized-Naming/OPNS_TRACK_SCOPE_beta5_deferred.md` (⛔ **out of this release** — planning note D4) | Resolve and register OpNS names against BRC-174, with a live overlay proof-of-concept |
| 4 | **On-chain backup & sync** | `track-4-onchain-backup-sync/` | Delta-chain backup measured against real token workloads, multi-device sync |
| — | Tickets | `tickets/` | Reviewed and assigned into tracks by the owner, not worked ad hoc |

## Why this order — settled, do not relitigate

**Guard → 1Sat → OpNS → Backup.** Each one is a prerequisite for the next, not a preference:

1. **Guard is first because the hazard is already live.** Users can receive a 1-sat output at a Hodos
   address today, and three code paths treat it as spendable value — one of them
   (`monitor/task_consolidate_dust.rs`) runs automatically every 24 hours with no user action. Every
   later track *creates* the assets those paths destroy. Shipping track 2 before track 1 means
   shipping a feature and its own destroyer in the same release. **This track is needed even if we
   never ship ordinals.**
2. **1Sat before OpNS** because an OpNS name is carried by a 1-sat output. Names inherit ordinal
   handling; building names first means building ordinal handling badly, twice.
3. **OpNS before Backup** because names are the second real token workload, and the backup track's
   central open question is measured against real rows.
4. **Backup last** because BRC-150 provenance rows (`beefB64`) *are* the token-heavy workload that
   deltas exist to solve. Sequencing it last means measuring a workload that exists rather than
   designing for one that does not.

## Decisions taken at kickoff (owner, 2026-08-29)

| # | Question | Decision |
|---|---|---|
| 1 | Minimal defensive floor in beta.3? | ✅ **Yes — ship now.** `satoshis > 1` floor in the dust consolidator, recovery sweep and the coin-selection dust pass, landing in beta.3 under its existing ticket. Do **not** wait on the exposure question; it changes urgency, not correctness. The full classification guard stays track 1. |
| 2 | Harness: inherit or fork? | ✅ **Reference beta.3, extend by delta.** `../0.4.0-beta.3/HARNESS.md` and `REGRESSION_SET.md` are the standard. beta.5's additions live in `HARNESS_DELTA.md` and `REGRESSION_ADDITIONS.md` only. Fold both into a version-neutral `development-docs/HARNESS.md` **after** beta.3 closes. |
| 3 | Guard reach in track 1? | ✅ **General seam, one classifier.** Build the classification point with a general shape (`Spendable` / `Token` / `Unknown`, fail closed on `Unknown`); implement only the 1-sat/inscription classifier. BSV-20/21 slots in later as a classifier, not as a rewrite of the call sites. |

## The harness — inherited, not copied

⛔ **The standard is `../0.4.0-beta.3/HARNESS.md`.** Read it before writing a phase contract. It is
not restated here and it has not been forked. The reason for referencing rather than copying: its §9
gate baseline registry is *measured state*, and beta.3 is still lowering baselines. Two copies of a
measurement diverge silently.

| File | Where it lives |
|---|---|
| The standard — phase contracts, four-column evidence table, tiers, ratchets, adversarial posture | `../0.4.0-beta.3/HARNESS.md` |
| The standing regression set — R-INTEXT, R-GOLD, R-CLOSE, R-PERIM, R-COUNT, R-UPDATE | `../0.4.0-beta.3/REGRESSION_SET.md` |
| Phase contract template | `../0.4.0-beta.3/PHASE_CONTRACT_TEMPLATE.md` |
| **What beta.5 adds** — new tiers, new gates, the token-specific evidence rules | **`HARNESS_DELTA.md`** |
| **What beta.5 adds to the standing set** — new invariants that must survive every later phase | **`REGRESSION_ADDITIONS.md`** |

## The one rule carried forward from beta.3

> ⛔ **The claim follows the matrix, never the other way round.** A green result is reported with its
> red half or not at all. A row with an empty RED or SUBJECT cell is not done.

And its beta.5 restatement, because this release handles assets that cannot be recovered:

> ⛔ **Fail closed.** An output the wallet cannot classify is not spendable. "We could not tell, so we
> spent it" is the failure mode that destroys a user's asset permanently, and it has no undo.

## Folder mechanics — done 2026-08-29

Both prior standalone track folders were **moved into this folder**, not copied — originals no
longer exist, and BSV-21 is out of the ordinals folder name per §2 of the kickoff:

| Was | Now |
|---|---|
| `development-docs/1SatOrdinals-BSV21/` | `0.4.0-beta.5/track-2-1sat-ordinals/` |
| `development-docs/Onchain-Backup-and-Sync/` | `0.4.0-beta.5/track-4-onchain-backup-sync/` |

Cross-references rewritten in `development-docs/README.md`, the moved
`TRACK_KICKOFF_PROMPT.md`, and four `research/*.md` files. Verified: **zero stale references remain
outside `0.4.0-beta.3/`.**

⚠️ **Six files under `0.4.0-beta.3/` still carry the old paths** — five `SESSION_PROMPT_beta3_*.md`
and `TICKET_token_outputs_destroyed_by_dust_paths.md`. They were **deliberately not edited**, because
another session owns that folder. Five of the six are historical session prompts and are archaeology.
The ticket is live and its "Links" section points at the old ordinals path — **that one is owed**, and
belongs to whoever next touches beta.3.

## What each track folder contains right now

| Folder | State |
|---|---|
| `track-1-utxo-safety-guard/` | ⬜ **New. Scope only** (`README.md`). No prior research existed; track 1 was created at this kickoff. |
| `track-2-1sat-ordinals/` | 🟡 Carried in: `README.md` (scope + the 2026-08-05 BRC-147/150 decision) and `RESEARCH_FINDINGS.md` (protocol mechanics, provider APIs, indexer infrastructure). Both predate the guard work — read against `TELESCOPE.md` before trusting scope claims. |
| `../Future-Features/Decentralized-Naming/OPNS_TRACK_SCOPE_beta5_deferred.md` (⛔ **out of this release** — planning note D4) | ⬜ **New. Scope only.** ⛔ A new naming track doc is **required** — see below. |
| `track-4-onchain-backup-sync/` | 🟢 Carried in and **authoritative**: `IMPLEMENTATION_PLAN.md` (8 phases, D1–D15 decisions, two adversarial reviews, owner sign-offs), `README.md`, `ADVERSARIAL_REVIEW.md`, `research/`. ⛔ **Not to be redesigned.** |

⛔ **`Future-Features/Decentralized-Naming/`** (README, `OPNS_REVIEW.md`, `OPNS_RESOLVER_SCOPE.md`,
Xanaverse review) **predates BRC-174 and will be archived.** It stays where it is, is read **once**
during the track-3 outline pass, and is **not referenced after that**. Track 3 gets a new doc built
on the merged BRC, not on the old research.

## Ecosystem position — as of 2026-08-29

| | |
|---|---|
| **BRC-174** (ours, OpNS) | **MERGED** 2026-08-28 as `tokens/0174.md`, zero review comments. The development base for track 3. ⚠️ Merging is publication, **not endorsement** — §4 and §10.1 are unimplemented by anyone. |
| **Collectables** — BRC-147, 150, 159, 160, 165 | All merged and mutually coherent. **Safe to build on.** This is track 2's foundation. |
| **Fungibles** — BRC-163 vs BRC-175 | 🔴 **Contested. Do not build on.** See `WATCH_fungibles.md` for the state and the explicit re-check gate. |
| **BSV-21 encodings** | Two exist: BRC-161 (JSON) and BRC-162 (binary/CBOR). Relevant only if the fungibles gate ever opens. |

## Verified code findings carried in — do not re-derive

Read by direct inspection of `rust-wallet` on 2026-08-29. Full detail in
`0.4.0-beta.3/TICKET_token_outputs_destroyed_by_dust_paths.md` (read-only) and in
`track-1-utxo-safety-guard/README.md`.

**Nothing files a 1-sat output into a protective basket, and three paths treat it as spendable:**

| # | Path | Trigger | Effect |
|---|---|---|---|
| 1 | `monitor/task_consolidate_dust.rs` | **Automatic, every 86,400 s** (`monitor/mod.rs:79`) | Selects `satoshis <= 1000` from the default basket, fires at 20 accumulated, packs into one output. **Destroys ordinals with no user action and no prompt.** |
| 2 | `recovery.rs` | User-triggered — but it is the **restore** flow | External scan sums every UTXO with no value filter; `build_sweep_transactions` batches into one P2PKH output. Only guard is a dust check on the *output*. |
| 3 | `handlers.rs:7257` `select_utxos_with_preference` | Ordinary coin selection | `dust_threshold_sats: 5000`, and line 7330 *deliberately includes* UTXOs at or under it. |

⭐ **The exclusion logic already exists and works.** `database/output_repo.rs:98,148` filter payment
selection to `basket_id IS NULL OR b.name = 'default'`, so a correctly-basketed output is already
safe. **The gap is classification on ingest, not exclusion.** That is why decision 3 above builds a
classification seam rather than adding value checks at five sites.

⚠️ **Unverified and owed:** whether an ordinary incoming 1-sat payment becomes a tracked
default-basket row without a recovery scan. If yes, path 1 is live for any user who has ever received
one. This changes **urgency, not the fix** — which is why the beta.3 floor ships without waiting.

### Basket and permission mechanics — verified against BRC-46, 99, 147, 165

- `p ` is **request-time routing only**. BRC-165 normalization rewrites the storage basket to `1sat`;
  the DB never stores a `p ` name.
- `database/basket_repo.rs:62-65` already rejects `p ` names per BRC-99 — the correct fail-closed
  default. It becomes route-if-supported when we implement a scheme.
- Our `domain_basket_permissions` (V18, `domain_permission_repo.rs:407`) grants **one domain, one
  basket, binary**. BRC-99/165 scopes let a grant name an axis (`all` / `collection` / `app` /
  `creator` / `id`) with the value carried in tags. Ours is narrower than the spec.
- ⛔ **Spend is a `createAction` label** (`p 1sat input id <key>`), not a basket. **BRC-147 says pay
  and auto-pay grants MUST NOT authorize ordinal spends.** So the auto-approve engine needs a
  separate token-spend permission class, and the approval modals must show which baskets and
  sub-categories a grant covers. This lands in track 2, and it touches the privacy-perimeter
  surface — treat it with the seriousness of `R-PERIM`.

## What "done" looks like for beta.5

To be filled in at the end of the microscope pass, not now. Two structural rules that hold regardless:

1. **Every track doc sets specific goals with well-defined outcomes** for functions and tests,
   against the testing methods identified by research (c) — see `research/`.
2. **A track that ships a token capability without the guard in front of it is not done**, however
   green its own evidence table is.

## Decisions owed before track 1 implementation starts

1. **Exposure question** (above) — answer by experiment, not by reading. Sizes the guard's urgency
   and tells us whether existing users are already affected.
2. **Where the classification seam sits** — ingest-only, or ingest plus a verification pass on
   reconcile. Microscope decision; do not settle it here.
3. **The BSV-20/21 review phase's own gate question** (owner's): *do wallets need BSV20/21 code at
   all, or is it only the apps that talk to wallets?* Carried into track 2 as a **review** phase,
   not a build phase. Record the testing problem with it: most 1Sat/BSV21 apps ship their own
   wallets, so we may have nothing to test against.
4. ~~**Whether the OpNS overlay PoC is in scope**~~ — ✅ **moot: OpNS is out of this release** (planning
   note D4, 2026-09-25).
5. ⭐ **RQ-1 and RQ-2** — the two research questions below. Both are **research tasks, not decisions**.

## Research questions owed before track work *(carried from `RESUME_beta4.md` §3, archived 2026-09-25)*

⭐ Two questions sit **between** tracks. If each track's session answered them on its own, you would
get incompatible answers — so they are researched once, up front, and then the owner decides.
⛔ **Owner correction, 2026-08-30: both are RESEARCH TASKS.** Research them and present them for
decision; do not settle them on the spot or from first principles (`CLAUDE.md` working rule 5).
⭐ Per planning note "Research step one": **re-fetch the BRCs and SDKs first**; our copies are stale.

### RQ-1 — What does "this output is a token" get saved as?

When the wallet decides an output is a token rather than spendable money, **where is that fact
stored, and in what form?** The guard track has to build it; the ordinals track then implements a
spec with its own rules for how tokens are filed. If the guard guesses and the spec disagrees, the
ordinals track rewrites the one thing the release's safety rests on.

| # | Read | Looking for |
|---|---|---|
| 1 | **BRC documentation** — 46, 99, 147, 150, 165 in particular | What the spec *requires* to be persisted, versus what it leaves to the implementer |
| 2 | **BSV Association `wallet-toolbox`** — **TypeScript** and **Go** | How a conforming wallet actually stores basket/token classification — the closest thing to a reference answer |
| 3 | **The other BSV SDKs**, across languages | Where they agree, that is the convention. ⭐ **Where they disagree, that is the real design question** — report it as such |
| 4 | Our own code — `output_repo.rs`, `basket_repo.rs`, `domain_permission_repo.rs` | What we already have, and how far it is from the above |

⚠️ There is no Rust `wallet-toolbox`: we port **patterns and semantics, never code**.
**Output:** a short document — what the ecosystem does, where implementations disagree, what we
should do **and why**, with the trade-offs visible. Then the owner decides.

### RQ-2 — What does restore do with an output it cannot identify?

Restoring from a recovery phrase is the moment the wallet knows least. ⛔ **Owner correction,
2026-08-30: this needs a full conversation and a good / bad / ugly outcome matrix, not a two-option
recommendation.**

| Required | Meaning |
|---|---|
| ⭐ **A good / bad / ugly outcome matrix** | For **each** candidate behaviour: the good case, the bad case, and **the ugly case** — the one where the user loses something and does not find out for months |
| The candidate behaviours | At least: show as held-but-unidentified · block the restore · classify-later-on-reconcile · anything the ecosystem does that we have not thought of |
| ⭐ **How other wallets handle it** | `wallet-toolbox`'s recovery path, the BSV SDKs, any BRC that speaks to recovery. Somebody has hit this already |
| The user-facing consequence of each | In plain language, not database state |

**Why it matters this much:** the two failure modes are opposites, and a naive fix for one causes the
other. An output that is **silently spendable** breaks the release's core rule at the worst moment.
One that is **silently dropped** is safe from spending and effectively lost. That is why `R-RESTORE`
in `REGRESSION_ADDITIONS.md` is two checks that are each other's control. It is also the sharpest
edge between the guard track and the **backup** track, which owns restore.

### Also still owed *(from `RESUME_beta4.md` §5)*

- **The exposure question** (decision 1 above) — ⚠️ it has been *read* twice already; **answer it by
  running something**.
- **`../SCOPING_PROCESS.md` §7a / §7b / §7b-ii** — ~19 proposed adoptions, one owner decision each,
  none in force. Not urgent.
- `tickets/TICKET_e2e_specs_wrong_subject_and_never_run.md` is second-hand and unverified; its first
  step is verification.
- ⭐ **The belief to test first** (`TELESCOPE.md` §6): *the classification seam is cheap because the
  exclusion logic already works.* The guard track's day-one call-site sweep tests it. If there are
  UTXO-selecting paths that bypass `output_repo`, the guard is bigger than scoped and the plan changes.

## Reading order for a fresh session

⛔ **Resuming after a gap?** Start at the **Planning notes** section near the top of this file — it
carries the owner's latest decisions and the cleanup state. (`RESUME_beta4.md` did this job until
2026-09-25; its live content now lives in this README and it is archived.)

```
0. this README — Planning notes, then Research questions owed
1. TELESCOPE.md            ← how the four tracks interact, and how to run the microscope pass
2. RELEASE_PLAN.md          ← the track-level breakdown
3. ../0.4.0-beta.3/HARNESS.md + REGRESSION_SET.md   ← the standard
4. HARNESS_DELTA.md + REGRESSION_ADDITIONS.md       ← what beta.5 adds to it
5. the one track folder you are working in — and no others
```

⛔ **Do not read all four track folders in one context.** That is the specific mistake
`TELESCOPE.md` §"Context strategy" exists to prevent.
