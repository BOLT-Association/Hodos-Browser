// Shared helpers for the teratestnet spv harness. The wallet is started by run-wallet.ps1.
// The harness talks only to the wallet's HTTP API, the public Arcade (status, headers) and the
// faucet; the wallet itself talks only to Arcade.
import { Beef, Transaction, Utils } from '@bsv/sdk'

export { Beef, Transaction, Utils }

export const WALLET = process.env.WALLET_URL ?? 'http://127.0.0.1:31401'
export const ARCADE = (process.env.ARCADE_URL ?? 'https://arcade-ttn-us-1.bsvblockchain.tech').replace(/\/$/, '')
export const CHAINTRACKS = process.env.CHAINTRACKS_URL ?? `${ARCADE}/chaintracks/v2`
export const FAUCET = (process.env.FAUCET_URL ?? 'https://faucet-ttn.bsvblockchain.tech').replace(/\/$/, '')
export const EXPLORER = 'https://woc-ttn.bsvblockchain.tech'

export const sleep = ms => new Promise(r => setTimeout(r, ms))
export const ok = (c, m) => { if (!c) { console.error('FAIL', m); process.exit(1) } console.log('ok  ', m) }

export async function wallet (path, body) {
  const r = await fetch(WALLET + path, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body ?? {}) })
  const text = await r.text()
  let json; try { json = JSON.parse(text) } catch { json = { raw: text } }
  return { status: r.status, json }
}

export async function walletGet (path) {
  const r = await fetch(WALLET + path)
  return { status: r.status, json: await r.json().catch(() => ({})) }
}

export async function until (label, fn, { timeout = 60000, every = 1000 } = {}) {
  const end = Date.now() + timeout
  while (Date.now() < end) { const v = await fn().catch(() => null); if (v) return v; await sleep(every) }
  throw new Error('timeout: ' + label)
}

export const arcadeStatus = async txid => (await fetch(`${ARCADE}/tx/${txid}`)).json()

export async function chaintracksHeight () {
  return (await (await fetch(`${CHAINTRACKS}/height`)).json()).height
}

/** The 80-byte header at `height`, as the chaintracks server serves it (bulk endpoint). */
export async function chaintracksHeaderHex (height) {
  const r = await fetch(`${CHAINTRACKS}/headers?height=${height}&count=1`)
  const b = Buffer.from(await r.arrayBuffer())
  if (b.length !== 80) throw new Error(`header ${height}: ${b.length} bytes`)
  return b.toString('hex')
}

export async function identityKey () {
  const r = await wallet('/getPublicKey', { identityKey: true })
  if (r.status !== 200 || !r.json.publicKey) throw new Error('no wallet identity key: ' + JSON.stringify(r))
  return r.json.publicKey
}

export const balance = async () => (await walletGet('/wallet/balance')).json.balance

/** Ask the faucet to pay the wallet's identity key (BRC-29). `FAUCET_API_KEY` raises its limits. */
export async function claim (identityKey, amount) {
  const headers = { 'content-type': 'application/json' }
  if (process.env.FAUCET_API_KEY) headers.authorization = `Bearer ${process.env.FAUCET_API_KEY}`
  const body = { identityKey }
  if (amount) body.amount = amount
  const r = await fetch(`${FAUCET}/api/claim/wallet`, { method: 'POST', headers, body: JSON.stringify(body) })
  const json = await r.json().catch(() => ({}))
  if (r.status !== 200) throw new Error(`faucet ${r.status}: ${JSON.stringify(json)}`)
  return json
}
