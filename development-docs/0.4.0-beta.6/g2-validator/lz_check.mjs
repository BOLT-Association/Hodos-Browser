import { PrivateKey, KeyDeriver, Hash, ProtoWallet } from '@bsv/sdk'
const priv = new PrivateKey('e8f32e723decf4051aefac8e2c93c9c5b214313817cdb01a1494b917c8436b35', 'hex')
const kd = new KeyDeriver(priv); const w = new ProtoWallet(priv)
for (let i = 0; i < 40; i++) {
  const keyID = 'k' + i
  const arr = kd.deriveSymmetricKey([2, 'server hmac'], keyID, 'self').toArray()
  if (arr.length === 32) continue
  const data = [1, 2, 3]
  console.log('calling createHmac for', keyID)
  const { hmac } = await Promise.race([w.createHmac({ protocolID: [2, 'server hmac'], keyID, data, counterparty: 'self' }), new Promise((_, r) => setTimeout(() => r(new Error('createHmac timed out')), 20000))])
  const h31 = Hash.sha256hmac(arr, data); const h32 = Hash.sha256hmac([0, ...arr], data)
  console.log(keyID, 'keylen', arr.length, 'sdkHmac==31-byte-key:', hmac.join() === h31.join(), 'sdkHmac==32-byte-padded:', hmac.join() === h32.join())
  break
}
