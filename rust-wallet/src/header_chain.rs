//! Verified block-header chain (Wallet-Hardening WS4).
//!
//! Pure logic, no I/O: validates headers, tracks every known fork, and picks the
//! active chain by **most cumulative work** — never by what a server calls its tip.
//! Merkle proofs are then checked against this chain's roots, not against a
//! third-party API's say-so.
//!
//! Enforced per header: 80-byte layout, proof-of-work (`hash <= target(bits)`),
//! `bits` equal to the network's fixed difficulty, parent known, `height = parent + 1`,
//! timestamp no more than 2 h in the future. The first header must be the pinned
//! genesis.
//!
//! **Regtest only for now.** Mainnet needs a checkpoint and the retarget/DAA rules;
//! `Params` has no mainnet constructor on purpose so nothing can silently run
//! unverified under a mainnet name. Median-time-past is not enforced yet.

use std::collections::HashMap;

use sha2::{Digest, Sha256};

/// Maximum accepted clock skew into the future (Bitcoin's 2 hours).
const MAX_FUTURE_SECS: u32 = 2 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderError {
    BadLength(usize),
    BadHex(String),
    BadBits(u32),
    WrongDifficulty { got: u32, want: u32 },
    InsufficientWork,
    UnknownParent(String),
    NotGenesis(String),
    TimeTooFarInFuture,
}

impl std::fmt::Display for HeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for HeaderError {}

// ---------------------------------------------------------------------------
// 256-bit unsigned integer (just what work/target arithmetic needs)
// ---------------------------------------------------------------------------

/// Little-endian limbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct U256(pub [u64; 4]);

impl U256 {
    pub const ZERO: U256 = U256([0; 4]);
    pub const ONE: U256 = U256([1, 0, 0, 0]);
    pub const MAX: U256 = U256([u64::MAX; 4]);

    pub fn from_be_bytes(b: [u8; 32]) -> U256 {
        let mut l = [0u64; 4];
        for i in 0..4 {
            let mut w = [0u8; 8];
            w.copy_from_slice(&b[24 - i * 8..32 - i * 8]);
            l[i] = u64::from_be_bytes(w);
        }
        U256(l)
    }

    pub fn to_be_bytes(self) -> [u8; 32] {
        let mut out = [0u8; 32];
        for i in 0..4 {
            out[24 - i * 8..32 - i * 8].copy_from_slice(&self.0[i].to_be_bytes());
        }
        out
    }

    pub fn is_zero(&self) -> bool {
        self.0 == [0; 4]
    }

    fn cmp_(&self, o: &U256) -> std::cmp::Ordering {
        for i in (0..4).rev() {
            if self.0[i] != o.0[i] {
                return self.0[i].cmp(&o.0[i]);
            }
        }
        std::cmp::Ordering::Equal
    }

    pub fn checked_add(self, o: U256) -> Option<U256> {
        let mut r = [0u64; 4];
        let mut carry = false;
        for i in 0..4 {
            let (s1, c1) = self.0[i].overflowing_add(o.0[i]);
            let (s2, c2) = s1.overflowing_add(carry as u64);
            r[i] = s2;
            carry = c1 || c2;
        }
        if carry { None } else { Some(U256(r)) }
    }

    fn sub(self, o: U256) -> U256 {
        let mut r = [0u64; 4];
        let mut borrow = false;
        for i in 0..4 {
            let (s1, b1) = self.0[i].overflowing_sub(o.0[i]);
            let (s2, b2) = s1.overflowing_sub(borrow as u64);
            r[i] = s2;
            borrow = b1 || b2;
        }
        U256(r)
    }

    fn shl1(self) -> U256 {
        let mut r = [0u64; 4];
        let mut carry = 0u64;
        for i in 0..4 {
            r[i] = (self.0[i] << 1) | carry;
            carry = self.0[i] >> 63;
        }
        U256(r)
    }

    fn bit(&self, n: usize) -> bool {
        (self.0[n / 64] >> (n % 64)) & 1 == 1
    }

    /// Long division; `d` must be non-zero.
    fn div(self, d: U256) -> U256 {
        let mut q = U256::ZERO;
        let mut r = U256::ZERO;
        for i in (0..256).rev() {
            r = r.shl1();
            if self.bit(i) {
                r.0[0] |= 1;
            }
            if r.cmp_(&d) != std::cmp::Ordering::Less {
                r = r.sub(d);
                q.0[i / 64] |= 1u64 << (i % 64);
            }
        }
        q
    }

    pub fn to_hex(self) -> String {
        hex::encode(self.to_be_bytes())
    }

