# AAR — the beta.3 release cycle

**Event:** the beta.3 release cycle, closing with **`v0.4.0-beta.4` promoted 2026-09-24**.
**Written:** 2026-09-24, immediately after promotion, while the cycle was still in memory.
**Participants:** 👤 owner, 🪟 Windows agent, 🍎 macOS agent.

> ⭐ **An AAR that ends in a document is a failed AAR.** §7 is the point of this file; everything
> above it exists to justify §7. Anything that could not be changed during the review became a
> ticket with a release attached — never a bullet that evaporates.
>
> ⛔ **Not a blame exercise and not a status report.** The unit of analysis is **the instrument**,
> never the person. Every "we missed X" below is really "no instrument was watching for X."

---

## §1 — What shipped

| | |
|---|---|
| Shipped | **`v0.4.0-beta.4`**, public + Latest, 2026-09-24 |
| Superseded | `v0.4.0-beta.3` (`868aef6`) — built, tested, **never published** |
| Previous public | `v0.3.0-beta.29`, 2026-07-20 |
| ⭐ Significance | **the first 0.4.0 the public has ever had.** beta.1 and beta.2 were built and never promoted, so the line jumped `0.3.0-beta.29 → 0.4.0-beta.4` |

**Content of the release** — three owner-found defects fixed during the cycle, then two logging
fixes found by the release process itself:

| | |
|---|---|
| level-0 protocol calls prompted when the reference wallet never does | `matrix_c.rs` |
| connect prompts arriving after approval were orphaned; the page hung 10 min | prompt-queue re-show |
| `createAction` accepted an empty locking script | `ERR_EMPTY_LOCKING_SCRIPT` |
| macOS aborted on **every** quit | `c4ef538` — never destroy `LogMutex` |
| Chromium wrote `debug.log` into `{app}` | `9559191` — two lines INFO → DEBUG |

**Phase 13 (bot detection) was parked**, correctly: Step 0 showed the complaint's mechanism had
already been deleted in 0.4.0, and the verdict matrix's own negative control did not go red.

## §2 — What it cost

| | |
|---|---|
| Release-execution span | ~1 day, 2026-09-23 → 2026-09-24 |
| Tags burned | **2** (beta.3 superseded by beta.4) |
| CI release runs | 3 (1 validation dispatch, 2 tag builds) |
| Promote runs | 3 (1 rehearsal, 1 failed, 1 successful) |
| Local gate runs | 3 × `preflight -Full`, 2 × farbling gate + 2 × its negative control |
| 👤 Owner time | ~4 h at the keyboard, including two full install/test passes and two AV submissions |

⭐ **The second tag was worth it.** It cost ~2 h and removed a crash the user saw on every quit of
the first public 0.4.0.

---

## §3 — ⭐ Defects found after the code was "done", and WHICH INSTRUMENT FOUND EACH

⭐ **This is the most valuable table in the AAR.** Over several cycles it says which instruments earn
their keep and which are theatre.

| # | Defect | Found by | Class |
|---|---|---|---|
| 1 | level-0 protocol calls prompted forever behind a stale prompt | 👤 **owner, using the product** | human |
| 2 | connect prompts orphaned after approval; page hangs 10 min | 👤 **owner, using the product** | human |
| 3 | `createAction` accepts `lockingScript:''` and spends into an unlocked output | 👤 **owner**, and **only the chain showed it** | human + ground truth |
| 4 | macOS aborts on every quit | 👤 **owner, on the beta.3 draft** | human |
| 5 | `{app}\debug.log` created during normal use | 🪟 **the `I4` before/after folder diff** | manual instrument |
| 6 | `WEBSITE_DEPLOY_TOKEN` expired mid-promote | 🔁 **the promote run itself** | production |
| 7 | `SHA256SUMS` cannot be pasted into the web form | 👤 **owner, instantly** — then the rehearsal | human |
| 8 | the C1 byte-flip control tested corruption, not the signature | 🍎 **macOS agent auditing its own harness** | self-review |
| 9 | `createSignature` rejects a missing `counterparty` | 🪟 a test script, incidentally | accident |

### What this table says

⛔ **Zero of the nine were found by an automated gate.** `preflight -Full` was green on the tree that
shipped #4 and #5, and green throughout.

⭐ **Four of nine were found by the owner simply using the product.** That is not a failure of
process — it is the measurement that says *our automated instruments do not reach where users live*:
native input, real quits, real money, the install tree.

