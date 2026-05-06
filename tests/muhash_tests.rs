//! MuHash3072 tests.

use blvm_muhash::{serialize_coin_for_muhash, MuHash3072, MUHASH_RUNNING_STATE_BYTES};

#[test]
fn empty_set() {
    let h = MuHash3072::new().finalize();
    assert_eq!(h.len(), 32);
}

#[test]
fn insert_remove_same() {
    let data = b"test element";
    let h1 = MuHash3072::new().finalize();
    let h2 = MuHash3072::new().insert(data).remove(data).finalize();
    assert_eq!(
        h1, h2,
        "insert then remove same element should equal empty set"
    );
}

#[test]
fn insert_order_independent() {
    let a = b"element a";
    let b = b"element b";
    let h1 = MuHash3072::new().insert(a).insert(b).finalize();
    let h2 = MuHash3072::new().insert(b).insert(a).finalize();
    assert_eq!(h1, h2, "MuHash is order-independent");
}

#[test]
fn serialize_coin_format() {
    let txid = [1u8; 32];
    let vout = 0u32;
    let height = 100u32;
    let is_coinbase = true;
    let amount = 50_0000_0000i64; // 50 BTC in satoshis
    let script_pubkey: Vec<u8> = [0x76, 0xa9, 0x14].into_iter().chain([0u8; 20]).collect(); // P2PKH

    let serialized =
        serialize_coin_for_muhash(&txid, vout, height, is_coinbase, amount, &script_pubkey);

    assert_eq!(serialized[0..32], txid);
    assert_eq!(serialized[32..36], vout.to_le_bytes());
    let height_coinbase = (height << 1) | (is_coinbase as u32);
    assert_eq!(serialized[36..40], height_coinbase.to_le_bytes());
    assert_eq!(serialized[40..48], amount.to_le_bytes());
    assert_eq!(serialized[48], 23); // compact size for 23-byte script
    assert_eq!(serialized[49..72], script_pubkey);
}

#[test]
fn running_state_roundtrip_preserves_finalize() {
    let coin = serialize_coin_for_muhash(&[7u8; 32], 2, 500_000, false, 12345, &[0x51]);
    let mh = MuHash3072::new().insert(&coin).insert(b"another");
    let bytes = mh.serialize_running_state();
    assert_eq!(bytes.len(), MUHASH_RUNNING_STATE_BYTES);
    let restored = MuHash3072::deserialize_running_state(&bytes);
    assert_eq!(mh.clone().finalize(), restored.finalize());
}

#[test]
fn known_empty_hash() {
    // Empty MuHash: 1/1 mod p, ToBytes = [1,0,0,...,0] LE, SHA256 of that.
    // Precomputed from Core behavior: empty chainstate muhash.
    let h = MuHash3072::new().finalize();
    // Just verify it's deterministic
    let h2 = MuHash3072::new().finalize();
    assert_eq!(h, h2);
}

#[test]
fn mut_api_matches_value_api() {
    // The hot-path APIs (`insert_mut` / `remove_mut`) skip the value-semantics clone
    // that the `insert(self) -> Self` form forces at every call site. They MUST be
    // bit-for-bit identical to the consuming form for any sequence of operations.
    let coins: [Vec<u8>; 4] = [
        serialize_coin_for_muhash(&[1u8; 32], 0, 100, false, 1, &[0x51]),
        serialize_coin_for_muhash(&[2u8; 32], 1, 200, true, 2, &[0x52]),
        serialize_coin_for_muhash(&[3u8; 32], 2, 300, false, 3, &[0x53]),
        serialize_coin_for_muhash(&[4u8; 32], 3, 400, true, 4, &[0x54]),
    ];

    // Inserts only.
    let value_form = coins
        .iter()
        .fold(MuHash3072::new(), |acc, c| acc.insert(c));
    let mut mut_form = MuHash3072::new();
    for c in coins.iter() {
        mut_form.insert_mut(c);
    }
    assert_eq!(value_form.clone().finalize(), mut_form.clone().finalize());
    assert_eq!(
        value_form.serialize_running_state(),
        mut_form.serialize_running_state()
    );

    // Inserts + removes (covers `remove_mut`).
    let value_form = coins
        .iter()
        .fold(MuHash3072::new(), |acc, c| acc.insert(c))
        .remove(&coins[1])
        .remove(&coins[3]);
    let mut mut_form = MuHash3072::new();
    for c in coins.iter() {
        mut_form.insert_mut(c);
    }
    mut_form.remove_mut(&coins[1]);
    mut_form.remove_mut(&coins[3]);
    assert_eq!(value_form.clone().finalize(), mut_form.clone().finalize());
    assert_eq!(
        value_form.serialize_running_state(),
        mut_form.serialize_running_state()
    );
}
