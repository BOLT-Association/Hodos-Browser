# AAR notes — beta.5 release cycle

**Append-only.** One entry per lesson, the moment it happens (`../RELEASE_CYCLE.md` §4.4). The beta.5
AAR is assembled from this file.

```
WHAT:       what happened, as fact
COST:       time, rework, or a burned artifact
INSTRUMENT: which instrument should have caught this, and why it didn't
```

---

## 2026-09-25 — ⭐ REVIEW AT THE AAR: GitHub issues, one per phase (trial)

```
WHAT:       Adopted RELEASE_CYCLE v5 §4.1a — one GitHub issue per phase, opened at G6, closed at G7 by
            the pushed commit; markdown stays the source of truth. Owner's reasons: link work to code;
            a place future developers and their AI tools expect to look.
COST:       none yet — this entry exists so the AAR judges it: did the issues stay in sync with the
            contracts, did anyone (owner, agent, future dev) actually use them, keep / change / drop?
INSTRUMENT: n/a — a trial, not a defect
```

## 2026-09-25 — the ticket status line is not evidence

```
WHAT:       A file-by-file review of beta.3's 51 tickets found 17 that read "OPEN" but were already
            fixed, and 2 beta.5 tickets presumed fixed that were only partly fixed. A status-line count
            ("~20 open") was wrong in both directions.
COST:       one review agent (~11 min); would have been weeks of re-investigating fixed defects.
INSTRUMENT: nothing checks that a ticket's status matches the code; closure is a manual edit that
            phase close does not require. Candidate: phase sign-off names the tickets it closes.
```

## 2026-09-28 — G3's parallel Fable run hit the usage limit in ~12 minutes

```
WHAT:       G3 launched 8 per-track contract agents at once (4 Fable, 4 Opus). Each read ~275k tokens
            of docs and code before writing. The account's usage limit was reached ~12 minutes in; the
            Fable agents were stopped and the unfinished tracks restarted on Opus, 3 at a time.
COST:       ~6 agents' in-flight work lost (their finished files were kept, 9f0c7dc); a wait for the
            limit to reset; the planned Fable-vs-Opus comparison cut to what finished (T0, T4 = Opus).
INSTRUMENT: none — agent count and model were chosen with no usage budget in view. Candidate: a
            multi-agent plan states its expected token spend, and Fable-heavy batches run at end of day.
```

## 2026-09-28 — G3 model comparison, as far as it got

```
WHAT:       Contracts: T0/T2/T4/T6 + most of T1/T3a/T5 by Opus; T1-P1, T3a-P0/P1, T5-P1 by Fable (before the
            stop). Independent controls: controls-A/B/C by Fable (103 cells), D/E/F by Opus (105) after Fable
            hit the usage limit a second time. Every control group, on both models, found GREENs that could
            pass with the feature absent — ~20 across the round.
COST:       two usage-limit stops in one day; the owner's review of the contracts is the real score, not yet run.
INSTRUMENT: the comparison needs the owner's review per contract (Fable- vs Opus-authored, Fable- vs
            Opus-controlled). Record it here at that review. Early signal only: the independent-control round
            earned its keep — every group found an assertion that could not fail.
```