⭐ **#5 is the encouraging one.** A dumb instrument — snapshot a folder, use the browser, snapshot
again, diff — caught what a purpose-built static gate could not (see §4). ⇒ **cheap runtime
observation beats clever static analysis at the boundary where the OS and dependencies live.**

⭐ **#8 is the one to be proudest of**: an agent audited its own passing test, found it passed for the
wrong reason, and said so. That is the behaviour the whole negative-control rule exists to produce.

---

## §4 — ⛔ Instruments that SHOULD have caught it and did not

⭐ Most teams never write this section. It is where the compounding happens.

| Instrument | Should have caught | Why it didn't |
|---|---|---|
| **`G1`** — *"bare-filename file sinks (relative path resolves against CWD, i.e. `{app}`)"* | #5, the file in `{app}` | ⛔ **It greps OUR source. The file was opened by Chromium's logging inside libcef.** The gate's *name* claims an outcome its *implementation* cannot deliver. 🚨 And `G1` **has** a negative control and passes it — a synthetic sink in our code does trip it. ⇒ **a negative control validates the mechanism you thought of, never the one you did not** |
| **`preflight -Full`** | #4 and #5 | Not a defect: it is a *code* gate, and both were *runtime* behaviours. ⚠️ But it is quoted as though it were a release verdict. Its scope needs saying out loud |
| **nothing at all** | #6, the expired secret | No instrument watches credential expiry. The release *was* the instrument |
| **the promote retry loop** | #6, correctly | It caught the failure but **named the wrong cause** — printed *"push rejected (concurrent update on main?)"* three times for what was a 401. ⇒ an instrument that misreports sends the next person down the wrong path |
| **the `C1` byte-flip control** | nothing — it was the instrument | 🍎 It went red by **corrupting the archive**, not by failing the signature. A control that goes red for the wrong reason is indistinguishable from one that works |

### ⭐ The pattern across §3 and §4 — the kaleidoscope finding

**Four separate instruments this cycle were green, or red, for reasons unrelated to what they claimed
to measure:**

1. `G1` — green while the defect it is named for shipped
2. the C1 byte-flip — red by corruption, not by signature
3. the Phase 13 `--enable-automation` cell — the score did not move because the demo's score is a
   **sample**, so the matrix proved reachability and nothing else
4. (carried in) the three 0.4.0 farbling harnesses that would each have passed with the feature absent

⇒ **The recurring shape is not "we forgot a test." It is "the test did not measure its subject."**
That is the single most expensive defect class this project has, across three separate releases now.

---

## §5 — Process failures, separate from code

| # | What | Cost |
|---|---|---|
| P1 | `WEBSITE_DEPLOY_TOKEN` expired **90 days to the day** after it was set, mid-promote | ~40 min + one failed run; left a **half-promoted** state |
| P2 | `SHA256SUMS` is multi-line; `workflow_dispatch` string inputs are **single-line** | one failed rehearsal, and the error read as *"the bytes changed"* — alarming and wrong |
| P3 | MS Defender's form now **requires** a Detection name; a clean file has none | risk of fabricating a threat name. Resolved honestly (`N/A` + explanation) |
| P4 | the AAR/lessons capture did not exist | this document was written from memory, which works only because it was written the same day |

⭐ **P1's real lesson is not "rotate the token."** It is that **Admin & Logistics has no slot in our
planning**. Environments, CI capacity, signing certs, credential expiry — none of it appears in a
track scope doc, and it took a release down.

---

## §6 — Communications review (the relay)

👤 Owner asked for this specifically. **Verdict: the relay works, and it is the reason two platforms
stayed coherent. Four defects, all mine, all in how it was used rather than what it is.**

| # | What happened | Fix |
|---|---|---|
| C1 | 🪟 **I appended three rounds to the BOTTOM** of a file whose own header says *"newest round first"*. macOS opens it at the top ⇒ they would have missed all three | Round `23e` re-posted at the top pointing down. ⛔ **Read a doc's own conventions before writing into it** |
| C2 | 🪟 **Round 23a stated a wrong conclusion** to the other platform (that we had never had Sparkle channels). 👤 The owner corrected it from memory; the history proved him right | `23b` retracted it. ⭐ **A relay round is published to another agent that will act on it — the bar is higher than a note to self** |
| C3 | **Round-number collision**: macOS had `23b/23c/23d` and Windows had `23a/23b/23c` on the same day, disambiguated only by a `(Mac)`/`(Windows)` suffix | ⇒ **prefix rounds by platform** (`W-23a`, `M-23a`) or use a single shared counter |
| C4 | Both agents nearly queued work for the owner at once — the 2026-09-21 *"we are causing each other conflicts"* shape | Mitigated live with an explicit **one-driver** instruction in `23e §1`. ⇒ make it standing, not ad hoc |

