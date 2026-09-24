# The release cycle

**Created:** 2026-09-24, out of the beta.3 cycle's AAR (`0.4.0-beta.3/AAR.md`). **Version: v2.**
⭐ **This document is an output of every AAR.** It is expected to change each cycle; when it does,
the AAR that changed it says so (§7).

> **What this is.** The spine of a **release cycle** — plan → execute → release → review — and the
> **contract** that gates it (§2). It is the document we follow.
>
> **What it is not.** It does not restate scoping mechanics (`SCOPING_PROCESS.md`) or how to build
> and ship (`DevOps-CICD/BUILD_AND_RELEASE.md`). ⛔ Where they disagree with this file, **they win**.

---

## ⚠️ Read this before anything else — the cycle WILL be interrupted

A release cycle runs **one week to several months**. It will not run in a straight line, and the
process does not assume it does:

- 👤 **A user can report a critical bug at any moment.** Depending on severity it is fixed
  *immediately*, or injected as a new track mid-execution. Both are normal.
- **A defect found during execution can invalidate planning** — beta.3's Phase 13 was parked
  outright when Step 0 showed the complaint's mechanism had already been deleted.
- **A shipped release can be superseded before it is promoted** — beta.3 was, by beta.4, hours later.

⛔ **Interruption is not process failure.** Treat a stage's gate as *"have we done this, or
consciously decided not to"* — never as *"have we done this in order."* 🚨 The one thing that must
not happen is an interruption going **unrecorded**: it goes in `AAR_NOTES.md` the moment it happens,
and if it changed scope, it becomes a ticket or a track — never a silent addition.

---

## 0. Vocabulary — settled, use it

| Rung | Name | What it is | Short id |
|---|---|---|---|
| 1 | **Release** | the version we ship | `B5` |
| 2 | **Track** | one of 4–5 parallel efforts — a feature, or a bundle of related defects | `B5-T2` |
| 3 | **Phase** | a slice of a track finished and **verified on its own** — owns a contract | `B5-T2-P3` |
| 3.5 | **Sub-phase** | ⭐ optional pressure valve, only when a phase cannot carry one honest evidence table | `B5-T2-P3.1` |
| 4 | **Item** | one numbered row inside a phase | `A1` |

- **"Release cycle"** is the whole loop. **"Release"** alone is rung 1.
- ⛔ **"Sprint" is retired** — it names a timebox everywhere else in the industry.
- ⛔ **Never write a bare rung number.** Every reference starts at the release.
- **AAR** — After-Action Review. One per cycle.

### The three scopes — each must earn its name

⭐ **A named scope earns its name only if it makes you ask a question you would otherwise skip.**

| Scope | Direction | The question |
|---|---|---|
| **Telescope** | outward | What exists already in the world? Ecosystem, BRCs, other implementations ⇒ `PRIOR_ART.md` |
| **Microscope** | inward | What exactly changes? The specific code, the specific call sites |
| **Kaleidoscope** | across | ⭐ **What shape is this, and where else does it appear?** — in defects **and in our own code** |

⛔ **Rejected, with reasons, so they are not re-proposed:** *cosmic* (Telescope already looks
outward) · *spectroscope* (decomposition is Telescope→Microscope) · *atomic* (per-item detail belongs
in execution, not planning — it is the bog this process avoids) · *gyroscope* (drift control is a
**checkpoint**, not a scope — §4.3).

---

## 1. ⭐ The planning phase, at a glance — for humans

Read top to bottom. Every row produces something; nothing is "thinking about it".

