# spv-ttn: Hodos in spv mode on the Teranode testnet

Live tests for a wallet that uses only the public teratestnet Arcade and a header chain it
verifies itself (DAA, 20-minute minimum difficulty, checkpoints), funded by the testnet faucet.

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

Things to know:

- The faucet submits through its own Arcade, so the public Arcade answers `transaction not found`
  for a fresh payout; `internalizeAction` then broadcasts the Atomic BEEF itself.
- Payouts can chain on earlier unmined faucet transactions the public Arcade does not track, so
  zero-conf leaves the output unconfirmed until the block; `--wait-mined` covers that path.
- Each `fund.mjs` run claims a real payout (100 000 sats by default, `FUND_SATS` to change;
  `FAUCET_API_KEY` raises the faucet's limits).
- Stop the wallet with Ctrl+C or `POST /shutdown`, never by image name.
