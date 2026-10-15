//! Verified block-header chain (Wallet-Hardening WS4).
//!
//! Pure logic, no I/O: validates headers, tracks every known fork, and picks the
//! active chain by **most cumulative work** — never by what a server calls its tip.
//! Merkle proofs are then checked against this chain's roots, not against a
//! third-party API's say-so.
//!
//! Enforced per header: 80-byte layout, proof-of-work (`hash <= target(bits)`),
//! `bits` equal to what the network's difficulty rule requires, parent known,
//! `height = parent + 1`, timestamp no more than 2 h in the future, and the pinned hash at
//! every checkpoint height. The first header must be the pinned **anchor**: the genesis for a
//! full chain, or a pinned checkpoint for a **lite** chain that verifies only a window at the tip.
//! Below a lite chain's `assume_valid_height` the difficulty rule cannot be computed (its window
//! lies below the anchor) and is not checked; proof-of-work on the header's own bits, bounded by
//! the network's powLimit, linkage, time and checkpoints still are.
//!
//! Networks: **regtest** (fixed difficulty), **teratestnet** (cw-144 DAA plus the testnet
//! 20-minute minimum-difficulty rule, ported from Teranode `services/blockchain/Difficulty.go`
//! and `go-chaincfg` `TeraTestNetParams`), full from the genesis or lite from the 18000
//! checkpoint, and **mainnet**, lite only (`Params::mainnet_lite`: the same DAA without the
//! minimum-difficulty rule, anchored at the 945000 checkpoint). A full mainnet chain is not
//! offered: it would need the pre-DAA difficulty regimes. Median-time-past is not enforced yet.

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
    NotAnchor(String),
    TimeTooFarInFuture,
    CheckpointMismatch { height: u32, got: String },
    /// The DAA's projected work does not fit in 256 bits (needs 144 blocks of work above 2^246).
    DifficultyOverflow,
    /// The DAA produced a zero target (projected work above 2^255).
    ZeroTarget,
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

    fn checked_mul_u64(self, m: u64) -> Option<U256> {
        let mut r = [0u64; 4];
        let mut carry = 0u128;
        for i in 0..4 {
            let p = self.0[i] as u128 * m as u128 + carry;
            r[i] = p as u64;
            carry = p >> 64;
        }
        if carry != 0 { None } else { Some(U256(r)) }
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

/// Encode a target as compact `bits` (Bitcoin's `GetCompact` / Teranode `BigToCompact`).
pub fn target_to_compact(target: U256) -> u32 {
    let b = target.to_be_bytes();
    let Some(first) = b.iter().position(|&x| x != 0) else { return 0 };
    let mut size = (32 - first) as u32;
    let byte = |i: usize| if i < 32 { b[i] as u32 } else { 0 };
    // Fewer than 3 significant bytes: the zero padding read past the end left-aligns the
    // mantissa, which is Bitcoin's `low64 << 8 * (3 - size)`.
    let mut mant = (byte(first) << 16) | (byte(first + 1) << 8) | byte(first + 2);
    if mant & 0x0080_0000 != 0 {
        mant >>= 8;
        size += 1;
    }
    (size << 24) | mant
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

/// Wire-order bytes from display-order hex, as explorers report a hash.
fn wire_hash(display: &str) -> Result<[u8; 32], HeaderError> {
    let b = hex::decode(display).map_err(|e| HeaderError::BadHex(e.to_string()))?;
    if b.len() != 32 {
        return Err(HeaderError::BadLength(b.len()));
    }
    let mut wire = [0u8; 32];
    wire.copy_from_slice(&b);
    wire.reverse();
    Ok(wire)
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

    /// Assemble a header from its fields as block explorers report them: display-order hex for
    /// the two hashes, `bits` as the compact integer.
    pub fn from_fields(
        version: u32,
        prev_hash_display_hex: &str,
        merkle_root_display_hex: &str,
        time: u32,
        bits: u32,
        nonce: u32,
    ) -> Result<Header, HeaderError> {
        Ok(Header {
            version,
            prev_hash: wire_hash(prev_hash_display_hex)?,
            merkle_root: wire_hash(merkle_root_display_hex)?,
            time,
            bits,
            nonce,
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

/// How a network sets the `bits` a header must carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    /// Every header carries exactly these `bits` (no retargeting).
    Fixed(u32),
    /// cw-144 DAA (bitcoin-sv `pow.cpp`, Teranode `Difficulty.go`). With
    /// `reduce_min_difficulty`, a header more than two spacings after its parent may use
    /// `pow_limit_bits` (the testnet rule).
    Daa { pow_limit_bits: u32, spacing_secs: u32, reduce_min_difficulty: bool },
}

/// DAA window, and the parent height below which the window does not exist yet
/// (Teranode: `blockHeight < DifficultyAdjustmentWindow + 4` -> powLimit).
const DAA_WINDOW: u32 = 144;
const DAA_MIN_PARENT_HEIGHT: u32 = DAA_WINDOW + 4;

#[derive(Debug, Clone)]
pub struct Params {
    pub name: &'static str,
    pub difficulty: Difficulty,
    /// `(height, display-hex hash)` of the first header the chain accepts: the genesis for a full
    /// chain, a pinned checkpoint for a lite one.
    pub anchor: (u32, &'static str),
    /// Below this height the difficulty rule is not checked (its window would lie below the
    /// anchor); `0` = always checked.
    pub assume_valid_height: u32,
    /// `(height, display-hex hash)` every chain must pass through.
    pub checkpoints: &'static [(u32, &'static str)],
    /// Real coins. Decides what `getNetwork` answers and that the header trackers must agree
    /// two-to-one; every test network is `false`.
    pub is_mainnet: bool,
}

/// Bitcoin SV mainnet checkpoints: the union of bitcoin-sv `src/chainparams.cpp`
/// `checkpointData` (16, up to 530359) and `bsv-blockchain/go-chaincfg` `MainNetParams`
/// (34, up to 945000), both fetched from GitHub master on 2026-10-15; the twelve heights they
/// share carry identical hashes. Pinned from these two node configurations; the live cross-check
/// against chaintracks and WhatsOnChain (`tests/spv-ttn/mainnet-anchor.mjs`) is still owed.
const MAINNET_CHECKPOINTS: &[(u32, &str)] = &[
                (11111, "0000000069e244f73d78e8fd29ba2fd2ed618bd6fa2ee92559f542fdb26e7c1d"),
                (33333, "000000002dd5588a74784eaa7ab0507a18ad16a236e7b1ce69f00d7ddfb5d0a6"),
                (74000, "0000000000573993a3c9e41ce34471c079dcf5f52a0e824a81e7f953b8661a20"),
                (105000, "00000000000291ce28027faea320c8d2b054b2e0fe44a773f3eefb151d6bdc97"),
                (134444, "00000000000005b12ffd4cd315cd34ffd4a594f430ac814c91184a0d42d2b0fe"),
                (168000, "000000000000099e61ea72015e79632f216fe6cb33d7899acb35b75c8303b763"),
                (193000, "000000000000059f452a5f7340de6682a977387c17010ff6e6c3bd83ca8b1317"),
                (210000, "000000000000048b95347e83192f69cf0366076336c639f9b7228e9ba171342e"),
                (216116, "00000000000001b4f4b433e81ee46494af945cf96014816a4e2370f11b23df4e"),
                (225430, "00000000000001c108384350f74090433e7fcf79a606b8e797f065b130575932"),
                (250000, "000000000000003887df1f29024b06fc2200b55f8af8f35453d7be294df2d214"),
                (267300, "000000000000000a83fbd660e918f218bf37edd92b748ad940483c7c116179ac"),
                (279000, "0000000000000001ae8c72a0b0c301f67e3afca10e819efa9041e458e9bd7e40"),
                (295000, "00000000000000004d9b4ef50f0f9d686fd69db2e03af35a100370c64632a983"),
                (300255, "0000000000000000162804527c6e9b9f0563a280525f9d08c12041def0a0f3b2"),
                (319400, "000000000000000021c6052e9becade189495d1c539aa37c58917305fd15f13b"),
                (343185, "0000000000000000072b8bf361d01a6ba7d445dd024203fafc78768ed4368554"),
                (352940, "000000000000000010755df42dba556bb72be6a32f3ce0b6941ce4430152c9ff"),
                (382320, "00000000000000000a8dc6ed5b133d0eb2fd6af56203e4159789b092defd8ab2"),
                (400000, "000000000000000004ec466ce4732fe6f1ed1cddc2ed4b328fff5224276e3f6f"),
                (430000, "000000000000000001868b2bb3a285f3cc6b33ea234eb70facf4dcdf22186b87"),
                (470000, "0000000000000000006c539c722e280a0769abd510af0073430159d71e6d7589"),
                (478558, "0000000000000000011865af4122fe3b144e2cbeea86142e8ff2fb4107352d43"),
                (504031, "0000000000000000011ebf65b60d0a3de80b8175be709d653b4c1a1beeb6ab9c"),
                (510000, "00000000000000000367922b6457e21d591ef86b360d78a598b14c2f1f6b0e04"),
                (530359, "0000000000000000011ada8bd08f46074f44a8f155396f43e38acf9501c49103"),
                (552979, "0000000000000000015648768ac1b788a83187d706f858919fcc5c096b76fbf2"),
                (556767, "000000000000000001d956714215d96ffc00e0afda4cd0a96c96f8d802b1662b"),
                (600000, "00000000000000000866448ef293f900812d4af8e08cbe7ef62888eee9d29c4c"),
                (650000, "00000000000000000310c17bbb4f3f8e5371a41ec2cee36a39876042019b725b"),
                (700000, "00000000000000000e155235fd83a8757c44c6299e63104fb12632368f3f0cc9"),
                (750000, "000000000000000006296f1e5437dd6c01b9b5471691a89a9c7d8e9f06920da5"),
                (800000, "000000000000000000ad9056924410005d91b57f100bce345944e5caf56e8565"),
                (850000, "0000000000000000039302a65227ab75fd93904ebe2e62421d1c66b15808b23b"),
                (868500, "00000000000000000a4c8747ee369c2f4645cf7b55db534851fdc1a040f74de4"),
                (900000, "000000000000000002feb6a36e1b8bf81409d0252e285449e3d0ef2388c5506a"),
                (938000, "000000000000000002616a5ad2413acf7cf122c5aa27fbd29bfa0e8c12dd455b"),
                (945000, "00000000000000000c39d94e19d6a55cfb0454918df1814fbcd919353a6e1f82")
];

impl Params {
    /// Teranode / Bitcoin regtest: no retargeting, easy PoW.
    pub fn regtest() -> Params {
        Params {
            name: "regtest",
            difficulty: Difficulty::Fixed(0x207f_ffff),
            anchor: (0, "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206"),
            assume_valid_height: 0,
            checkpoints: &[],
            is_mainnet: false,
        }
    }

    /// Teranode test network (`go-chaincfg` v1.5.8 `TeraTestNetParams`).
    pub fn teratestnet() -> Params {
        Params {
            name: "teratestnet",
            difficulty: Difficulty::Daa { pow_limit_bits: 0x1d00_ffff, spacing_secs: 600, reduce_min_difficulty: true },
            anchor: (0, "000000000499eabba0a88f5b3747231c74b9191c1a4a04b2c2ea817976b7776d"),
            assume_valid_height: 0,
            checkpoints: &[
                (5000, "00000000038392cdae17df19464dfcccadb754b24441d11f3294cc04a90749b8"),
                (9500, "00000000bd750801352dd82fcb0e675d62a308b38a6486698308ae06fcb3bc10"),
                (18000, "00000000174095c3c94343b28d29f45010b44bf221e22df5cea6e0ce547a10b5"),
            ],
            is_mainnet: false,
        }
    }

    /// Bitcoin SV mainnet, **lite only**: a verified window from the 945000 checkpoint (the
    /// newest in `go-chaincfg`) to the tip. The rule is the cw-144 DAA above the DAA fork at
    /// 504031 (`go-chaincfg` `DaaForkHeight`), **without** the testnet minimum-difficulty rule
    /// (`ReduceMinDifficulty: false`), powLimit `0x1d00ffff`, 600 s spacing. The full rule
    /// resumes at anchor + 147; below it PoW is still enforced. Heights below the anchor are the
    /// header trackers' to answer, and on mainnet two of them must agree.
    pub fn mainnet_lite() -> Params {
        Params {
            name: "mainnet-lite",
            difficulty: Difficulty::Daa { pow_limit_bits: 0x1d00_ffff, spacing_secs: 600, reduce_min_difficulty: false },
            anchor: (945000, "00000000000000000c39d94e19d6a55cfb0454918df1814fbcd919353a6e1f82"),
            assume_valid_height: 945000 + 147,
            checkpoints: MAINNET_CHECKPOINTS,
            is_mainnet: true,
        }
    }

    /// Teranode test network, **lite**: a verified window from the pinned checkpoint at 18000
    /// (`go-chaincfg` `TeraTestNetParams`, confirmed by a full live sync on 2026-10-14) to the
    /// tip. Heights below the anchor are not this chain's to answer. The full difficulty rule
    /// resumes at anchor + 147, the first height whose DAA window (median-of-3 at `parent` and at
    /// `parent - 144`) lies inside the chain; below it PoW is still enforced.
    pub fn teratestnet_lite() -> Params {
        Params {
            name: "teratestnet-lite",
            anchor: (18000, "00000000174095c3c94343b28d29f45010b44bf221e22df5cea6e0ce547a10b5"),
            assume_valid_height: 18000 + 147,
            ..Params::teratestnet()
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

    /// Index of `height` in `active` (the anchor is index 0); `None` below the anchor.
    fn idx(&self, height: u32) -> Option<usize> {
        height.checked_sub(self.params.anchor.0).map(|i| i as usize)
    }

    /// Hash of the active header at `height`, if the chain has one.
    fn active_at(&self, height: u32) -> Option<&String> {
        self.idx(height).and_then(|i| self.active.get(i))
    }

    pub fn header_at_height(&self, height: u32) -> Option<&Entry> {
        self.active_at(height).and_then(|h| self.entries.get(h))
    }

    pub fn get(&self, hash: &str) -> Option<&Entry> {
        self.entries.get(hash)
    }

    pub fn is_active(&self, hash: &str) -> bool {
        self.entries
            .get(hash)
            .map(|e| self.active_at(e.height).map(|h| h == hash).unwrap_or(false))
            .unwrap_or(false)
    }

    /// Does `merkle_root_hex` (display order) match the active chain's header at `height`?
    pub fn verify_merkle_root(&self, height: u32, merkle_root_hex: &str) -> bool {
        self.header_at_height(height)
            .map(|e| e.header.merkle_root_hex().eq_ignore_ascii_case(merkle_root_hex))
            .unwrap_or(false)
    }

    /// Does `merkle_root_hex` belong to a header at `height` on a competing branch that is still
    /// in the running, i.e. a branch whose tip has at least the active tip's work? Such a root
    /// is undecided. A branch that has clearly lost (less work, e.g. orphaned by a reorg) is not
    /// in the running: its proofs are plainly not on the active chain.
    pub fn side_branch_has_root(&self, height: u32, merkle_root_hex: &str) -> bool {
        let Some(active_tip) = self.tip() else { return false };
        for t in self.entries.values() {
            if t.hash == active_tip.hash || t.chainwork < active_tip.chainwork || t.height < height {
                continue;
            }
            // Walk this competing tip back to `height`.
            let mut cur = t;
            while cur.height > height {
                let mut prev = cur.header.prev_hash;
                prev.reverse();
                match self.entries.get(&hex::encode(prev)) {
                    Some(p) => cur = p,
                    None => break,
                }
            }
            if cur.height == height
                && self.active_at(height).map(|h| h != &cur.hash).unwrap_or(true)
                && cur.header.merkle_root_hex().eq_ignore_ascii_case(merkle_root_hex)
            {
                return true;
            }
        }
        false
    }

    pub fn add_header(&mut self, header: Header, now: u32) -> Result<AddOutcome, HeaderError> {
        let hash = header.hash_hex();
        if self.entries.contains_key(&hash) {
            return Ok(AddOutcome::Known);
        }

        // Linkage first: the difficulty rule is a function of the parent's ancestry. The anchor
        // (the genesis, or a lite chain's pinned checkpoint) has no parent here; it is accepted
        // only as the pinned header, at its pinned height.
        let (height, parent_work) = if self.entries.is_empty() {
            if hash != self.params.anchor.1 {
                return Err(HeaderError::NotAnchor(hash));
            }
            (self.params.anchor.0, U256::ZERO)
        } else {
            let prev = display_hex(&header.prev_hash);
            let Some(p) = self.entries.get(&prev) else {
                return Err(HeaderError::UnknownParent(prev));
            };
            let height = p.height + 1;
            // Below `assume_valid_height` the rule's window would reach below the anchor, so the
            // header's own bits stand: bounded by powLimit and proven by its hash, below.
            if height >= self.params.assume_valid_height {
                let want = self.required_bits(p, header.time)?;
                if header.bits != want {
                    return Err(HeaderError::WrongDifficulty { got: header.bits, want });
                }
            }
            (height, p.chainwork)
        };

        // Proof-of-work. A target easier than the network's powLimit is not a target at all
        // (Bitcoin Core `CheckProofOfWork`); on a lite chain this is what bounds how cheap a
        // header below `assume_valid_height` can be.
        let target = compact_to_target(header.bits)?;
        if let Difficulty::Daa { pow_limit_bits, .. } = self.params.difficulty {
            if target > compact_to_target(pow_limit_bits)? {
                return Err(HeaderError::BadBits(header.bits));
            }
        }
        let mut h = header.hash();
        h.reverse(); // to big-endian for numeric comparison
        if U256::from_be_bytes(h) > target {
            return Err(HeaderError::InsufficientWork);
        }
        if header.time > now.saturating_add(MAX_FUTURE_SECS) {
            return Err(HeaderError::TimeTooFarInFuture);
        }
        if let Some((_, want)) = self.params.checkpoints.iter().find(|(ch, _)| *ch == height) {
            if hash != *want {
                return Err(HeaderError::CheckpointMismatch { height, got: hash });
            }
        }

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

    /// The `bits` a header on top of `parent`, timestamped `time`, must carry.
    fn required_bits(&self, parent: &Entry, time: u32) -> Result<u32, HeaderError> {
        let (pow_limit_bits, spacing, reduce_min) = match self.params.difficulty {
            Difficulty::Fixed(bits) => return Ok(bits),
            Difficulty::Daa { pow_limit_bits, spacing_secs, reduce_min_difficulty } => {
                (pow_limit_bits, spacing_secs, reduce_min_difficulty)
            }
        };
        if reduce_min && time as u64 > parent.header.time as u64 + 2 * spacing as u64 {
            return Ok(pow_limit_bits);
        }
        if parent.height < DAA_MIN_PARENT_HEIGHT {
            return Ok(pow_limit_bits);
        }
        let last = self.suitable(parent)?;
        let first = self.suitable(self.ancestor(parent, DAA_WINDOW)?)?;

        // Bound the timespan so one adjustment moves difficulty at most 2x (72..288 spacings).
        let spacing = spacing as u64;
        let span = (last.header.time as u64).saturating_sub(first.header.time as u64).clamp(72 * spacing, 288 * spacing);
        let work = last.chainwork.sub(first.chainwork);
        let projected = work
            .checked_mul_u64(spacing)
            .ok_or(HeaderError::DifficultyOverflow)?
            .div(U256([span, 0, 0, 0]));
        if projected.is_zero() {
            return Ok(last.header.bits);
        }
        // target = (2^256 - projected) / projected; 2^256 - x is ~x + 1 in 256 bits.
        let neg = U256([!projected.0[0], !projected.0[1], !projected.0[2], !projected.0[3]])
            .checked_add(U256::ONE)
            .unwrap_or(U256::ZERO);
        let target = neg.div(projected).min(compact_to_target(pow_limit_bits)?);
        if target.is_zero() {
            return Err(HeaderError::ZeroTarget);
        }
        Ok(target_to_compact(target))
    }

    fn parent_of(&self, e: &Entry) -> Result<&Entry, HeaderError> {
        let prev = display_hex(&e.header.prev_hash);
        self.entries.get(&prev).ok_or(HeaderError::UnknownParent(prev))
    }

    fn ancestor<'a>(&'a self, mut e: &'a Entry, back: u32) -> Result<&'a Entry, HeaderError> {
        for _ in 0..back {
            e = self.parent_of(e)?;
        }
        Ok(e)
    }

    /// Median-by-time of `e`, its parent and grandparent, sorted oldest-first with the same
    /// three-step network as bitcoin-sv / Teranode so ties pick the same block.
    fn suitable<'a>(&'a self, e: &'a Entry) -> Result<&'a Entry, HeaderError> {
        let p = self.parent_of(e)?;
        let mut s = [self.parent_of(p)?, p, e];
        if s[0].header.time > s[2].header.time {
            s.swap(0, 2);
        }
        if s[0].header.time > s[1].header.time {
            s.swap(0, 1);
        }
        if s[1].header.time > s[2].header.time {
            s.swap(1, 2);
        }
        Ok(s[1])
    }

    fn reorg_to(&mut self, new_tip: &str) -> AddOutcome {
        let old_tip = self.active.last().cloned().unwrap_or_default();
        let old_height = self.params.anchor.0 + self.active.len() as u32 - 1;
        // Walk the new branch back until it meets the active chain.
        let mut branch: Vec<String> = Vec::new();
        let mut cur = new_tip.to_string();
        loop {
            let e = &self.entries[&cur];
            if self.active_at(e.height).map(|h| h == &cur).unwrap_or(false) {
                break;
            }
            branch.push(cur.clone());
            if e.height == self.params.anchor.0 {
                break;
            }
            cur = display_hex(&e.header.prev_hash);
        }
        let fork_height = self.entries[&cur].height;
        self.active.truncate(self.idx(fork_height).unwrap_or(0) + 1);
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
        let params = Params { name: "test", difficulty: Difficulty::Fixed(0x207f_ffff), anchor: (0, ghash), assume_valid_height: 0, checkpoints: &[], is_mainnet: false };
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
        assert_eq!(h.hash_hex(), Params::regtest().anchor.1);
        let mut c = HeaderChain::new(Params::regtest());
        assert_eq!(c.add_header(h, NOW), Ok(AddOutcome::Extended));
    }

    #[test]
    fn wrong_genesis_is_rejected() {
        let mut c = HeaderChain::new(Params::regtest());
        let h = mine([0; 32], 7, 0x207f_ffff);
        assert!(matches!(c.add_header(h, NOW), Err(HeaderError::NotAnchor(_))));
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

    // -----------------------------------------------------------------------
    // teratestnet: real headers 0..=700 served by the public Arcade chaintracks
    // (`/chaintracks/v2/headers?height=0&count=701`). Heights 0-148 carry powLimit,
    // 149 is the first DAA-computed target, 398/399 are 20-minute min-difficulty blocks.
    // -----------------------------------------------------------------------

    const TTN_HEADERS: &[u8] = include_bytes!("../tests/fixtures/ttn_headers_0_700.bin");

    fn ttn(height: usize) -> Header {
        Header::from_hex(&hex::encode(&TTN_HEADERS[height * 80..height * 80 + 80])).unwrap()
    }

    /// A teratestnet chain holding the real headers `0..=last`.
    fn ttn_chain(last: usize) -> HeaderChain {
        let mut c = HeaderChain::new(Params::teratestnet());
        for h in 0..=last {
            assert_eq!(c.add_header(ttn(h), NOW), Ok(AddOutcome::Extended), "real header {}", h);
        }
        c
    }

    #[test]
    fn ttn_genesis_hashes_to_pinned_value() {
        assert_eq!(ttn(0).hash_hex(), Params::teratestnet().anchor.1);
    }

    #[test]
    fn ttn_real_chain_is_accepted() {
        let c = ttn_chain(TTN_HEADERS.len() / 80 - 1);
        assert_eq!(c.tip_height(), Some(700));
        assert_eq!(c.tip().unwrap().hash, ttn(700).hash_hex());
    }

    #[test]
    fn ttn_daa_header_with_other_bits_is_rejected() {
        let mut c = ttn_chain(148);
        let mut h = ttn(149);
        h.bits = 0x1c7f_ff81; // one off the DAA target
        assert!(matches!(c.add_header(h, NOW), Err(HeaderError::WrongDifficulty { .. })));
        assert_eq!(c.add_header(ttn(149), NOW), Ok(AddOutcome::Extended));
    }

    #[test]
    fn ttn_min_difficulty_needs_more_than_twenty_minutes() {
        let mut c = ttn_chain(397);
        let mut h = ttn(398); // powLimit bits, real gap ~20 days
        h.time = ttn(397).time + 1200; // exactly 2x spacing: not enough
        assert!(matches!(c.add_header(h, NOW), Err(HeaderError::WrongDifficulty { .. })));
        assert_eq!(c.add_header(ttn(398), NOW), Ok(AddOutcome::Extended));
    }

    #[test]
    fn ttn_pow_limit_bits_at_a_daa_height_are_rejected() {
        let mut c = ttn_chain(149);
        let mut h = ttn(150); // 63 s after its parent
        h.bits = 0x1d00_ffff;
        assert!(matches!(c.add_header(h, NOW), Err(HeaderError::WrongDifficulty { .. })));
    }

    #[test]
    fn regtest_params_reject_the_ttn_genesis() {
        let mut c = HeaderChain::new(Params::regtest());
        assert!(matches!(c.add_header(ttn(0), NOW), Err(HeaderError::NotAnchor(_))));
    }

    #[test]
    fn ttn_from_stored_revalidates_the_real_chain() {
        let mut stored: Vec<(u32, String)> = (0..=700u32).map(|h| (h, ttn(h as usize).to_hex())).collect();
        stored.reverse(); // order in storage does not matter
        let (c, rejected) = HeaderChain::from_stored(Params::teratestnet(), stored, NOW);
        assert_eq!(rejected, 0);
        assert_eq!(c.tip_height(), Some(700));
    }

    // -----------------------------------------------------------------------
    // Lite chain: a window anchored at a pinned checkpoint instead of the genesis. The same real
    // teratestnet headers, anchored at 300: heights 301..=446 lie below `assume_valid_height`
    // (the DAA window would reach below the anchor), 447 is the first height with the full rule.
    // -----------------------------------------------------------------------

    const LITE_ANCHOR: u32 = 300;

    /// teratestnet rules anchored at real header `LITE_ANCHOR`.
    fn ttn_lite_params() -> Params {
        let anchor_hash: &'static str = Box::leak(ttn(LITE_ANCHOR as usize).hash_hex().into_boxed_str());
        Params {
            name: "ttn-lite-test",
            anchor: (LITE_ANCHOR, anchor_hash),
            assume_valid_height: LITE_ANCHOR + 147,
            ..Params::teratestnet()
        }
    }

    fn ttn_lite_chain(last: usize) -> HeaderChain {
        let mut c = HeaderChain::new(ttn_lite_params());
        for h in LITE_ANCHOR as usize..=last {
            assert_eq!(c.add_header(ttn(h), NOW), Ok(AddOutcome::Extended), "real header {}", h);
        }
        c
    }

    #[test]
    fn lite_chain_starts_at_the_anchor_and_accepts_the_window() {
        let c = ttn_lite_chain(700);
        assert_eq!(c.tip_height(), Some(700));
        assert_eq!(c.len(), 401);
        assert_eq!(c.header_at_height(LITE_ANCHOR).unwrap().hash, ttn(LITE_ANCHOR as usize).hash_hex());
        assert_eq!(c.header_at_height(LITE_ANCHOR).unwrap().height, LITE_ANCHOR);
        assert!(c.header_at_height(LITE_ANCHOR - 1).is_none(), "below the anchor is not the window's to answer");
        assert!(c.header_at_height(0).is_none());
        assert!(c.verify_merkle_root(500, &ttn(500).merkle_root_hex()));
        assert!(!c.verify_merkle_root(500, &ttn(501).merkle_root_hex()));
        assert!(c.is_active(&ttn(700).hash_hex()));
    }

    #[test]
    fn lite_first_header_must_be_the_anchor() {
        let mut c = HeaderChain::new(ttn_lite_params());
        for h in [0usize, 299, 301] {
            assert!(matches!(c.add_header(ttn(h), NOW), Err(HeaderError::NotAnchor(_))), "header {}", h);
        }
        assert_eq!(c.add_header(ttn(LITE_ANCHOR as usize), NOW), Ok(AddOutcome::Extended));
        assert_eq!(c.tip_height(), Some(LITE_ANCHOR));
    }

    #[test]
    fn below_assume_valid_pow_is_enforced_but_bits_are_not_compared() {
        // 300..=349 are accepted although the DAA cannot be computed for them (the window test
        // above); what is still checked here is the proof-of-work on each header's own bits.
        let mut c = ttn_lite_chain(349);
        let mut nonce = ttn(350);
        nonce.nonce = nonce.nonce.wrapping_add(1);
        assert_eq!(c.add_header(nonce, NOW), Err(HeaderError::InsufficientWork));
        let mut off = ttn(350);
        off.bits = off.bits.wrapping_add(1);
        assert_eq!(c.add_header(off, NOW), Err(HeaderError::InsufficientWork), "changed bits: the hash no longer meets its own target");
        let mut easy = ttn(350);
        easy.bits = 0x1e00_ffff; // easier than teratestnet's powLimit: not a target at all
        assert_eq!(c.add_header(easy, NOW), Err(HeaderError::BadBits(0x1e00_ffff)));
        assert_eq!(c.add_header(ttn(350), NOW), Ok(AddOutcome::Extended));
    }

    #[test]
    fn the_full_difficulty_rule_resumes_at_assume_valid_height() {
        let mut c = ttn_lite_chain(446);
        let mut h = ttn(447);
        h.bits = h.bits.wrapping_add(1);
        assert!(matches!(c.add_header(h, NOW), Err(HeaderError::WrongDifficulty { .. })), "447 is the first height whose DAA window lies inside the chain");
        assert_eq!(c.add_header(ttn(447), NOW), Ok(AddOutcome::Extended));
    }

    #[test]
    fn assume_valid_height_must_be_at_least_anchor_plus_147() {
        // One less and the median-of-3 at `parent - 144` needs the header below the anchor.
        let mut p = ttn_lite_params();
        p.assume_valid_height = LITE_ANCHOR + 146;
        let mut c = HeaderChain::new(p);
        for h in LITE_ANCHOR..LITE_ANCHOR + 146 {
            c.add_header(ttn(h as usize), NOW).unwrap();
        }
        assert!(matches!(c.add_header(ttn(446), NOW), Err(HeaderError::UnknownParent(_))));
    }

    #[test]
    fn teratestnet_lite_is_anchored_at_the_18000_checkpoint() {
        let full = Params::teratestnet();
        let lite = Params::teratestnet_lite();
        assert_eq!(lite.name, "teratestnet-lite");
        assert_eq!(lite.anchor, (18000, full.checkpoints.iter().find(|(h, _)| *h == 18000).unwrap().1));
        assert_eq!(lite.assume_valid_height, 18000 + 147);
        assert_eq!(lite.difficulty, full.difficulty);
        assert_eq!(lite.checkpoints, full.checkpoints);
        assert_eq!(full.anchor.0, 0, "the full chain is anchored at its genesis");
        assert_eq!(full.assume_valid_height, 0);
        assert_eq!(Params::regtest().anchor.0, 0);
    }

    // -----------------------------------------------------------------------
    // Mainnet, lite only.
    // -----------------------------------------------------------------------

    #[test]
    fn mainnet_lite_is_anchored_at_the_newest_pinned_checkpoint_with_the_mainnet_rule() {
        let m = Params::mainnet_lite();
        assert_eq!(m.name, "mainnet-lite");
        assert!(m.is_mainnet);
        assert_eq!(
            m.difficulty,
            Difficulty::Daa { pow_limit_bits: 0x1d00_ffff, spacing_secs: 600, reduce_min_difficulty: false },
            "the cw-144 DAA without the testnet minimum-difficulty rule"
        );
        let newest = m.checkpoints.iter().max_by_key(|(h, _)| *h).unwrap();
        assert_eq!(m.anchor, (newest.0, newest.1), "anchored at the newest checkpoint");
        assert_eq!(m.anchor.0, 945000);
        assert_eq!(m.assume_valid_height, m.anchor.0 + 147);
        assert!(m.anchor.0 > 504031, "above the DAA fork: no pre-DAA difficulty regime is ever evaluated");
        assert!(m.checkpoints.windows(2).all(|w| w[0].0 < w[1].0), "ascending, no duplicates");
        assert!(m.checkpoints.iter().any(|(h, _)| *h == 504031), "the DAA fork block is pinned");
        for (h, hash) in m.checkpoints {
            assert_eq!(hash.len(), 64, "checkpoint {}", h);
            assert!(hash.starts_with("00000000"), "a mainnet block hash: checkpoint {}", h);
            assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), "checkpoint {}", h);
        }
        for p in [Params::regtest(), Params::teratestnet(), Params::teratestnet_lite()] {
            assert!(!p.is_mainnet, "{}", p.name);
        }
    }

    #[test]
    fn the_mainnet_rule_has_no_minimum_difficulty_exception_the_testnet_rule_has() {
        // Real teratestnet header 398 carries powLimit bits about 20 days after its parent: the
        // testnet rule allows that, the mainnet rule does not. Both chains are anchored at 200 so
        // that 398 lies above assume-valid (347) and the rule is actually consulted.
        let anchor: &'static str = Box::leak(ttn(200).hash_hex().into_boxed_str());
        let testnet_rule = Params { name: "ttn-rule", anchor: (200, anchor), assume_valid_height: 347, ..Params::teratestnet() };
        let mainnet_rule = Params { name: "main-rule", difficulty: Params::mainnet_lite().difficulty, ..testnet_rule.clone() };
        let mut t = HeaderChain::new(testnet_rule);
        let mut m = HeaderChain::new(mainnet_rule);
        for h in 200..=397 {
            assert_eq!(t.add_header(ttn(h), NOW), Ok(AddOutcome::Extended), "testnet rule, header {}", h);
            assert_eq!(m.add_header(ttn(h), NOW), Ok(AddOutcome::Extended), "mainnet rule, header {}", h);
        }
        assert_eq!(ttn(398).bits, 0x1d00_ffff, "398 is a minimum-difficulty block");
        assert_eq!(t.add_header(ttn(398), NOW), Ok(AddOutcome::Extended), "testnet: allowed after 20 minutes");
        assert!(
            matches!(m.add_header(ttn(398), NOW), Err(HeaderError::WrongDifficulty { got: 0x1d00_ffff, .. })),
            "mainnet: the DAA target is required whatever the gap"
        );
    }

    #[test]
    fn lite_from_stored_revalidates_the_window_and_drops_rows_below_the_anchor() {
        let mut stored: Vec<(u32, String)> = (299..=700u32).map(|h| (h, ttn(h as usize).to_hex())).collect();
        stored.reverse();
        let (c, rejected) = HeaderChain::from_stored(ttn_lite_params(), stored, NOW);
        assert_eq!(rejected, 1, "299 is below the anchor");
        assert_eq!(c.tip_height(), Some(700));
        assert_eq!(c.header_at_height(LITE_ANCHOR).unwrap().height, LITE_ANCHOR);
    }

    #[test]
    fn reorg_inside_a_lite_chain_forks_at_or_above_the_anchor() {
        // Regtest-style rules anchored at a mined header at height 5.
        let anchor = mine([0x55; 32], 0, 0x207f_ffff);
        let params = Params {
            name: "lite-test",
            difficulty: Difficulty::Fixed(0x207f_ffff),
            anchor: (5, Box::leak(anchor.hash_hex().into_boxed_str())),
            assume_valid_height: 0,
            checkpoints: &[],
            is_mainnet: false,
        };
        let mut c = HeaderChain::new(params);
        assert_eq!(c.add_header(anchor.clone(), NOW), Ok(AddOutcome::Extended));
        assert_eq!(c.tip_height(), Some(5));
        let a1 = extend(&mut c, &anchor, 1);
        let a2 = extend(&mut c, &a1, 2);
        assert_eq!(c.tip_height(), Some(7));
        let b1 = mine(anchor.hash(), 11, 0x207f_ffff);
        assert_eq!(c.add_header(b1.clone(), NOW), Ok(AddOutcome::Fork));
        let b2 = mine(b1.hash(), 12, 0x207f_ffff);
        assert_eq!(c.add_header(b2.clone(), NOW), Ok(AddOutcome::Fork));
        let b3 = mine(b2.hash(), 13, 0x207f_ffff);
        match c.add_header(b3.clone(), NOW).unwrap() {
            AddOutcome::Reorg { fork_height, depth, old_tip, new_tip } => {
                assert_eq!((fork_height, depth), (5, 2));
                assert_eq!(old_tip, a2.hash_hex());
                assert_eq!(new_tip, b3.hash_hex());
            }
            other => panic!("expected reorg, got {:?}", other),
        }
        assert_eq!(c.tip_height(), Some(8));
        assert_eq!(c.header_at_height(5).unwrap().hash, anchor.hash_hex());
        assert!(c.is_active(&b1.hash_hex()) && !c.is_active(&a1.hash_hex()));
        assert!(c.header_at_height(4).is_none());
        assert!(c.verify_merkle_root(6, &b1.merkle_root_hex()));
    }

    #[test]
    fn a_header_other_than_the_checkpoint_is_rejected_at_its_height() {
        let (mut c, g) = fixture();
        let a1 = extend(&mut c, &g, 1);
        let a2 = extend(&mut c, &a1, 2);
        let mut params = c.params().clone();
        params.checkpoints = Box::leak(vec![(2u32, &*Box::leak(a2.hash_hex().into_boxed_str()))].into_boxed_slice());
        let mut c = HeaderChain::new(params);
        c.add_header(g.clone(), NOW).unwrap();
        c.add_header(a1.clone(), NOW).unwrap();
        let b2 = mine(a1.hash(), 22, 0x207f_ffff);
        assert!(matches!(c.add_header(b2, NOW), Err(HeaderError::CheckpointMismatch { height: 2, .. })));
        assert_eq!(c.add_header(a2, NOW), Ok(AddOutcome::Extended));
    }

    #[test]
    fn from_fields_rebuilds_a_header_from_explorer_fields() {
        let h = ttn(5);
        let rebuilt = Header::from_fields(h.version, &display_hex(&h.prev_hash), &h.merkle_root_hex(), h.time, h.bits, h.nonce).unwrap();
        assert_eq!(rebuilt, h);
        assert_eq!(rebuilt.hash_hex(), h.hash_hex());
        assert!(matches!(Header::from_fields(1, "zz", &h.merkle_root_hex(), 0, 0, 0), Err(HeaderError::BadHex(_))));
        assert!(matches!(Header::from_fields(1, "abcd", &h.merkle_root_hex(), 0, 0, 0), Err(HeaderError::BadLength(2))));
    }

    #[test]
    fn target_to_compact_round_trips() {
        for bits in [0x1d00_ffffu32, 0x1c7f_ff80, 0x1c0a_8eba, 0x207f_ffff, 0x0312_3456, 0x0212_3400, 0x0112_0000] {
            assert_eq!(target_to_compact(compact_to_target(bits).unwrap()), bits, "{:#x}", bits);
        }
        // A mantissa with the sign bit set moves up a byte.
        assert_eq!(target_to_compact(U256([0x80, 0, 0, 0])), 0x0200_8000);
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
