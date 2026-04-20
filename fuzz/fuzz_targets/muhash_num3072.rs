#![no_main]
//! Direct `Num3072` field arithmetic plus coin serialization feeding `MuHash3072`.
use blvm_muhash::num3072::BYTE_SIZE;
use blvm_muhash::{serialize_coin_for_muhash, MuHash3072, Num3072};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut buf_a = [0u8; BYTE_SIZE];
    let n = data.len().min(BYTE_SIZE);
    buf_a[..n].copy_from_slice(&data[..n]);
    let a = Num3072::from_bytes(&buf_a);

    let mut buf_b = buf_a;
    if !data.is_empty() {
        buf_b[0] ^= data[0];
    }
    if data.len() > 32 {
        buf_b[BYTE_SIZE - 1] ^= data[31];
    }
    let b = Num3072::from_bytes(&buf_b);

    let mut x = a.clone();
    x.multiply(&b);
    let mut y = x;
    y.divide(&b);
    let _ = y.is_overflow();
    y.full_reduce();

    let inv = b.get_inverse();
    let mut prod = inv;
    prod.multiply(&b);

    let mut raw = [0u8; BYTE_SIZE];
    prod.to_bytes(&mut raw);

    let mut txid = [0u8; 32];
    for i in 0..32.min(data.len()) {
        txid[i] = data[i];
    }
    let script_len = data.len().saturating_sub(48).min(512);
    let script = if data.len() > 48 {
        &data[48..48 + script_len]
    } else {
        &[][..]
    };
    let vout = data
        .get(32..36)
        .map(|s| u32::from_le_bytes(s.try_into().unwrap()))
        .unwrap_or(0);
    let height = data
        .get(36..40)
        .map(|s| u32::from_le_bytes(s.try_into().unwrap()))
        .unwrap_or(0);
    let is_coinbase = data.get(40).map(|b| (b & 1) != 0).unwrap_or(false);
    let amount = data
        .get(41..49)
        .filter(|s| s.len() == 8)
        .map(|s| i64::from_le_bytes(s.try_into().unwrap()))
        .unwrap_or(0);

    let coin = serialize_coin_for_muhash(&txid, vout, height, is_coinbase, amount, script);
    let _ = MuHash3072::new().insert(&coin).insert(data).finalize();
});
