# 🔍 Evaluate BRC-188 (User Management Protocol) as an optional account-recovery layer

**Found:** 2026-09-27, reading the draft in full (`wallet/0188.md` on the registry PR #278 branch,
ty-everett, opened 2026-09-27 — not yet merged).
**Status:** ⬜ UNASSIGNED · **Track:** beta.6 intake — evaluation, not a defect · **Filed by:** owner + Claude

> ⚠️ **Method note.** Everything below is a reading of the draft spec; nothing was run, and the
> draft may change before merge. The one thing not verified: whether any wallet besides the
> Metanet/CWI lineage has implemented UMP.

---

## What this is

BRC-188 specifies the deployed CWI-style account system: the wallet's two root keys never change,
and three factors — a presentation key (returned by a Wallet Authentication Backend after e.g.
phone verification), a password, and a user-saved recovery key — are arranged so **any two recover
the roots**. The encrypted material lives in a spendable on-chain descriptor (PushDrop UTXO,
`tm_users` overlay / `ls_users` lookup); factor changes spend the descriptor and publish a
successor. It sits beneath BRC-100 and is invisible to applications.

## Why it matters to us

- **It answers "what if I lose my seed phrase" with something other than "you lose everything."**
  Hodos today is mnemonic + encrypted backups. Two-of-three recovery is a materially different UX
  for the casual-user north star, and it is being standardized by the most active author in the
  registry.
- **Scope boundary we can build against now:** the draft explicitly does NOT cover wallet
  transaction-storage backup — that is our backup-and-sync track's territory. UMP recovers keys;
  our on-chain backup recovers wallet state. Complementary layers, and the backup track's BRC
  review should cite this boundary (cheap, do first).
- **Compatibility-first:** if we ever add factor-based recovery, doing it to BRC-188 rather than
  inventing our own keeps accounts portable across conforming wallets.

## How exposed are we — answer this first

Not exposed; nothing breaks without it. The cost of ignoring it is product (recovery UX and
interop), not correctness. The evaluation should answer:

| Question | Why it decides |
|---|---|
| Does UMP coexist with a mnemonic-rooted wallet, or does adopting it mean a different root-key model? | Decides whether this is an add-on or an architecture change |
| What does running or choosing a WAB imply for us (host one? support third-party?) | Decides the operational cost |
| Factor rotation does not revoke old roots (spec §7.3) — how does that interact with our identity-rotation design? | Decides whether it helps or conflicts with track 5 |
| Implementation size honestly estimated after reading §5–§13 (formats, overlay, vectors) | Owner expects "a lot"; confirm with numbers |

## What already protects us

Nothing needed today — seed + encrypted backups work. This ticket exists so the evaluation happens
on purpose in beta.6 planning rather than reactively when a user loses a phrase or a competitor
ships recovery.

## Proposed next step (the floor, not the system)

1. Backup-and-sync track cites BRC-188's scope line in its BRC review (one paragraph).
2. At beta.6 telescope: a half-day read of §5–§13 with the four questions above answered in
   writing, then decide build / defer / decline.

## Marketing note

👤 Owner, 2026-09-27: **Video 3 says nothing about this.** Not implemented, large, out of scope.
If a guest or commenter raises key recovery, we can speak to it: our wallet is seed + encrypted
backups today; BRC-188 is the ecosystem's emerging two-of-three answer and we're watching it.
Guest-prep pocket only.
