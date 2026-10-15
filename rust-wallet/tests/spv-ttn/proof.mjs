// Proof (mainnet): a mined transaction the wallet does not own, wrapped as Atomic BEEF with its
// BUMP, is offered to internalizeAction. The block is below the verified window, so the wallet must
// get its header from the header trackers (chaintracks + WhatsOnChain agreeing, JungleBus
// confirming) before it can judge the BUMP: the answer must be ERR_NO_OUTPUTS_OWNED (the proof
// verified; the output is not ours), never ERR_PROOF_NOT_VERIFIED. The same BEEF with one sibling
// hash corrupted must be ERR_PROOF_NOT_VERIFIED. Residue: in spv mode the wallet caches every
// transaction a BEEF carries in `parent_transactions` before the ownership gate, so the genuine
// run leaves that one raw transaction in the throwaway wallet's cache (no output, no balance).
//
//   NETWORK=mainnet node proof.mjs        (wallet started with run-wallet.ps1 -Network mainnet)
//   PROOF_HEIGHT=<h>                      the block to take the transaction from (default 1000
//                                         below the anchor; a height inside the window skips the
//                                         trackers and checks the window instead)
import { wallet, ok, woc, pinnedMainnet, merklePathFromTsc, identityKey, Transaction, Utils, MAINNET, EXPLORER } from './lib.mjs'

if (!MAINNET) { console.error('proof.mjs needs NETWORK=mainnet'); process.exit(2) }
const { anchor } = pinnedMainnet()
const height = Number(process.env.PROOF_HEIGHT ?? anchor.height - 1000)
const where = height < anchor.height ? 'below the window: header from the trackers' : 'inside the window'

const block = await woc(`/block/height/${height}`)
ok(block?.hash && block.merkleroot, `block ${height} is ${block?.hash} (${where})`)
const txids = Array.isArray(block.tx) && block.tx.length ? block.tx : await woc(`/block/hash/${block.hash}/page/1`)
ok(Array.isArray(txids) && txids.length > 0, `block ${height} lists ${txids?.length} transaction(s)`)
const txid = txids[Math.min(1, txids.length - 1)] // not the coinbase when there is a choice
const rawHex = String(await woc(`/tx/${txid}/hex`)).trim()
const tx = Transaction.fromHex(rawHex)
ok(tx.id('hex') === txid, `raw transaction hashes to ${txid}`)
const tsc = await woc(`/tx/${txid}/proof/tsc`)
const proof = Array.isArray(tsc) ? tsc[0] : tsc
ok(proof && Array.isArray(proof.nodes) && Number.isInteger(proof.index), `WhatsOnChain has a TSC proof (index ${proof?.index}, ${proof?.nodes?.length} node(s))`)
const bump = () => merklePathFromTsc(height, txid, proof)
ok(bump().computeRoot(txid) === block.merkleroot, "the BUMP computes the block's merkle root")

const sender = await identityKey()
const body = beef => ({
  tx: beef,
  outputs: [{
    outputIndex: 0,
    protocol: 'wallet payment',
    paymentRemittance: { senderIdentityKey: sender, derivationPrefix: Utils.toBase64(Utils.toArray('hodos-proof', 'utf8')), derivationSuffix: Utils.toBase64(Utils.toArray(String(height), 'utf8')) }
  }],
  description: 'mainnet proof check: a stranger\'s mined transaction'
})
const code = r => r.json.code ?? JSON.stringify(r.json).slice(0, 120)

// ---- negative control: one corrupted sibling hash in the BUMP ---------------------------------
const bad = Transaction.fromHex(rawHex)
bad.merklePath = bump()
const sib = bad.merklePath.path.flat().find(l => !l.txid && !l.duplicate && l.hash)
ok(sib, 'the BUMP has a sibling hash to corrupt')
sib.hash = sib.hash === 'ab'.repeat(32) ? 'cd'.repeat(32) : 'ab'.repeat(32)
const r1 = await wallet('/internalizeAction', body(bad.toAtomicBEEF()))
ok(r1.status === 400 && r1.json.code === 'ERR_PROOF_NOT_VERIFIED', `tampered BUMP rejected (${r1.status} ${code(r1)})`)

// ---- the genuine proof: verified, then refused because the output is not ours ----------------
tx.merklePath = bump()
const r2 = await wallet('/internalizeAction', body(tx.toAtomicBEEF()))
ok(r2.json.code !== 'ERR_PROOF_NOT_VERIFIED', `genuine BUMP at ${height} verified (${where})`)
ok(r2.status === 400 && r2.json.code === 'ERR_NO_OUTPUTS_OWNED', `then refused as not ours (${r2.status} ${code(r2)})`)
console.log(`     ${EXPLORER}/tx/${txid}`)
console.log('PASS proof')
