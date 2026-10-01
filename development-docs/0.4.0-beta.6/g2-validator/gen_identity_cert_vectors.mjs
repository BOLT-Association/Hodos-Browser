// beta.6 P4 — BRC-52 identity-certificate vectors for rust-wallet/src/identity_resolver.rs, made by the
// REFERENCE implementation (@bsv/sdk, version recorded in the output), never by our code.
// Run: node gen_identity_cert_vectors.mjs > ../../../rust-wallet/tests/fixtures/identity_cert_vectors.json
import { PrivateKey, ProtoWallet, MasterCertificate, VerifiableCertificate, Utils } from '@bsv/sdk'
import { readFileSync } from 'node:fs'

const sdkVersion = JSON.parse(readFileSync(new URL('./node_modules/@bsv/sdk/package.json', import.meta.url))).version
const TYPE_TWITTER = 'vdDWvftf1H+5+ZprUw123kjHlywH+v20aPQTuXgMpNc=' // identity_resolver.rs :: TYPE_TWITTER

// Fixed keys so the fixture is reproducible.
const certifier = PrivateKey.fromHex('11'.repeat(32))
const attacker = PrivateKey.fromHex('22'.repeat(32))
const subject = PrivateKey.fromHex('33'.repeat(32))
const other = PrivateKey.fromHex('44'.repeat(32))
const pub = k => k.toPublicKey().toString()
const fields = { userName: 'alice', profilePhoto: 'https://example.com/alice.png' }

async function publicCert (signer) {
  const master = await MasterCertificate.issueCertificateForSubject(
    new ProtoWallet(signer), pub(subject), fields, TYPE_TWITTER,
    async () => '00'.repeat(32) + '.0', Utils.toBase64(new Array(32).fill(7)))
  const keyring = await MasterCertificate.createKeyringForVerifier(
    new ProtoWallet(subject), pub(signer), 'anyone', master.fields, Object.keys(fields),
    master.masterKeyring, master.serialNumber)
  return {
    type: master.type, serialNumber: master.serialNumber, subject: master.subject,
    certifier: master.certifier, revocationOutpoint: master.revocationOutpoint,
    fields: master.fields, keyring, signature: master.signature
  }
}

async function sdkSays (c) {
  const v = new VerifiableCertificate(c.type, c.serialNumber, c.subject, c.certifier,
    c.revocationOutpoint, c.fields, c.keyring, c.signature)
  let verifies = false
  try { verifies = await v.verify() } catch { verifies = false }
  let decrypted = null
  try { decrypted = await v.decryptFields(new ProtoWallet('anyone')) } catch { decrypted = null }
  return { verifies, decrypted }
}

const genuine = await publicCert(certifier)
const byAttacker = await publicCert(attacker)
// Claims the trusted certifier but was signed by the attacker.
const forgedCertifier = { ...byAttacker, certifier: pub(certifier) }
// Signature bytes altered.
const sig = Utils.toArray(genuine.signature, 'hex'); sig[sig.length - 1] ^= 1
const badSignature = { ...genuine, signature: Utils.toHex(sig) }

// The victim's genuinely signed certificate, with the attacker's fields + keyring swapped in.
const fieldSwap = { ...genuine, fields: byAttacker.fields, keyring: byAttacker.keyring }

const cases = {
  genuine,
  field_swap: fieldSwap,
  untrusted_certifier: byAttacker,
  forged_certifier: forgedCertifier,
  bad_signature: badSignature
}
const out = {
  generator: 'development-docs/0.4.0-beta.6/g2-validator/gen_identity_cert_vectors.mjs',
  sdk: `@bsv/sdk ${sdkVersion}`,
  trusted_certifier: pub(certifier),
  subject: pub(subject),
  other_key: pub(other),
  expected_name: fields.userName,
  cases: {}
}
for (const [name, cert] of Object.entries(cases)) {
  out.cases[name] = { cert, sdk: await sdkSays(cert) }
}
// The vectors are only meaningful if the SDK itself agrees with them.
if (!out.cases.genuine.sdk.verifies || out.cases.genuine.sdk.decrypted?.userName !== 'alice') throw new Error('genuine vector does not verify in the SDK')
for (const n of ['forged_certifier', 'bad_signature', 'field_swap']) if (out.cases[n].sdk.verifies) throw new Error(`${n} verifies in the SDK`)
if (!out.cases.untrusted_certifier.sdk.verifies) throw new Error('untrusted_certifier should be a VALID signature by an untrusted key')
console.log(JSON.stringify(out, null, 2))
