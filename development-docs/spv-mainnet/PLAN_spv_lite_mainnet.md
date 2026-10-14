# Mainnet spv-lite for PR #3 (`spv-testnet`)

## Context

PR #3 gives the wallet an spv mode (Arcade + a header chain it verifies itself) on regtest and the
public Teranode testnet. The owner wants mainnet support in the same PR. A full verified mainnet
chain (~971k headers) would cost 150–340 MB of disk, 185 MB of RAM or a windowed refactor, and a
10-minute first sync. The owner chose **spv-lite** instead, designed so a later `full` switch
reuses the same path:

- a **verified window** at the tip (the existing `HeaderChain`, seeded from a pinned recent
  checkpoint instead of genesis; full DAA/PoW/linkage inside it; reorgs handled exactly as now);
- **on-demand headers from a quorum** of independent trackers for heights below the window
  (Arcade EU, Arcade US, WhatsOnChain, GorillaPool/JungleBus; ≥2 must answer, all answers must
  agree, every answer must hash to what it claims, and any pinned checkpoint must match);
- **pinned checkpoints** (bitcoin-sv `chainparams.cpp` mainnet list + recent ones verified
  against two sources at implementation time).

This is the model BSV's SDK `ChainTracker` / `wallet-toolbox` use (one trusted tracker), plus a
quorum and a verified tip window. The trade-off, stated in the docs: a forged proof at an old
height needs the quorum's independent operators to collude; the full chain (later) removes even
that.

Facts verified live on 2026-10-14 (see `development-docs/architecture/architecture-diff.md` §5.1):
`arcade-v2-eu-1.bsvblockchain.tech` serves mainnet `/chaintracks/v2` (headers by height/hash,
bulk, tip stream), SSE; the US hosts reset their header store mid-session (so a receding server
tip must be treated as *unavailable*); Arcade `GET /tx/:id` only knows txs it tracks, and
re-submitting a mined tx returns `202 MINED` + merkle path (the existing `internalizeAction`
re-broadcast covers this). WoC: `/block/height/{h}` and `/block/hash/{hash}` carry the six header
fields; its `/block/hash/{hash}/header` route is gone (404), so the existing WoC
`get_block_header` is broken today in public mode too. JungleBus: `/block_header/get/{hash}`
(hash only). Mainnet DAA = the cw-144 algorithm already implemented for teratestnet, without the
20-minute minimum-difficulty rule.

## Decisions already made with the owner

- Mainnet uses spv-lite now; `Params` must let a later full-from-genesis mode reuse the code
  (anchor at height 0 + assume-valid height = "full"). No full-chain work in this PR.
- Quorum sources: Arcade EU + Arcade US chaintracks + WhatsOnChain + GorillaPool (JungleBus).
- `HODOS_CHAIN_MODE` stays `public` by default; `HODOS_CHAIN_NETWORK` default stays `teratestnet`.
- No schema change (quorum headers are cached in memory only; the window persists in the existing
  V26 `header_chain` table under network `mainnet`, growing ~11 KB/day).

## Design (what changes where)

