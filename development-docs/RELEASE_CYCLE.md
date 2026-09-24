# The release cycle

**Created:** 2026-09-24, out of the beta.3 cycle's AAR (`0.4.0-beta.3/AAR.md`).
**Status:** v1. ⭐ **This document is an output of every AAR.** It is expected to change each cycle;
when it does, the AAR that changed it says so.

> **What this is.** The spine of a **release cycle**: plan → execute → release → review. It is the
> document we follow, not a document about writing a document.
>
> **What it is not.** It does not restate the scoping mechanics — `SCOPING_PROCESS.md` owns those
> and this points at it. It does not restate how to build and ship — `DevOps-CICD/BUILD_AND_RELEASE.md`
> owns that. ⛔ Where they disagree with this file, **they win**; this is the map, they are the ground.

---

## 0. Vocabulary — settled, use it

| Rung | Name | What it is | Short id |
|---|---|---|---|
| 1 | **Release** | the version we ship, 1–3 months | `B5` |
| 2 | **Track** | one of 4–5 parallel efforts — a feature, or a bundle of related defects | `B5-T2` |
| 3 | **Phase** | a slice of a track finished and **verified on its own** — owns a contract | `B5-T2-P3` |
| 3.5 | **Sub-phase** | ⭐ optional pressure valve, only when a phase is too big for one honest evidence table | `B5-T2-P3.1` |
| 4 | **Item** | one numbered row inside a phase | `A1` |

- **"Release cycle"** is the whole loop, plan through review. **"Release"** alone is rung 1.
- ⛔ **"Sprint" is retired.** It names a timebox everywhere else in the industry, which is what made
  "the beta.4 sprint" collide with "sprint 2".
- ⛔ **Never write a bare rung number.** Every reference starts at the release.
- **AAR** — After-Action Review. One per cycle, `AAR.md` in the release folder.

### The scopes — three, and each must earn its name

⭐ **A named scope earns its name only if it makes you ask a question you would otherwise skip.** If
two scopes ask overlapping questions, one of them dies.

| Scope | Direction | The question |
|---|---|---|
| **Telescope** | outward, far | What exists already? Ecosystem, BRCs, other implementations, prior art. ⇒ `CLAUDE.md` working rule 5 and `PRIOR_ART.md` |
| **Microscope** | inward, near | What exactly changes? The specific code, the specific call sites |
| **Kaleidoscope** | across | ⭐ **What *shape* is this, and where else does it appear?** Both in defects *and in our own code* — the same function in three places, the same wrong pattern in four |

⛔ **Rejected, with the reason** — so they are not re-proposed: *cosmic* (Telescope already looks
outward) · *spectroscope* (decomposition is what Telescope→Microscope already does) · *atomic*
(per-item detail belongs in execution, not planning — it is the bog this process exists to avoid) ·
*gyroscope* (drift control is a **checkpoint**, not a scope — it lives in the contract, §2.3).

---

## 1. Stage A — PLAN

Structured as **SMEAC**, because two of its five paragraphs cover things software planning templates
routinely omit — and one of those omissions took a release down in the beta.3 cycle.

Scoping mechanics (the Scope → Telescope → Microscope → Telescope-close passes) are in
`SCOPING_PROCESS.md`. ⛔ That process does **not** apply to small work; a single ticket does not get
a four-stage run.

### 1.1 Situation — orientation

Before any ticket is read:

- **The project**: what Hodos is, the three layers, the invariants. ⇒ root `CLAUDE.md`
- **Higher**: what constrains us — the engine pin, the signing chain, the BRC specs we conform to
- **Adjacent**: what the other platform is doing, what is live in the field, what users are running
- **Our own state**: the last cycle's **AAR**, and every open ticket

⭐ **The AAR is a required input here.** A cycle that starts without reading the last AAR has thrown
away the only mechanism that makes the next one better.

### 1.2 Mission — what this release is for

One paragraph, in plain words, that a person could repeat from memory. Then the tracks.

⛔ **If the mission cannot be stated without listing the tracks, it is not a mission — it is a
backlog.** Send it back.

### 1.3 Execution — tracks, phases, items

Tickets are read, grouped, and become tracks; tracks decompose into phases; phases into items.
Sub-phases only where a phase cannot carry one honest evidence table.

Each **track scope doc** owes, as a required section:

> **Integration check** — three questions, answered in writing:
> 1. What existing invariant could this violate?
> 2. What does it touch that we did not write? *(CEF, Chromium, Sparkle, the OS, a BRC)*
> 3. What would we have to un-ship if we are wrong?