| # | Step | The question it answers | Produces | Gate |
|---|---|---|---|---|
| 1 | **Orientation — the project** | What is Hodos? Three layers, invariants, principles | *(read only)* root `CLAUDE.md` | G0 |
| 2 | **Orientation — higher** | What constrains us? Engine pin, signing chain, BRC specs we conform to | notes in the release README | G0 |
| 3 | **Orientation — adjacent** | What is the other platform doing? What is live in the field? What are users running? | notes | G0 |
| 4 | **Orientation — ourselves** | ⭐ What did we learn last cycle? What is already open? | **last AAR read** + ticket inventory | G0 |
| 5 | **Standing serialization** | ⭐ **What must be serial, and why?** (known before any ticket — §3.6) | the constraint list, restated | G0 |
| 6 | **Mission** | What is this release *for*, in one paragraph a person could repeat? | mission statement | G1 👤 |
| 7 | **Telescope** | What exists already? Who solved this first? | `PRIOR_ART.md` rows | G2 |
| 8 | **Kaleidoscope (open)** | ⭐ **Do we already have this? Are we about to duplicate a shape?** | reuse notes / "extend, don't add" calls | G2 |
| 9 | **Tracks** | How do the tickets group into 4–5 parallel efforts? | track folders + scope docs | G2 |
| 10 | **Integration check** | ⭐ What invariant could this violate? What do we touch that we didn't write? What would we un-ship? | a required section per track | G2 |
| 11 | **Microscope** | What exactly changes, and in what order? | phases, items, contracts | G3 |
| 12 | **Admin & Logistics** | ⭐ Credentials, environments, CI, pins, test money — **what expires?** | the A&L checklist (§3.4) | G4 |
| 13 | **Command & Signal** | Who leads, who decides, how do we talk? | relay opened (§3.5) | G5 |
| 14 | **Serialization map** | Given *these* tracks, what else must be serial? | the map | G5 |
| 15 | **Planning close** | Do deliverables exist and are open questions answered? | 👤 sign-off | G6 👤 |

⛔ **Stop at G6.** Deliverables exist and open questions are answered ⇒ **stop planning and build.**
Over-planning is the failure this process exists to prevent, not the one it exists to cause.

---

## 2. ⭐ THE CYCLE CONTRACT — the gates

> 👤 Owner decision, 2026-09-24: *"it should always go back to that one master cycle document that
> needs to be checked off as a gate to the next step."* This is that document, and this is that list.
> Sub-contracts (track scope docs, phase contracts) hang off it; they never replace it.

⛔ A gate is **signed off**, not assumed. A gate that is deliberately skipped is **written down as
skipped, with the reason** — an unrecorded skip is the thing this contract exists to prevent.

### Planning gates

- [ ] **G0 — Oriented.** Project, higher, adjacent, and **ourselves** (steps 1–5). ⛔ The last AAR
      has been read. A cycle that starts without it has discarded the only mechanism that makes this
      one better
- [ ] **G1 — Mission stated** and 👤 signed off. ⛔ If it cannot be stated without listing the
      tracks, it is a backlog, not a mission — send it back
- [ ] **G2 — Tracks defined.** Telescope + kaleidoscope(open) done; every track has a scope doc and
      an **integration check**
- [ ] **G3 — Phases and items defined.** Every phase has a contract; sub-phases only where justified
- [ ] **G4 — Admin & Logistics checked** (§3.4). ⛔ Nothing we depend on expires inside this cycle
- [ ] **G5 — Comms plan set and serialization mapped.** Relay opened; lead named; constraints listed
- [ ] **G6 — 👤 PLANNING CLOSED.** Owner sign-off. ⇒ **execution may begin**

### Execution gates *(per track — the detail lives in the phase contracts)*

- [ ] **G7 — Each phase closed**: evidence table complete · every assertion has a **negative
      control** · **adversarial review of the evidence** done · `AAR_NOTES.md` swept
- [ ] **G8 — Each track closed**: **kaleidoscope(close)** · regression at the boundary ·
      ⭐ *does anything we learned change what the next track should do?*

### Release gates

- [ ] **G9 — Release definition-of-done** complete (§5)
- [ ] **G10 — Promoted and verified live** — feed, download redirects, **and N−1 → N self-update**

### Review gates

- [ ] **G11 — AAR written**, with §C and §D tables and changes in past tense
- [ ] **G12 — Closing actions**: next release's relay opened → this folder archived →
      **this document updated** if the AAR changed it

⭐ **The release is not done until G12.** An AAR deferred is an AAR never written.

---

## 3. Stage A — PLAN (detail)

Structured as **SMEAC**. Scoping mechanics are in `SCOPING_PROCESS.md`; ⛔ that process does **not**
apply to small work — a single ticket does not get a four-stage run.

### 3.1 Situation — orientation
Steps 1–5 of §1. ⭐ The last AAR and the open ticket inventory are **required inputs**, not optional
reading.

### 3.2 Mission
One paragraph, plain words, repeatable from memory.

### 3.3 Execution — tracks, phases, items
Tickets group into tracks; tracks decompose into phases; phases into items.

Each **track scope doc** owes an **Integration check**, answered in writing:
1. What existing invariant could this violate?
2. ⭐ What does it touch that **we did not write**? *(CEF, Chromium, Sparkle, the OS, a BRC)*
3. What would we have to un-ship if we are wrong?

