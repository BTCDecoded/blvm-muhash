//! Coin serialization for MuHash.
//!
//! Matches Bitcoin Core TxOutSer in src/kernel/coinstats.cpp.

/// Serialize (outpoint, coin) for MuHash insertion.
/// Matches Core TxOutSer: outpoint, (height<<1)|coinbase, amount, script_pubkey.
pub fn serialize_coin_for_muhash(
    txid: &[u8; 32],
    vout: u32,
    height: u32,
    is_coinbase: bool,
    amount: i64,
    script_pubkey: &[u8],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(32 + 4 + 4 + 8 + 9 + script_pubkey.len()); // compact size max 9

    out.extend_from_slice(txid);
    out.extend_from_slice(&vout.to_le_bytes());

    let height_coinbase = (height << 1) | (is_coinbase as u32);
    out.extend_from_slice(&height_coinbase.to_le_bytes());

    out.extend_from_slice(&amount.to_le_bytes());
    out.extend(compact_size_encode(script_pubkey.len()));
    out.extend_from_slice(script_pubkey);

    out
}

fn compact_size_encode(len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(9);
    if len < 0xfd {
        out.push(len as u8);
    } else if len <= 0xffff {
        out.push(0xfd);
        out.extend_from_slice(&(len as u16).to_le_bytes());
    } else if len <= 0xffff_ffff {
        out.push(0xfe);
        out.extend_from_slice(&(len as u32).to_le_bytes());
    } else {
        out.push(0xff);
        out.extend_from_slice(&(len as u64).to_le_bytes());
    }
    out
}
