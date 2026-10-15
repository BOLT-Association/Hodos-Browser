// Pre-flight for Params::mainnet_lite(): every hash header_chain.rs pins (the anchor and the
// checkpoints) is re-read from three sources that do not share an operator -- the EU and US Arcade
// chaintracks and WhatsOnChain -- and must match; the window after the anchor is fetched in bulk
// and must link. Live only; it never touches the wallet. A mismatch prints both sides and exits 1.
//
//   node mainnet-anchor.mjs                      check the pins
//   node mainnet-anchor.mjs --fixture            also write tests/fixtures/main_headers_<from>_<to>.bin
//   node mainnet-anchor.mjs --negative-control   flip one pinned hash first: this run must FAIL
//   CHAINTRACKS_US_URL=<base>                    the second chaintracks (default arcade-v2-us-1)
import { writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { ok, sha256d, prevHashOf, pinnedMainnet, woc, chaintracksHeadersHex, chaintracksHeight, CHAINTRACKS } from './lib.mjs'

const US = (process.env.CHAINTRACKS_US_URL ?? 'https://arcade-v2-us-1.bsvblockchain.tech/chaintracks/v2').replace(/\/$/, '')
const negative = process.argv.includes('--negative-control')
const WINDOW_BEFORE = 147 // the difficulty rule's look-back; a fixture from here can show NotAnchor below the anchor
const WINDOW_AFTER = 300

const { anchor, checkpoints } = pinnedMainnet()
ok(checkpoints.some(c => c.height === anchor.height && c.hash === anchor.hash), `the anchor ${anchor.height} is itself a pinned checkpoint`)
const pins = checkpoints.map(c => ({ ...c }))
if (negative) {
  const a = pins.find(c => c.height === anchor.height)
  a.hash = a.hash.slice(0, -1) + (a.hash.endsWith('0') ? '1' : '0')
  console.log('NEGATIVE CONTROL: the anchor pin is corrupted; this run must fail at height', anchor.height)
}

const tips = { eu: await chaintracksHeight(CHAINTRACKS).catch(e => e.message), us: await chaintracksHeight(US).catch(e => e.message) }
console.log(`     tips: EU ${tips.eu} (${CHAINTRACKS}), US ${tips.us} (${US})`)
if (Number.isInteger(tips.eu)) ok(tips.eu - anchor.height >= 500, `the anchor is ${tips.eu - anchor.height} blocks below the EU tip (at least 500, so no reorg reaches it)`)

const chaintracksHash = async (base, h) => {
  try { const [x] = await chaintracksHeadersHex(h, 1, base); return x ? sha256d(x) : null } catch (e) { console.log(`     ${base} at ${h}: ${e.message}`); return null }
}
const wocHash = async h => { try { return (await woc(`/block/height/${h}`))?.hash ?? null } catch (e) { console.log(`     WhatsOnChain at ${h}: ${e.message}`); return null } }

// ---- every pin against every source ----------------------------------------------------------
let bad = 0
for (const pin of pins) {
  const got = { eu: await chaintracksHash(CHAINTRACKS, pin.height), us: await chaintracksHash(US, pin.height), woc: await wocHash(pin.height) }
  const answered = Object.entries(got).filter(([, h]) => h)
  const wrong = answered.filter(([, h]) => h !== pin.hash)
  if (wrong.length || answered.length < 2) {
    bad++
    console.error(`MISMATCH at ${pin.height}: pinned ${pin.hash}`)
    for (const [s, h] of Object.entries(got)) console.error(`     ${s.padEnd(4)} ${h ?? '(no answer)'}${h && h !== pin.hash ? '  <-- differs' : ''}`)
    if (answered.length < 2) console.error('     fewer than two sources answered')
  } else {
    console.log(`ok   ${String(pin.height).padStart(7)} ${pin.hash.slice(0, 20)}… agreed by ${answered.map(([s]) => s).join(', ')}`)
  }
}
ok(bad === 0, `${pins.length} pinned hashes, ${bad} mismatch(es)`)

// ---- the window around the anchor links -----------------------------------------------------
const from = anchor.height - WINDOW_BEFORE
const headers = await chaintracksHeadersHex(from, WINDOW_BEFORE + 1 + WINDOW_AFTER, CHAINTRACKS)
ok(headers.length === WINDOW_BEFORE + 1 + WINDOW_AFTER, `EU serves ${headers.length} headers from ${from}`)
ok(sha256d(headers[WINDOW_BEFORE]) === anchor.hash, `header ${anchor.height} in the bulk range is the anchor`)
let unlinked = 0
for (let i = 1; i < headers.length; i++) if (prevHashOf(headers[i]) !== sha256d(headers[i - 1])) unlinked++
ok(unlinked === 0, `every header in ${from}..=${from + headers.length - 1} names the one before it (${unlinked} that do not)`)

if (process.argv.includes('--fixture')) {
  const to = from + headers.length - 1
  const path = fileURLToPath(new URL(`../fixtures/main_headers_${from}_${to}.bin`, import.meta.url))
  writeFileSync(path, Buffer.from(headers.join(''), 'hex'))
  console.log(`     wrote ${path} (${headers.length} headers)`)
}
console.log(`     Params::mainnet_lite(): anchor: (${anchor.height}, "${anchor.hash}")`)
console.log('PASS mainnet-anchor')
