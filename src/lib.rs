//! MuHash3072 for Bitcoin UTXO set hashing.
//!
//! Matches Bitcoin Core `gettxoutsetinfo` muhash output for AssumeUTXO compatibility.
//!
//! # Example
//!
//! ```rust
//! use blvm_muhash::{MuHash3072, serialize_coin_for_muhash};
//!
//! let txid = [0u8; 32];
//! let serialized = serialize_coin_for_muhash(&txid, 0, 100, true, 50_0000_0000, &[0x76, 0xa9, 0x14]);
//! let hash = MuHash3072::new().insert(&serialized).finalize();
//! assert_eq!(hash.len(), 32);
//! ```

pub mod coin;
pub mod muhash;
pub mod num3072;

pub use coin::serialize_coin_for_muhash;
pub use muhash::MuHash3072;
pub use num3072::Num3072;
