import { readFileSync } from 'node:fs'
import { validateWalletResult, snapshotWalletResultRequest } from '@bsv/sdk'
const rec = JSON.parse(readFileSync('./recording.json','utf8'))
const la = rec[8]
const b = JSON.parse(JSON.stringify(la.body))
const steps = [
 ['status', x => { x.status = { aborted: 'nosend', created: 'unsigned' }[x.status] ?? x.status }],
 ['inputs -> sourceOutpoint/sourceSatoshis/sourceLockingScript/inputDescription/sequenceNumber', x => { x.inputs = x.inputs.map(i => ({ sourceOutpoint: `${i.txid}.${i.vout}`, sourceSatoshis: i.satoshis, sourceLockingScript: i.script || undefined, inputDescription: 'probe input', sequenceNumber: 0xffffffff })) }],
 ['outputs -> outputIndex/lockingScript/spendable/outputDescription/tags', x => { x.outputs = x.outputs.map(o => ({ outputIndex: o.vout, satoshis: o.satoshis, lockingScript: o.script, spendable: false, outputDescription: 'probe output', tags: [], basket: 'default' })) }],
 ['drop extra keys (referenceNumber, timestamp, confirmations)', x => { x.reference = x.referenceNumber; delete x.referenceNumber; delete x.timestamp; delete x.confirmations }],
]
for (const [name, f] of [['as recorded', () => {}], ...steps]) {
  b.actions.forEach(f)
  try { validateWalletResult('listActions', b, snapshotWalletResultRequest('listActions', la.args)); console.log(`after "${name}": PASS`) }
  catch (e) { console.log(`after "${name}": FAIL — ${e.message}`) }
}
