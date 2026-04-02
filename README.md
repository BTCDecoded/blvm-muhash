# blvm-muhash

[![crates.io](https://img.shields.io/crates/v/blvm-muhash.svg)](https://crates.io/crates/blvm-muhash)
[![docs.rs](https://docs.rs/blvm-muhash/badge.svg)](https://docs.rs/blvm-muhash)
[![CI](https://github.com/BTCDecoded/blvm-muhash/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/BTCDecoded/blvm-muhash/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

MuHash3072 for Bitcoin UTXO set hashing. Matches Bitcoin Core `gettxoutsetinfo` muhash output for AssumeUTXO compatibility.

## Features

- **Core-compatible** — Produces identical hashes to Bitcoin Core's MuHash3072
- **Incremental** — Insert and remove elements without full recomputation
- **Mergeable** — Combine hashes for parallel processing
- **No bitcoin-* deps** — Pure Rust with sha2, chacha20, cipher

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
blvm-muhash = "0.1"
```

## Usage

```rust
use blvm_muhash::{MuHash3072, serialize_coin_for_muhash};

// Serialize a UTXO for hashing (matches Core TxOutSer format)
let txid = [0u8; 32];
let vout = 0u32;
let height = 100u32;
let is_coinbase = true;
let amount = 50_0000_0000i64; // 50 BTC in satoshis
let script_pubkey = vec![0x76, 0xa9, 0x14]; // P2PKH prefix

let serialized = serialize_coin_for_muhash(&txid, vout, height, is_coinbase, amount, &script_pubkey);

// Hash the UTXO set (iterate in lexicographic order by outpoint)
let hash = MuHash3072::new()
    .insert(&serialized)
    .finalize();

assert_eq!(hash.len(), 32);
```

## API

| Item | Description |
|------|-------------|
| `MuHash3072::new()` | Empty set |
| `insert(self, data)` | Add element |
| `remove(self, data)` | Remove element |
| `multiply(self, other)` | Union of sets |
| `divide(self, other)` | Difference of sets |
| `finalize(self)` | Produce 32-byte hash |
| `serialize_coin_for_muhash(...)` | Serialize (outpoint, coin) for insertion |

## License

MIT
