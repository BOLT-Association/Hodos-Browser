// G2 recorder part 3 (beta.6 P5): createAction with a dApp-SUPPLIED input, the path ordinals and
// token dApps use. Money-free: everything is noSend, nothing is broadcast.
//   1. the wallet pays 1000 sats to a throwaway "dApp" key (noSend);
//   2. the dApp spends that output in a new createAction, no unlockingScript ⇒ signableTransaction;
//   3. the dApp signs its input with @bsv/sdk and calls signAction with spends (noSend).
// Residue: two nosend rows + their reservations, released by the NOSEND timeout (10 min).
// Writes recording3.json; validate with: node validate.mjs recording3.json
import { writeFileSync } from 'node:fs'
import { P2PKH, PrivateKey, Transaction, Utils } from '@bsv/sdk'

const BASE = 'http://127.0.0.1:31401'
const rec = []
async function call (name, args) {
  const res = await fetch(`${BASE}/${name}`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(args)
  })
  const text = await res.text()
  let body; try { body = JSON.parse(text) } catch { body = text }
  rec.push({ call: name, args, status: res.status, body })
  console.log(name, res.status, text.length > 300 ? text.slice(0, 300) + '…' : text)
  return body
}

const dappKey = PrivateKey.fromRandom()
const dappLock = new P2PKH().lock(dappKey.toPublicKey().toAddress())

// 1. fund the dApp key (noSend: the parent never reaches the network)
const fund = await call('createAction', {
  description: 'G2 probe fund dApp key',
  labels: ['g2probe'],
  outputs: [{ lockingScript: dappLock.toHex(), satoshis: 1000, outputDescription: 'g2 probe dapp funding' }],
  options: { noSend: true, randomizeOutputs: false }
})
const fundTx = Transaction.fromAtomicBEEF(fund.tx)
const fundTxid = fundTx.id('hex')

// 2. the dApp spends it; it will sign its own input later
const ca = await call('createAction', {
  description: 'G2 probe dapp supplied input',
  labels: ['g2probe'],
  inputBEEF: fund.tx,
  inputs: [{ outpoint: `${fundTxid}.0`, inputDescription: 'g2 probe dapp input', unlockingScriptLength: 107 }],
  outputs: [{ lockingScript: dappLock.toHex(), satoshis: 500, outputDescription: 'g2 probe dapp output' }],
  options: { noSend: true, randomizeOutputs: false }
})
const signable = ca?.signableTransaction
if (signable) {
  // 3. sign input 0 (the dApp's) with the SDK, over the transaction the wallet returned
  const tx = Transaction.fromAtomicBEEF(signable.tx)
  const idx = tx.inputs.findIndex(i => (i.sourceTXID ?? i.sourceTransaction?.id('hex')) === fundTxid && i.sourceOutputIndex === 0)
  tx.inputs[idx].sourceTransaction = fundTx
  tx.inputs[idx].unlockingScriptTemplate = new P2PKH().unlock(dappKey)
  const unlocking = await tx.inputs[idx].unlockingScriptTemplate.sign(tx, idx)
  await call('signAction', { reference: signable.reference, spends: { [idx]: { unlockingScript: unlocking.toHex() } }, options: { noSend: true } })
}
writeFileSync(new URL('./recording3.json', import.meta.url), JSON.stringify(rec, null, 1))
console.log('recorded', rec.length)
