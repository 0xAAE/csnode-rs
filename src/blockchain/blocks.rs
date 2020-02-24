/// Block complete structure: bytes serialized as follow:
/// 
/// hashed data:
///     meta:
///         version:                u8
///         previous hash:          Hash
///         sequence:               u64
///         user fields:
///             count:              u8
///             user field [
///                 key:            u32
///                 value:
///                     type:       u8
///                     value:
///                                 | u64   (type == 1)
///                                 | vec   (type == 2)
///                                     len: u32
///                                     data:   u8[len]
///                                 | money (type == 3)
///                                     integral: i32
///                                     fraction: u64
///             ] * count
///         round cost: money
///                     integral:   i32
///                     fraction:   u64
/// 
///     transactions:
///         count:                  u32
///         transaction [
///             inner ID:
///                 lo:             u16
///                 hi:             u32
///             source:
///                                 | u32   (hi & 0x8000_0000)
///                                 | PublicKey
///             target:
///                                 | u32   (hi & 0x4000_0000)
///                                 | PublicKey
///             sum: money
///                 integral:       i32
///                 fraction:       u64
///             max fee:            u16
///             currency:           u8
///             user fields:
///                 count_uf:       u8
///                 user filed [
///                     key:        u32
///                     value:
///                         type:   u8
///                         value:
///                                 | u64   (type == 1)
///                                 | vec   (type == 2)
///                                     len: u32
///                                     data:   u8[len]
///                                 | money (type == 3)
///                                     integral: i32
///                                     fraction: u64
///                 ] * count_uf
///             signature:          Signature
///             fee:                u16
///         ] * count
/// 
///     new wallets:
///         count_nw:               u32
///         new wallet [
///             address id:         u64 (1b - source/target, 63b - transaction index in block)
///             wallet id:          u32
///         ] * count_nw
/// 
///     trusted info:
///         count:                  u8
///         actual:                 u64 (biteset)           - count of 1 -> sig_blk_cnt
///         keys:                   PublicKey[count]
///         next rt:
///             count:              u8                      - ?
///             actual_rt:          u64 (bitset)            - count of 1 -> sig_next_rt_cnt
///             signatures:         Signature[sig_next_rt_cnt]
///     hashed len:                 usize
///
/// signatures:                     Signature[sig_blk_cnt]
///     
/// contract signatures:
///     count_contract_sig:         u8
///     contract_sig [
///         key:                    PublicKey
///         round:                  u64
///         trusted_cnt:            u8
///         trusted [
///             n:                  u8
///             sig:                Signature
///         ] * trusted_cnt
///     ] * count_contract_sig

use super::raw_block::RawBlock;

use rkv::{Manager, Rkv, SingleStore, Value, StoreOptions};

use std::fs;
use std::path::Path;

pub struct Blocks {
    deferred: Option<RawBlock>,
    store: SingleStore,
    /// the top of blockchain
    chain_top: u64
}

impl Blocks {

    pub fn new() -> Blocks {
        let path = Path::new("db/blockchain/blocks");
        fs::create_dir_all(path).unwrap();
        let created_arc = Manager::singleton().write().unwrap().get_or_create(path, Rkv::new).unwrap();
        let env = created_arc.read().unwrap();
        let store: SingleStore = env.open_single("blocks", StoreOptions::create()).unwrap();

        Blocks {
            deferred: RawBlock::new(Vec::<u8>::new()),
            store: store,
            chain_top: 0
        }
    }

    pub fn top(&self) -> u64 {
        self.chain_top
    }
}