| Piece | Today | After |
|---|---|---|
| `header_chain.rs :: Params` | `genesis_hash` | `anchor: (height, hash)` (genesis = `(0, hash)`), `assume_valid_height: u32` (0 = never skip). Below `assume_valid_height` the bits-equality check is skipped (PoW on the header's own bits, linkage, time, checkpoints still enforced); at/above it the full rule runs. Mainnet: `anchor = (970853, …)`, `assume_valid_height = 971000` (anchor + 147, the first height whose DAA window lies inside the chain), checkpoint `(971000, …)`, DAA with `reduce_min_difficulty: false`, checkpoints from bitcoin-sv + verified recent ones |
| `header_chain.rs :: add_header` | empty chain accepts only the genesis | accepts only the anchor, at the anchor height. `HeaderError::NotGenesis` → `NotAnchor` |
| `header_sync.rs :: sync_into` | starts at 0, walks back to 0 (`if h == 0 { "source genesis differs" }`) | starts at `anchor.height`; the walk-back stops at `h == anchor.height` ("source anchor differs"), never asks the source for a height below the anchor; `server_tip < anchor.height` or `server_tip + MAX_WALK_BACK < local_tip` → `Err("source unavailable…")`, chain untouched (the US reset case). Neither rule can fire on regtest/teratestnet (anchor 0) except where today's "no common ancestor" already errors |
| new `header_quorum.rs` | — | `HeaderQuorum { sources }`, pure `decide(answers, checkpoint) -> Result<Header, QuorumError>`, async `header_at(height)` with an in-process cache. Rules: an answer whose bytes do not hash to its claimed hash is **discarded**; an answer that contradicts a pinned checkpoint is **discarded** (that source is on the wrong chain or reset; logged), never fatal by itself; `Ok` needs ≥2 remaining answers that are identical; `Err(Disagree)` if remaining answers differ, `Err(InsufficientAnswers)` if < 2. Height-capable sources (chaintracks ×N, WoC) are asked concurrently with **concurrency capped at 2 and heights deduplicated** (WoC unauthenticated is ~3 req/s; a BEEF with many BUMPs must not burst); hash-only sources (JungleBus) confirm the agreed hash afterwards, where `NotFound` is *abstain* (it lags at the tip), a different header is `Disagree` |
| `header_sync.rs :: HeaderService` | window only | `quorum: Option<HeaderQuorum>`; `async ensure_header(height)` (window hit → Ok; else quorum → cache); `check_merkle_root` answers from window, then quorum cache; `Err` (cannot say) when neither has it. New async `header_hex_at_height_or_fetch` for `/getHeaderForHeight` |
| callers of `check_merkle_root` | sync | the full set (reviewed): (a) `cache_helpers::verify_tsc_proof_against_block` (async) awaits `ensure_header(h)` before calling the sync `verify_tsc_proof_against_header_chain`; this one choke point covers `task_check_for_proofs` ≈L613/L868 and `handlers.rs` ≈L15965; (b) `task_check_for_proofs` ≈L740 closure: `ensure_header` awaited just before; (c) `task_recheck_proofs :: run` prefetches the heights of every stored proof it is about to scan **before** calling the sync `find_orphaned` (≈L33, calls `check_merkle_root` at ≈L69) and before its direct `verify_tsc_proof_against_header_chain` call at ≈L162; (d) `internalize_action` prefetches every `beef.bumps[*].block_height` (`MerkleProof.block_height` is `pub`; `parsed_beef` is in scope before ≈L13183) before `verify_beef_bumps`, **with one retry + backoff**, because ≈L13193 turns any closure `Err` into `400 ERR_PROOF_NOT_VERIFIED` and a transient source failure must not reject a payment the sender may never retry. `block_hash_at_height(...).unwrap_or(block_hash)` at ≈L760 is a fallback and needs nothing |
| `services/providers/whatsonchain.rs :: get_block_header` | fetches `/block/hash/{h}/header` (404 today) | rebuilds the 80-byte header from the block JSON (`version, previousblockhash, merkleroot, time, bits, nonce`) and rejects it unless it hashes to `hash`, same as `chaintracks.rs :: parse_header`. Fixes public mode too |
| `chain_mode.rs` | `teratestnet` / `regtest` | `+ mainnet`; `network_name()` → `"mainnet"` when the params are mainnet, else `"testnet"`; `HODOS_HEADER_QUORUM` (optional, comma list of chaintracks base URLs; default for mainnet: EU + US). No `chain_mode::get` change: WoC/JungleBus providers use their own client and are blocked in spv by *provider selection*, so the quorum simply constructs them for `get_block_header` only. Add a unit test that the spv `get_raw_tx` / `outspend` / `fetch_utxos` chains still hold only `SpvNoIndexerProvider` on mainnet (the privacy guarantee is about address/tx lookups, which stay blocked) |
| `main.rs` header-chain init | `header_params()` | also builds the quorum when `params.anchor.0 > 0` (lite); `regtest_hint` unchanged |
| `tests/spv-ttn/` | testnet only | `run-wallet.ps1 -Network mainnet` (Arcade default per network: mainnet → `arcade-v2-eu-1`); `connect.mjs` unchanged in spirit (its heights 0 and 18000 now come from the quorum, tip-1 from the window) + compares a checkpoint height with WoC; new `proof.mjs` (mainnet, zero coins): real mined tx + WoC TSC proof → Atomic BEEF → `internalizeAction` must **not** fail with `ERR_PROOF_NOT_VERIFIED` (it fails later, "no outputs for us"), and a tampered copy **must** |

Reorgs: inside the window, unchanged (`AddOutcome::Reorg` → `request_proof_recheck`). Deeper
than the window: out of scope, documented; the walk-back failure path triggers a one-time
`request_proof_recheck(0)` so every stored proof is re-verified through the quorum.

Quorum `Err` paths are already "cannot say", not verdicts: `pending_proofs::classify_proof` maps
`Err` → `HoldPending` (≈L40) and `find_orphaned` maps it to "cannot judge" (≈L73). A *persistent*
disagreement therefore cycles hold → expire (6 h) → re-poll; acceptable, and said in the docs.

Anchor arithmetic (reviewed): for height 971000, `suitable(parent 970999)` needs 970997..970999
and `suitable(ancestor 970855)` needs 970853..970855, so **970853 is the exact minimum anchor**
with zero slack. The fixture starts at 970853 and a unit test pins
`assume_valid_height == anchor.0 + 147`; the "971000 is `Extended`" test is what proves the walk
never reaches below the anchor.

## Steps (each: red test commit, then green commit; `Co-Authored-By` as the branch does)

1. **`header_chain`: anchor + assume-valid + mainnet params.**
   - Fixture `rust-wallet/tests/fixtures/main_headers_970853_971087.bin` (235 real headers from the
     EU chaintracks bulk endpoint; the fetch script asserts each hash against WoC before the file
     is committed; commit message records both sources).
   - Red: `Params::mainnet()` accepts all 235 (`Extended`); the first header must be the anchor
     (`NotAnchor` otherwise); a header **below** `assume_valid_height` with one byte flipped →
     `InsufficientWork` (PoW still enforced, bits not compared); header 971000 with bits one off →
     `WrongDifficulty`; header 971001 with powLimit bits → `WrongDifficulty` (no min-difficulty rule
     on mainnet); a mined regtest header **before** the anchor height on a `Params` with a higher
     anchor → `NotAnchor`; `from_stored` of the 235 (shuffled) → 0 rejected; regtest/teratestnet
     tests unchanged (their anchor is genesis, `assume_valid_height = 0`).
   - Green: the `Params` change (`anchor`, `assume_valid_height`), `required_bits` skip below
     assume-valid, `NotGenesis` → `NotAnchor` (also the doc comment in `chain_mode.rs` ≈L93,
     `tests/spv-ttn/README.md`, root `CLAUDE.md` spv paragraph), `Params::mainnet()` with the
     checkpoint list.
     Checkpoint hashes: bitcoin-sv `chainparams.cpp` mainnet `checkpointData` (11111 … 530359) +
     556767 (BSV split) + 600000, 800000, 971000, each **verified against EU chaintracks and WoC by
     a script at implementation time**; a hash that fails either source is not pinned.
     Update the two test `Params {…}` literals in `header_sync.rs` / `header_chain.rs`.

2. **`header_sync`: start at the anchor; receding source is unavailable.**
   - Red (fake sources): empty chain with anchor 100 syncs from a source serving 100..110 → tip 110,
     persisted 11; source whose tip is 50 → `Err` containing "unavailable", chain untouched, no
     reorg reported; source tip 100 after local tip 10_200 (> `MAX_WALK_BACK` behind) → same;
     walk-back that reaches the anchor without agreement → `Err`, not a reorg.
   - Green: `sync_into` changes; `HeaderSource` unchanged.

3. **`header_quorum.rs` (new) + WoC header fix.**
   - Red (pure `decide`): two identical answers → `Ok`; three with one different → `Err(Disagree)`;
     one answer → `Err(InsufficientAnswers)`; an answer whose bytes do not hash to its claimed hash
     is discarded (two good + one bad → `Ok`); an answer contradicting the pinned checkpoint is
     discarded (two good + one wrong-chain → `Ok`; one good + one wrong-chain →
     `Err(InsufficientAnswers)`); hash-only confirmer `NotFound` → abstain (`Ok`), different
     header → `Err(Disagree)`; `header_at` with 5 duplicate heights makes one fetch per source.
   - Red (WoC provider, offline with a mocked JSON): header rebuilt from the block JSON hashes to
     `hash`; a JSON whose fields disagree with `hash` → `InvalidResponse`.
   - Red (`HeaderService` with a fake quorum): height inside the window → no quorum call;
     below the anchor → quorum result cached, second call no fetch; quorum `Err` →
     `check_merkle_root` is `Err` (never `Ok(false)`); `ensure_header` on a height above the tip →
     `Err` (window pending).
   - Green: module, `HeaderService.quorum`, `ensure_header`, the caller sites (a)–(d) from the
     design table (one choke point in `verify_tsc_proof_against_block`, prefetch in
     `task_recheck_proofs::run`, the L740 closure, the `internalize_action` prefetch with one
     retry), `/getHeaderForHeight` async path, `WalletServices` test that mainnet spv chains for raw tx / outspend / utxos are
     still `SpvNoIndexerProvider` only.
   - Live (ignored): `header_quorum_live_mainnet` — heights 504031, 600000, 800000 agree across all
     four sources and match the pinned hashes; `header_sync_live_mainnet` — window syncs from the
     anchor to the tip in seconds.

4. **`chain_mode`: `mainnet`, `network_name`, `HODOS_HEADER_QUORUM`.**
   - Red: `parse_network("mainnet")`; `network_name()` = `mainnet` for spv+mainnet, `testnet` for
     spv+teratestnet/regtest, `mainnet` for public; `HODOS_HEADER_QUORUM` parsing (trim, drop
     empties, reject a public-indexer host so nobody points the chaintracks slot at WoC).
   - Green; `main.rs` wiring; `validate_startup` refuses `mainnet` + a chaintracks URL on
     `localhost` (that is the regtest trap inverted).

5. **Harness.** `run-wallet.ps1 -Network mainnet`, `connect.mjs` checkpoint check, `proof.mjs`
   (positive: not `ERR_PROOF_NOT_VERIFIED`; negative: tampered BUMP → `ERR_PROOF_NOT_VERIFIED`).
   Re-run the **testnet** `connect.mjs` + `fund.mjs` (header_chain changed).

6. **Docs.** `architecture-diff.md` §3.1 (two header models), §5 (mainnet: lite now, full later,
   the trust statement), root `CLAUDE.md` spv paragraph (mainnet contacts WoC/JungleBus for
   headers only), `rust-wallet/src/CLAUDE.md` rows (`chain_mode`, `header_chain`, `header_sync`,
   new `header_quorum`), harness README, PR body.

## Verification

- `cargo test` (lib/bin/integration) green at every green commit; each red commit seen failing.
- Negative controls, named in commit messages: the quorum `decide` disagreement/insufficient
  cases; PoW below assume-valid; `NotAnchor`; checkpoint mismatch; `check_merkle_root` never
  `Ok(false)` on a quorum error; `proof.mjs` tampered BUMP.
- Live: `cargo test --lib -- --ignored header_quorum_live_mainnet header_sync_live_mainnet`;
  `run-wallet.ps1 -Network mainnet` + `node connect.mjs` + `node proof.mjs`; testnet
  `connect.mjs` + `fund.mjs` still pass; `run-wallet.ps1 -Network regtest` still rejects the
  testnet genesis (`NotAnchor`).
- Reorg path: the existing `header_sync` reorg tests + `task_recheck_proofs` unit tests cover the
  window; the "source unavailable" tests cover the US-reset case.

## Out of scope (say so in the PR)

Full-chain mainnet (`HODOS_HEADERS=full`: anchor `(0, genesis)`, assume-valid 971000, plus the
disk/memory work); reorgs deeper than the window; median-time-past; a mainnet funding test with
real coins (the owner can run `fund`-style manually by sending a BRC-29 BEEF to the identity key).
