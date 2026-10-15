// Shared helpers for the spv harness: the Teranode testnet by default, `NETWORK=mainnet` for
// mainnet. The wallet is started by run-wallet.ps1. The harness talks only to the wallet's HTTP
// API, the public Arcade (status, headers), the faucet (teratestnet) and WhatsOnChain (mainnet:
// an independent reading of block hashes, and the mined transaction + proof that proof.mjs feeds
// the wallet); the wallet itself talks only to Arcade and, on mainnet, to its header trackers.
import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { Beef, MerklePath, Transaction, Utils } from '@bsv/sdk'

export { Beef, MerklePath, Transaction, Utils }

export const NETWORK = process.env.NETWORK ?? 'teratestnet'
export const MAINNET = NETWORK === 'mainnet'
const DEFAULT_ARCADE = MAINNET ? 'https://arcade-v2-eu-1.bsvblockchain.tech' : 'https://arcade-ttn-us-1.bsvblockchain.tech'

export const WALLET = process.env.WALLET_URL ?? 'http://127.0.0.1:31401'
export const ARCADE = (process.env.ARCADE_URL ?? DEFAULT_ARCADE).replace(/\/$/, '')
export const CHAINTRACKS = process.env.CHAINTRACKS_URL ?? `${ARCADE}/chaintracks/v2`
export const FAUCET = (process.env.FAUCET_URL ?? 'https://faucet-ttn.bsvblockchain.tech').replace(/\/$/, '')
export const EXPLORER = MAINNET ? 'https://whatsonchain.com' : 'https://woc-ttn.bsvblockchain.tech'
/** WhatsOnChain's mainnet API: the wallet's second header tracker there, and the harness's independent source. */
export const WOC = 'https://api.whatsonchain.com/v1/bsv/main'

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

export async function chaintracksHeight (base = CHAINTRACKS) {
  return (await (await fetch(`${base}/height`)).json()).height
}

/** `count` 80-byte headers from `height`, hex each, as the chaintracks bulk endpoint serves them (fewer at the tip). */
export async function chaintracksHeadersHex (height, count, base = CHAINTRACKS) {
  const r = await fetch(`${base}/headers?height=${height}&count=${count}`)
  if (!r.ok) throw new Error(`${base} headers ${height}+${count}: HTTP ${r.status}`)
  const b = Buffer.from(await r.arrayBuffer())
  if (b.length % 80 !== 0) throw new Error(`${base} headers ${height}+${count}: ${b.length} bytes`)
  const out = []
  for (let i = 0; i < b.length; i += 80) out.push(b.subarray(i, i + 80).toString('hex'))
  return out
}

/** The 80-byte header at `height`, as the chaintracks server serves it (bulk endpoint). */
export async function chaintracksHeaderHex (height, base = CHAINTRACKS) {
  const [h] = await chaintracksHeadersHex(height, 1, base)
  if (!h) throw new Error(`header ${height}: not served by ${base}`)
  return h
}

/** The block hash (display hex) of an 80-byte header given as hex. */
export function sha256d (headerHex) {
  const h = b => createHash('sha256').update(b).digest()
  return h(h(Buffer.from(headerHex, 'hex'))).reverse().toString('hex')
}

/** The parent hash (display hex) an 80-byte header names. */
export const prevHashOf = headerHex => Buffer.from(headerHex.slice(8, 72), 'hex').reverse().toString('hex')

/**
 * The mainnet anchor and pinned checkpoints, read from `Params::mainnet_lite()` and
 * `MAINNET_CHECKPOINTS` in header_chain.rs, so the harness checks what the wallet pins and
 * cannot drift from it.
 */
export function pinnedMainnet () {
  const src = readFileSync(fileURLToPath(new URL('../../src/header_chain.rs', import.meta.url)), 'utf8')
  const num = s => Number(s.replace(/_/g, ''))
  const tuple = /\(\s*(\d[\d_]*)\s*,\s*"([0-9a-f]{64})"\s*\)/
  const fn = src.slice(src.indexOf('fn mainnet_lite'))
  const a = tuple.exec(fn.slice(fn.indexOf('anchor:')))
  const list = src.slice(src.indexOf('MAINNET_CHECKPOINTS:'))
  const body = list.slice(list.indexOf('&['), list.indexOf('];'))
  const checkpoints = [...body.matchAll(new RegExp(tuple.source, 'g'))].map(m => ({ height: num(m[1]), hash: m[2] }))
  if (!a || checkpoints.length < 10) throw new Error('could not read Params::mainnet_lite / MAINNET_CHECKPOINTS from header_chain.rs')
  return { anchor: { height: num(a[1]), hash: a[2] }, checkpoints }
}

/** WhatsOnChain (mainnet), paced to the free tier's 3 requests a second. 404 → null; JSON or text by content type. */
let wocNext = 0
export async function woc (path) {
  const wait = wocNext - Date.now()
  if (wait > 0) await sleep(wait)
  wocNext = Date.now() + 350
  const r = await fetch(WOC + path)
  if (r.status === 404) return null
  if (!r.ok) throw new Error(`WhatsOnChain ${path}: HTTP ${r.status}`)
  return (r.headers.get('content-type') ?? '').includes('json') ? r.json() : r.text()
}

/**
 * A BUMP (`MerklePath`) at `height` from a TSC proof `{index, nodes}` for `txid`: level 0 holds
 * the txid and its sibling, each level above the one sibling on the path; `*` is a duplicate
 * (the odd node pairs with itself).
 */
export function merklePathFromTsc (height, txid, tsc) {
  const leaf = (level, offset) => tsc.nodes[level] === '*' ? { offset, duplicate: true } : { offset, hash: tsc.nodes[level] }
  const level0 = [{ offset: tsc.index, hash: txid, txid: true }]
  if (tsc.nodes.length > 0) level0.push(leaf(0, tsc.index ^ 1))
  const path = [level0.sort((x, y) => x.offset - y.offset)]
  for (let level = 1; level < tsc.nodes.length; level++) path.push([leaf(level, (tsc.index >> level) ^ 1)])
  return new MerklePath(height, path)
}

export async function identityKey () {
  const r = await wallet('/getPublicKey', { identityKey: true })
  if (r.status !== 200 || !r.json.publicKey) throw new Error('no wallet identity key: ' + JSON.stringify(r))
  return r.json.publicKey
}

export const balance = async () => (await walletGet('/wallet/balance')).json.balance

/** Ask the faucet to pay the wallet's identity key (BRC-29). `FAUCET_API_KEY` raises its limits. Teratestnet only. */
export async function claim (identityKey, amount) {
  if (MAINNET) throw new Error('there is no faucet on mainnet')
  const headers = { 'content-type': 'application/json' }
  if (process.env.FAUCET_API_KEY) headers.authorization = `Bearer ${process.env.FAUCET_API_KEY}`
  const body = { identityKey }
  if (amount) body.amount = amount
  const r = await fetch(`${FAUCET}/api/claim/wallet`, { method: 'POST', headers, body: JSON.stringify(body) })
  const json = await r.json().catch(() => ({}))
  if (r.status !== 200) throw new Error(`faucet ${r.status}: ${JSON.stringify(json)}`)
  return json
}
