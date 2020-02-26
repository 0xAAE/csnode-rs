#[cfg(test)]
use super::raw_block::{validate_user_fields, validate_transaction, validate_raw_block};
#[cfg(test)]
use super::super::primitive::{HASH_SIZE, PUBLIC_KEY_SIZE, SIGNATURE_SIZE};
#[cfg(test)]
use bincode::serialize_into;
#[cfg(test)]
use std::io::Write;

#[cfg(test)]
fn mock_serialize_user_fields_to(bytes: &mut Vec<u8>) {
    serialize_into(bytes.by_ref(), &3u8).unwrap();
    serialize_into(bytes.by_ref(), &10u32).unwrap();
    serialize_into(bytes.by_ref(), &1u8).unwrap();
    serialize_into(bytes.by_ref(), &11u64).unwrap();
    serialize_into(bytes.by_ref(), &20u32).unwrap();
    serialize_into(bytes.by_ref(), &2u8).unwrap();
    let l = bytes.len() as u32;
    serialize_into(bytes.by_ref(), &l).unwrap();
    bytes.extend_from_slice(&bytes.clone()[..l as usize]);
    serialize_into(bytes.by_ref(), &30u32).unwrap();
    serialize_into(bytes.by_ref(), &3u8).unwrap();
    serialize_into(bytes.by_ref(), &12u32).unwrap();
    serialize_into(bytes.by_ref(), &14u64).unwrap();
}

#[cfg(test)]
fn mock_serialize_transaction_to(bytes: &mut Vec<u8>, src_id: bool, tgt_id: bool) {
    serialize_into(bytes.by_ref(), &3u16).unwrap(); // lo
    let mut hi: u32 = 10;
    if src_id {
        hi += 0x8000_0000;
    }
    if tgt_id {
        hi += 0x4000_0000;
    }
    serialize_into(bytes.by_ref(), &hi).unwrap();
    if src_id {
        serialize_into(bytes.by_ref(), &55u32).unwrap();
    }
    else {
        bytes.extend_from_slice(&[211u8; PUBLIC_KEY_SIZE]);
    }
    if tgt_id {
        serialize_into(bytes.by_ref(), &88u32).unwrap();
    }
    else {
        bytes.extend_from_slice(&[212u8; PUBLIC_KEY_SIZE]);
    }
    serialize_into(bytes.by_ref(), &19u32).unwrap();
    serialize_into(bytes.by_ref(), &29u64).unwrap();
    serialize_into(bytes.by_ref(), &15771u16).unwrap();
    serialize_into(bytes.by_ref(), &1u8).unwrap();
    mock_serialize_user_fields_to(bytes.by_ref());
    let sig = [217u8; SIGNATURE_SIZE];
    bytes.extend_from_slice(&sig[..]);
    serialize_into(bytes.by_ref(), &17552u16).unwrap();
}

#[cfg(test)]
fn mock_serialize_raw_block_to(bytes: &mut Vec<u8>) {
    serialize_into(bytes.by_ref(), &0u8).unwrap();
    serialize_into(bytes.by_ref(), &32u8).unwrap(); // sizeof HASH
    serialize_into(bytes.by_ref(), &[214u8; HASH_SIZE]).unwrap();
    serialize_into(bytes.by_ref(), &17_000_000u64).unwrap();
    mock_serialize_user_fields_to(bytes.by_ref());
    serialize_into(bytes.by_ref(), &100u32).unwrap();
    serialize_into(bytes.by_ref(), &100u64).unwrap();
    serialize_into(bytes.by_ref(), &4u32).unwrap(); // transactions count
    mock_serialize_transaction_to(bytes.by_ref(), true, true);
    mock_serialize_transaction_to(bytes.by_ref(), false, true);
    mock_serialize_transaction_to(bytes.by_ref(), true, false);
    mock_serialize_transaction_to(bytes.by_ref(), false, false);
    // introduced new wallets
    let nw_cnt: u32 = 2;
    serialize_into(bytes.by_ref(), &nw_cnt).unwrap();
    for _ in 0..nw_cnt {
        serialize_into(bytes.by_ref(), &99u64).unwrap();
        serialize_into(bytes.by_ref(), &99u32).unwrap();
    }
    // trusted info: consensus
    let cnt_blk_sig: u8;
    {
        let cons_cnt: u8 = 7;
        serialize_into(bytes.by_ref(), &cons_cnt).unwrap();
        let actual = 0b0111_1101 as u64;
        serialize_into(bytes.by_ref(), &actual).unwrap();
        for _ in 0..cons_cnt {
            serialize_into(bytes.by_ref(), &[215u8; PUBLIC_KEY_SIZE]).unwrap();
        }
        cnt_blk_sig = actual.count_ones() as u8;
    }
    // trusted info: next RT
    {
        let rt_cnt: u8 = 8;
        serialize_into(bytes.by_ref(), &rt_cnt).unwrap();
        let actual_rt = 0b1101_1111 as u64;
        serialize_into(bytes.by_ref(), &actual_rt).unwrap();
        for _ in 0..actual_rt.count_ones() {
            let sig = [229u8; SIGNATURE_SIZE];
            bytes.extend_from_slice(&sig[..]);
        }
    }
    // hashed length
    let hashed_len = bytes.len();
    serialize_into(bytes.by_ref(), &hashed_len).unwrap();
    // trusted info signatures
    for _ in 0..cnt_blk_sig {
        let sig = [237u8; SIGNATURE_SIZE];
        bytes.extend_from_slice(&sig[..]);
    }
    // contracts signatures
    let cnt_contr = 5u8;
    serialize_into(bytes.by_ref(), &cnt_contr).unwrap();
    for _ in 0..cnt_contr {
        serialize_into(bytes.by_ref(), &[225u8; PUBLIC_KEY_SIZE]).unwrap();
        serialize_into(bytes.by_ref(), &10_234_567u64).unwrap();
        let cnt_trusted = 7u8;
        serialize_into(bytes.by_ref(), &cnt_trusted).unwrap();
        for _ in 0..cnt_trusted {
            serialize_into(bytes.by_ref(), &1u8).unwrap();
            let sig = [241u8; SIGNATURE_SIZE];
            bytes.extend_from_slice(&sig[..]);
        }
    }
}