    pub fn from_hex(s: &str) -> Option<U256> {
        let b = hex::decode(s).ok()?;
        let arr: [u8; 32] = b.try_into().ok()?;
        Some(U256::from_be_bytes(arr))
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp_(o))
    }
}

impl Ord for U256 {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.cmp_(o)
    }
}

/// Expand compact `bits` to a target. Rejects negative, zero and overflowing values.
pub fn compact_to_target(bits: u32) -> Result<U256, HeaderError> {
    let exp = (bits >> 24) as usize;
    let mant = bits & 0x007f_ffff;
    if bits & 0x0080_0000 != 0 || mant == 0 {
        return Err(HeaderError::BadBits(bits));
    }
    let mut b = [0u8; 32];
    // mantissa occupies bytes [exp-3, exp) counted from the least-significant end
    for i in 0..3usize {
        let byte = ((mant >> (8 * i)) & 0xff) as u8;
        let pos = exp as isize - 3 + i as isize; // little-endian byte index
        if byte != 0 {
            if !(0..32).contains(&pos) {
                return Err(HeaderError::BadBits(bits));
            }
            b[31 - pos as usize] = byte;
        }
    }
    Ok(U256::from_be_bytes(b))
}

/// Expected work for a header with this target: `2^256 / (target + 1)`.
pub fn work_for_target(target: U256) -> U256 {
    // (~target / (target + 1)) + 1, as in Bitcoin Core, avoiding a 257-bit numerator.
    let not_t = U256([!target.0[0], !target.0[1], !target.0[2], !target.0[3]]);
    let denom = target.checked_add(U256::ONE).unwrap_or(U256::ZERO);
    if denom.is_zero() {
        return U256::ONE; // target == 2^256-1
    }
    not_t.div(denom).checked_add(U256::ONE).unwrap_or(U256::MAX)
}

// ---------------------------------------------------------------------------
// Header + params
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub version: u32,
    /// Wire (internal) byte order.
    pub prev_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub time: u32,
    pub bits: u32,
    pub nonce: u32,
}

/// Display-order (reversed) hex, as used by RPCs and explorers.
pub fn display_hex(wire: &[u8; 32]) -> String {
    let mut r = *wire;
    r.reverse();
    hex::encode(r)
}

impl Header {
    pub fn from_hex(s: &str) -> Result<Header, HeaderError> {
        let b = hex::decode(s).map_err(|e| HeaderError::BadHex(e.to_string()))?;
        if b.len() != 80 {
            return Err(HeaderError::BadLength(b.len()));
        }
        let u = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        Ok(Header {
            version: u(0),
            prev_hash: b[4..36].try_into().unwrap(),
            merkle_root: b[36..68].try_into().unwrap(),
            time: u(68),
            bits: u(72),
            nonce: u(76),
        })
    }

