//! Byte layout of a MuHash coin, and insert/remove returning to the empty set.

use blvm_muhash::{MuHash3072, serialize_coin_for_muhash};

#[test]
fn serialize_coin_sets_the_coinbase_bit_in_the_height_word() {
    let txid = [0x11u8; 32];
    let bytes = serialize_coin_for_muhash(&txid, 7, 100, true, 50, &[0x51]);
    assert_eq!(&bytes[..32], &txid, "txid");
    assert_eq!(u32::from_le_bytes(bytes[32..36].try_into().unwrap()), 7);
    assert_eq!(
        u32::from_le_bytes(bytes[36..40].try_into().unwrap()),
        (100 << 1) | 1,
        "coinbase height word"
    );
    assert_eq!(i64::from_le_bytes(bytes[40..48].try_into().unwrap()), 50);
    assert_eq!(bytes[48], 1, "script compact size");
    assert_eq!(&bytes[49..], &[0x51]);
}

#[test]
fn serialize_coin_clears_the_coinbase_bit_for_a_normal_output() {
    let bytes = serialize_coin_for_muhash(&[0u8; 32], 0, 100, false, 0, &[]);
    assert_eq!(
        u32::from_le_bytes(bytes[36..40].try_into().unwrap()),
        100 << 1
    );
    assert_eq!(bytes[48], 0, "empty script is a one-byte compact size");
    assert_eq!(bytes.len(), 49);
}

#[test]
fn insert_then_remove_finalizes_to_the_empty_set() {
    let coin = serialize_coin_for_muhash(&[9u8; 32], 1, 1, false, 1, &[0x00]);
    let empty = MuHash3072::new().finalize();
    let round = MuHash3072::new().insert(&coin).remove(&coin).finalize();
    assert_eq!(round, empty);
}
