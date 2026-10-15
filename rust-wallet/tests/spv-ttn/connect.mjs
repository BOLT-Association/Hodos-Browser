// Connect: the wallet (spv mode) is up, has a wallet, and its own verified header chain has
// reached the public chaintracks tip and holds the same headers.
//
//   node connect.mjs                        (wallet started with run-wallet.ps1)
//   HEADERS=lite node connect.mjs           (started with -Headers lite: also checks the anchor and
//                                            the height below it, which the header trackers answer)
//   NETWORK=mainnet node connect.mjs        (started with -Network mainnet: getNetwork says mainnet;
//                                            the anchor and the heights below the window hash to
//                                            the checkpoints header_chain.rs pins)
import { wallet, walletGet, until, ok, chaintracksHeight, chaintracksHeaderHex, sha256d, pinnedMainnet, MAINNET, WALLET } from './lib.mjs'

ok((await fetch(WALLET + '/health').catch(() => ({}))).status === 200, 'wallet /health answers')

const status = await walletGet('/wallet/status')
if (!status.json.exists) {
  const c = await wallet('/wallet/create', {})
  ok(c.status === 200 && c.json.success, 'created a throwaway wallet') // the mnemonic is not printed
} else {
  ok(!status.json.locked, 'wallet exists and is unlocked')
}

const net = await wallet('/getNetwork', {})
ok(net.status === 200 && net.json.network === (MAINNET ? 'mainnet' : 'testnet'), `getNetwork answers ${net.json.network}`)

// First run syncs ~36k headers (about a minute; lite: ~18k from the 18000 checkpoint; mainnet:
// the window from the pinned anchor to the tip); later runs only the new ones.
const serverTip = await chaintracksHeight()
let last = -1
const tip = await until(`wallet header chain reaches ${serverTip - 2}`, async () => {
  const r = await wallet('/getHeight', {})
  if (r.status !== 200) return null
  if (r.json.height !== last) { last = r.json.height; console.log('     wallet height', last) }
  return r.json.height >= serverTip - 2 ? r.json.height : null
}, { timeout: 10 * 60_000, every: 3000 })
ok(true, `wallet verified headers to ${tip} (chaintracks tip ${serverTip})`)

const headerAt = async h => {
  const r = await wallet('/getHeaderForHeight', { height: h })
  return r.status === 200 ? r.json.header : null
}

if (MAINNET) {
  const { anchor, checkpoints } = pinnedMainnet()
  for (const h of [anchor.height, tip - 1]) {
    ok(await headerAt(h) === await chaintracksHeaderHex(h), `header ${h} (${h === anchor.height ? 'anchor' : 'window'}) matches chaintracks`)
  }
  ok(sha256d(await headerAt(anchor.height)) === anchor.hash, `the anchor hashes to the pinned ${anchor.hash.slice(0, 20)}…`)
  // Below the window the header trackers answer (chaintracks + WhatsOnChain, two must agree,
  // JungleBus confirms the hash): the height under the anchor and the two newest pinned
  // checkpoints below it, which must hash to what header_chain.rs pins.
  const below = [anchor.height - 1, ...checkpoints.filter(c => c.height < anchor.height).slice(-2).map(c => c.height)]
  for (const h of below) {
    const mine = await headerAt(h)
    ok(mine && mine === await chaintracksHeaderHex(h), `header ${h} (header trackers) matches chaintracks`)
    const pinned = checkpoints.find(c => c.height === h)
    if (pinned) ok(sha256d(mine) === pinned.hash, `header ${h} hashes to the pinned checkpoint`)
  }
} else {
  for (const h of [0, 18000, tip - 1]) {
    ok(await headerAt(h) === await chaintracksHeaderHex(h), `header ${h} matches chaintracks`)
  }
  if (process.env.HEADERS === 'lite') {
    // 18000 is the anchor (the window's first header); 17999 is below it and comes from the trackers.
    for (const h of [17999, 18000]) {
      ok(await headerAt(h) === await chaintracksHeaderHex(h), `lite: header ${h} (${h < 18000 ? 'header trackers' : 'anchor'}) matches chaintracks`)
    }
  }
}
console.log('PASS connect')
