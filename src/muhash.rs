//! MuHash3072: Multiplicative hash for UTXO set.
//!
//! Matches Bitcoin Core gettxoutsetinfo muhash output.

use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use sha2::{Digest, Sha256};

use crate::num3072::{Num3072, BYTE_SIZE};

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
}

impl Default for MuHash3072 {
    fn default() -> Self {
        Self::new()
    }
}
