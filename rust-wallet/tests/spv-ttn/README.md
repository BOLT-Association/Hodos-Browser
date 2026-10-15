# spv-ttn: Hodos in spv mode on the Teranode testnet (and mainnet)

Live tests for a wallet that uses only the public teratestnet Arcade and a header chain it
verifies itself (DAA, 20-minute minimum difficulty, checkpoints), funded by the testnet faucet.
The same scripts run against mainnet with `-Network mainnet` / `NETWORK=mainnet` (below); nothing
there is funded and nothing spends.

| Service | URL |
|---|---|
| Arcade (broadcast, status, BUMPs, SSE at `/events`) | `https://arcade-ttn-us-1.bsvblockchain.tech` |
| Chaintracks (headers; bulk `/headers?height=&count=`) | `<Arcade>/chaintracks/v2` |
| Faucet (`POST /api/claim/wallet {identityKey}` → Atomic BEEF) | `https://faucet-ttn.bsvblockchain.tech` |
| Explorer (by eye only; the wallet never calls it) | `https://woc-ttn.bsvblockchain.tech` |

```powershell
cd rust-wallet; cargo build
.\tests\spv-ttn\run-wallet.ps1 -Push        # 127.0.0.1:31401, data in %TEMP%\hodos-ttn-wallet
```

```bash
cd rust-wallet/tests/spv-ttn && npm install
node connect.mjs              # wallet created, ~36k headers verified (about a minute), headers match chaintracks
node fund.mjs                 # faucet payout internalized; a tampered BUMP is rejected first
node fund.mjs --wait-mined    # then waits for the block and the wallet's own proof check (minutes to ~45 min)
```

Negative control: `run-wallet.ps1 -Network regtest` makes the wallet reject the teratestnet
genesis (`NotAnchor`), so `connect.mjs` never reaches the tip.

## Lite headers (`-Headers lite`)

`run-wallet.ps1 -Headers lite` sets `HODOS_HEADERS=lite`: the wallet verifies a window from the
pinned 18000 checkpoint to the tip instead of the whole chain from the genesis, and answers a
height below the window from the **header trackers**: `<Arcade>/chaintracks/v2` plus any URLs given
with `-Trackers` (`HODOS_HEADER_TRACKERS`, comma-separated). Two sources must agree when two or more
are configured; with the single public teratestnet tracker the wallet runs on one and warns at
startup that headers below the window rest on that tracker (an owner decision for test networks,
2026-10-15; mainnet must insist on two).

```powershell
.\tests\spv-ttn\run-wallet.ps1 -Headers lite
```

```bash
HEADERS=lite node connect.mjs   # as above, plus 17999 (trackers) and 18000 (anchor) match chaintracks
node fund.mjs                   # unchanged: the faucet's BUMPs are at the tip, inside the window
```

Negative controls:

- `run-wallet.ps1 -Headers lite -Network regtest` refuses to start: regtest has no pinned
  checkpoint to anchor a window at.
- `run-wallet.ps1 -Headers lite -Trackers http://127.0.0.1:9/chaintracks/v2` adds a second source
  that cannot answer, so two must agree and only one can: `connect.mjs` fails at
  `header 0 matches chaintracks` (the wallet answers 404 for every height below 18000), while
  18000 and the tip still pass. Trackers that answered from one source here would be the bug.

## Mainnet (`-Network mainnet`)

`run-wallet.ps1 -Network mainnet` runs the same wallet against Bitcoin SV mainnet:
`HODOS_CHAIN_NETWORK=mainnet`, headers `lite` (the only depth there: the window runs from the
pinned **945000** checkpoint, `Params::mainnet_lite`, to the tip, under the mainnet DAA with no
minimum-difficulty rule), Arcade `arcade-v2-eu-1`, data in `%TEMP%\hodos-mainnet-wallet`. Below the
window the header trackers are `<Arcade>/chaintracks/v2` **and WhatsOnChain**; two must agree, and
JungleBus must have the agreed hash at that height. `getNetwork` answers `mainnet`. ⚠️ Real coins:
the wallet is a throwaway, the harness never funds it, and no script spends.

| Service | URL |
|---|---|
| Arcade EU (default; broadcast, status, BUMPs, chaintracks at `/chaintracks/v2`) | `https://arcade-v2-eu-1.bsvblockchain.tech` |
| Arcade US (the pre-flight's second chaintracks) | `https://arcade-v2-us-1.bsvblockchain.tech` |
| WhatsOnChain (the wallet's second tracker; the harness's independent reading) | `https://api.whatsonchain.com/v1/bsv/main` |
| JungleBus (the wallet's hash confirmer) | `https://junglebus.gorillapool.io` |

```bash
node mainnet-anchor.mjs                     # pre-flight, no wallet: every hash header_chain.rs pins (the anchor + 37 checkpoints)
                                            # agreed by EU + US chaintracks + WhatsOnChain; the 448 headers around the anchor link
node mainnet-anchor.mjs --negative-control  # one pin corrupted first: this run must FAIL at 945000
```

```powershell
.\tests\spv-ttn\run-wallet.ps1 -Network mainnet   # 127.0.0.1:31401, data in %TEMP%\hodos-mainnet-wallet
```

```bash
NETWORK=mainnet node connect.mjs   # getNetwork mainnet; the window from 945000 verified; 944999 and the two
                                   # newest pinned heights below it answered by the trackers and hashing to the pins
NETWORK=mainnet node proof.mjs     # a stranger's mined tx from block 944000, below the window: the tampered BUMP is
                                   # ERR_PROOF_NOT_VERIFIED; the genuine one verifies through the trackers and is then
                                   # refused as ERR_NO_OUTPUTS_OWNED (not ours). PROOF_HEIGHT=<h> picks another block
```

Negative controls:

- `run-wallet.ps1 -Network mainnet -Headers full` refuses to start: mainnet is lite only.
- `run-wallet.ps1 -Network mainnet -Arcade http://localhost:8080` refuses to start: a chaintracks on
  this machine is the regtest stack (the regtest trap, inverted).
- `run-wallet.ps1 -Network mainnet -Trackers https://arcade-ttn-us-1.bsvblockchain.tech/chaintracks/v2`
  adds a testnet tracker, which serves different headers at the same heights: the sources disagree,
  every height below the window is a 404, and `connect.mjs` fails at `header 944999 (header
  trackers)` while the anchor and the window still pass. Trackers that answered here would be the bug.
- `mainnet-anchor.mjs --negative-control`, above.

Residue: `proof.mjs` leaves the stranger's raw transaction in the throwaway wallet's
`parent_transactions` cache (spv mode caches every transaction a BEEF carries before the ownership
gate); no output, no balance, nothing spendable.

> **Status 2026-10-15:** the mainnet scripts were written on a machine that could not reach the chain
> hosts and have **not been run**. Run `mainnet-anchor.mjs` first; a pin it rejects is not pinned.

Things to know:

- The faucet submits through its own Arcade, so the public Arcade answers `transaction not found`
  for a fresh payout; `internalizeAction` then broadcasts the Atomic BEEF itself.
- Payouts can chain on earlier unmined faucet transactions the public Arcade does not track, so
  zero-conf leaves the output unconfirmed until the block; `--wait-mined` covers that path.
- Each `fund.mjs` run claims a real payout (100 000 sats by default, `FUND_SATS` to change;
  `FAUCET_API_KEY` raises the faucet's limits).
- Stop the wallet with Ctrl+C or `POST /shutdown`, never by image name.
