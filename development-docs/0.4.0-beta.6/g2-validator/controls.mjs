// Negative/positive controls for validate.mjs: each mutation must flip the verdict for the stated reason.
import { readFileSync } from 'node:fs'
import { validateWalletResult, snapshotWalletResultRequest, Transaction } from '@bsv/sdk'

const rec = JSON.parse(readFileSync(new URL('./recording.json', import.meta.url), 'utf8'))
const clone = x => JSON.parse(JSON.stringify(x))
function run (label, call, body, args, expect) {
  let got
  try { validateWalletResult(call, body, args === undefined ? undefined : snapshotWalletResultRequest(call, args)); got = 'PASS' } catch (e) { got = 'FAIL — ' + e.message }
  const ok = got.startsWith(expect)
  console.log(`${ok ? 'OK ' : 'BAD'} [expect ${expect}] ${label}: ${got}`)
}

const ca = rec[6]
// what the validator sees inside our BEEF
const tx = Transaction.fromAtomicBEEF(ca.body.tx)
console.log('createAction #6 outputs:', tx.outputs.map(o => `${o.satoshis}:${o.lockingScript.toAddress?.() ?? o.lockingScript.toHex().slice(0, 12)}`).join(' | '))

run('#6 as recorded', 'createAction', ca.body, ca.args, 'PASS')
{ const a = clone(ca.args); a.outputs[0].satoshis = 1001; run('#6 request asks 1001 sats (tx pays 1000)', 'createAction', ca.body, a, 'FAIL') }
{ const a = clone(ca.args); a.outputs.unshift({ ...a.outputs[0], satoshis: 1000 }); run('#6 randomizeOutputs:false, request output moved to vout 1', 'createAction', ca.body, a, 'FAIL') }
{ const b = clone(ca.body); b.txid = '00'.repeat(32); run('#6 txid swapped', 'createAction', b, ca.args, 'FAIL') }
{ const a = clone(ca.args); a.lockTime = 5; run('#6 request lockTime 5 (tx has 0)', 'createAction', ca.body, a, 'FAIL') }

const uns = rec[7]
{ const b = clone(uns.body); const a = clone(uns.args); a.options.signAndProcess = true; run('#7 same body, request signAndProcess:true', 'createAction', b, a, 'FAIL') }

const la = rec[8]
{ const b = clone(la.body); b.actions.forEach(x => { x.status = { aborted: 'nosend', created: 'unsigned' }[x.status] ?? x.status }); run('#8 statuses mapped only (inputs/outputs still legacy — full mapping in la.mjs)', 'listActions', b, la.args, 'FAIL') }

const lc = rec[23]
{ const b = { totalCertificates: lc.body.total_certificates, certificates: lc.body.certificates }; run('#23 renamed totalCertificates', 'listCertificates', b, lc.args, 'PASS') }

run('#29 verifySignature valid:true shape', 'verifySignature', { valid: true }, undefined, 'PASS')
