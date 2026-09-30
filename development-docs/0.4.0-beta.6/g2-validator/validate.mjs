// G2: run recorded Hodos responses through @bsv/sdk's hardened result validators,
// the same call WalletClient makes on every substrate (WalletClient.js validatedResult).
import { readFileSync } from 'node:fs'
import { validateWalletResult, snapshotWalletResultRequest } from '@bsv/sdk'

const rec = JSON.parse(readFileSync(new URL(process.argv[2] ?? './recording.json', import.meta.url), 'utf8'))
const BOUND = new Set(['createAction', 'signAction', 'listActions', 'listOutputs', 'revealCounterpartyKeyLinkage',
  'revealSpecificKeyLinkage', 'acquireCertificate', 'listCertificates', 'proveCertificate', 'discoverByIdentityKey', 'discoverByAttributes'])
let pass = 0; let fail = 0
for (const [i, e] of rec.entries()) {
  if (e.status !== 200) { console.log(`#${i} ${e.call} HTTP ${e.status} — error path, not a result`); continue }
  try {
    const req = BOUND.has(e.call) ? snapshotWalletResultRequest(e.call, e.args) : undefined
    validateWalletResult(e.call, e.body, req)
    pass++; console.log(`#${i} ${e.call} PASS`)
  } catch (err) {
    fail++; console.log(`#${i} ${e.call} FAIL — ${err.message}`)
  }
}
console.log(`pass ${pass} fail ${fail}`)
