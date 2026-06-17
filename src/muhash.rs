//! MuHash3072: Multiplicative hash for UTXO set.
//!
//! Matches Bitcoin Core gettxoutsetinfo muhash output.

use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use sha2::{Digest, Sha256};

use crate::num3072::{BYTE_SIZE, Num3072};

/// Serialized [`MuHash3072`] rolling state (numerator ‖ denominator). Persist between flushes for incremental IBD UTXO hashing.
pub const MUHASH_RUNNING_STATE_BYTES: usize = BYTE_SIZE * 2;

/// MuHash3072 state. Empty set: numerator=1, denominator=1.
#[derive(Clone)]
pub struct MuHash3072 {
    numerator: Num3072,
    denominator: Num3072,
}

fn to_num3072(data: &[u8]) -> Num3072 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();

    let key: [u8; 32] = hash.into();
    let nonce = [0u8; 12];

    let mut cipher = ChaCha20::new(&key.into(), &nonce.into());
    let mut tmp = [0u8; BYTE_SIZE];
    cipher.apply_keystream(&mut tmp);

    Num3072::from_bytes(&tmp)
}

impl MuHash3072 {
    /// Empty set.
    pub fn new() -> Self {
        MuHash3072 {
            numerator: Num3072::default(),
            denominator: Num3072::default(),
        }
    }

    /// Insert element into the set.
    pub fn insert(mut self, data: &[u8]) -> Self {
        let elem = to_num3072(data);
        self.numerator.multiply(&elem);
        self
    }

    /// Remove element from the set.
    pub fn remove(mut self, data: &[u8]) -> Self {
        let elem = to_num3072(data);
        self.denominator.multiply(&elem);
        self
    }

    /// In-place insert. Hot-path equivalent of [`Self::insert`] without the value-semantics
    /// move. The owning `insert(self) -> Self` form is convenient at call sites that don't
    /// already have `&mut`, but it forces callers like
    /// `*mh = mh.clone().insert(&pre)` to clone the running state on every row — which is
    /// 768 B per row (two `Num3072`s) and dominates IBD-flush CPU at high heights.
    ///
    /// Mathematically identical to [`Self::insert`].
    pub fn insert_mut(&mut self, data: &[u8]) {
        let elem = to_num3072(data);
        self.numerator.multiply(&elem);
    }

    /// In-place remove. See [`Self::insert_mut`] for the rationale.
    /// Mathematically identical to [`Self::remove`].
    pub fn remove_mut(&mut self, data: &[u8]) {
        let elem = to_num3072(data);
        self.denominator.multiply(&elem);
    }

    /// Finalize to 32-byte hash. Consumes self.
    pub fn finalize(mut self) -> [u8; 32] {
        self.numerator.divide(&self.denominator);

        let mut data = [0u8; BYTE_SIZE];
        self.numerator.to_bytes(&mut data);

        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    /// Multiply by another MuHash (union of sets). For parallel/merge use.
    pub fn multiply(mut self, other: &MuHash3072) -> Self {
        self.numerator.multiply(&other.numerator);
        self.denominator.multiply(&other.denominator);
        self
    }

    /// Divide by another MuHash (difference of sets). For parallel/merge use.
    pub fn divide(mut self, other: &MuHash3072) -> Self {
        self.numerator.multiply(&other.denominator);
        self.denominator.multiply(&other.numerator);
        self
    }

    /// Encode rolling numerator/denominator for persistence (not the finalized 32-byte muhash).
    pub fn serialize_running_state(&self) -> [u8; MUHASH_RUNNING_STATE_BYTES] {
        let mut out = [0u8; MUHASH_RUNNING_STATE_BYTES];
        let mut num_buf = [0u8; BYTE_SIZE];
        let mut den_buf = [0u8; BYTE_SIZE];
        self.numerator.to_bytes(&mut num_buf);
        self.denominator.to_bytes(&mut den_buf);
        out[..BYTE_SIZE].copy_from_slice(&num_buf);
        out[BYTE_SIZE..].copy_from_slice(&den_buf);
        out
    }

    /// Decode [`serialize_running_state`] output.
    pub fn deserialize_running_state(bytes: &[u8; MUHASH_RUNNING_STATE_BYTES]) -> Self {
        let numerator = Num3072::from_bytes(bytes[..BYTE_SIZE].try_into().unwrap());
        let denominator = Num3072::from_bytes(bytes[BYTE_SIZE..].try_into().unwrap());
        MuHash3072 {
            numerator,
            denominator,
        }
    }
}

impl Default for MuHash3072 {
    fn default() -> Self {
        Self::new()
    }
}
