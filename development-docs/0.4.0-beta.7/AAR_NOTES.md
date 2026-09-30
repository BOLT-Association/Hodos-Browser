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

## 2026-09-28 — how a test passes with its feature missing: five shapes the control round found

```
WHAT:       The independent-control round (208 cells, 6 designers) found ~20 GREENs that would pass with the
            feature absent. They fall into five repeatable shapes:
            1. An OLDER guard already produces the green — the 1-sat floor, `is_p2pkh_script` len==25, the
               basket filter — so the test proves the old guard, not the new classifier (T1-P5-A5a, T2-P1-A6,
               T2-P3-A11, T3b-P5-A14).
            2. The fixture is one today's code already handles — rescan already reaches index 28; a sender's
               `1sat` basket is already honoured (T1-P6-A6, T2-P1-A2).
            3. A helper turns an error into an answer BEFORE the test's switch — `check_tx_exists_on_chain`
               maps Unknown to Ok(false) inside, so removing the caller's `.unwrap_or` changes nothing
               (T1-P2-A10, T4-P1-A2b).
            4. A cache or fallback supplies the result — PaidContentCache serves the second visit; a truncated
               bulk body falls back to single-address fetch (T1-P1-A9, T1-P1-A2).
            5. The control itself would hit production — a hosts-file fault blinds the installed wallet; a
               release-profile binary opens the production data dir (T1-P2-A2, T4-P1-A9b).
COST:       none yet — caught at G3, before any code. Each would have been a green that proved nothing.
INSTRUMENT: the authors' own pre-mortems missed all of them; a second designer found them. Candidate for the
            contract template: per §4 row, "which OLDER guard, cache or fallback would also make this green?" —
            and a fixture rule: money fixtures need a >=2-sat case and a case today's code gets wrong.
```
