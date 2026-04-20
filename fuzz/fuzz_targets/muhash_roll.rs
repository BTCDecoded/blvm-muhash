#![no_main]
use blvm_muhash::{serialize_coin_for_muhash, MuHash3072};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut txid = [0u8; 32];
    for (i, b) in data.iter().take(32).enumerate() {
        txid[i] = *b;
    }

    let script_len = data.len().min(512);
    let script = &data[..script_len];
    let coin = serialize_coin_for_muhash(&txid, 0, 100, true, 50_000_000_000, script);

    let _ = MuHash3072::new()
        .insert(&coin)
        .insert(data)
        .remove(data)
        .finalize();

    let a = MuHash3072::new().insert(&coin);
    let b = MuHash3072::new().insert(data);
    let _ = MuHash3072::new().multiply(&a).divide(&b).finalize();
});
