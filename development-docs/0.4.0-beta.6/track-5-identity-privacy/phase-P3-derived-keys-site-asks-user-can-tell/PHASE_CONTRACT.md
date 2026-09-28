# B5-T5-P3 — Derived keys: the site asks, the user can tell · PHASE CONTRACT

> From `../../PHASE_CONTRACT_TEMPLATE.md` (copied 2026-09-28 from `../../../0.4.0-beta.3/PHASE_CONTRACT_TEMPLATE.md`, read-only).
> Written at G3 by the T5 track agent (relaunch). ⛔ Documents only — no code, no schema, no issue.

**Track:** B5-T5 Identity & privacy · **Tickets:** `../../tickets/TICKET_derived_public_keys_have_no_prompt_and_can_match_across_sites.md` (with `TICKET_derivation_params_unbound_to_origin.md` merged in — closed as duplicate 2026-09-27; its three unique points (a) site B can ask for site A's keyID, (b) the caller survey, (c) the app declares its own BRC-43 level, are carried by rows `P3-A0`, `P3-A11`, §1), `../../tickets/TICKET_createSignature_requires_counterparty.md` · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), T5 track agent, repo head `0777b26` on `0.4.0` · **Platforms:** both (Rust only) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:**
- **Decision 11 — parts 1 + 2.** Part 1: derived `getPublicKey` at levels 1–2 goes through `permission_service::dispatch_scoped_grant`, the same per-site grant `createSignature` has. Part 2: record the requesting site and **warn + log** when two different sites are given the same `(invoice, counterparty)` — refusing waits for real traffic. **Schema change approved (invariant 2)** at decision 2a's testing bar (*"test and test and test every little negative control possible"*). Shape per T3 SCOPE §6a (b): a **child table**, because `derived_key_cache.derived_pubkey` is `UNIQUE` + `INSERT OR REPLACE`. Part 3 → BRC draft only. **Level 0 stays open** — every BRC-100 wallet does this — and it is stated plainly (§1, `P3-A5`, docs row `P3-A12`).
- **Decision 12 — missing `counterparty`.** `createSignature` ⇒ `anyone`; `createHmac` and `verifySignature` ⇒ `self` (`@bsv/sdk` 2.8.7 per-call defaults). ⛔ Not via `handlers.rs :: resolve_counterparty_pubkey`'s `None → self` for signing. **Validate the request before the permission prompt.** Signing change **owner-approved (invariant 3)**; the signing math is unchanged — only the default filling a blank field. The detector watches **`anyone` as well as `self`**.
- **Owner-decided negative controls, cited as such:** (i) two origins, `[2,'login']`, keyID `'1'`, `counterparty:'self'` ⇒ same key, no prompt — the RED that must go GREEN after part 1 (`P3-A0`→`P3-A1`); (ii) part 2's detector disabled ⇒ the second site's request produces **no** warning (`P3-A6`); (iii) a signature made without a counterparty verifies under **`anyone`** and **fails** under **`self`** (`P3-A9`).

---

## 1. Goal

A site that asks for a derived public key at security level 1 or 2 needs the user's per-site grant (asked once, the same grant signing already uses), the wallet notices and logs when two different sites are handed the same derived key, and conforming SDK calls that omit `counterparty` get the SDK's answer instead of a parse error. ⚠️ **Stated plainly:** a level-0 derived key is still handed out without a prompt, in every BRC-100 wallet including ours; any two sites can agree on one. Part 2's log is how we will see whether that is abused.

## 2. Done means

- [ ] Owner-decided RED reproduced on today's build first (`P3-A0`); if it does not reproduce, the ticket closes instead.
- [ ] `get_public_key`'s derived branch (levels 1–2) calls `dispatch_scoped_grant` with `ScopedCall::Protocol`, placed **after** the identity-key branch and only on the derived path. First call from a site with no grant ⇒ 202 `protocol_permission_prompt`; a site holding the grant ⇒ silent; the owner-decided RED goes GREEN (`P3-A1`, `P3-A2`).
- [ ] A login flow (`getPublicKey` then `createSignature`, same protocol) shows **one** prompt, not two (`P3-A3`).
- [ ] Counterparty spellings are normalised **before** the scope is classified: a hex key equal to the `anyone` point or to the user's own master key is treated as `anyone` / `self` — otherwise a site bypasses part 1 by spelling `anyone` as hex (see §5 finding, `P3-A4`).
- [ ] The identity-key path behaves exactly as before (`P3-A5`); level 0 still silent (`SilentProtocolLevelZero`), asserted, not assumed.
- [ ] New migration (next free version — serialized with T1-P3's money-index migration, §11) adds a child table of requesters; every external derived `getPublicKey` (`forSelf` **true and false**, all levels incl. 0) records `(derived key, requesting site)`; a second, different site handed the same derived key ⇒ one `log::warn!` naming the key's first 16 hex, both sites, protocol, level and counterparty kind; the first site's row **survives** the second request (`P3-A6`, `P3-A7`).
- [ ] The detector covers `anyone` as well as `self` (`P3-A8`).
- [ ] `CreateSignatureRequest.counterparty` optional ⇒ `anyone`; `CreateHmacRequest.counterparty` and `VerifySignatureRequest.counterparty` optional ⇒ `self`; the request is parsed and validated **before** `dispatch_scoped_grant` runs (`P3-A9`, `P3-A10`).
- [ ] Caller survey done (K6): which real dApps call derived `getPublicKey` at levels 1–2 before any signature (`P3-A11`).
- [ ] Docs: the misleading comment in `get_public_key` (*"their own protocol/basket gates fire via createHmac/createSignature"*) replaced; `rust-wallet/src/CLAUDE.md` line claiming `sign_action` reads `derived_key_cache` corrected (it does not — §5); the level-0 residual written in the user-facing permission docs (`P3-A12`).

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-PERIM` — identity-key reveal | Silent when `identity_key_disclosure_allowed=1` or session opt-in; otherwise prompts | P3 edits `get_public_key`, which holds this gate. A new gate placed before the `wants_identity_key` branch, or a changed `wants_identity_key` predicate, changes identity behaviour. `P3-A5` + `R-PERIM` T1/T2 |
| `R-PERIM` — key-linkage reveal | — | Not edited; `revealCounterpartyKeyLinkage` shares `resolve_counterparty_pubkey`, which must **not** change (decision 12 says do not reuse it for signing — so it stays as is for its current callers) |
| `R-INTEXT` | Internal never prompts | The wallet UI calls `getPublicKey` header-free (PeerPay, certificates). `dispatch_scoped_grant` returns `Proceed` with no `X-Requesting-Domain`, and the detector must not record internal calls. `P3-A2` internal half |
| `R-TOKENPERM` / T2-P2 | Token spends prompt per action | `create_signature` is edited by both phases — serialize (§11) |
| Invariant 2 (schema) | Owner-approved for **one** child table + (if §12 Q1 chooses it) the parent write changing from `INSERT OR REPLACE` to an upsert | Migration must apply on a beta.4 DB, on a restored-backup DB, and be idempotent; rollback plan in §7 |
| Invariant 3 (crypto) | Owner-approved: default-filling only | `git diff rust-wallet/src/crypto/` empty; `derive_child_*` untouched; `resolve_counterparty_pubkey` untouched |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. Every row below is crypto (key derivation / signing defaults) or schema, so — except the three **owner-decided** controls and the docs row — the RED is designed by a second agent (`../../../RELEASE_CYCLE.md` §4.2).

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P3-A0` | **Step 0, today's build.** Two approved scratch origins A and B each call `getPublicKey({protocolID:[2,'login'], keyID:'1', counterparty:'self', forSelf:true})` ⇒ the **same** key, **no** prompt. Also: B asks for A's keyID/protocol and gets A's key (merged ticket point a) | 👤 **Owner-decided control (decision 11 (i)).** This is the RED; recorded first. If keys differ or a prompt fires, the ticket is wrong and closes | Rust log `R-INTEXT trust: path=/getPublicKey requesting_domain=<A>` then `<B>` + `✅ Derived child public key` with equal hex; absence of any `PermissionDecision` audit line for the call | T2 (two scratch https origins on the dev build) | ⬜ |
| `P3-A1` | After part 1: A's first call ⇒ 202 `protocol_permission_prompt` (`protocolLevel=2`, `protocolName=login`); the owner-decided scenario of `P3-A0` is **no longer silent** for either site until each approves | 👤 **Owner-decided control (decision 11 (i)):** the same two-origin sequence on the pre-change build stays silent — the row is GREEN only if the pre/post pair is shown together | The Rust `PermissionDecision` = `Prompt` with reason in `permission_service/audit.rs` output for `/getPublicKey`, **not** the modal's appearance (R-PERIM SUBJECT rule) | T1 (handler test with a scratch DB + header) + T2 | ⬜ |
| `P3-A2` | A site holding the `[2,'login']` grant ⇒ silent (`SilentScopedGrantExists`); an internal (header-free) derived call ⇒ `Proceed`, nothing recorded; level-1 behaves like level-2 for a site without a grant | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Audit reason string per call; for internal, the `<none:internal>` trust line and zero requester rows | T1 + T2 | ⬜ |
| `P3-A3` | Login flow on a real BRC-100 site: one prompt at `getPublicKey`, `createSignature` then silent. K6 recorded (prompts added vs moved) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Count of `protocol_permission_prompt` 202s in the Rust log for the site's login = 1; the site logs in | T2 👤 (2 real BRC-100 sites) | ⬜ |
| `P3-A4` | Counterparty normalisation: `counterparty:"<hex of G>"` is classified as `anyone`, `counterparty:"<user master pubkey hex>"` as `self` ⇒ both prompt like their named forms; a genuine third-party hex counterparty keeps today's `SilentCounterpartyDefault` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Audit decision + reason for each spelling; the derived keys for `"anyone"` and `"<hex G>"` are byte-equal (proves they are the same request) | T1 | ⬜ |
| `P3-A5` | Identity-key path unchanged: `getPublicKey({identityKey:true})` from a site with V17 on ⇒ silent master key; with V17 off ⇒ `Prompt{identity_key_reveal}`; a level-0 derived call ⇒ `Silent{SilentProtocolLevelZero}` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Audit decisions; `matrix_c.rs` engine tests unchanged and green (`git diff crates/hodos_permission_engine` empty) | T1 + T2 | ⬜ |
| `P3-A6` | Part 2: A then B (both approved, both granted) request the same derived key ⇒ exactly **one** `warn` line on B's request naming both sites; two requester rows; A's row still present after B's request | 👤 **Owner-decided control (decision 11 (ii)):** detector disabled (its call removed in a scratch build) ⇒ B's request produces **no** warning and no second row. Must be observed together with the GREEN | The wallet log **file** (`RUST_LOG=hodos_wallet=debug`) warn line + `SELECT` on the child table in the scratch profile's DB | T1 + T2 | ⬜ |
| `P3-A7` | Schema: migration applies on (i) a fresh DB, (ii) a copy of a **beta.4** DB, (iii) a DB produced by restoring a backup, and (iv) the migrated DB still opens under the **previous** binary (downgrade-open); re-running is a no-op; `derived_key_cache` rows unchanged by it; the first site's requester row **survives** a second write of the same derived key (the `INSERT OR REPLACE` trap: a REPLACE deletes the parent row, and with `PRAGMA foreign_keys=ON` an `ON DELETE CASCADE` child would be wiped — or the REPLACE would fail) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Row counts per table before/after on each DB copy; `PRAGMA foreign_key_check` empty; the schema version row | T1 | ⬜ |
| `P3-A8` | Detector covers `anyone`: A and B each request `[1,'x']`, keyID `'1'`, `counterparty:'anyone'`, `forSelf:false` (and `true`) ⇒ warn on B, as for `self` | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Warn line names `counterparty=anyone`; requester rows keyed on the derived key | T1 | ⬜ |
| `P3-A9` | `createSignature` with **no** counterparty succeeds and the signature **verifies under `anyone`**: `@bsv/sdk` 2.8.7 `ProtoWallet(new PrivateKey(1)).verifySignature({… counterparty: <wallet master pubkey>})` ⇒ `valid:true` | 👤 **Owner-decided control (decision 12 (iii)):** the **same** signature checked against the `self`-derived key (`ProtoWallet(<wallet master key>)` / our `/verifySignature` with `counterparty:'self'`) ⇒ **fails**. Both verdicts recorded together — pins the default, not "no error" | The **SDK's** verdicts (an independent verifier), not our own `/verifySignature` alone | T1 (Node script vs the running dev wallet, internal path) | ⬜ |
| `P3-A10` | `createHmac` and `verifySignature` with no counterparty default to `self` (HMAC equals the explicit-`self` HMAC; a self-signed signature verifies); a malformed body (bad `protocolID`, missing `data`) ⇒ 400 **with no 202 prompt first** (validate-before-prompt) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Byte-equality of the two HMACs; for validation, the absence of any `PermissionDecision` audit line before the 400 | T1 | ⬜ |
| `P3-A11` | Caller survey (merged ticket point b): for Xanadu (`[0,"xanaverse"]`, keyID `"1"`, `self` — known) and 2–3 other BRC-100 sites, the derived-key calls seen (level, counterparty kind, `forSelf`) are tabled; ⚠️ note which would now prompt | Measurement row. Control: the collector run on a scratch page that makes one known call must list it (shows the collector sees calls) | The Rust log's `BRC-43 invoice:` lines grouped by `requesting_domain` | T2 👤 | ⬜ |
| `P3-A12` | Docs: the `get_public_key` comment no longer claims derived keys are gated elsewhere; `rust-wallet/src/CLAUDE.md`'s "`sign_action` uses this cache" corrected; user-facing permission doc states the level-0 residual in one plain paragraph | `grep` for the old comment text / the stale CLAUDE.md sentence ⇒ >0 before, 0 after | The three files' text | T0 | ⬜ |
| `P3-A13` | Boundary: `R-PERIM` T1 + T2 (identity-key prompt), `R-INTEXT` both halves, `R-TOKENPERM` if T2-P2 has landed, `cargo test` green, `git diff --stat rust-wallet/src/crypto/` empty | Per `REGRESSION_SET.md` / `REGRESSION_ADDITIONS.md` | Per those docs | T1–T2 | ⬜ |

**Two-sided rows:** `P3-A1` (no grant ⇒ prompt) ⇄ `P3-A2` (grant ⇒ silent). `P3-A6` pairs warn-on-B with no-warn-on-A's-repeat (the same site asking twice is **not** a cross-site match — a detector that warns on every repeat fails it). `P3-A9` is two-sided by construction.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/handlers.rs :: get_public_key` | ✅ | `wants_identity_key = identity_key \|\| protocol_id.is_none() \|\| key_id.is_none()` → `dispatch_privacy_perimeter`; derived branch has **no** gate; misleading comment present; `forSelf=true` ⇒ in-memory insert + `INSERT OR REPLACE INTO derived_key_cache` |
| `rust-wallet/src/handlers.rs :: peek_scoped_grant_scope_protocol` | ✅ | Maps `self` / `anyone` / `""` ⇒ `None`; a hex string ⇒ `Some(hex)` **without** comparing it to `G` or the master key — the normalisation gap `P3-A4` closes. Accepts string `protocolID` as level 2, which `get_public_key` rejects with 400 — the validate-before-prompt case |
| `rust-wallet/src/permission_service/request_gate.rs :: dispatch_scoped_grant`, `ScopedCall::Protocol`, `ScopedCall::call_kind` | ✅ | `counterparty: Some(_)` ⇒ `CallKind::CounterpartyUse`; header-free ⇒ `Proceed` |
| `rust-wallet/crates/hodos_permission_engine/src/matrix_c.rs :: decide_scoped_grant` | ✅ | `CounterpartyUse` ⇒ `SilentCounterpartyDefault` **first**; then level 0 ⇒ `SilentProtocolLevelZero`; then `scoped_grant_exists`. ⚠️ So part 1 prompts only for `self`/`anyone`/omitted counterparty at levels 1–2 — a site-supplied hex counterparty stays silent, **unlike** wallet-toolbox (which prompts at 1–2 regardless). Carried as decided ("the same grant `createSignature` has"); the divergence is stated in §12 Q2. Engine not edited |
| `rust-wallet/src/handlers.rs :: resolve_counterparty_pubkey` | ✅ | `None ⇒ self`. ⛔ Not reused for signing defaults; not edited |
| `rust-wallet/src/handlers.rs :: CreateSignatureRequest`, `create_signature` | ✅ | `counterparty: serde_json::Value` (required — the observed `missing field` error); `dispatch_scoped_grant` runs **before** the body is parsed into the struct (the ticket's "prompt before a call that was always going to fail") |
| `rust-wallet/src/handlers.rs :: CreateHmacRequest`, `create_hmac`; `VerifySignatureRequest` (`counterparty: String`), `verify_signature`; `VerifyHmacRequest` (`counterparty: serde_json::Value`), `verify_hmac` | ✅ | `verify_hmac` has the same required field; decision 12 does not name it — §12 Q3 |
| `rust-wallet/src/database/migrations.rs` — `derived_key_cache` (`id INTEGER PRIMARY KEY AUTOINCREMENT, derived_pubkey TEXT NOT NULL UNIQUE, invoice, counterparty_pubkey, created_at`); `migrate_v24_to_v25` (latest) | ✅ | The next migration is V26 — T1-P3's money-index table also needs one ⇒ serialize |
| `rust-wallet/src/database/connection.rs` — `PRAGMA foreign_keys=ON` | ✅ | On in production ⇒ the CASCADE-on-REPLACE trap in `P3-A7` is live, not theoretical |
| `rust-wallet/src/main.rs :: AppState.derived_key_cache`, `DerivedKeyInfo` | ✅ | ⚠️ **Finding: write-only.** Nothing in `rust-wallet/src` reads the in-memory map or the table (grep 2026-09-28: the only references are the write in `get_public_key`, the struct, the initialiser and the `CREATE TABLE`). `rust-wallet/src/CLAUDE.md`'s *"`sign_action` uses this cache to find the correct BRC-42 key for PushDrop signing"* is stale. ⇒ T3 SCOPE §6a's "keep the PushDrop-signing lookup working" constraint has no live subject; the design is freer than §6a assumed (§12 Q1) |
| `rust-wallet/src/backup.rs :: BackupPayload` | ✅ (per T3 SCOPE §6a) | `derived_key_cache` not carried ⇒ the new child table must not be added to backups either; the detector restarts empty after restore (§11) |
| `cef-native/src/core/HttpRequestInterceptor.cpp` — `protocol_permission_prompt` extra-params builder, `openProtocolPermissionPromptModal` | ✅ | Generic over endpoint — **no C++ change expected**. `extractProtocolScope` (excludes `/getPublicKey`) has **no caller** — dead since 2.6-G; reported, not deleted |

## 6. Out of scope

- Part 3 (wallet adds the origin to the keyID) — BRC draft `originator-scoped-authentication-keys` only.
- The owner's level-0 idea (silence level 0 only when a higher grant exists) — decided after part 2's data (decision 11).
- **Refusing** on a cross-site match — warn + log only this release.
- Changing `SilentCounterpartyDefault` (Fix #3) in the engine — see §12 Q2; not changed without the owner.
- A UI for the detector's log (owner reads it via the wallet log / a query this release).
- `revealCounterpartyKeyLinkage` / `revealSpecificKeyLinkage` defaults.
- Tempted by: deleting the write-only `derived_key_cache` altogether — no; report it, the owner decided the detector hangs off it.

## 7. Rollback

Three commits: (1) part 1 + normalisation, (2) decision 12 defaults + validate-before-prompt, (3) migration + detector. (1) and (2) revert cleanly. (3) reverts the code; the migration is **additive** (a new table, plus at most the parent write changing to an upsert), so an older binary on a migrated DB ignores the child table — stated and tested as `P3-A7` (iv) *downgrade-open* before landing.

## 8. Pre-mortem (adversarial review — before)

| Story: it shipped and failed because… | Row that catches it |
|---|---|
| The child table used FK + `ON DELETE CASCADE` on `derived_key_cache`, and the parent's `INSERT OR REPLACE` deleted the row on every repeat request — the evidence part 2 exists to keep was wiped each time (or, without CASCADE, the REPLACE started failing and logging `DB cache WRITE FAILED`) | `P3-A7` survive-the-second-write clause |
| The detector hung off `derived_key_cache`, which only records `forSelf=true` — a site asking with `forSelf:false` was never seen | `P3-A8` runs both `forSelf` values |
| A tracker spelled `anyone` as the hex of G and sailed through part 1 as a "specific counterparty" | `P3-A4` |
| The new gate went before the identity-key branch, so `getPublicKey({})` started asking a protocol prompt instead of the identity prompt | `P3-A5` |
| The detector warned on the same site's repeat call — noise the owner learns to ignore | `P3-A6` two-sided clause |
| `createSignature`'s default was implemented by calling `resolve_counterparty_pubkey(None)` — signs as `self`, error-free, and every conforming verifier rejects it | `P3-A9` (owner-decided, SDK verdict) |
| The body was made optional in the struct but `peek_scoped_grant_scope_protocol` and the handler parse differently, so a body the gate reads as "no scope" skips the prompt and the handler signs | `P3-A10` + a T1 test that feeds every body shape to **both** parsers |
| Migration numbering collided with T1-P3's money index (both "V26") | §11 serialize; `P3-A7` on the post-T1 DB |
| The login prompt now reads "sign" wording while the call is a key fetch — the user cannot judge it (a popup the user can't judge is not a mitigation) | `P3-A3` human reads the modal; wording edge to T6-P7 |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | Dev wallet `31401` |
| macOS | `P3-A1`, `A6`, `A7` (migration on a mac beta.4 DB copy), `A9` | Rust only, same code; the mac wallet is a separate binary and DB, and the migration must be seen applying there. No C++ ⇒ no relay rebuild note beyond naming the migration |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~2 h**: login on 2 real BRC-100 sites reading the prompt (`P3-A3`, ~40 min), the caller survey sitting (`P3-A11`, ~30 min), §12 answers (~30 min), reading the first detector log after a week of use (~20 min) |
| Human-bound rows | `P3-A3`, `P3-A11` |
| Unknowns (K) — uncertainty, not difficulty | **K6** how often real dApps fetch derived keys at levels 1–2 before signing (prompts added vs moved) · **K7** level 0 stays open — decided; its abuse rate is what part 2 measures · **K21 (new)** SQLite's behaviour for FK actions on a REPLACE-deleted parent in our exact pragma set — settled by a T1 test before the table shape is fixed |

## 11. Cross-track edges

| Direction | Other phase | What is given / needed |
|---|---|---|
| gives | **B5-T3a / T3b** | ⭐ `derived_key_cache` **and** the new requester table are **not** in backups/exports (T3 SCOPE §6a): the detector **restarts empty after a restore**. T3's restore report states it; if the detector ever refuses, this must be revisited |
| serialize | **B5-T1-P3** | Both add a migration; T1 lands first (decision 2a order); P3 takes the next free version and runs `P3-A7` on a post-T1 DB |
| serialize | **B5-T2-P2** | Both edit `handlers.rs :: create_signature` (T2's token binding; this phase's default + validation). T2's contract says T5 lands first or both rebase — agreed |
| needs | **B5-T6-P7** (brand/prompt surfaces) | The `protocol_permission_prompt` modal now also fires for a key fetch — its wording must make sense for "share a key" as well as "sign" |
| needs | **B5-T0 Build 1** | `P3-A3` / `P3-A11` real-site rows on the new engine |
| gives | BRC draft `originator-scoped-authentication-keys` | Part 2's data (level-0 and cross-site counts) is the evidence the draft needs |

## 12. Open questions for the owner

1. **Table shape (invariant 2 — the approved change, exact form).** Recommendation: a child table `derived_key_requesters` keyed `(derived_pubkey, requesting_domain, for_self)` with `protocol_level`, `invoice`, `counterparty_kind` (`self`/`anyone`/`other`), `first_seen`, `last_seen`, `request_count` — joined to the parent **by `derived_pubkey`** (UNIQUE, stable across REPLACE), **no `ON DELETE CASCADE`**, and the parent write changed from `INSERT OR REPLACE` to `INSERT … ON CONFLICT(derived_pubkey) DO NOTHING` so the parent id is stable too. `forSelf=false` requests get a requester row even though the parent only caches `forSelf=true` (the parent is write-only today — §5 — so no signing path is affected). Two readings exist: (a) this; (b) the literal `cert_field_permissions` pattern (FK on `id` + CASCADE), which the REPLACE trap makes unsafe unless the parent write changes. Recommend **(a)**. Which?
2. **Hex counterparties stay silent (engine Fix #3).** Part 1 via `dispatch_scoped_grant` prompts only for `self`/`anyone`/omitted at levels 1–2; a site-chosen hex counterparty is `SilentCounterpartyDefault` — wallet-toolbox prompts there too. This is a **deviation from the reference**, carried because decision 11 says "the same grant `createSignature` has". Part 2's detector still sees any cross-site match regardless of spelling (it keys on the derived key). Keep as decided and state it? Recommendation: **yes**, plus `P3-A4`'s normalisation so `anyone`/`self` cannot be re-spelled as hex.
3. **`verifyHmac`** has the same required `counterparty` (SDK default `self`), but decision 12 names only `createHmac` and `verifySignature`. Include `verifyHmac` ⇒ `self` under the same approval? Recommendation: **yes** (same family, verify-only, no key revealed). Yes / no?
4. No evidence that a G2 decision is wrong. The write-only finding (§5) makes T3 §6a's constraint moot, it does not contradict decision 11.

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
