# spv-lite on mainnet: the second half

> **Status 2026-10-15 — built on branch `spv-lite` (PR #5 into `spv-testnet`), unit-tested, not yet run
> live.** Steps 1–5 landed as red/green commit pairs: `Params::mainnet_lite()` (38 checkpoints, anchor
> 945000), `Header::from_fields` + `verified_block_header` + the WoC block-JSON rebuild and
> `WhatsOnChainTracker`, `HashConfirmer` / `JungleBusConfirmer`, `HODOS_CHAIN_NETWORK=mainnet` with
> the roster by network, and the harness (`run-wallet.ps1 -Network mainnet`, `mainnet-anchor.mjs`,
> `connect.mjs`, `proof.mjs`). **Deviations from the plan below, all because the build container could
> not reach the chain hosts:** (a) step 0 did not run — the anchor is the newest checkpoint the BSVA
> node's own `go-chaincfg` `MainNetParams` pins (945000), cross-checked against bitcoin-sv
> `chainparams.cpp` on the 12 shared heights, and `mainnet-anchor.mjs` now re-reads every pin from the
> EU + US chaintracks and WhatsOnChain and is the first thing to run on a machine that can; (b) no
> mainnet header fixture exists yet (`mainnet-anchor.mjs --fixture` writes it) — the mainnet-rule tests
> run the teratestnet headers under `Params::mainnet_lite`'s rule instead, which is what pins
> `reduce_min_difficulty: false`; (c) step 3's "a confirmer that abstains (404) → `Ok`" became **a
> contradiction**: the trackers serve only heights far below the tip, so a JungleBus that does not know
> the hash is evidence against it, not lag (unreachable still abstains). Owner to confirm (c). The
> first half (the verified window, the anchored sync, `header_trackers.rs`, `HODOS_HEADERS=lite`) was
> proven on teratestnet; this document is the remainder: what makes the same machinery run against
> Bitcoin SV mainnet. It supersedes the mainnet-specific rows of `PLAN_spv_lite_mainnet.md`, whose
> network-agnostic rows are done.

## What is already there

| Done on `spv-lite` | Where |
|---|---|
| Anchor + assume-valid window; PoW bounded by powLimit below assume-valid | `header_chain.rs :: Params { anchor, assume_valid_height }`, `add_header` |
| Sync from the anchor; receding source is *unavailable* | `header_sync.rs :: sync_into` |
| Trackers below the window: usable-answer rules, agreement, cache, two in flight | `header_trackers.rs :: decide`, `HeaderTrackers::header_at` |
| Every proof check makes its height answerable first | `HeaderService::ensure_header(s)` and its five callers |
| Depth switch, tracker list, agreement minimum | `chain_mode.rs :: HODOS_HEADERS`, `HODOS_HEADER_TRACKERS`, `min_agreeing` |

Mainnet needs **no new mechanism**. It needs its own `Params`, two more tracker types, the network
value, and a rule that the test-network allowance (one tracker) does not apply.

## Facts this plan rests on

- **Chain rules.** From bitcoin-sv `src/chainparams.cpp` (master, fetched 2026-10-15): genesis
  `000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f`, powLimit bits `0x1d00ffff`,
  spacing 600 s, UAHF at 478558, **DAA from 504031**. Above 504031 the rule is the cw-144 DAA that
  `header_chain.rs` already implements for teratestnet, **without** the 20-minute minimum-difficulty
  rule (`reduce_min_difficulty: false`). An anchor above 504031 never meets the EDA or the 2016-block
  retarget, so neither is implemented.
- **Pinned checkpoints** (bitcoin-sv `checkpointData`, same fetch). Every one of these is still
  re-verified against two live sources at implementation time before it is pinned; a hash that fails
  either source is not pinned.

| Height | Hash |
|---|---|
| 11111 | `0000000069e244f73d78e8fd29ba2fd2ed618bd6fa2ee92559f542fdb26e7c1d` |
| 33333 | `000000002dd5588a74784eaa7ab0507a18ad16a236e7b1ce69f00d7ddfb5d0a6` |
| 74000 | `0000000000573993a3c9e41ce34471c079dcf5f52a0e824a81e7f953b8661a20` |
| 105000 | `00000000000291ce28027faea320c8d2b054b2e0fe44a773f3eefb151d6bdc97` |
| 134444 | `00000000000005b12ffd4cd315cd34ffd4a594f430ac814c91184a0d42d2b0fe` |
| 168000 | `000000000000099e61ea72015e79632f216fe6cb33d7899acb35b75c8303b763` |
| 193000 | `000000000000059f452a5f7340de6682a977387c17010ff6e6c3bd83ca8b1317` |
| 210000 | `000000000000048b95347e83192f69cf0366076336c639f9b7228e9ba171342e` |
| 216116 | `00000000000001b4f4b433e81ee46494af945cf96014816a4e2370f11b23df4e` |
| 225430 | `00000000000001c108384350f74090433e7fcf79a606b8e797f065b130575932` |
| 250000 | `000000000000003887df1f29024b06fc2200b55f8af8f35453d7be294df2d214` |
| 279000 | `0000000000000001ae8c72a0b0c301f67e3afca10e819efa9041e458e9bd7e40` |
| 295000 | `00000000000000004d9b4ef50f0f9d686fd69db2e03af35a100370c64632a983` |
| 478558 | `0000000000000000011865af4122fe3b144e2cbeea86142e8ff2fb4107352d43` |
| 504031 | `0000000000000000011ebf65b60d0a3de80b8175be709d653b4c1a1beeb6ab9c` |
| 530359 | `0000000000000000011ada8bd08f46074f44a8f155396f43e38acf9501c49103` |

  Plus the anchor's checkpoint `C` (below), verified the same way.
- **Trackers that exist** (`architecture-diff.md` §5.1, probed 2026-10-14): Arcade chaintracks at
  `arcade-v2-eu-1.bsvblockchain.tech` (healthy; use first) and the US hosts (`arcade-v2-us-1`,
  `arcade-us-1`), all serving `/chaintracks/v2` with bulk headers; the US header store reset to
  height 0 mid-session once, which the "unavailable" rule now covers. WhatsOnChain `/block/height/{h}`
  and `/block/hash/{hash}` carry the six header fields; its `/block/hash/{hash}/header` route is
  gone (404), which is why `whatsonchain.rs :: get_block_header` is broken today in public mode too.
  JungleBus `/block_header/get/{hash}` answers by hash only (`junglebus.rs :: parse_jb_block_header`
  already parses it).
- **Network.** This build container cannot reach `bsvblockchain.tech`, `whatsonchain.com` or
  `gorillapool.io`. Every live step below runs on a machine that can, or after those hosts are added
  to the environment's allowed domains.

## Decisions needed before step 1

1. **WhatsOnChain and JungleBus as header trackers on mainnet.** The original plan said yes. For:
   independent operators from the Arcade hosts, so two agreeing answers mean something. Against: spv
   mode's posture is "no public indexers for chain state". The leak is small and specific: a request
   for *a header at height h* tells WhatsOnChain the wallet cares about height h, nothing about
   addresses or transactions; the address and tx chains stay `SpvNoIndexerProvider`-only and a test
   pins that. **Recommendation: yes, headers only, built in for mainnet, no toggle.**
2. **Anchor policy.** A fixed anchor pinned in code and bumped per release, versus anything
   automatic. The window grows by ~144 headers (~11 KB) a day, so a pin lasts a year without trouble.
   **Recommendation: fixed; the pre-flight script below prints the literal to paste.**
3. **`HODOS_HEADERS=full` on mainnet.** Refuse to start with a message, or allow the ~971k-header
   sync (10 minutes, ~200 MB in memory, the `side_branch_has_root` scan). **Recommendation: refuse
   for now;** full-chain mainnet stays out of scope as in the first plan.
4. **Who runs the live pre-flight.** Either the owner runs `mainnet-anchor.mjs` on their machine and
   pastes its output, or `bsvblockchain.tech`, `whatsonchain.com` and `gorillapool.io` are added to
   the cloud environment's allowed domains so the implementing session can run it and the live tests
   itself. **Recommendation: allow the three hosts.**

## Design (what changes where)

| Piece | Today | After |
|---|---|---|
| `header_chain.rs :: Params` | `regtest()`, `teratestnet()`, `teratestnet_lite()` | `+ mainnet_lite()`: name `mainnet-lite`, `Daa { pow_limit_bits: 0x1d00_ffff, spacing_secs: 600, reduce_min_difficulty: false }`, `anchor = (C - 147, hash)`, `assume_valid_height = C`, checkpoints = the table above + `C`. A `pub is_mainnet: bool` (false everywhere else) so chain_mode can tell real coins from test coins without string-matching the name |
| `header_trackers.rs` | `TrackerSource` (by height), `ChaintracksTracker` | `+ WhatsOnChainTracker` (by height; 404 abstains). `+ HashConfirmer` trait (`confirm(hash) -> Result<Option<Header>, String>`) and `JungleBusConfirmer`. `HeaderTrackers` gains `confirmers`; after `decide()` agrees on a header, each confirmer is asked for that hash: a **different** header is `Disagree` (nothing cached), `None` (404) abstains, a transport failure abstains with a warning (a confirmer that is down cannot veto; one that answers differently can) |
| `services/providers/whatsonchain.rs :: get_block_header` | `/block/height/{h}` → hash → `/block/hash/{hash}/header` (404) | rebuilds the 80 bytes from the block JSON (`version, previousblockhash, merkleroot, time, bits, nonce`) and rejects it unless it hashes to `hash`, through one shared helper with `chaintracks.rs :: parse_header` (reuse-first: both do the same thing). Fixes public mode too. ⚠️ Check whether the WoC provider's requests pass through `chain_mode::get`, which blocks `whatsonchain.com` in spv mode; the tracker needs a header-only path that spv mode permits, and the test that address/tx lookups stay blocked must still pass |
| `chain_mode.rs` | `teratestnet` / `regtest`; `network_name()` = `testnet` in spv; `min_agreeing(n) = n.clamp(1, 2)` | `+ mainnet` → `Params::mainnet_lite()` for `lite`; `mainnet` + `full` refuses startup ("not available yet"); `network_name()` = `mainnet` when the params are mainnet, else `testnet`; `min_agreeing(params, n)` = **2 on mainnet regardless of n**, `n.clamp(1, 2)` on test networks; `validate_startup` refuses `mainnet` with a chaintracks URL on localhost (the regtest trap inverted). Mainnet's height-capable sources are the chaintracks list + WhatsOnChain, so the minimum of two holds without `HODOS_HEADER_TRACKERS`; a second Arcade region is optional there |
| `main.rs` | builds chaintracks trackers from the URL list | builds by network: test networks as today; mainnet adds `WhatsOnChainTracker` and the `JungleBusConfirmer`, logs the roster, and never prints the single-source warning (it cannot start that way) |
| `tests/spv-ttn/` | teratestnet only | `run-wallet.ps1 -Network mainnet` (Arcade default per network: mainnet → `arcade-v2-eu-1`; `-Headers` defaults to `lite` when the network is mainnet), `lib.mjs` defaults by `NETWORK`, `connect.mjs` also compares `C` with WhatsOnChain, new `proof.mjs` (zero coins, below), new `mainnet-anchor.mjs` (pre-flight) |

Reorgs inside the window: unchanged. Deeper than the window: out of scope, as before; the
walk-back failure still triggers a full proof re-check through the trackers.

## Steps (each: red test commit, then green commit; Claude as author)

0. **Pre-flight (live, no code change to the wallet).** `tests/spv-ttn/mainnet-anchor.mjs`:
   reads the EU chaintracks tip; picks `C` = the largest multiple of 1000 at least 500 below the tip
   (a few days of margin, so no reorg reaches the anchor); fetches headers `C-147 ..= C+300` in bulk
   from the EU host; cross-checks the hash at `C-147`, at `C`, and at every bitcoin-sv checkpoint
   height against WhatsOnChain `/block/height/{h}` **and** the US chaintracks; writes
   `tests/fixtures/main_headers_<anchor>_<end>.bin`; prints the `Params::mainnet_lite()` literal.
   Any mismatch is a hard failure that prints both sides. Committed with the fixture, and the commit
   message records the two sources and the tip at the time.
1. **`Params::mainnet_lite()`.**
   - Red: the fixture's headers are all accepted from the anchor (`Extended`); the header before the
     anchor is `NotAnchor`; `C` with bits one off is `WrongDifficulty`; a header more than 20 minutes
     after its parent carrying powLimit bits is `WrongDifficulty` on mainnet **and accepted under
     `Params::teratestnet()`'s rule**, the pair being the negative control for `reduce_min_difficulty`;
     a flipped nonce below assume-valid is `InsufficientWork`; bits easier than powLimit are `BadBits`;
     `from_stored` of the shuffled fixture rejects 0; `assume_valid_height == anchor + 147`;
     `is_mainnet` is true only here.
   - Green: the constructor, the field, the checkpoint list.
2. **WhatsOnChain: header rebuild + tracker.**
   - Red: offline, a mocked `/block/height` JSON rebuilds to 80 bytes that hash to `hash`; a tampered
     field and a missing field are `InvalidResponse`; `WhatsOnChainTracker` maps 404 to an
     abstention and anything else to a failure; a `WalletServices` test that in spv mode on mainnet
     `get_raw_tx` / `outspend` / `fetch_utxos` still hold only `SpvNoIndexerProvider`.
   - Green: the shared rebuild helper, the provider fix (public mode benefits), the tracker, and
     whatever header-only egress path spv mode needs for it.
3. **JungleBus confirmer.**
   - Red: with two agreeing height sources, a confirmer that agrees → `Ok`; one that returns a
     different header → `Err(Disagree)` and nothing cached (the negative control); one that abstains
     (404) → `Ok`; one that is unreachable → `Ok` with a warning. `JungleBusConfirmer` parses the
     real `/block_header/get` shape (fixture from `junglebus.rs`'s existing tests).
   - Green: `HashConfirmer`, `HeaderTrackers::confirmers`, the confirmer.
4. **`chain_mode` mainnet.**
   - Red: `parse_network("mainnet", Lite)` is `mainnet-lite`; `("mainnet", Full)` refuses naming
     `HODOS_HEADERS`; `network_name()` is `mainnet` for spv + mainnet and still `testnet` for spv +
     teratestnet/regtest; `min_agreeing` is 2 on mainnet for 1, 2 and 5 sources and unchanged on test
     networks; `validate_startup` refuses mainnet with `HODOS_CHAINTRACKS_URL` on localhost.
   - Green: the parsing, `is_mainnet` plumbing, `main.rs` roster by network.
5. **Harness.** `run-wallet.ps1 -Network mainnet`; `connect.mjs` checkpoint comparison; `proof.mjs`:
   take a recently mined mainnet transaction (any; the wallet owns none of its outputs), fetch its
   TSC proof from WhatsOnChain, wrap raw tx + BUMP as Atomic BEEF with `@bsv/sdk`, call
   `/internalizeAction` with a `wallet payment` claim: the response must **not** be
   `ERR_PROOF_NOT_VERIFIED` (it fails later, no outputs for us); a copy with one BUMP sibling
   corrupted **must** be `ERR_PROOF_NOT_VERIFIED`. Negative controls: `-Network mainnet -Headers full`
   refuses to start; `-Network mainnet -Trackers https://arcade-ttn-us-1.bsvblockchain.tech/chaintracks/v2`
   (a testnet tracker, which serves different headers at the same heights) makes the trackers
   disagree, so every height below the window is a 404 while the window still passes, and
   `connect.mjs` fails at the below-window check. Re-run the teratestnet `connect.mjs` + `fund.mjs`
   (full and lite), because `header_trackers.rs` changed.
6. **Docs.** `architecture-diff.md` §3.1 (mainnet column), §3.3 (mainnet trust: two independent
   operators must agree; a lone compromised tracker can delay by disagreeing, never forge), §5
   (what remains is the full chain); root `CLAUDE.md` spv paragraph; `rust-wallet/src/CLAUDE.md`
   rows for `chain_mode`, `header_chain`, `header_trackers`, `whatsonchain`, `junglebus`; harness
   README; a status note here and in `PLAN_spv_lite_mainnet.md`; `PRIOR_ART.md` row (bitcoin-sv
   `chainparams.cpp` checkpoints and DAA height; WhatsOnChain block JSON as a header source).

## Verification

- `cargo test` (lib, bin, integration) green at every green commit; each red commit seen failing.
- Negative controls named in commit messages: the min-difficulty pair (mainnet rejects what
  teratestnet accepts), `NotAnchor`, `BadBits`, confirmer disagreement, `min_agreeing` never 1 on
  mainnet, `full` refused on mainnet, the testnet-tracker disagreement in the harness.
- Live, on a machine that reaches the hosts: `mainnet-anchor.mjs` (step 0), `run-wallet.ps1 -Network
  mainnet` + `connect.mjs` + `proof.mjs`, the two harness negative controls, and the teratestnet
  runs again.

## Out of scope (say so in the PR)

Full-from-genesis mainnet (`HODOS_HEADERS=full`: three difficulty regimes, ~971k headers, the memory
and `side_branch_has_root` work); reorgs deeper than the window; median-time-past; a funded mainnet
test (the owner can send a BRC-29 BEEF to the identity key by hand); an anchor that advances by
itself.
