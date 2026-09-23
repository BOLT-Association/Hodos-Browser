# TICKET — gate `G1` was written for exactly the `{app}\debug.log` defect and passed straight through it

**Filed:** 2026-09-23, immediately after fixing the defect it failed to catch (`9559191`).
**Severity:** 🟡 the gate is not wrong, it is **narrower than its own name**, which is worse than
either a pass or a fail — it reads as coverage we do not have.

> ⛔ **Filed here, not fixed on the release candidate.** `HARNESS.md` rule 6: the instrument is not
> edited by the change it measures. Widening `G1` in the same commit that fixed the file it missed
> would be exactly that.

## What happened

`G1` is *"Bare-filename file sinks (relative path resolves against CWD, i.e. `{app}`)"*. It exists
because a file landing in the install root has bitten this project before
(`TICKET_stray_log_in_install_root.md`).

📏 On 2026-09-23, `INSTALL_TEST_BATCH` row `I4` found **`{app}\debug.log`** on a real signed install.
📏 `G1` **passed on every run before, during and after** — including the run on the exact tree that
produced the file.

## Why it could not have caught it

`G1` greps **our own source** for file sinks opened with a bare filename — `std::ofstream("x.log")`
and friends. The file it missed was opened by **Chromium's logging**, from inside libcef, because two
of our `LOG_INFO` calls ran before `CefInitialize` applied `settings.log_file` and Chromium fell back
to its documented default: `debug.log` in the process working directory.

⇒ **the defect's mechanism is in a dependency, and the gate only reads our tree.** No pattern over our
source can see it. The gate is not broken; its *name* claims a guarantee its *implementation* cannot
give, and I read it as coverage.

## What a real gate for this looks like

⭐ The honest instrument is not static at all — it is `I4` itself: **snapshot `{app}`, run the browser,
snapshot again, diff.** That catches the file whatever wrote it, ours or a dependency's, and it is what
actually worked. Options, cheapest first:

1. ⭐ **Promote the `{app}` before/after diff from a manual install row to a scripted check** that can
   run against any built tree with a temporary install root. Its negative control is free: revert
   `9559191` and it must go red.
2. Narrow `G1`'s **name and description** to what it actually covers (*"our own bare-filename file
   sinks"*), so nobody else reads it as "nothing lands in `{app}`". ⚠️ Do this even if 1 is done.
3. Assert positively at startup that `settings.log_file` is applied **before** the first `LOG_*` —
   i.e. make the ordering the invariant rather than policing the symptom.

⛔ **Do not just add `debug.log` to a denylist.** That closes this filename and leaves the class open;
the next dependency default lands somewhere else and `G1` passes again.

## The general lesson, which is the reason this is a ticket and not a one-liner

⭐ **A gate that reads only our source cannot see a defect produced by a dependency's default.** When a
gate's name describes an *outcome* (*"nothing resolves against `{app}`"*) but its implementation is a
grep over *our code*, the gap is invisible precisely because the gate is green.

⚠️ Sibling of the standing rule that a test must be shown to fail: **`G1` has a negative control and
passes it** — a synthetic bare-filename sink in our source does trip it. So the control proved the gate
works on the input it models, and the model was too small. ⇒ *a negative control validates the
mechanism you thought of, never the one you did not.*

## First step

Decide between 1 and 3 above. 2 is unconditional and takes a minute.
