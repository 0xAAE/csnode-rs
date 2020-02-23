use super::super::primitive::{HASH_SIZE, PUBLIC_KEY_SIZE, SIGNATURE_SIZE};

use bincode::{deserialize_from, serialize_into};

pub struct RawBlock {
    pub data: Vec<u8>
}

impl RawBlock {

    pub fn new(bytes: Vec<u8>) -> Option<RawBlock> {
        if ! validate(&bytes[..]) {
            return None;
        }
        Some(RawBlock {
            data: bytes
        })
    }


}

fn validate(bytes: &[u8]) -> bool {
    let total = bytes.len();

    let mut pos =
        1 +         // version
        HASH_SIZE + // prev hash
        8;          // sequence
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
    pos += 12;     // round cost (money)
    if total <= pos {
        return false;
    }
    // transactions

    total == pos
}

/// None - failed to validate
/// Some(pos) - the position immediately after user fields, which starts from 1, pos[0] is fileds count
fn validate_user_fields(bytes: &[u8]) -> Option<usize> {
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

#[test]
fn test_validate_user_fields() {
    let mut bytes = Vec::<u8>::new();
    serialize_into(&mut bytes, &3u8).unwrap();
    serialize_into(&mut bytes, &10u32).unwrap();
    serialize_into(&mut bytes, &1u8).unwrap();
    serialize_into(&mut bytes, &11u64).unwrap();
    serialize_into(&mut bytes, &20u32).unwrap();
    serialize_into(&mut bytes, &2u8).unwrap();
    let l = bytes.len() as u32;
    serialize_into(&mut bytes, &l).unwrap();
    bytes.extend_from_slice(&bytes.clone()[..l as usize]);
    serialize_into(&mut bytes, &30u32).unwrap();
    serialize_into(&mut bytes, &3u8).unwrap();

    assert_eq!(validate_user_fields(&bytes[..]), None);

    serialize_into(&mut bytes, &12u32).unwrap();

    assert_eq!(validate_user_fields(&bytes[..]), None);

    serialize_into(&mut bytes, &14u64).unwrap();

    assert_eq!(validate_user_fields(&bytes[..]), Some(bytes.len()));
}

/// None - failed to validate
/// Some(pos) - th eposition immediately after transaction
fn validate_transaction(bytes: &[u8]) -> Option<usize> {
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

#[test]
fn test_validate_transaction_id_id() {
    let mut bytes = Vec::<u8>::new();
    serialize_into(&mut bytes, &3u16).unwrap(); // lo
    let hi: u32 = 0x8000_0000 + 0x4000_0000 + 10;
    serialize_into(&mut bytes, &hi).unwrap();

    let src = 55u32; // [1u8; PUBLIC_KEY_SIZE];
    // bytes.extend_from_slice(&src);
    serialize_into(&mut bytes, &src).unwrap();

    let tgt = 88u32; // [2u8; PUBLIC_KEY_SIZE];
    // bytes.extend_from_slice(&tgt);
    serialize_into(&mut bytes, &tgt).unwrap();

    serialize_into(&mut bytes, &19u32).unwrap();
    serialize_into(&mut bytes, &29u64).unwrap();
    serialize_into(&mut bytes, &15771u16).unwrap();
    serialize_into(&mut bytes, &1u8).unwrap();
    // user fields
    serialize_into(&mut bytes, &3u8).unwrap();
    serialize_into(&mut bytes, &10u32).unwrap();
    serialize_into(&mut bytes, &1u8).unwrap();
    serialize_into(&mut bytes, &11u64).unwrap();
    serialize_into(&mut bytes, &20u32).unwrap();
    serialize_into(&mut bytes, &2u8).unwrap();
    let l = bytes.len() as u32;
    serialize_into(&mut bytes, &l).unwrap();
    bytes.extend_from_slice(&bytes.clone()[..l as usize]);
    serialize_into(&mut bytes, &30u32).unwrap();
    serialize_into(&mut bytes, &3u8).unwrap();
    serialize_into(&mut bytes, &12u32).unwrap();
    serialize_into(&mut bytes, &14u64).unwrap();
    let sig = [17u8; SIGNATURE_SIZE];
    bytes.extend_from_slice(&sig[..]);

    assert_eq!(validate_user_fields(&bytes[..]), None);

    serialize_into(&mut bytes, &17552u16).unwrap();

    assert_eq!(validate_transaction(&bytes[..]), Some(bytes.len()));
}

#[test]
fn test_validate_transaction_pk_id() {
    let mut bytes = Vec::<u8>::new();
    serialize_into(&mut bytes, &3u16).unwrap(); // lo
    let hi: u32 = 0x4000_0000 + 10;
    serialize_into(&mut bytes, &hi).unwrap();

    let src = [1u8; PUBLIC_KEY_SIZE];
    bytes.extend_from_slice(&src);

    let tgt = 88u32; // [2u8; PUBLIC_KEY_SIZE];
    serialize_into(&mut bytes, &tgt).unwrap();

    serialize_into(&mut bytes, &19u32).unwrap();
    serialize_into(&mut bytes, &29u64).unwrap();
    serialize_into(&mut bytes, &15771u16).unwrap();
    serialize_into(&mut bytes, &1u8).unwrap();
    // user fields
    serialize_into(&mut bytes, &3u8).unwrap();
    serialize_into(&mut bytes, &10u32).unwrap();
    serialize_into(&mut bytes, &1u8).unwrap();
    serialize_into(&mut bytes, &11u64).unwrap();
    serialize_into(&mut bytes, &20u32).unwrap();
    serialize_into(&mut bytes, &2u8).unwrap();
    let l = bytes.len() as u32;
    serialize_into(&mut bytes, &l).unwrap();
    bytes.extend_from_slice(&bytes.clone()[..l as usize]);
    serialize_into(&mut bytes, &30u32).unwrap();
    serialize_into(&mut bytes, &3u8).unwrap();
    serialize_into(&mut bytes, &12u32).unwrap();
    serialize_into(&mut bytes, &14u64).unwrap();
    let sig = [17u8; SIGNATURE_SIZE];
    bytes.extend_from_slice(&sig[..]);

    assert_eq!(validate_user_fields(&bytes[..]), None);

    serialize_into(&mut bytes, &17552u16).unwrap();

    assert_eq!(validate_transaction(&bytes[..]), Some(bytes.len()));
}

#[test]
fn test_validate_transaction_id_pk() {
    let mut bytes = Vec::<u8>::new();
    serialize_into(&mut bytes, &3u16).unwrap(); // lo
    let hi: u32 = 0x8000_0000 + 10;
    serialize_into(&mut bytes, &hi).unwrap();

    let src = 55u32; // [1u8; PUBLIC_KEY_SIZE];
    serialize_into(&mut bytes, &src).unwrap();

    let tgt = [2u8; PUBLIC_KEY_SIZE];
    bytes.extend_from_slice(&tgt);

    serialize_into(&mut bytes, &19u32).unwrap();
    serialize_into(&mut bytes, &29u64).unwrap();
    serialize_into(&mut bytes, &15771u16).unwrap();
    serialize_into(&mut bytes, &1u8).unwrap();
    // user fields
    serialize_into(&mut bytes, &3u8).unwrap();
    serialize_into(&mut bytes, &10u32).unwrap();
    serialize_into(&mut bytes, &1u8).unwrap();
    serialize_into(&mut bytes, &11u64).unwrap();
    serialize_into(&mut bytes, &20u32).unwrap();
    serialize_into(&mut bytes, &2u8).unwrap();
    let l = bytes.len() as u32;
    serialize_into(&mut bytes, &l).unwrap();
    bytes.extend_from_slice(&bytes.clone()[..l as usize]);
    serialize_into(&mut bytes, &30u32).unwrap();
    serialize_into(&mut bytes, &3u8).unwrap();
    serialize_into(&mut bytes, &12u32).unwrap();
    serialize_into(&mut bytes, &14u64).unwrap();
    let sig = [17u8; SIGNATURE_SIZE];
    bytes.extend_from_slice(&sig[..]);

    assert_eq!(validate_user_fields(&bytes[..]), None);

    serialize_into(&mut bytes, &17552u16).unwrap();

    assert_eq!(validate_transaction(&bytes[..]), Some(bytes.len()));
}

#[test]
fn test_validate_transaction_pk_pk() {
    let mut bytes = Vec::<u8>::new();
    serialize_into(&mut bytes, &3u16).unwrap(); // lo
    let hi: u32 = 10;
    serialize_into(&mut bytes, &hi).unwrap();

    let src = [1u8; PUBLIC_KEY_SIZE];
    bytes.extend_from_slice(&src);
    //serialize_into(&mut bytes, &src).unwrap();

    let tgt = [2u8; PUBLIC_KEY_SIZE];
    bytes.extend_from_slice(&tgt);
    //serialize_into(&mut bytes, &tgt).unwrap();

    serialize_into(&mut bytes, &19u32).unwrap();
    serialize_into(&mut bytes, &29u64).unwrap();
    serialize_into(&mut bytes, &15771u16).unwrap();
    serialize_into(&mut bytes, &1u8).unwrap();
    // user fields
    serialize_into(&mut bytes, &3u8).unwrap();
    serialize_into(&mut bytes, &10u32).unwrap();
    serialize_into(&mut bytes, &1u8).unwrap();
    serialize_into(&mut bytes, &11u64).unwrap();
    serialize_into(&mut bytes, &20u32).unwrap();
    serialize_into(&mut bytes, &2u8).unwrap();
    let l = bytes.len() as u32;
    serialize_into(&mut bytes, &l).unwrap();
    bytes.extend_from_slice(&bytes.clone()[..l as usize]);
    serialize_into(&mut bytes, &30u32).unwrap();
    serialize_into(&mut bytes, &3u8).unwrap();
    serialize_into(&mut bytes, &12u32).unwrap();
    serialize_into(&mut bytes, &14u64).unwrap();
    let sig = [17u8; SIGNATURE_SIZE];
    bytes.extend_from_slice(&sig[..]);

    assert_eq!(validate_user_fields(&bytes[..]), None);

    serialize_into(&mut bytes, &17552u16).unwrap();

    assert_eq!(validate_transaction(&bytes[..]), Some(bytes.len()));
}