⭐ **What worked, and should be kept deliberately:**
- Rounds carrying **measurements, not conclusions** — hashes, timestamps, log lines. Every
  disagreement this cycle was settled by a number rather than an argument.
- **Explicit "do NOT inherit this"** markers when a result was platform-specific. Used three times;
  each one prevented a wrong assumption.
- 🍎 macOS **correcting our spec** rather than satisfying it (the C1 client version, the byte-flip
  control). A relay that only reports compliance is worth much less.

👤 **Owner decision, recorded:** 🪟 **Windows is the lead**, tasking 🍎 macOS. To be written into the
release-cycle doc rather than re-derived each time.

---

## §7 — ⭐ CHANGES MADE (past tense — this is the point of the AAR)

| # | Change | Where |
|---|---|---|
| 1 | The four promote findings written into the **status block** of the release guide, where the next person hits them before starting | `BUILD_AND_RELEASE.md` §Current Status |
| 2 | The CLI promote invocation documented, with the reason the web form cannot carry `SHA256SUMS` | same |
| 3 | The half-promoted state documented as **recoverable** (re-run skips the flip) | same |
| 4 | The Defender "Detection name" answer for a clean file, plus the local scan that establishes there is no detection | same |
| 5 | `WEBSITE_DEPLOY_TOKEN` rotated to **no expiry**, minimal scope recorded | GitHub secret + doc |
| 6 | Signing-CA ledger row: beta.4 is the **third consecutive** release on `EOC CA 04` | `BUILD_AND_RELEASE.md` §2.5.1 |
| 7 | Release-boundary regression row, with R-INTEXT's subject **verified** rather than assumed | `REGRESSION_SET.md` |
| 8 | Owner's gate checklist written and corrected mid-cycle | `RELEASE_GATE_CHECKLIST_beta3.md` |

**Tickets opened** (all → the next release): `G1` gate gap · Chromium `debug.log` *(fixed, closed)* ·
tracked mkcert dev key · the visible update (3 tiers, costed) · wallet shared across OS accounts
*(🍎)* · plus the four carried from the cycle itself.

## §8 — What we deliberately did NOT change, and why

⭐ Recorded so the next session does not re-litigate settled calls.

| Decision | Why |
|---|---|
| **Ship the feed unlabelled** (no `sparkle:channel`) | `beta.29` postdates the client subscription's removal, so a labelled item is invisible to every installed macOS build. Re-adding the subscription is free and additive — it just cannot help machines already in the field |
| **Did not widen `G1` in this release** | Harness rule 6: the instrument is not edited by the change it measures. Ticketed instead |
| **Did not adopt Chrome's side-by-side updater** | Tiers 1+2 give the same *experience* for ~a tenth of the engineering; tier 3's gain is download size. Revisit only if 130 MB becomes a complaint |
| **Did not fix the named-profile taskbar label** | Known, tracked as `P3-A5d`, and a deliberate consequence of the 2026-08-31 decision to drop per-profile shortcuts. The mechanism is unverified and the file has been wrong twice — it gets measured before it gets written |
| **Did not chase `createSignature`'s counterparty** | Signing code; `CLAUDE.md` invariant 3 says ask first. Ticketed |

## §9 — Carried forward

- ⬜ **Regenerate `WEBSITE_DEPLOY_TOKEN`** — it never expires and was pasted into a chat log
- ⬜ **Open the next release's relay**, then archive this folder *(the closing action of this AAR)*
- ⬜ Consolidate `0.4.0-beta.4/` + its tickets into **`0.4.0-beta.5/`** — the beta.4 version is now spent
- ⬜ Phase 12 (adblock wrong render process) remains **open**; engine fix parked in `NEXT_CHROMIUM_BUILD.md`
- ⬜ Phase 13 parked; reopen only on a reproduction against a current build

---

## §10 — ⭐ The three findings worth carrying into the process itself

1. **Our instruments do not reach where users live.** Four of nine defects were found by a human
   using the product. ⇒ the release gate must keep a *human-at-the-keyboard* leg; it is not a
   stopgap until automation improves.
2. **The dominant defect class is "the test did not measure its subject"** — four instances this
   cycle, three releases running. ⇒ this is what the **kaleidoscope** pass exists to catch, and it
   deserves to be a named step rather than a habit.
3. **Admin & Logistics has no home in our planning**, and it took the release down. ⇒ it becomes a
   required paragraph, not an afterthought.
