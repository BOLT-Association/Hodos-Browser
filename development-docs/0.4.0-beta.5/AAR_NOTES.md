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