⭐ Question 2 catches the expensive ones. Both defects shipped in the beta.3 cycle were **our code
meeting a dependency's default** — Chromium's log target, and libc++'s static destruction order.

### 3.4 Admin & Logistics — ⭐ required; its absence took a release down
🚨 On 2026-09-24 `WEBSITE_DEPLOY_TOKEN` expired **exactly 90 days** after it was set, mid-promote,
leaving the release half-published. No plan had a slot for it.

- [ ] **Credentials and expiry** — signing certs, deploy tokens, API keys. Owner, expiry date, blast radius
- [ ] **Environments** — dev stack, build host, test accounts, the **N−1 rig** for the update test
- [ ] **CI capacity and pins** — minutes, runner images (⛔ pinned, never floating)
- [ ] **Engine/toolchain pins** — CEF asset, vcpkg, deployment target
- [ ] **Test money** — funded wallets for money-path rows

### 3.5 Command & Signal — the comms plan
| | |
|---|---|
| **Lead** | 🪟 **Windows leads** and tasks 🍎 macOS. 👤 Owner decision, 2026-09-24 |
| **Channel** | `MAC_RELAY_<release>.md` in the release folder. Opened at G5, closed at G12 |
| **Ordering** | ⛔ **newest round first.** Read a file's own conventions before writing into it |
| **Round ids** | ⛔ **platform-prefixed** — `W-24a`, `M-24a`. Same-day collisions happened in beta.3 |
| **Content** | ⭐ **measurements, not conclusions.** Every cross-platform disagreement in beta.3 was settled by a number |
| **Platform-specific results** | ⛔ mark **"do NOT inherit this"**. Used three times in beta.3; each prevented a wrong assumption |
| **One driver** | ⛔ **only one agent queues work for 👤 the owner at a time.** The other banks its human rows and waits |

### 3.6 ⭐ Orchestration — ask what must be SERIAL
⛔ **Do not ask "where can we parallelize."** Ask **"what must be serial, and why?"** Everything else
is parallel by default and needs no planning.

**Standing constraints — known before any ticket is read (step 5, gate G0):**

| Constraint | Consequence |
|---|---|
| 👤 **The owner's attention** | one driver at a time (§3.5) |
| **Shared files** — relay, `CLAUDE.md`, ticket indexes | append-only where possible; append-vs-append conflicts are normal — **keep both sides** |
| **Shared machine state** — loopback ports, the wallet, the profile lock | ⚠️ ports are **machine-wide, not per-user**: two browsers on one machine share one wallet |
| **The build** | cannot compile while the dev browser runs — `scripts/stop-dev.ps1` first |
| **The tag** | one release build at a time; a burned tag costs a version number |

**Per-cycle map (step 14, gate G5):** given *these* tracks, what else is serial? Cross-platform
dependencies, shared files two tracks both edit, anything needing the owner.

---

## 4. Stage B — EXECUTE

> ⭐ *No plan survives first contact.* Contracts do not prevent adjustment — they make it **visible
> and recorded** instead of silent.

### 4.1 The phase contract
Carries the items, acceptance assertions, evidence table, and the checks below.

### 4.2 Where each check belongs

| Check | Where | The question |
|---|---|---|
| **Kaleidoscope (open)** | ⭐ **track open**, and before writing anything new | *Do we already have this? Are we about to duplicate a shape?* |
| **Negative control** | ⛔ **every acceptance assertion**, item level | *Can this test fail?* Turn the feature off; it must go red **for the right reason** |
| **Adversarial review — before** | phase contract, pre-code | *Does this plan have a hole?* A **pre-mortem**: assume it failed, say why |
| **Adversarial review — after** | ⭐ **phase close, on the evidence** | *Does this evidence prove what it claims?* |
| **Kaleidoscope (close)** | track close | *What shape did we create or find, and where else does it appear?* |
| **Regression** | track + release boundaries | *Did we disturb someone else's guarantee?* ⇒ `REGRESSION_SET.md` |

⭐ **Kaleidoscope runs at both ends and they are different questions.** Open prevents work — it is
the reuse-first rule with teeth. Close catalogues what we found, and feeds the AAR and the next
cycle's tickets. ⛔ A pattern found at close is **noted and ticketed, never chased** — we do not stop
a build to start a cleanup project.

