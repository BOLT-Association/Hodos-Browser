// G2 recorder: calls the DEV wallet (31401) and saves each request + raw response.
// No broadcast: every createAction/signAction here is noSend or signAndProcess:false,
// and every output pays the wallet's own identity-key address. abortAction is NOT called
// (TSA-260 re-lock). Residue: nosend/unsigned rows released by TaskFailAbandoned (5 min)
// and TaskCheckForProofs NOSEND_TIMEOUT (10 min).
import { writeFileSync } from 'node:fs'
import { P2PKH, PublicKey, Utils } from '@bsv/sdk'

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

const TAG = 'g2probe'
const LABEL = 'g2probe'
const { publicKey: idKey } = await call('getPublicKey', { identityKey: true })
const lockingScript = new P2PKH().lock(PublicKey.fromString(idKey).toAddress()).toHex()

await call('getVersion', {})
await call('getNetwork', {})
await call('getHeight', {})
await call('isAuthenticated', {})
await call('getPublicKey', { protocolID: [2, 'g2 probe test'], keyID: '1', counterparty: 'self' })

// createAction, noSend, signed — includes the 1000-sat service fee output
const ca = await call('createAction', {
  description: 'G2 validator probe nosend',
  labels: [LABEL],
  outputs: [{ lockingScript, satoshis: 1000, outputDescription: 'g2 probe output', basket: 'g2probe', tags: [TAG] }],
  options: { noSend: true, randomizeOutputs: false }
})

// createAction two-phase: signAndProcess:false then signAction noSend
const ca2 = await call('createAction', {
  description: 'G2 validator probe two phase',
  labels: [LABEL],
  outputs: [{ lockingScript, satoshis: 1000, outputDescription: 'g2 probe output 2' }],
  options: { signAndProcess: false, noSend: true, randomizeOutputs: false }
})
const ref = ca2?.signableTransaction?.reference
if (ref) await call('signAction', { reference: ref, spends: {}, options: { noSend: true } })

await call('listActions', { labels: [LABEL], includeLabels: true, includeInputs: true, includeOutputs: true, includeOutputLockingScripts: true, limit: 10 })
await call('listActions', { labels: ['nonexistent-label-zz'], limit: 10 })
await call('listActions', { labels: [], limit: 5 })
await call('listOutputs', { basket: 'default', include: 'entire transactions', limit: 3 })
await call('listOutputs', { basket: 'default', include: 'locking scripts', includeTags: true, includeLabels: true, includeCustomInstructions: true, limit: 3 })
await call('listOutputs', { basket: 'g2probe', tags: [TAG], include: 'entire transactions', includeTags: true, limit: 5 })
await call('listOutputs', { basket: 'default', tags: ['no-such-tag-zz'], limit: 5 })

// signatures / hmac
const data = Array.from(Utils.toArray('g2 probe message', 'utf8'))
const other = Array.from(Utils.toArray('g2 probe DIFFERENT', 'utf8'))
const sigArgs = { protocolID: [2, 'g2 probe test'], keyID: '1', counterparty: 'self' }
const sig = await call('createSignature', { ...sigArgs, data })
await call('verifySignature', { ...sigArgs, data, signature: sig.signature, forSelf: true })
await call('verifySignature', { ...sigArgs, data: other, signature: sig.signature, forSelf: true })
const h = await call('createHmac', { ...sigArgs, data })
await call('verifyHmac', { ...sigArgs, data, hmac: h.hmac })
await call('verifyHmac', { ...sigArgs, data: other, hmac: h.hmac })
const enc = await call('encrypt', { ...sigArgs, plaintext: data })
if (enc?.ciphertext) await call('decrypt', { ...sigArgs, ciphertext: enc.ciphertext })
await call('listCertificates', { certifiers: [], types: [], limit: 5 })

// errors
await call('createSignature', { protocolID: [2, 'g2 probe test'], keyID: '1' }) // missing data
await call('internalizeAction', { tx: [1, 2, 3], outputs: [], description: 'g2 bad tx probe' })

writeFileSync(new URL('./recording.json', import.meta.url), JSON.stringify(rec, null, 1))
console.log('recorded', rec.length)
