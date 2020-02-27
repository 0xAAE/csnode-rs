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

extern crate rkv;
use rkv::{Manager, Rkv, SingleStore, IntegerStore, Value, PrimitiveInt, StoreOptions, EnvironmentBuilder, DatabaseFlags, EnvironmentFlags, StoreError, DataError};
use serde_derive::Serialize;

use std::fs;
use std::path::Path;
use std::sync::{RwLock, Arc};
use std::convert::From;
use log::{info, error};

#[derive(Serialize)]
struct U64(u64);
impl PrimitiveInt for U64 {}
impl From<u64> for U64 {
    fn from(v: u64) -> Self {
        U64(v)
    }
}

static START_BLOCKCHAIN_SIZE: usize = 1 * 1024 * 1024 * 1024; // 1G
static INCREASE_BLOCKCHAIN_SIZE: usize = 500 * 1024 * 1024; // 500M

pub struct Blocks {
    environment: Arc<RwLock<rkv::Rkv>>,
    db: IntegerStore<U64>,
    /// the top of blockchain
    chain_top: u64
}

impl Blocks {

    pub fn new() -> Blocks {
        let path = Path::new("db/blockchain/blocks");
        fs::create_dir_all(path).unwrap();
        let mut builder: EnvironmentBuilder = Rkv::environment_builder();
        builder.
            set_flags(EnvironmentFlags::NO_SYNC | EnvironmentFlags::WRITE_MAP | EnvironmentFlags::MAP_ASYNC).
            set_map_size(START_BLOCKCHAIN_SIZE).
            set_max_dbs(2);
        let created_arc = Manager::singleton().write().unwrap().get_or_create(path, |path| { Rkv::from_env(path, builder) }).unwrap();
        let environment = created_arc.read().unwrap();
        let store = environment.open_integer::<&str, U64>("blocks", StoreOptions::create()).unwrap();

        Blocks {
            environment: created_arc.clone(),
            db: store,
            chain_top: 0
        }
    }

    pub fn top(&self) -> u64 {
        self.chain_top
    }

    pub fn store(&mut self, block: RawBlock) -> bool {
        let block_sequence = block.sequence().unwrap();
        if block_sequence <= self.chain_top {
            return false;
        }
        if block_sequence == self.chain_top + 1 {
            if !self.check_map_size() {
                error!("failed to store block {}", block_sequence);
                return false;
            }
            let guard = self.environment.read().unwrap();
            let mut writer = guard.write().unwrap();
            match self.db.put(&mut writer, U64(block.sequence().unwrap()), &Value::Blob(&block.data[..])) {
                Err(e) => {
                    error!("failed to store block: {}", e);
                    return false;
                }
                _ => {
                    match writer.commit() {
                        Err(e) => {
                            error!("failed to store block: {}", e);
                            return false;
                        }
                        _ => {
                            self.chain_top = block_sequence;
                        }
                    }
                }
            }
        }
        // cache for further usage

        true
    }

    fn check_map_size(&self, ) -> bool {
        let guard = self.environment.read().unwrap();
        match guard.info() {
            Err(e) => {
                error!("failed to get free space: {}", e);
                return false;
            }
            Ok(info) => {
                let current_size = info.map_size();
                let last_pgno = info.last_pgno();
                match guard.stat() {
                    Err(e) => {
                        error!("failed to get free space: " {});
                        return false;
                    }
                    Ok(stat) => {
                        let free_space = current_size - (stat.page_size() as usize * last_pgno);
                        if free_space < INCREASE_BLOCKCHAIN_SIZE / 2 {
                            let new_size = current_size + INCREASE_BLOCKCHAIN_SIZE;
                            match guard.set_map_size(new_size) {
                                Err(e) => {
                                    error!("failed to increase map size from {} to {}: {}", current_size, new_size, e);
                                    return false;
                                }
                                Ok(_) => {
                                    info!("blocks storage increased to {}", new_size);
                                }
                            }
                        }
                    }
                }
            }
        }

        true
    }
}