⭐ Question 2 is the one that catches the expensive ones. The beta.3 cycle's two shipped defects were
both **our code meeting a dependency's default behaviour** — Chromium's log target, and libc++'s
static destruction order.

### 1.4 Admin & Logistics — ⭐ new, and it took a release down

⛔ **Required. It has no home anywhere else and its absence is not theoretical.** On 2026-09-24
`WEBSITE_DEPLOY_TOKEN` expired exactly 90 days after it was set, mid-promote, leaving the release
half-published. Nothing in any plan had a slot for it.

At minimum, every cycle:

- [ ] **Credentials and their expiry** — signing certs, deploy tokens, API keys. Who owns each, when
      does it die, what breaks when it does
- [ ] **Environments** — dev stack, build host, test accounts, the N−1 rig for the update test
- [ ] **CI capacity and cost** — minutes, runner images and their pins
- [ ] **Engine/toolchain pins** — CEF asset, vcpkg, runner OS. Does anything expire or drift?
- [ ] **Test money** — funded wallets for the money-path rows

### 1.5 Command & Signal — the comms plan

⭐ Software planning rarely has this paragraph. We do, and it is the reason two platforms stay
coherent.

| | |
|---|---|
| **Lead** | 🪟 **Windows is the lead** and tasks 🍎 macOS. 👤 Owner decision, 2026-09-24 |
| **Channel** | `MAC_RELAY_<release>.md` in the release folder. One per release; opened at plan, closed at AAR |
| **Ordering** | ⛔ **newest round first.** Read a file's own conventions before writing into it |
| **Round ids** | ⛔ **prefix by platform** — `W-24a`, `M-24a`. Same-day collisions happened in beta.3 |
| **Content** | ⭐ **measurements, not conclusions** — hashes, timestamps, log lines. Every cross-platform disagreement in beta.3 was settled by a number |
| **Platform-specific results** | ⛔ mark **"do NOT inherit this"** explicitly. Used three times in beta.3; each prevented a wrong assumption |
| **One driver** | ⛔ **only one agent may queue work for the owner at a time.** The other banks its human rows and waits for a relay line |

### 1.6 Orchestration — ⭐ ask what must be SERIAL

⛔ **Do not ask "where can we parallelize."** Ask **"what must be serial, and why?"** Everything else
is parallel by default and needs no planning at all.

The standing serialization constraints, all discovered by hitting them:

| Constraint | Consequence |
|---|---|
| 👤 **The owner's attention** | one driver at a time (§1.5) |
| **Shared files** — the relay, `CLAUDE.md`, ticket indexes | append-only where possible; expect append-vs-append conflicts and **keep both sides** |
| **Shared machine state** — loopback ports, the wallet, the profile lock | ⚠️ ports are **machine-wide, not per-user**. Two browsers on one machine share one wallet |
| **The build** | cannot compile while the dev browser runs; `scripts/stop-dev.ps1` first |
| **The tag** | one release build at a time; a burned tag costs a version number |

⭐ Name these at plan time and the parallel work falls out for free.

### 1.7 Exit condition

⛔ **Deliverables exist and open questions are answered ⇒ stop planning.** Development sometimes just
requires building. The loop limit and human decision points are in `SCOPING_PROCESS.md`.

---

## 2. Stage B — EXECUTE

> ⭐ *No plan survives first contact.* The point of the contracts is not to prevent adjustment — it is
> to make adjustment **visible and recorded** instead of silent.

### 2.1 The phase contract

Every phase owns one. It carries the items, the acceptance assertions, the evidence table, and the
checks below. The contracts worked well in beta.3 and are the main thing to keep refining.

### 2.2 Where each check belongs

| Check | Where | The question it asks |
|---|---|---|
| **Negative control** | ⛔ **every acceptance assertion**, at item level | *Can this test fail?* Turn the feature off; it must go red **for the right reason** |
| **Adversarial review — before** | phase contract, pre-code | *Does this plan have a hole?* (a **pre-mortem**: assume it failed, say why) |
| **Adversarial review — after** | ⭐ **phase close, on the evidence** | *Does this evidence prove what it claims?* |
| **Kaleidoscope** | track close | *What shape is this, and where else does it appear?* |
| **Regression** | track + release boundaries | *Did we disturb someone else's guarantee?* ⇒ `REGRESSION_SET.md` |

