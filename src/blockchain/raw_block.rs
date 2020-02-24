use std::mem::{size_of_val, size_of};
use super::super::primitive::{HASH_SIZE, PUBLIC_KEY_SIZE, SIGNATURE_SIZE};

use bincode::deserialize_from;

pub struct RawBlock {
    pub data: Vec<u8>
}

impl RawBlock {

    pub fn new(bytes: Vec<u8>) -> Option<RawBlock> {
        if ! validate_raw_block(&bytes[..]) {
            return None;
        }
        Some(RawBlock {
            data: bytes
        })
    }


}

pub fn validate_raw_block(bytes: &[u8]) -> bool {
    let total = bytes.len();

    let mut pos =
        size_of::<u8>() +   // version
        HASH_SIZE +         // prev hash
        size_of::<u64>();   // sequence
    if total <= pos {
        return false;
    }
    // block user fields
    match validate_user_fields(&bytes[pos..]) {
        None => {
            return false;
        },
        Some(len) => {
            pos += len;
        }
    }
    let sizeof_money = size_of::<u32>() + size_of::<u64>();
    pos += sizeof_money;     // round cost (money)
    if total <= pos {
        return false;
    }
    // transactions
    if total < pos + size_of::<u32>() {
        return false;
    }
    let t_cnt: u32 = deserialize_from(&bytes[pos..]).unwrap();
    pos += size_of_val(&t_cnt);
    for _ in 0..t_cnt {
        match validate_transaction(&bytes[pos..]) {
            None => {
                return false;
            },
            Some(len) => {
                pos += len;
            }
        }
    }

    // introduced new wallets
    if total < pos + size_of::<u32>() {
        return false;
    }
    let w_cnt: u32 = deserialize_from(&bytes[pos..]).unwrap();
    pos += size_of_val(&w_cnt) + w_cnt as usize * (size_of::<u32>() + size_of::<u64>());

    // trusted info
    if total < pos + size_of::<u8>() + size_of::<u64>() {
        return false;
    } 

    // trusted info - consensus
    let consensus_cnt: u8 = deserialize_from(&bytes[pos..]).unwrap();
    let consensus_bits: u64 = deserialize_from(&bytes[pos + size_of_val(&consensus_cnt)..]).unwrap();
    let sig_blk_cnt = consensus_bits.count_ones() as usize;
    pos += size_of_val(&consensus_cnt) + size_of_val(&consensus_bits) + (consensus_cnt as usize * PUBLIC_KEY_SIZE);
    // trusted info - next RT
    if total < pos + size_of::<u8>() + size_of::<u64>() {
        return false;
    }
    let rt_cnt: u8 = deserialize_from(&bytes[pos..]).unwrap();
    let rt_bits: u64 = deserialize_from(&bytes[pos + size_of_val(&rt_cnt)..]).unwrap();
    let sig_rt_cnt = rt_bits.count_ones() as usize;
    pos += size_of_val(&rt_cnt) + size_of_val(&rt_bits) + sig_rt_cnt * SIGNATURE_SIZE;
    // hashed length
    pos += size_of::<usize>();
    // signatures
    pos += sig_blk_cnt * SIGNATURE_SIZE;
    // contract signatures
    if total < pos + 1 {
        return false;
    }
    let contr_cnt: u8 = deserialize_from(&bytes[pos..]).unwrap();
    pos += size_of_val(&contr_cnt);
    for _ in 0..contr_cnt {
        pos += PUBLIC_KEY_SIZE + size_of::<u64>();
        if total < pos {
            return false;
        }
        let item_sig_cnt: u8 = deserialize_from(&bytes[pos..]).unwrap();
        pos += size_of_val(&item_sig_cnt) + (size_of::<u8>() + SIGNATURE_SIZE) * item_sig_cnt as usize;
    }

    total == pos
}

/// None - failed to validate
/// Some(pos) - the position immediately after user fields, which starts from 1, pos[0] is fileds count
pub fn validate_user_fields(bytes: &[u8]) -> Option<usize> {
    let total = bytes.len();
    if total <= 0 {
        return None;
    }
    let count: usize = bytes[0] as usize;
    let mut pos = 1usize;
    for _ in 0..count {
        // field key
        pos += 4; // u32
        // field type
        if total <= pos {
            return None;
        }
        match bytes[pos] {
            1 => { // integer u64
                pos += 9; // type + u64
            }
            2 => { // vec
                if total <= pos + 1 + 4 {
                    return None;
                }
                let vec_size: u32 = deserialize_from(&bytes[pos + 1..]).unwrap_or(0);
                if vec_size == 0 {
                    return None;
                }
                pos += 5usize + vec_size as usize; // type + vec_size + vec_data
            },
            3 => { // money
                pos += 13;  // type + money
            }
            _ => {
                return None;
            }
        }
    }
    if total < pos {
        return None;
    }
    Some(pos)
}

/// None - failed to validate
/// Some(pos) - th eposition immediately after transaction
pub fn validate_transaction(bytes: &[u8]) -> Option<usize> {
    let total = bytes.len();
    if total <= 6 {
        return None;
    }
    
    // innerID, source, target
    let mut pos = 2;
    let hi: u32 = deserialize_from(&bytes[pos..]).unwrap();
    pos += 4;
    if (hi & 0x8000_0000) != 0 {
        pos += 4;
    }
    else {
        pos += PUBLIC_KEY_SIZE;
    }
    if (hi & 0x4000_0000) != 0 {
        pos += 4;
    }
    else {
        pos += PUBLIC_KEY_SIZE;
    }

    // money + max_fee + currency
    pos += 12 + 2 + 1;
    if total <= pos {
        return None;
    }

    // user fields
    match validate_user_fields(&bytes[pos..]) {
        None => {
            return None;
        },
        Some(len) => {
            pos += len;
        }
    }

    // signature + fee
    pos += SIGNATURE_SIZE + 2;

    if total < pos {
        return None;
    }
    Some(pos)
}
