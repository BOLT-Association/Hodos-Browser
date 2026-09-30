// G2 recorder part 2 — appends to recording.json. Same no-broadcast rules as record.mjs.
import { readFileSync, writeFileSync } from 'node:fs'
import { Utils } from '@bsv/sdk'

const BASE = 'http://127.0.0.1:31401'
const path = new URL('./recording.json', import.meta.url)
const rec = JSON.parse(readFileSync(path, 'utf8'))
async function call (name, args) {
  const res = await fetch(`${BASE}/${name}`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(args)
  })
  const text = await res.text()
  let body; try { body = JSON.parse(text) } catch { body = text }
  rec.push({ call: name, args, status: res.status, body })
  console.log(name, res.status, text.length > 400 ? text.slice(0, 400) + '…' : text)
  return body
}

const unsigned = rec.find(e => e.call === 'createAction' && e.args.options.signAndProcess === false)
await call('signAction', { reference: unsigned.body.reference, spends: {}, options: { noSend: true } })

const idKey = rec[0].body.publicKey
const data = Array.from(Utils.toArray('g2 probe message', 'utf8'))
const other = Array.from(Utils.toArray('g2 probe DIFFERENT', 'utf8'))
const sigArgs = { protocolID: [2, 'g2 probe test'], keyID: '1', counterparty: idKey }
const sig = await call('createSignature', { ...sigArgs, data })
await call('verifySignature', { ...sigArgs, data, signature: sig.signature, forSelf: true })
await call('verifySignature', { ...sigArgs, data: other, signature: sig.signature, forSelf: true })

await call('listOutputs', { basket: 'g2probe', include: 'entire transactions', includeTags: true, includeLabels: true, limit: 5 })
await call('listOutputs', { basket: 'g2probe', tags: ['no-such-tag-zz'], limit: 5 })
await call('listOutputs', { basket: 'g2probe', include: 'locking scripts', includeCustomInstructions: true, limit: 5 })
await call('listActions', { labels: ['g2probe'], limit: 10 })

writeFileSync(path, JSON.stringify(rec, null, 1))
console.log('recorded', rec.length)