⭐ **The phase-close adversarial review is the highest-value slot and the newest.** Reviewing a plan
is cheap and common; reviewing **your own green result** is rare — and it is where this project's
most expensive defect class hides. In beta.3 it is what caught a passing signature test that was
actually testing corruption.

> 🚨 **The dominant defect class, three releases running: "the test did not measure its subject."**
> Four instances in the beta.3 cycle alone. ⛔ A negative control validates the mechanism you thought
> of, never the one you did not — so where the stakes are high, **the control should be designed by
> someone who did not write the assertion.**

### 2.3 Drift control — the checkpoint formerly called the gyroscope

The contract states the phase's objective in one sentence. At each item's close, and again at phase
close: **are we still doing that?** Scope that grew goes in `AAR_NOTES.md` and, if it is real work,
becomes a ticket — never a silent addition.

### 2.4 Lessons capture — ⭐ write it the moment it happens

One file per release: **`AAR_NOTES.md`**, append-only. An entry is three lines:

```
WHAT:       what happened, as fact
COST:       time, rework, or a burned artifact
INSTRUMENT: which instrument should have caught this, and why it didn't
```

⭐ **The third line is the one that matters.** Naming the instrument while it is fresh means the
AAR's two hard tables assemble themselves instead of needing archaeology.

- **Capture: any time.** Do not wait for a checkpoint.
- **Mandatory sweep: phase close**, as a line in the contract.
- ⛔ **Not per item.** Dozens of "nothing to report" entries train people to skip the one that mattered.

### 2.5 Checkpoints

| When | What |
|---|---|
| **Item close** | drift check (§2.3). Capture anything learned |
| **Phase close** | `AAR_NOTES.md` sweep · adversarial review of the evidence · contract signed off |
| **Track close** | **kaleidoscope** pass · regression at the boundary · ⭐ **one question: does anything we learned change what the next track should do?** |

---

## 3. Stage C — RELEASE

`DevOps-CICD/BUILD_AND_RELEASE.md` is the procedure and it wins. This is only the shape:

**propagate → validation build → tag → verified DRAFT → 👤 human gate → rehearse → promote → verify live**

⛔ **A tag can never reach a customer on its own.** Only a manual promote can, and that is deliberate.

### Release definition-of-done

- [ ] Every track closed, or explicitly cut and recorded
- [ ] `preflight -Full` **PASS — all checks ran** *(⚠️ a code gate, not a release verdict)*
- [ ] Regression set green at the release boundary
- [ ] Both platforms built, signed, notarized
- [ ] 👤 **Human-at-the-keyboard gate** — install, smoke, a **real payment**, the install-tree diff
- [ ] Farbling rotation token **+ its negative control**
- [ ] AV seeding — VirusTotal (hash-checked) + MS Defender
- [ ] Promote **rehearsal** green before the real run
- [ ] Live verification: feed, download redirects, **and the N−1 → N self-update**
- [ ] **AAR written** ⇐ the release is not done until this exists

---

## 4. Stage D — AAR

Immediately after promote, while it is fresh. ⛔ **Not at the start of the next cycle** — what decays
fastest is *why*, and that is the part worth keeping.

Template: `0.4.0-beta.3/AAR.md`. The two sections that carry the value:

- **C — defects found after "done", each attributed to the instrument that found it.** Over several
  cycles this says which instruments earn their keep and which are theatre.
- **D — instruments that should have caught it and did not.** Where the compounding happens.

Plus: process failures (separate from code), a **comms review**, changes **made** in past tense with
locations, decisions **deliberately not taken** so the next cycle does not re-litigate them, and what
is carried forward.

⛔ **An AAR that ends in a document is a failed AAR.** Its last section is changes made. Anything not
doable in the session becomes a ticket with an owner and a release.

**Closing actions:** open the next release's relay → archive this release folder.
⭐ **Archive means reviewed, not merely shipped** — a folder moves to `archived-docs/` only after its
AAR is closed.

---

## 5. How this document improves

Each AAR may change this file. When it does, the AAR says which section and why, and this header's
version increments. ⛔ No silent edits — a process document nobody can trust the provenance of is
worse than none.

**v1 — 2026-09-24**, from the beta.3 AAR. New in v1: Admin & Logistics (§1.4), Command & Signal as a
required paragraph (§1.5), orchestration framed as serialization constraints (§1.6), the phase-close
adversarial review (§2.2), `AAR_NOTES.md` capture-at-the-moment (§2.4), and the kaleidoscope scope.
