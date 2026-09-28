# B5-T3b-P5 — Move a wallet in and out of Hodos: standard export/import files and a recovery phrase from any wallet · PHASE CONTRACT

**Track:** B5-T3b Sync & portability · **Tickets:** `../../tickets/TICKET_brc140_key_shares_vs_bip39.md` (decision only, no build — SCOPE Q5) · **Status:** ⬜ NOT STARTED
**Opened:** 2026-09-28 · **Author:** Claude (Opus 5.5), G3 agent for T3b · **Platforms:** both (Rust wallet + the wallet overlay's React pages) · **GitHub issue:** *(opened at G6)*
**Standard:** `../../../0.4.0-beta.3/HARNESS.md` (inherited, read-only) + `../../HARNESS_DELTA.md` + `../../REGRESSION_ADDITIONS.md`.
**G2 decisions carried:** **5** (same wallet, two formats — the file is strict BRC-38/39 from the **full** DB; the on-chain copy never claims BRC-38; ⭐ the export ⇄ on-chain round-trip test), **6** ⚠️ PROVISIONAL (tiered phrase import ①–④ with fall-through; BIP-32 recovery code stays; BRC-157 for new wallets **not** decided), **2 / 2a** (money is positively marked; the money index is derived and rebuilt, never exported), **7** (anything unidentified is held, shown, never auto-spent; import never fails outright on it), T1 Q5 (a BIP-32 hit is swept into a BRC-42 address), `../SCOPE.md` §0, §2.3, §2.4, §6a, Q1/Q5 (approved recommendations).

> ⭐ **Decision 5's required test is owned HERE.** T3a-P1 builds the instrument (`canonical()` comparator, mock chain, H2 manifest) and says so in its §11: *"T3b-P5 owns running it (it is the first phase where a strict `.brc39` file exists)."* Row `P5-A4` below.

---

## 0. Fresh read — 2026-09-28 (only what changed since `../SCOPE.md` §2, 2026-09-25, and T3a-P0 §0 this morning)

T3a-P0 §0 re-read BRC-38/39/40, BRC-155, BRC-157, BRC-177, wallet-toolbox v2.14.3 `storage/portable` and the Go toolbox **today**; nothing there is repeated. What this contract adds:

| Source | Fetched | Finding | Consequence for P5 |
|---|---|---|---|
| BRC-75 `key-derivation/0075.md` | BRCs HEAD `8f36bdf` (2026-09-28); last content change `932cc22` **2023-11-18** | Root = `SHA-256(BIP-39 seed)`. ⚠️ Its only printed example derives from **random** entropy — **no reproducible test vector** | Tier ② needs a vector we **generate** from the SDK and cross-check against HandCash's `vault.ts :: rootKeyFromMnemonicBrc75` (`f8c874f`) — two independent implementations must agree before the row counts |
| BRC-157 `key-derivation/0157.md` | same | Unchanged. ⭐ **Has a vector:** `legal winner thank year wave sausage worth useful legal winner thank yellow` ⇒ root `m/0'/0'` = `27e442c8015fc055789d6628f3b30461e8b2598aff74dc87ceef00dd8e670e55`. Also MUST-reject rules for a phrase whose entropy is zero (`abandon ×11 about`) | Tier ② BRC-157 has a spec vector; P5-A11 pins it. The zero-entropy reject rule applies only to wallets **conforming** to 157 — Hodos is not (root `m`), so we reject it only on the 157 branch |
| wallet-toolbox `storage/portable/index.ts :: validateBRC38`, `importBRC38` | ts-stack, toolbox **2.14.3**, file last touched `b3155fa` 2026-09-22 | Beyond SCOPE §2.4: `validateBRC38` checks 12 named table arrays and **does not reject unknown top-level keys or unknown row fields**; `restoreBRC38` then does `insertOutput({ ...row })`, so an **extra field inside a row may break the reference insert**; `importBRC38` throws `BRC-38 chain mismatch` if `sourceStorage.chain` ≠ target. The importer never checks that anyone controls `user.identityKey` — that is left to the wallet | Hodos-only data (permissions, address high-water mark) goes in a **separate top-level key**, never as extra fields inside the 12 tables (P5-A2). The identity check is **ours** to do (P5-A6) |
| HandCash `src/wallet/phraseSweep.ts` | HEAD `f8c874f` (2026-09-28); file last changed `d749709` 2026-09-22 | ⚠️ **SCOPE §2.3 summarised this as "tries BRC-75 and legacy-HD". It does more:** it scans BRC-75, legacy-HD `m`, five Yours/RelayX/Twetch BIP-44 branches, **and** 40 "Centi" addresses (`m/44'/145'/0'/{0,1}/{0..19}`) — and it scans **all of them every time**: *"a phrase may have been used by more than one wallet, so finding a Yours/HandCash hit is not evidence that its Centi branches are empty. Recovery favours completeness."* It **sweeps** (coins move into the active wallet, signed by the foreign key); it never adopts a foreign root or imports data. It keeps funding (≥ a sweep floor) and 1-sat items apart, and migrates items separately | Evidence bearing on decision 6's triage — §12 Q2. And ⚠️ **"Centi" is not Centbee**: HandCash's path is `m/44'/145'/0'`, ours (`recovery.rs :: ExternalWalletConfig::centbee`) is `m/44'/0/{0,1}/{i}` with the 4-digit PIN as BIP-39 passphrase |
| BRC PR **#283** — BRC-191 *Thoughts on Identity, Privacy and Recovery on the Metanet* (Ty Everett, `opinions/0191.md`) | open, updated 2026-09-28 | An opinion, not a standard. Says BRC-38 **excludes root-key/profile material**; lists the portability questions an export must answer (*can another implementation read it, reconstruct relationships, keep certificates usable, keep assets' meaning?*); names as a failure mode *"a user should not have to reveal a recovery secret to obtain records already belonging to them"* | ⭐ Our private `.hodos-wallet` export carries the **mnemonic** (`backup.rs :: BackupPayload.mnemonic`); the BRC-38 file must not (P5-A3). BRC-191's question list is the checklist behind the capability diff (P5-A7/A8) |
| Two shipping importers (T3a-P0 §0: peacock-wallet PR #30 merged 2026-09-24; bsv-browser PR #152 open) | via T3a-P0 | **Preview → isolated restore → identity/network compatibility check → explicit activation** | Adopted as P5's import shape (P5-A7, A9). Not re-derived |

**What the fresh read changed:** (1) the HandCash triage evidence (§12 Q2); (2) where Hodos-only data may live in a BRC-38 file (P5-A2); (3) no BRC-75 vector exists — we generate and cross-check; (4) a concrete export defect found in our own code while checking what a conforming wallet can spend — P5-A8.

## 1. Goal

A user can export their Hodos wallet as a standard `.brc39` file that the reference importer accepts, import a standard file from another wallet after being told exactly what will and will not carry over, and recover funds from a recovery phrase made by another wallet — and Hodos never invents, drops or silently spends a coin in any of these moves.

## 2. Done means

- [ ] **Step 0 (no code, may start now — see §11 "May P5 go earlier?"):** BRC-38 vectors generated by the **reference** exporter (toolbox 2.14.3) from P1's fixture wallets, checked in as append-only fixtures; BRC-75 vector generated and cross-checked by two implementations; §12's owner questions answered; the BRC-140 decision recorded in the ticket (decision only, build no earlier than beta.7)
- [ ] `.brc39` **export** from the full local DB: accepted by the reference `importBRC39` in `restore` mode (P5-A1); no nulls, no dangling references (A2); no mnemonic or root secret in the file (A3); outputs a conforming wallet cannot spend are named as such (A8)
- [ ] `.brc38` / `.brc39` **import**: preview before any write, isolated restore, identity check before any row, explicit activation (A6, A7, A9); every imported output goes through T1's classifier before the money index is rebuilt (A5); a foreign table or field we do not use survives a re-export unchanged (A10)
- [ ] ⭐ **Decision 5's round-trip:** one wallet ⇒ `.brc39` file import **and** seed-only on-chain restore ⇒ two wallets identical to each other and to the source, money index included (A4)
- [ ] **Tiered phrase import** ①–④ with fall-through, reusing today's recovery code and T1-P6's sweep; the BIP-32 recovery code (`recovery.rs`) and the Centbee path stay compiled and tested even if the UI entry moves (A10–A14)
- [ ] Interop sitting: HandCash `.brc39` → Hodos (T8) and Hodos `.brc39` → HandCash (T9), each result recorded honestly — a refusal for a stated reason is a result, not a failure to hide (A15)

## 3. Invariants preserved

| ID | Invariant | Why this phase could break it |
|---|---|---|
| `R-CLASSIFY` | classification reaches every ingest path | ⭐ This phase **adds two ingest routes** (BRC-38 import; tier ②/③ sweeps landing coins). A route that inserts outputs without the classifier re-creates the defect the release exists to close. Both routes go on T1's route list |
| `R-NOSPEND` / `R-DUST` | nothing automatic spends an unclassified or 1-sat output | A phrase sweep that treats every UTXO at a foreign address as funding **burns ordinals**. `recovery.rs :: split_token_reserved` already separates them for the Centbee sweep; HandCash keeps funding and items apart. A14's RED |
| `R-RESTORE` | fail-closed survives recovery | File import is a recovery path. Both halves (not spendable / not lost) must hold for an imported unclassifiable output |
| Invariant 1 | no private key or phrase reaches web content | The export file must not carry the phrase (BRC-191); the phrase is typed only inside the wallet overlay, as today (`WalletPanelPage.tsx`) |
| Invariant 2 | no schema change without asking | None planned: Hodos-only data travels in the file, not in new columns. ⚠️ If §12 Q1 is answered "adopt the foreign root", a per-wallet root-convention field **is** a schema change — asked there |
| Invariant 3 | no crypto/derivation change without asking | Tier ② computes BRC-75 and BRC-157 roots — new derivation code paths for **foreign** phrases. Hodos's own root stays `m` (decision 6). §12 Q1 |
| Service fee | every outgoing tx carries 1,000 sats | Tier sweeps are spends. T1-P6 decides the sweep's fee treatment (T1 Q5: "a sweep is a normal spend — network fee + the 1,000-sat service fee"); P5 reuses T1-P6's builder, never a second one |
| Production isolation | dev never touches `HodosBrowser/` | All import tests run on scratch profiles (`HARNESS_DELTA.md` §1.2) |
| Gold pill | — | Untouched: no dApp payment path |

## 4. Evidence table

⛔ No empty RED or SUBJECT cells. A green result is reported with its red half or not at all.
⛔ Money, schema and crypto rows: RED designed by a **second agent** (`../../../RELEASE_CYCLE.md` §4.2) — placeholder below. Record who designed it. Rows whose RED I designed are format, disclosure or UI rows.
⭐ **SUBJECT obligation for this release** (`HARNESS_DELTA.md` §1.1): name the **output** — txid:vout, satoshis, basket — not just the process.

| ID | 🟢 GREEN — must be true | 🔴 RED — must be *seen* to fail, and how | 🎯 SUBJECT — proves the right thing was measured | Tier | Result |
|---|---|---|---|---|---|
| `P5-A0` *vectors are the reference's* | BRC-38 vectors are produced by the reference `exportBRC38Json` (toolbox 2.14.3, pinned) from P1's fixtures A and B loaded into a toolbox SQLite; checked in append-only with the toolbox version in the filename | Insert one `null` into a vector and feed it to the reference `importBRC38` ⇒ it must throw its own `BRC-38 … must omit null values`. A vector the reference accepts in any state was not produced by the reference | The reference code's own error string and version, pasted into the fixture README; never our exporter's output standing in for a vector | T1 | ⬜ |
| `P5-A1` *our export is importable by the reference* | Fixtures A and B ⇒ Hodos `.brc39` export ⇒ reference `importBRC39(…, {mode:'restore'})` into an **empty** toolbox SQLite succeeds; per-table row counts in the reference DB equal the file's; every output row names its outpoint | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The **reference** DB after import (row counts + one outpoint per basket listed with satoshis), never our own importer reading our own file | T1 | ⬜ |
| `P5-A2` *strict: no nulls, no dangling refs, no foreign fields in the 12 tables* | Our exporter's pre-write validator enforces the reference rules (omit-not-null; `requireRef` on `output.transactionId`, `output.spentBy`, `commission.transactionId`, `txLabelMap.transactionId`, `provenTxReq.provenTxId`; binaries base64 with padding; arrays sorted by PK). Hodos-only sections (permissions, the address high-water mark) are a **separate top-level key**, never fields inside the 12 tables | Feed the validator an output carrying the on-chain strip's nulled `transaction_id` (the `backup.rs :: compress_for_onchain` "Null out orphan FK references" shape) ⇒ our export must **refuse to write the file** naming the row; second RED: put `hodos_domain` inside an output row ⇒ our validator refuses. A validator that writes the file anyway tested nothing | Our validator's refusal message and the absence of the output file on disk | T1 | ⬜ |
| `P5-A3` *no secret in the file* | The decrypted export contains no mnemonic word sequence, no master private key (hex or WIF), no PIN-derived material — BRC-38 carries none and BRC-191 names the failure | Build the exporter on today's `collect_payload` unchanged (which fills `BackupPayload.mnemonic`) ⇒ the scan must go red on the phrase. A scan that stays green on that build cannot see a secret | The decrypted **file bytes**, scanned for the fixture's 12 words in order and its master key in hex and WIF — not the Rust struct | T1 | ⬜ |
| `P5-A4` ⭐ **decision 5 — export file ⇄ on-chain restore** | From **one** fixture wallet (A, then B): (i) `.brc39` export → import into a fresh profile; (ii) on-chain backup on the mock → wipe → seed-only `wallet_recover_onchain` into another fresh profile. `canonical(i) == canonical(ii) == canonical(source)` minus exactly the H2 manifest's exclusions, **money-index rows included** (both rebuilt through the classifier, decision 2a / §6a). Also run on a **pre-beta.6 backup** (received payments carry `change=0`) ⇒ still identical after classification | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The two **restored** DBs (new processes, fresh data dirs), diffed with T3a-P1's `canonical()`; each diff line names an H2 manifest row. ⚠️ The on-chain half runs on **today's** format at P5 close and is **re-run at T3a-P4's freeze** — a pass on a superseded format is not carried forward | T1 (mock) | ⬜ |
| `P5-A5` *import classifies, then rebuilds* | Every output arriving by BRC-38 import passes T1's classifier (on its **real** script from the file's `lockingScript`, not one synthesised from an address) before the money index is rebuilt; plain P2PKH multi-sat ⇒ money; 1-sat or unreadable ⇒ **held and shown**; the import completes and reports held items (decision 7) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | Imported `outputs` rows by outpoint (txid:vout, sats, basket) and the money-index table; the classifier's input field named and shown to be an observation (`REGRESSION_ADDITIONS.md` R-CLASSIFY note) | T1 | ⬜ — **gated on T1-P3 + T1-P5** |
| `P5-A6` *the phrase must control the file* | Import refuses, **before writing any row**, a file whose `user.identityKey` is not derivable from the phrase under the convention the user's tier names (today's check is `m` only — `handlers.rs :: wallet_import` "mnemonic does not match identity key"); a refused import leaves the profile byte-identical | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The profile directory's file hashes before and after; the refusal reason shown | T1 | ⬜ |
| `P5-A7` *the preview told the truth (T11)* | Before any write the user sees, per table, what will import, what will be held, and what has no home in Hodos; after import the actual counts equal the preview's | Import a file carrying a table the preview code does not count (plant certificates in a fixture while stubbing the preview's certificate count) ⇒ the post-import comparison must go red naming the table | Preview JSON vs post-import row counts, both logged; the rendered screen checked by the owner in the sitting (A15) | T1 + T4 (human, visual) | ⬜ |
| `P5-A8` *export says which coins another wallet cannot spend* | 🔍 **Found in today's code while re-verifying:** Hodos's own receive outputs are derived with invoice `2-receive address-{i}` (`database/connection.rs :: create_wallet_with_first_address` and `create_wallet_from_existing_mnemonic`, both `"2-receive address-0"`; `crypto/brc42.rs` tests) and stored with `derivation_prefix = "2-receive address"`; a BRC-29 conforming signer builds `2-3241645161d8-{prefix} {suffix}` (`handlers.rs :: derive_address_from_payment_remittance`) — so a foreign wallet given our file would likely derive the **wrong key** for those outputs, and cannot spend `bip32` or `1-wallet-backup` rows at all. GREEN: the export's capability section lists every output whose derivation is Hodos-specific, with count and sats, and the export screen says so in plain words. *(Hypothesis until A15's T9 run or a reference-signer test confirms it.)* | Remove the classification of `2-receive address` rows from the capability section ⇒ the test comparing the section against a `SELECT … GROUP BY derivation_prefix` of the source must go red | The source DB's `derivation_prefix` histogram vs the file's capability section | T1 | ⬜ |
| `P5-A9` *isolated, atomic, explicit* | Import restores into an **isolated** profile and becomes the active wallet only on an explicit user action; a kill at any step boundary leaves either no wallet or the previous one — never a half wallet. (Today's `wallet_import` runs `let _ = conn.execute("DELETE …")` **outside** the import transaction and updates `current_index` `WHERE id = payload.wallet.id` — the backup's old id — both shapes P5 must not copy) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The profile's DB after a kill at each step boundary (T3a-P1 harness), by row count and wallet id | T1 | ⬜ |
| `P5-A10` *preserve-unknown (T10)* | A foreign file carrying a basket, tag, label, certificate type or `customInstructions` Hodos does not act on imports, is kept, and re-exports **byte-equal** in those rows | Make the importer drop rows in unknown baskets ⇒ the re-export diff must go red naming them | Re-export vs original, diffed per table with the reference `canonicalize` | T1 | ⬜ |
| `P5-A11` 🔐 *tier ② derivations are right* | BRC-157: the spec phrase `legal winner … yellow` ⇒ root `27e442c8…0e55` (spec vector, exactly); BRC-75: a generated vector agrees between our code, `@bsv/sdk`, and HandCash `vault.ts :: rootKeyFromMnemonicBrc75`; zero-entropy phrase rejected **on the 157 branch only** | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The derived root's public key compared to the vector bytes; the three implementations named with versions/commits | T1 | ⬜ |
| `P5-A12` 🔐 *fall-through* | If the tier the user picks finds nothing, the other tiers are scanned anyway (decision 6 triage); "I don't know" (④) scans all; the result names **which** convention found each coin, in plain words | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | A funded scratch phrase whose only coin sits at the BRC-75 root, entered as tier ③ ⇒ the coin (txid:vout, sats) must be found and attributed to "another BRC-100 wallet". ⚠️ Shape depends on §12 Q2 | T1 (mock) + T2 (funded scratch phrase) | ⬜ |
| `P5-A13` 🔐 *tier ① and ③ unchanged* | Tier ① = today's `wallet_recover_onchain` → on `backup_found:false` fall back to `wallet_recover` (`WalletPanelPage.tsx :: doRecoverWallet`), behaviour unchanged; tier ③ = `recover_wallet_from_mnemonic` (BIP-32 `m/{i}` + BRC-42) and the Centbee path (`wallet_recover_external`, `ExternalWalletConfig::centbee`) — BIP-32 hits are **swept to BRC-42 via T1-P6**, never written into `addresses` as spendable BIP-32 rows (T1 Q5) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | The restored DB's `addresses` and `outputs` rows by derivation, and the sweep txid(s) | T1 + T2 | ⬜ — **gated on T1-P6** |
| `P5-A14` 🔐 *a sweep never burns an item* | Any tier-②/③ sweep moves only funding outputs; every 1-sat output found at a foreign address is **held and shown**, never an input of a funding sweep (`recovery.rs :: split_token_reserved` is the existing seam) | ⏳ independent control — second agent (RELEASE_CYCLE §4.2) | A real 1-sat output staged at a scratch foreign-phrase address, named by txid:vout — the sweep tx's input list must not contain it (`HARNESS_DELTA.md` §1.2: scratch profile, destructive RED) | T2 | ⬜ |
| `P5-A15` 👤 *interop sitting (T8 + T9)* | HandCash `.brc39` → Hodos: imported per A5/A6/A7 **or** refused with a stated reason (root convention — §12 Q1); Hodos `.brc39` → HandCash: HandCash's importer accepts **or** refuses with its reason. Both recorded verbatim with file hashes | The same HandCash file with one ciphertext byte flipped ⇒ `BRC-39 authentication failed` on both sides — proves each side actually opened the file under test | The two wallets' own UI/log text and the file's SHA-256; the HandCash version | T4 (owner) | ⬜ |
| `P5-A16` *address high-water mark travels* | After a file import the wallet's next receive index is **≥** the source's `max(wallets.current_index, MAX(addresses.index))` (BRC-155 `nextIndex`, the one mark T3a owes T1) — carried in the Hodos top-level section, or recomputed from the imported derivations when a foreign file has none | Import a file with the Hodos section removed and no recompute ⇒ the next address handed out equals one already in the file ⇒ red (the address-reuse shape the README's "Confirmed by the orchestrating session" recorded for `wallet_import`) | The next address the imported wallet issues vs the set of addresses in the file | T1 | ⬜ |

**Two-sided rows:** `P5-A5` (not spendable) ⇄ `P5-A7` (not hidden) — `R-RESTORE`'s pairing, applied to file import. `P5-A2` (strict for the reference) ⇄ `P5-A10` (nothing unknown lost): an exporter that passes A2 by dropping what it cannot express fails A10.

## 5. Blast radius

| Cited code (`file :: symbol`) | Verified 2026-09-28 | Note |
|---|---|---|
| `rust-wallet/src/backup.rs :: collect_payload`, `BackupPayload` (field `mnemonic`), `BackupOutput` (`change`, `spendable`, `basket_id`) | ✅ | The `.brc39` exporter is a **sibling reusing the collector's queries**, not a second collector (SCOPE §3.1) — and it must not carry `mnemonic` (A3) |
| `rust-wallet/src/backup.rs :: encrypt_backup`, `decrypt_backup`, `export_to_json`; `EncryptedBackup` | ✅ | Private `.hodos-wallet` format (PIN-derived key via `crypto::pin::derive_key_from_pin`). Stays as is; BRC-39 uses Argon2id per spec — a **new** KDF call site (rule 4: cite the Argon2 crate docs in the kickoff) |
| `rust-wallet/src/backup.rs :: import_to_db_with_ids`, `import_entities` | ✅ | One DB transaction — reuse for the isolated restore; the cleanup around it is the part not to copy (A9) |
| `rust-wallet/src/handlers.rs :: wallet_export`, `wallet_import` | ✅ | `wallet_import`: identity check against `m` only; `let _ = conn.execute("DELETE …")` outside the transaction; `UPDATE wallets … WHERE id = payload.wallet.id` (old id). The `current_index` bug is **T1's** (README "Confirmed by the orchestrating session") — P5 does not fix it in `wallet_import`, it just must not reproduce it |
| `rust-wallet/src/handlers.rs :: wallet_recover_onchain`, `wallet_recover`, `wallet_recover_external` | ✅ | Tiers ①/③. ⚠️ Tier ① inherits BS-C2: an indexer error can read as "no backup" and fall to a scan-only restore — fixed in **T3a-P2**, not here (§11) |
| `rust-wallet/src/recovery.rs :: recover_wallet_from_mnemonic`, `derive_bip32_address`, `derive_brc42_address`, `derive_private_key_bip32`, `derive_key_at_path`, `ExternalWalletConfig::centbee`, `scan_external_wallet`, `split_token_reserved`, `build_sweep_transactions` | ✅ | ⛔ Kept compiled and tested whatever the UI does (decision 6(1)). Tier ②'s root derivations are new code beside these |
| `rust-wallet/src/database/helpers.rs :: get_master_private_key_from_db` | ✅ | `XPrv::new(&seed)` — Hodos root `m`. Not changed by this phase |
| `rust-wallet/src/database/connection.rs :: create_wallet_with_first_address`, `create_wallet_from_existing_mnemonic`; `rust-wallet/src/crypto/brc42.rs` (invoice `2-receive address-{i}`) | ✅ | A8's finding |
| `rust-wallet/src/main.rs` route table (`/wallet/export`, `/wallet/import`, `/wallet/recover*`) and the internal-only path list (`path.starts_with("/wallet/recover")`) | ✅ | New endpoints go through the same table and the same internal-only guard — never reachable by a dApp |
| `frontend/src/pages/WalletPanelPage.tsx :: doRecoverWallet`, Centbee flow (`showCentbeeRecovery`, `handleCentbeeRecover`) | ✅ | The tier ask lives here (overlay rule). Native `<input>` and a **visible** `<input type="file">` (root `CLAUDE.md` CEF input patterns) |
| `frontend/src/components/wallet/SettingsTab.tsx` — "Export Backup — hidden for now" | ✅ | The export entry point; unhiding is in scope for the `.brc39` export only |

## 6. Out of scope

Moving **new** Hodos wallets to BRC-157 (decision 6 — a separate future call, invariant 3). Building BRC-140 shares (decision only here). `merge` import into a non-empty wallet (restore-into-isolated only; merge is a later item — it touches permission tables SCOPE P5 flagged as unknown). Yours ZIP import (README item 7 fallback — a finding if it happens, not a build). Fixing `wallet_import`'s `current_index` bug (T1). Fixing BS-C2 (T3a-P2). Changing the on-chain format (T3a-P4). Exporting `derived_key_cache` (T5-P3; the detector restarts empty — stated in the preview, A7). Scanning Yours/RelayX/Twetch/Centi branches as HandCash does — ⚠️ tempting, and a §12 Q2 decision, not a default.

## 7. Rollback

Export and import are new endpoints + one overlay screen: revert the phase's commits; the private `.hodos-wallet` path and today's recovery flow are untouched by design (A13 proves it). ⚠️ **Files users have already exported cannot be rolled back** (SCOPE §6.3) — which is why A0–A3 must be green before the export button is un-hidden.

## 8. Pre-mortem (adversarial review — before)

| Failure story | Caught by |
|---|---|
| The export passed our own importer and failed the reference's — "BRC-38" in name only | A1's SUBJECT is the **reference** DB |
| The export silently shipped the recovery phrase because it reused `collect_payload` | A3 |
| A HandCash user imported their file; the balance looked right; every received payment was frozen as "not money" because the file's `change` flags meant something else to us | A5 (classifier on real scripts before the rebuild) + A4's pre-beta.6 run |
| A HandCash user exported from Hodos into HandCash, which showed the coins but could not spend the `2-receive address` ones | A8 + A15 (T9) |
| The user chose "another BRC-100 wallet", we found nothing at BRC-75, stopped, and their coins sat at the BIP-32 root | A12 (fall-through); §12 Q2 (scan all, as HandCash does) |
| A phrase sweep swept a 1-sat inscription as funding | A14 |
| Decision 5's round-trip passed on today's on-chain format and was never re-run after T3a-P4 froze a new one | A4's SUBJECT: re-run at the freeze; a superseded pass is not carried |
| Import died half way and left a wallet with users but no outputs, which the next session read as fact (trip-wire 3) | A9 |
| A green A11 compared our BRC-75 code with a vector our own code generated | A11's SUBJECT: three implementations |

## 9. Platforms

| Platform | Rows that run here | Notes |
|---|---|---|
| Windows | all | Scratch profiles under `HodosBrowserDev` |
| macOS | A0–A14, A16 (`cargo test` + the overlay) | 🍎 Two platform-specific checks: (1) the export **save path** resolves through the cross-platform path helpers (invariant 9 — no hardcoded Windows path); (2) the visible file input for import works in the macOS overlay window (`GenericOverlayWindow`). A15 is owner-hosted, once, on either platform |

## 10. Owner-hours, human-bound rows, unknowns

| | |
|---|---|
| Owner-hours (estimate) | **~2.25 h**: answer §12 Q1–Q3, 0.5 h · tier-ask wording review (plain words, four choices), 0.25 h · interop sitting A15 with his HandCash account (SCOPE §7.1), 1.5 h |
| Human-bound rows | A15 (second wallet, owner's account); A7's rendered screen; A12/A14's funded scratch phrases need funding he already has (memory: funding is never a deferral reason) — agent-run once funded |
| Unknowns (K) — uncertainty, not difficulty | **Yes — one of T3b's two design-invalidating unknowns: the root convention.** What "import another wallet's file" means for a wallet whose root is not `m` (§12 Q1) decides whether T8 is an import or a sweep; and what the reference/HandCash importers do with our file in practice (A15) |

## 11. Cross-track edges

### May P5 go earlier? *(owner, 2026-09-27: "may deserve to go earlier")*

**Yes, partly — and it does not need to wait for T3a-P4.** P5's file is built from the full local DB, not from the on-chain format, so the format freeze is not upstream of it.

| Slice | Can start | Depends on |
|---|---|---|
| **Step 0** (vectors, derivation cross-checks, owner answers, BRC-140 decision) | **Now** — no code dependency | nothing |
| **Export + import build** (A1–A3, A5–A10, A16) | after **T1-P3** (money index) + **T1-P5** (classifier + `change=1` migration) + **T3a-P1** (harness, comparator) | ⚠️ Not before T1-P5: an export taken before the `change=1` migration tells a conforming wallet that received payments are **not money** (they are written `change=0` today — SCOPE §6a), and import has no classifier to call |
| **Tier ③ sweep** (A13) | after **T1-P6** | the BIP-32 → BRC-42 sweep builder |
| **Decision 5 round-trip** (A4) | runs when the build lands; **closes** only after **T3a-P2** | its on-chain half needs restore-classifies-before-rebuild (§6a). Before P2 its on-chain half is expected red — recorded as such, not rounded up. Re-run at **T3a-P4**'s freeze |

⇒ G5 can schedule P5 **in parallel with T3a-P2..P4**, not behind them.

| Direction | Other phase | What is given / needed |
|---|---|---|
| needs | **T3a-P0** A12 | HandCash `.brc39` field list + root-key consequence; the "no vectors exist — generate them" finding |
| needs | **T3a-P1** | `canonical()`, mock chain, H2 manifest, fixtures A/B/C |
| needs | **T3a-P2** | restore classifies before the money-index rebuild (A4's on-chain half); BS-C2 (tier ① must not read an indexer error as "no backup") |
| needs | **T3a-P4** | the frozen on-chain format — A4 is re-run against it |
| needs | **T1-P3 / T1-P5** | money index; classifier + migration (A5, and every export) |
| needs | **T1-P4** | real scripts on synced rows — an export of fabricated 25-byte scripts would publish fabrications as fact |
| needs | **T1-P6** | the BIP-32 → BRC-42 sweep (tier ③), incl. its service-fee decision |
| gives | **T1** `R-CLASSIFY` route list | two new ingest routes: BRC-38 import, tier ②/③ sweep landing |
| gives | **T2-P1** | imported `1sat`/`bsv21` baskets go through T2's classifier; held until identified |
| gives/needs | **T5-P3** | `derived_key_cache` (and its requester child table) is neither in the on-chain backup nor in BRC-38 — the detector restarts empty after a file import; the preview says so (A7) |
| gives | **P8** | the file format decisions (where Hodos-only sections live) the BRC text describes |

## 12. Open questions for the owner

1. **Q1 — "import a file from another BRC-100 wallet" has two readings; which one?** *(invariant 3; decides T8)*
   - **(a) Adopt the foreign root.** Hodos runs that wallet on its own root (BRC-75 or BRC-157): identity key, certificates and every BRC-42 relationship survive. Cost: a per-wallet "root convention" field (**schema**, invariant 2) and every place that calls `get_master_private_key_from_db` must honour it (**derivation**, invariant 3) — including the on-chain backup key.
   - **(b) Sweep with the file's data (HandCash's shipped model, extended).** The file tells us every derived output's keys; we sweep the coins into the user's Hodos wallet (root `m`). Coins survive; the old identity, certificates and labels come across as **history**, not as the user's live identity.
   - **Recommendation: (b) for beta.6**, (a) as a deliberate later decision. It leaves Hodos's own derivation untouched, matches what the one shipping wallet does, and is reversible; (a) is the "moving new wallets to BRC-157" question in another form, which decision 6 left undecided. Until answered, A6/A15 are written for (b).
2. **Q2 — fall back only if the chosen tier finds nothing, or always scan every tier?** Decision 6 says fall back when nothing is found. ⚠️ **Evidence:** HandCash scans **every** convention every time, because *"a phrase may have been used by more than one wallet"* — a hit in one tier is not proof the others are empty. **Recommendation: scan all tiers always; the user's answer only orders and labels the results.** Same cost (a few address lookups), and it cannot miss a coin the triage rule would. This is a refinement of decision 6, so I am asking rather than doing it.
3. **Q3 — the name.** Code, UI and the recovery comment all say **Centbee** (`recovery.rs :: ExternalWalletConfig::centbee`, `wallet_type: "centbee"`, `WalletPanelPage.tsx` Centbee flow). "SentBee" is Centbee — confirm? ⚠️ Not to be confused with **Centi**, which HandCash scans at a different path (`m/44'/145'/0'`). Should tier ③ also try Centi's path (HandCash tester-confirmed) — yes/no?
4. No evidence that decision 5 is wrong; the fresh read confirms it (the reference importer's strictness is unchanged).

---

## Sign-off

- [ ] Every evidence row GREEN **and** its RED observed (rows marked ⏳: RED designed by the second agent, name recorded)
- [ ] `scripts/preflight.ps1` run — result + date recorded below
- [ ] `scripts/preflight.ps1 -NegativeControl` run — every T0 gate seen to fail
- [ ] `../../../0.4.0-beta.3/REGRESSION_SET.md` + `../../REGRESSION_ADDITIONS.md` run in full at this boundary — result recorded (R-CLASSIFY with the two new routes named; R-RESTORE)
- [ ] Adversarial review of the evidence complete, four questions answered in writing
- [ ] Any baseline lowered in `../../../0.4.0-beta.3/HARNESS.md` §4, residuals listed with reasons
- [ ] Commit messages cite the row IDs they satisfy, and reference the phase issue (`Refs #N`)
- [ ] **Pushed, and the phase's GitHub issue CLOSED** by the closing commit (`Closes #N`) — `../../../RELEASE_CYCLE.md` §4.1a
- [ ] A4 re-run recorded at T3a-P4's freeze
- [ ] `../../../PRIOR_ART.md` rows added (BRC-75 vector gap; HandCash scan-all; BRC-191)
- [ ] `../../AAR_NOTES.md` swept
- [ ] Context: memory saved · boundary decided (continue / fresh session) — `../../../RELEASE_CYCLE.md` §4.6

| Item | Result | Date | By |
|---|---|---|---|
| preflight | | | |
| preflight -NegativeControl | | | |
| regression set | | | |
| adversarial review | | | |
