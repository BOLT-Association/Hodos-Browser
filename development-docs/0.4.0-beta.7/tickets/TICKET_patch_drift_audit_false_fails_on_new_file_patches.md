# TICKET — the patch drift audit HARD-FAILS on a clean tree for any patch that creates a file

**Opened** 2026-09-29, during the `v0.4.0-beta.5` engine security refresh (Windows, `.255` tree). **Severity: LOW — instrument defect, fails closed.**
⛔ Fix as **its own commit**, reason in the harness notes, re-run with a negative control (root `CLAUDE.md` working rule 6). Not fixed in the release that found it.

## What happened (measured)

`development-docs/DevOps-CICD/scripts/cef_patch_drift_audit.sh` on the freshly checked-out `.255` tree (fork `7d50c1cab`, no patches applied yet) printed:

```
AUDIT_FAIL: hodos_farble_session_cache targets a missing file: .../execution_context/hodos_session_cache.cc (upstream rename/delete?)
AUDIT_FAIL: hodos_farble_session_cache targets a missing file: .../execution_context/hodos_session_cache.h  (upstream rename/delete?)
AUDIT_RESULT: HARD FAIL — DO NOT START THE BUILD (exit 1)
```

Both files are **created** by that patch (`new file mode 100644`, `--- /dev/null`). The section-2 target check (≈ line 256) takes every `+++` path and drops only `/dev/null`, so a created file's `+++` path is checked for existence **before** the patch that creates it has run. On earlier runs the tree still carried the previous build's applied patches, so the files existed and the check passed. That's why it never showed until a tree was reverted.

Same run, section 3 (the real apply check): **120 would apply cleanly, 1 already applied (`runhooks.patch`, applied by the checkout itself), 0 offsets, 0 failures.** `git apply --check` of `hodos_farble_session_cache.patch` alone: clean. After the codec gate's `gclient_hook.py` (with `HODOS_FARBLING=1`): patcher `121 patches total (120 applied, 1 skipped, 0 failed)`, and all 7 `hodos_*` patches reverse-check as applied.

## Fix

In the target-file check, skip the `+++` path of any file section whose `---` is `/dev/null` (a created file). Optionally assert the opposite for those: the created file must **not** already exist unless the patch is already applied.

**Negative control:** delete one line of context from a patch that **edits** an existing file ⇒ still exit 1; rename a **created** file's target directory out of the tree ⇒ still caught by section 3's apply check.

## Also found

`cef_gn_args_gate.sh` (the codec gate) runs `gclient_hook.py`, which applies the patches, but it does **not** export `HODOS_FARBLING=1`. Run as written, it would apply the upstream patches without the Hodos privacy set. The build tree's `p3_build_gate2.sh` (2026-08-05) does export it; the repo copy predates the farbling patches. On 2026-09-29 it was run with `HODOS_FARBLING=1` set in the environment. Same fix discipline: its own commit.