#[test]
fn test_validate_user_fields() {
    let mut bytes = Vec::<u8>::new();
    mock_serialize_user_fields_to(bytes.by_ref());
    assert_eq!(validate_user_fields(&bytes[1..]), None);
    assert_eq!(validate_user_fields(&bytes[..bytes.len() - 1]), None);
    assert_eq!(validate_user_fields(&bytes[..]), Some(bytes.len()));
}

#[test]
fn test_validate_transaction_id_id() {
    let mut bytes = Vec::<u8>::new();
    mock_serialize_transaction_to(bytes.by_ref(), true, true);
    assert_eq!(validate_transaction(&bytes[1..]), None);
    assert_eq!(validate_transaction(&bytes[..bytes.len() - 1]), None);
    assert_eq!(validate_transaction(&bytes[..]), Some(bytes.len()));
}

#[test]
fn test_validate_transaction_pk_id() {
    let mut bytes = Vec::<u8>::new();
    mock_serialize_transaction_to(bytes.by_ref(), false, true);
    assert_eq!(validate_transaction(&bytes[1..]), None);
    assert_eq!(validate_transaction(&bytes[..bytes.len() - 1]), None);
    assert_eq!(validate_transaction(&bytes[..]), Some(bytes.len()));
}

#[test]
fn test_validate_transaction_id_pk() {
    let mut bytes = Vec::<u8>::new();
    mock_serialize_transaction_to(bytes.by_ref(), true, false);
    assert_eq!(validate_transaction(&bytes[1..]), None);
    assert_eq!(validate_transaction(&bytes[..bytes.len() - 1]), None);
    assert_eq!(validate_transaction(&bytes[..]), Some(bytes.len()));
}

#[test]
fn test_validate_transaction_pk_pk() {
    let mut bytes = Vec::<u8>::new();
    mock_serialize_transaction_to(bytes.by_ref(), false, false);
    assert_eq!(validate_transaction(&bytes[1..]), None);
    assert_eq!(validate_transaction(&bytes[..bytes.len() - 1]), None);
    assert_eq!(validate_transaction(&bytes[..]), Some(bytes.len()));
}

#[test]
fn test_validate_transactions() {
    let mut bytes = Vec::<u8>::new();
    mock_serialize_transaction_to(bytes.by_ref(), false, false);
    let pos1 = bytes.len();
    mock_serialize_transaction_to(bytes.by_ref(), false, true);
    let pos2 = bytes.len();
    mock_serialize_transaction_to(bytes.by_ref(), true, false);
    let pos3 = bytes.len();
    mock_serialize_transaction_to(bytes.by_ref(), true, true);
    let pos4 = bytes.len();

    assert_eq!(validate_transaction(&bytes[..]), Some(pos1));
    assert_eq!(validate_transaction(&bytes[pos1..]), Some(pos2 - pos1));
    assert_eq!(validate_transaction(&bytes[pos2..]), Some(pos3 - pos2));
    assert_eq!(validate_transaction(&bytes[pos3..]), Some(pos4 - pos3));
}

#[test]
fn test_validate_raw_block() {
    let mut bytes = Vec::<u8>::new();
    mock_serialize_raw_block_to(bytes.by_ref());

    assert_eq!(validate_raw_block(&bytes[..]), Some(bytes.len()));
    assert_eq!(validate_raw_block(&bytes[1..]), None);
    assert_eq!(validate_raw_block(&bytes[..bytes.len() - 1]), None);
}