⭐ **The phase-close adversarial review is the highest-value slot.** Reviewing a plan is cheap and
everyone does it; reviewing **your own green result** is rare — and is where this project's most
expensive defect class hides.

> 🚨 **The dominant defect class, three releases running: "the test did not measure its subject."**
> Four instances in the beta.3 cycle. ⛔ A negative control validates the mechanism you thought of,
> never the one you did not — so where stakes are high, **the control is designed by someone who did
> not write the assertion.**

### 4.3 Drift control *(the checkpoint formerly called the gyroscope)*
The contract states the phase's objective in one sentence. At each item's close, and at phase close:
**are we still doing that?** Scope that grew goes in `AAR_NOTES.md`, and if it is real work it
becomes a ticket — never a silent addition.

### 4.4 Lessons capture — ⭐ write it the moment it happens
One file per release: **`AAR_NOTES.md`**, append-only. Three lines per entry:

```
WHAT:       what happened, as fact
COST:       time, rework, or a burned artifact
INSTRUMENT: which instrument should have caught this, and why it didn't
```

⭐ The third line is the one that matters — it makes the AAR's hard tables assemble themselves.

- **Capture: any time.** Do not wait for a checkpoint.
- **Mandatory sweep: phase close**, as a line in the contract.
- ⛔ **Not per item** — dozens of "nothing to report" entries train people to skip the one that mattered.

### 4.5 Checkpoints
| When | What |
|---|---|
| **Item close** | drift check · capture anything learned |
| **Phase close** | `AAR_NOTES.md` sweep · **adversarial review of the evidence** · contract signed off *(G7)* |
| **Track close** | **kaleidoscope(close)** · regression at the boundary · ⭐ *does anything we learned change what the next track should do?* *(G8)* |

---

## 5. Stage C — RELEASE

`DevOps-CICD/BUILD_AND_RELEASE.md` is the procedure and wins. The shape:

**propagate → validation build → tag → verified DRAFT → 👤 human gate → rehearse → promote → verify live**

⛔ **A tag can never reach a customer on its own.** Only a manual promote can.

### Release definition-of-done *(gate G9)*
- [ ] Every track closed, or explicitly cut **and recorded**
- [ ] `preflight -Full` **PASS — all checks ran** *(⚠️ a code gate, **not** a release verdict)*
- [ ] Regression set green at the release boundary
- [ ] Both platforms built, signed, notarized
- [ ] 👤 **Human at the keyboard** — install, smoke, a **real payment**, the install-tree diff
- [ ] Farbling rotation token **+ its negative control**
- [ ] AV seeding — VirusTotal (hash-checked) + MS Defender
- [ ] Promote **rehearsal** green before the real run
- [ ] Live verification: feed, download redirects, **N−1 → N self-update**

---

## 6. Stage D — AAR

Immediately after promote, while it is fresh. ⛔ **Not at the start of the next cycle** — what decays
fastest is *why*.

Template: `0.4.0-beta.3/AAR.md`. The two sections carrying the value:
- **§C — defects found after "done", each attributed to the instrument that found it.**
- **§D — instruments that should have caught it and did not.**

Plus process failures, a **comms review**, changes **made** in past tense with locations, decisions
**deliberately not taken**, and what is carried forward.

⛔ **An AAR that ends in a document is a failed AAR.** Its last section is changes made; anything not
doable in the session becomes a ticket with an owner and a release.

**Closing actions (G12):** open the next release's relay → archive this folder → update this document.
⭐ **Archive means reviewed, not merely shipped.**

---

## 7. How this document improves

Each AAR may change this file. When it does, the AAR names the section and why, and the version
increments. ⛔ No silent edits.

- **v1 — 2026-09-24**, from the beta.3 AAR. Introduced Admin & Logistics, Command & Signal,
  serialization-first orchestration, the phase-close adversarial review, `AAR_NOTES.md`, kaleidoscope.
- **v2 — 2026-09-24**, 👤 owner review of v1: the **cycle contract with gates** (§2) — *"it should
  always go back to one master document checked off as a gate"*; the **human-readable planning
  table** (§1); **kaleidoscope at track OPEN as well as close** (§4.2) — *"shouldn't we look for
  shapes that already exist before we build it?"*; **serialization moved into orientation** (§1 step
  5, G0); and the ⚠️ **interruption notice** at the top — a cycle runs one week to several months and
  a user's critical bug can inject a track mid-execution.
