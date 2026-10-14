// Fund: the teratestnet faucet pays the wallet's identity key (BRC-29) and returns Atomic BEEF;
// the wallet internalizes it, verifying every BUMP against its own header chain.
//
// Before the real call it sends a copy whose BUMP has one sibling hash corrupted and requires
// the wallet to reject it (ERR_PROOF_NOT_VERIFIED): the header-chain check must be doing work.
// That copy is rejected before anything is stored, so it leaves no residue.
//
//   node fund.mjs                 internalize, balance rises by the faucet amount
//   node fund.mjs --wait-mined    then wait for the block (~10 min) until the wallet has
//                                 verified the proof itself and marks the tx completed
//   FUND_SATS=<n>                 ask the faucet for n sats (default: the faucet's payout)
import { Beef, Transaction, Utils, wallet, walletGet, until, ok, arcadeStatus, identityKey, balance, claim, EXPLORER } from './lib.mjs'

const waitMined = process.argv.includes('--wait-mined')
const key = await identityKey()
ok(true, `wallet identity key ${key.slice(0, 12)}…`)
const before = await balance()

const pay = await claim(key, process.env.FUND_SATS ? Number(process.env.FUND_SATS) : undefined)
ok(pay.txid && pay.atomicBEEF, `faucet paid ${pay.amount} sats in ${pay.txid}`)
const beefBytes = Utils.toArray(pay.atomicBEEF, 'hex')
ok(Transaction.fromAtomicBEEF(beefBytes).id('hex') === pay.txid, 'Atomic BEEF subject is the faucet txid')

const body = tx => ({
  tx,
  outputs: [{
    outputIndex: pay.outputIndex,
    protocol: 'wallet payment',
    paymentRemittance: { senderIdentityKey: pay.senderIdentityKey, derivationPrefix: pay.derivationPrefix, derivationSuffix: pay.derivationSuffix }
  }],
  description: 'teratestnet faucet'
})

// ---- negative control: one corrupted sibling hash in a BUMP ------------------------------
const bad = Beef.fromBinary(beefBytes)
let sib
for (const bump of bad.bumps) {
  for (const level of bump.path) { sib = level.find(l => !l.txid && !l.duplicate && l.hash); if (sib) break }
  if (sib) break
}
if (sib) {
  sib.hash = sib.hash === 'ab'.repeat(32) ? 'cd'.repeat(32) : 'ab'.repeat(32)
  const r = await wallet('/internalizeAction', body(bad.toBinaryAtomic(pay.txid)))
  ok(r.status === 400 && r.json.code === 'ERR_PROOF_NOT_VERIFIED', `tampered BUMP rejected (${r.status} ${r.json.code ?? JSON.stringify(r.json).slice(0, 120)})`)
} else {
  console.log(`  (BEEF has ${bad.bumps.length} BUMP(s) with no sibling hash to corrupt; negative check skipped)`)
}

// ---- the real payment ---------------------------------------------------------------------
const r = await wallet('/internalizeAction', body(beefBytes))
ok(r.status === 200, `internalizeAction accepted the faucet BEEF (${r.status} ${JSON.stringify(r.json).slice(0, 160)})`)
// The balance cache (60 s TTL) is not invalidated by internalizeAction, so poll.
const after = await until('balance to reflect the payment', async () => {
  const b = await balance()
  return b !== before ? b : null
}, { timeout: 90_000, every: 3000 })
ok(after - before === pay.amount, `balance rose by exactly ${pay.amount} sats (${before} -> ${after})`)
console.log(`     ${EXPLORER}/tx/${pay.txid}`)

if (waitMined) {
  let seen
  const mined = await until('Arcade reports MINED', async () => {
    const s = await arcadeStatus(pay.txid)
    if (s.txStatus !== seen) { seen = s.txStatus; console.log('     arcade:', seen) }
    return s.txStatus === 'MINED' ? s : null
  }, { timeout: 45 * 60_000, every: 15_000 })
  ok(true, `mined at height ${mined.blockHeight}`)
  const done = await until('wallet marks the tx completed', async () => {
    const items = (await walletGet('/wallet/activity?limit=100')).json.items ?? []
    const it = items.find(i => i.txid === pay.txid)
    return it?.status === 'completed' ? it : null
  }, { timeout: 10 * 60_000, every: 5000 })
  ok(done, 'wallet verified the proof against its own header chain and completed the tx')
}
console.log('PASS fund')
