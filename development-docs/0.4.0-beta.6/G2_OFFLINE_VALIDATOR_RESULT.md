# G2 offline test: Hodos's real wallet responses vs the hardened `@bsv/sdk` validators

**Run:** 2026-09-30 · **By:** Claude (Opus 5.5) · **Subject:** the **dev** wallet (`HODOS_DEV=1`, port 31401), live responses · **Validator:** `@bsv/sdk` **2.8.11** (npm, 2026-09-29), `validateWalletResult` + `snapshotWalletResultRequest`, the call every `WalletClient` substrate makes on every result (`WalletClient.js :: validatedResult`, `substrates/window.CWI.js`, `substrates/HTTPWalletJSON.js`).
**Reproduce:** `g2-validator/`: `npm i`, then `node validate.mjs` (21 pass / 6 fail) and `node controls.mjs` (all `OK`). `record.mjs`/`record2.mjs` re-record against a running dev wallet.

**Why the SDK result is what a dApp sees:** our `window.CWI` shim resolves the Rust JSON body unchanged (`CWIShimScript.h :: __hodos_walletResponse`), and the 3321/2121 bridges return the same bodies. An upgraded dApp's `WalletClient` validates that body before the dApp sees it. A failure means the dApp's call **throws**, with no change on our side.

## ⭐ The headline question: the 1000-sat service fee — ✅ does NOT break

`createAction` (`noSend`, `randomizeOutputs:false`, one requested output) → tx outputs `[requested 1000 | service fee 1000 | change]` → **PASS**.
The validator (`validateCreateActionTransaction`) requires every requested output at its requested amount and script (and, with `randomizeOutputs:false`, **at its requested index**), matching `version`/`lockTime`/sequences, and value conservation. **Extra outputs are allowed.** Our order (request → fee → change) satisfies the position rule.

**Negative controls (the check really engaged on our tx):** request 1001 sats → FAIL · requested output shifted to index 1 → FAIL · txid swapped → FAIL · request `lockTime` 5 → FAIL. Each fails for the stated reason (`controls.mjs`).

⚠️ `completeBoundAction` (TSA-082/-278/-401) is a **separate, opt-in** SDK helper, not on the default result path; not exercised here.

## ❌ Confirmed breaks (an upgraded dApp's call throws)

| # | Call | Failure | Cause (code) | Who is hit |
|---|---|---|---|---|
| B1 | `listActions` | `actions[0].status: expected a supported action status` | `action_storage.rs :: TransactionStatus::to_action_status` maps to legacy words: completed→`confirmed`, unproven→`unconfirmed`, nosend→`aborted`, unsigned/unprocessed/nonfinal→`created`, sending→`signed`. **Only `failed` survives** | Every `listActions` that returns any non-failed action |
| B1b | `listActions` + `includeInputs`/`includeOutputs` | after statuses are fixed: `inputs[].sourceOutpoint` missing; `outputs[].lockingScript` / `basket` missing | Inputs are `{txid,vout,satoshis,script}`, outputs `{vout,satoshis,script,address}`, not the BRC-100 shapes. Passes once mapped (`la.mjs`); extra keys are tolerated | Same, when the flags are set |
| B2 | `listCertificates` | `totalCertificates: expected an integer` | `certificate_handlers.rs :: ListCertificatesResponse` lacks `#[serde(rename = "totalCertificates")]` (its two `discover*` siblings have it). Passes once renamed (control) | Every `listCertificates` call |
| B3 | `createAction` `signAndProcess:false` (no dApp inputs) | `tx: expected a complete, exactly framed BEEF` | We ignore `signAndProcess:false` unless a dApp input lacks an unlocking script: we return `txid` + an **unsigned raw tx** (153 bytes, not BEEF) instead of `signableTransaction`. The row is stored `unsigned` | dApps using explicit two-phase signing |
| B4 | `verifySignature` / `verifyHmac` returning `{valid:false}` | `valid: expected true` | The SDK treats "invalid" as an error, not a verdict (TSA-040) | ⚠️ **Low:** the dApp already got a throw from the SDK's own verifier in this case. Only the error type and message differ |

## Not a break, but measured

| Item | Result |
|---|---|
| Error bodies (`{"error":…}`, `{"status":"error","code":…}`) | Via `window.CWI` any rejection is a throw (unchanged). Via `HTTPWalletJSON` a 400 without the BRC-100 `isError` envelope becomes a generic `HTTP status 400` error: **typed errors lost** (`WERR_INSUFFICIENT_FUNDS`, `WERR_REVIEW_ACTIONS`). A degradation, not a break |
| `getVersion`, `getNetwork`, `getHeight`, `isAuthenticated`, `getPublicKey`, `createSignature`, `createHmac`, `encrypt`, `decrypt`, `signAction` (noSend), `listOutputs` (tag, entire transactions, locking scripts) | PASS |

## Found on the way (not G2 validator results)

| Item | Evidence | Note |
|---|---|---|
| `verifySignature` rejects `counterparty:"self"` | `400 "Invalid counterparty public key hex: Invalid character 's'"` | BRC-100 allows `self`. A conformance bug independent of the SDK |
| `listOutputs` refuses basket `default` | `400 "Basket name is reserved…"` | Probably deliberate; noted only |
| `noSend` outputs flagged `spendable=0`, `"phantom: parent tx not on chain"` within 3 s of creation | dev DB rows for `2aa435e5…` | Written deliberately by our code, so **not** a rule-7 impossible row. But a dApp chaining `noSendChange` into its next `createAction` may find that change unusable. Owed: check against the `sendWith` flow |
| `abortAction` on a `noSend` action returns `aborted:true` **without releasing anything** | code reading: `nosend`→legacy `Aborted` → early `return` in `handlers.rs :: abort_action` | Sibling of TSA-260 (G3 abort) |

## Gaps (not measured)

- `listCertificates` / `proveCertificate` with a **non-empty** certificate list: the dev wallet holds none. Only the envelope was tested.
- `createAction` with **dApp-supplied inputs** (the `signableTransaction` path ordinals and token dApps use): no spendable fixture input left. Owed before closing G2.
- TSA-289 (tag filter returns the whole basket): the probe basket was emptied by the phantom check before a no-match query ran.
- `internalizeAction` success path.

## Residue (declared, per working rule 7)

Dev wallet only. None of the probe calls broadcast. Rows, label and basket `g2probe`:
- tx **986** `2aa435e5…` (`nosend`): **failed by the wallet at 19:41 UTC, input `a9a556e4…:4` restored** (checked in the DB).
- tx **987** `2859b885…` (`unsigned`) was **replaced by tx 988** `2ab8d10e…` when `signAction` signed it (`nosend`). Input `d37844d7…:2` stays reserved until `NOSEND_TIMEOUT` (10 min) fails it.
- `abortAction` was deliberately **not** called (TSA-260).

⚠️ **Indirect spend:** about 1 minute after the probe, the wallet's own `TaskBackup` saw the DB hash change (very probably from the probe rows) and broadcast a routine on-chain backup, tx **989** `23221252…` (`backup-23221252`), **13,002 sats mining fee** (~$0.003). This is designed behaviour, not a defect, but the test was not money-free. A future re-record should expect it.