    pub fn to_bytes(&self) -> [u8; 80] {
        let mut o = [0u8; 80];
        o[0..4].copy_from_slice(&self.version.to_le_bytes());
        o[4..36].copy_from_slice(&self.prev_hash);
        o[36..68].copy_from_slice(&self.merkle_root);
        o[68..72].copy_from_slice(&self.time.to_le_bytes());
        o[72..76].copy_from_slice(&self.bits.to_le_bytes());
        o[76..80].copy_from_slice(&self.nonce.to_le_bytes());
        o
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    /// Wire-order double-SHA256.
    pub fn hash(&self) -> [u8; 32] {
        Sha256::digest(Sha256::digest(self.to_bytes())).into()
    }

    pub fn hash_hex(&self) -> String {
        display_hex(&self.hash())
    }

    pub fn merkle_root_hex(&self) -> String {
        display_hex(&self.merkle_root)
    }
}

#[derive(Debug, Clone)]
pub struct Params {
    pub name: &'static str,
    /// Fixed difficulty: every header must carry exactly these `bits` (no retargeting).
    pub fixed_bits: u32,
    /// Display-hex hash of the pinned genesis header.
    pub genesis_hash: &'static str,
}

impl Params {
    /// Teranode / Bitcoin regtest: no retargeting, easy PoW.
    pub fn regtest() -> Params {
        Params {
            name: "regtest",
            fixed_bits: 0x207f_ffff,
            genesis_hash: "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
        }
    }
}

/// A validated header with its position and cumulative work.
#[derive(Debug, Clone)]
pub struct Entry {
    pub header: Header,
    pub hash: String,
    pub height: u32,
    pub chainwork: U256,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddOutcome {
    /// Already stored.
    Known,
    /// Extended the active chain by one.
    Extended,
    /// Stored on a side branch with less (or equal) work than the active tip.
    Fork,
    /// A side branch overtook the active chain.
    Reorg { fork_height: u32, depth: u32, old_tip: String, new_tip: String },
}

pub struct HeaderChain {
    params: Params,
    entries: HashMap<String, Entry>,
    /// Active chain: hash at each height (index = height).
    active: Vec<String>,
}

impl HeaderChain {
    pub fn new(params: Params) -> Self {
        HeaderChain { params, entries: HashMap::new(), active: Vec::new() }
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    /// Rebuild from stored headers, **re-validating every one** (storage is not trusted).
    /// Headers are applied parents-first; any that fail validation are dropped and counted.
    pub fn from_stored(params: Params, headers: Vec<(u32, String)>, now: u32) -> (HeaderChain, usize) {
        let mut chain = HeaderChain::new(params);
        let mut hs = headers;
        hs.sort_by_key(|(h, _)| *h);
        let mut rejected = 0;
        for (_, hex) in hs {
            match Header::from_hex(&hex).and_then(|h| chain.add_header(h, now)) {
                Ok(_) => {}
                Err(_) => rejected += 1,
            }
        }
        (chain, rejected)
    }

    pub fn tip(&self) -> Option<&Entry> {
        self.active.last().and_then(|h| self.entries.get(h))
    }

    pub fn tip_height(&self) -> Option<u32> {
        self.tip().map(|e| e.height)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn header_at_height(&self, height: u32) -> Option<&Entry> {
        self.active.get(height as usize).and_then(|h| self.entries.get(h))
    }

    pub fn get(&self, hash: &str) -> Option<&Entry> {
        self.entries.get(hash)
    }

    pub fn is_active(&self, hash: &str) -> bool {
        self.entries
            .get(hash)
            .map(|e| self.active.get(e.height as usize).map(|h| h == hash).unwrap_or(false))
            .unwrap_or(false)
    }

    /// Does `merkle_root_hex` (display order) match the active chain's header at `height`?
    pub fn verify_merkle_root(&self, height: u32, merkle_root_hex: &str) -> bool {
        self.header_at_height(height)
            .map(|e| e.header.merkle_root_hex().eq_ignore_ascii_case(merkle_root_hex))
            .unwrap_or(false)
    }

    pub fn add_header(&mut self, header: Header, now: u32) -> Result<AddOutcome, HeaderError> {
        let hash = header.hash_hex();
        if self.entries.contains_key(&hash) {
            return Ok(AddOutcome::Known);
        }

        // Difficulty and proof-of-work.
        if header.bits != self.params.fixed_bits {
            return Err(HeaderError::WrongDifficulty { got: header.bits, want: self.params.fixed_bits });
        }
        let target = compact_to_target(header.bits)?;
        let mut h = header.hash();
        h.reverse(); // to big-endian for numeric comparison
        if U256::from_be_bytes(h) > target {
            return Err(HeaderError::InsufficientWork);
        }
        if header.time > now.saturating_add(MAX_FUTURE_SECS) {
            return Err(HeaderError::TimeTooFarInFuture);
        }

        // Linkage.
        let (height, parent_work) = if self.entries.is_empty() {
            if hash != self.params.genesis_hash {
                return Err(HeaderError::NotGenesis(hash));
            }
            (0, U256::ZERO)
        } else {
            let prev = display_hex(&header.prev_hash);
            match self.entries.get(&prev) {
                Some(p) => (p.height + 1, p.chainwork),
                None => return Err(HeaderError::UnknownParent(prev)),
            }
        };

        let chainwork = parent_work
            .checked_add(work_for_target(target))
            .unwrap_or(U256::MAX);
        let entry = Entry { header, hash: hash.clone(), height, chainwork };
        let tip_work = self.tip().map(|t| t.chainwork);
        self.entries.insert(hash.clone(), entry);

        match tip_work {
            None => {
                self.active.push(hash);
                Ok(AddOutcome::Extended)
            }
            Some(tw) => {
                let extends_tip = self.active.last().map(|t| {
                    self.entries[&hash].header.prev_hash == self.entries[t].header.hash()
                }).unwrap_or(false);
                if extends_tip {
                    self.active.push(hash);
                    Ok(AddOutcome::Extended)
                } else if self.entries[&hash].chainwork > tw {
                    Ok(self.reorg_to(&hash))
                } else {
                    Ok(AddOutcome::Fork)
                }
            }
        }
    }

    fn reorg_to(&mut self, new_tip: &str) -> AddOutcome {
        let old_tip = self.active.last().cloned().unwrap_or_default();
        let old_height = self.active.len() as u32 - 1;
        // Walk the new branch back until it meets the active chain.
        let mut branch: Vec<String> = Vec::new();
        let mut cur = new_tip.to_string();
        loop {
            let e = &self.entries[&cur];
            if self.active.get(e.height as usize).map(|h| h == &cur).unwrap_or(false) {
                break;
            }
            branch.push(cur.clone());
            if e.height == 0 {
                break;
            }
            cur = display_hex(&e.header.prev_hash);
        }
        let fork_height = self.entries[&cur].height;
        self.active.truncate(fork_height as usize + 1);
        branch.reverse();
        self.active.extend(branch);
        AddOutcome::Reorg {
            fork_height,
            depth: old_height - fork_height,
            old_tip,
            new_tip: new_tip.to_string(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) const NOW: u32 = 2_000_000_000;

    /// Mine a header on top of `prev` (wire hash) at regtest difficulty.
    pub(crate) fn mine(prev: [u8; 32], salt: u32, bits: u32) -> Header {
        let target = compact_to_target(bits).unwrap();
        let mut h = Header { version: 1, prev_hash: prev, merkle_root: [salt as u8; 32], time: 1_700_000_000 + salt, bits, nonce: 0 };
        loop {
            let mut x = h.hash();
            x.reverse();
            if U256::from_be_bytes(x) <= target {
                return h;
            }
            h.nonce += 1;
        }
    }

    /// Test params whose genesis is a header we mine ourselves.
    pub(crate) fn fixture() -> (HeaderChain, Header) {
        let g = mine([0; 32], 0, 0x207f_ffff);
        let ghash: &'static str = Box::leak(g.hash_hex().into_boxed_str());
        let params = Params { name: "test", fixed_bits: 0x207f_ffff, genesis_hash: ghash };
        let mut c = HeaderChain::new(params);
        assert_eq!(c.add_header(g.clone(), NOW), Ok(AddOutcome::Extended));
        (c, g)
    }

    fn extend(c: &mut HeaderChain, parent: &Header, salt: u32) -> Header {
        let h = mine(parent.hash(), salt, 0x207f_ffff);
        c.add_header(h.clone(), NOW).unwrap();
        h
    }

    #[test]
    fn real_regtest_genesis_hashes_to_pinned_value() {
        // Header served by the spv-testnet stack at height 0.
        let h = Header {
            version: 1,
            prev_hash: [0; 32],
            merkle_root: hex::decode("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b").unwrap().into_iter().rev().collect::<Vec<_>>().try_into().unwrap(),
            time: 1296688602,
            bits: 545259519,
            nonce: 2,
        };
        assert_eq!(h.hash_hex(), Params::regtest().genesis_hash);
        let mut c = HeaderChain::new(Params::regtest());
        assert_eq!(c.add_header(h, NOW), Ok(AddOutcome::Extended));
    }

    #[test]
    fn wrong_genesis_is_rejected() {
        let mut c = HeaderChain::new(Params::regtest());
        let h = mine([0; 32], 7, 0x207f_ffff);
        assert!(matches!(c.add_header(h, NOW), Err(HeaderError::NotGenesis(_))));
    }

    #[test]
    fn extends_and_looks_up_by_height() {
        let (mut c, g) = fixture();
        let a = extend(&mut c, &g, 1);
        assert_eq!(c.tip_height(), Some(1));
        assert_eq!(c.header_at_height(1).unwrap().hash, a.hash_hex());
        assert!(c.verify_merkle_root(1, &a.merkle_root_hex()));
        assert!(!c.verify_merkle_root(1, &g.merkle_root_hex()));
        assert!(!c.verify_merkle_root(9, &a.merkle_root_hex()));
    }

    #[test]
    fn insufficient_work_is_rejected() {
        let (mut c, g) = fixture();
        // Find a header that fails PoW at regtest target.
        let mut h = Header { version: 1, prev_hash: g.hash(), merkle_root: [9; 32], time: 1_700_000_001, bits: 0x207f_ffff, nonce: 0 };
        let target = compact_to_target(h.bits).unwrap();
        loop {
            let mut x = h.hash();
            x.reverse();
            if U256::from_be_bytes(x) > target { break; }
            h.nonce += 1;
        }
        assert_eq!(c.add_header(h, NOW), Err(HeaderError::InsufficientWork));
    }

    #[test]
    fn wrong_difficulty_is_rejected() {
        let (mut c, g) = fixture();
        let h = mine(g.hash(), 3, 0x2000_ffff); // harder than allowed, valid PoW, wrong bits
        assert!(matches!(c.add_header(h, NOW), Err(HeaderError::WrongDifficulty { .. })));
    }

    #[test]
    fn unknown_parent_is_rejected() {
        let (mut c, _) = fixture();
        let h = mine([0xAB; 32], 4, 0x207f_ffff);
        assert!(matches!(c.add_header(h, NOW), Err(HeaderError::UnknownParent(_))));
    }

    #[test]
    fn future_timestamp_is_rejected() {
        let (mut c, g) = fixture();
        let mut h = Header { version: 1, prev_hash: g.hash(), merkle_root: [5; 32], time: NOW + MAX_FUTURE_SECS + 1, bits: 0x207f_ffff, nonce: 0 };
        let target = compact_to_target(h.bits).unwrap();
        loop {
            let mut x = h.hash();
            x.reverse();
            if U256::from_be_bytes(x) <= target { break; }
            h.nonce += 1;
        }
        assert_eq!(c.add_header(h, NOW), Err(HeaderError::TimeTooFarInFuture));
    }

    #[test]
    fn duplicate_is_known() {
        let (mut c, g) = fixture();
        let a = extend(&mut c, &g, 1);
        assert_eq!(c.add_header(a, NOW), Ok(AddOutcome::Known));
    }

    #[test]
    fn equal_work_fork_does_not_displace_active_chain() {
        let (mut c, g) = fixture();
        let a1 = extend(&mut c, &g, 1);
        let b1 = mine(g.hash(), 11, 0x207f_ffff);
        assert_eq!(c.add_header(b1.clone(), NOW), Ok(AddOutcome::Fork));
        assert_eq!(c.tip().unwrap().hash, a1.hash_hex());
        assert!(!c.is_active(&b1.hash_hex()));
    }

    #[test]
    fn heavier_branch_triggers_reorg_and_switches_active_chain() {
        let (mut c, g) = fixture();
        let a1 = extend(&mut c, &g, 1);
        let a2 = extend(&mut c, &a1, 2);
        let b1 = mine(g.hash(), 11, 0x207f_ffff);
        assert_eq!(c.add_header(b1.clone(), NOW), Ok(AddOutcome::Fork));
        let b2 = mine(b1.hash(), 12, 0x207f_ffff);
        assert_eq!(c.add_header(b2.clone(), NOW), Ok(AddOutcome::Fork)); // tie at height 2
        let b3 = mine(b2.hash(), 13, 0x207f_ffff);
        match c.add_header(b3.clone(), NOW).unwrap() {
            AddOutcome::Reorg { fork_height, depth, old_tip, new_tip } => {
                assert_eq!(fork_height, 0);
                assert_eq!(depth, 2);
                assert_eq!(old_tip, a2.hash_hex());
                assert_eq!(new_tip, b3.hash_hex());
            }
            other => panic!("expected reorg, got {:?}", other),
        }
        assert_eq!(c.tip_height(), Some(3));
        assert!(c.is_active(&b1.hash_hex()) && !c.is_active(&a1.hash_hex()));
        // Roots now come from the new branch.
        assert!(c.verify_merkle_root(1, &b1.merkle_root_hex()));
        assert!(!c.verify_merkle_root(1, &a1.merkle_root_hex()));
    }

    #[test]
    fn from_stored_revalidates_and_drops_garbage() {
        let (mut c, g) = fixture();
        let a1 = extend(&mut c, &g, 1);
        let params = c.params().clone();
        let wrong_bits = mine(a1.hash(), 2, 0x2000_ffff); // valid PoW, wrong difficulty
        let stored = vec![
            (1, a1.to_hex()),
            (0, g.to_hex()),
            (5, "zz".to_string()),
            (2, wrong_bits.to_hex()),
        ];
        let (r, rejected) = HeaderChain::from_stored(params, stored, NOW);
        assert_eq!(r.tip_height(), Some(1));
        assert_eq!(rejected, 2);
    }

    #[test]
    fn compact_and_work_known_values() {
        // Bitcoin genesis difficulty-1 target 0x1d00ffff -> work 0x100010001.
        let t = compact_to_target(0x1d00_ffff).unwrap();
        assert_eq!(work_for_target(t), U256([0x1_0001_0001, 0, 0, 0]));
        // Regtest: target ~2^255 -> work 2.
        let rt = compact_to_target(0x207f_ffff).unwrap();
        assert_eq!(work_for_target(rt), U256([2, 0, 0, 0]));
        assert!(compact_to_target(0x0180_0001).is_err()); // negative
        assert!(compact_to_target(0x0100_0000).is_err()); // zero mantissa
    }
}
