// Connect: the wallet (spv mode, teratestnet) is up, has a wallet, and its own verified header
// chain has reached the public chaintracks tip and holds the same headers.
//
//   node connect.mjs        (wallet started with run-wallet.ps1)
import { wallet, walletGet, until, ok, chaintracksHeight, chaintracksHeaderHex } from './lib.mjs'

ok((await fetch((process.env.WALLET_URL ?? 'http://127.0.0.1:31401') + '/health').catch(() => ({}))).status === 200, 'wallet /health answers')

const status = await walletGet('/wallet/status')
if (!status.json.exists) {
  const c = await wallet('/wallet/create', {})
  ok(c.status === 200 && c.json.success, 'created a throwaway wallet') // the mnemonic is not printed
} else {
  ok(!status.json.locked, 'wallet exists and is unlocked')
}

// First run syncs ~36k headers (about a minute); later runs only the new ones.
const serverTip = await chaintracksHeight()
let last = -1
const tip = await until(`wallet header chain reaches ${serverTip - 2}`, async () => {
  const r = await wallet('/getHeight', {})
  if (r.status !== 200) return null
  if (r.json.height !== last) { last = r.json.height; console.log('     wallet height', last) }
  return r.json.height >= serverTip - 2 ? r.json.height : null
}, { timeout: 10 * 60_000, every: 3000 })
ok(true, `wallet verified headers to ${tip} (chaintracks tip ${serverTip})`)

for (const h of [0, 18000, tip - 1]) {
  const mine = await wallet('/getHeaderForHeight', { height: h })
  ok(mine.status === 200 && mine.json.header === await chaintracksHeaderHex(h), `header ${h} matches chaintracks`)
}
console.log('PASS connect')
