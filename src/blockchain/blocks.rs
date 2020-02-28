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
use rkv::{Manager, Rkv, SingleStore, Value, StoreOptions, EnvironmentBuilder, EnvironmentFlags, StoreError};
use bincode::{deserialize_from, serialize_into};

use std::fs;
use std::path::Path;
use std::sync::{RwLock, Arc};
use std::mem::{size_of_val, size_of};
use std::io::Write;
use log::{info, error};

static START_BLOCKCHAIN_SIZE: usize = 1 * 1024 * 1024 * 1024; // 1G
static INCREASE_BLOCKCHAIN_SIZE: usize = 500 * 1024 * 1024; // 500M
static START_BLOCKCACHE_SIZE: usize = 10 * 1024 * 1024; // 10M
static INCREASE_BLOCKCACHE_SIZE: usize = 5 * 1024 * 1024; // 5M

pub struct Blocks {
    environment: Arc<RwLock<rkv::Rkv>>,
    // blockchain storage
    db: SingleStore,
    // current last block in chain
    chain_top: u64,
    // cached for future blocks storage
    cache: SingleStore,
    // current start block in cache
    cache_front: u64
}

impl Blocks {

    pub fn new() -> Blocks {
        let path = Path::new("db/blockchain/blocks");
        fs::create_dir_all(path).unwrap();
        let mut builder: EnvironmentBuilder = Rkv::environment_builder();
        builder.
            set_flags(EnvironmentFlags::NO_SYNC | EnvironmentFlags::WRITE_MAP | EnvironmentFlags::MAP_ASYNC).
            set_map_size(START_BLOCKCHAIN_SIZE).
            set_max_dbs(2); // chain & cache
        let created_arc = Manager::singleton().write().unwrap().get_or_create(path, |path| { Rkv::from_env(path, builder) }).unwrap();
        let environment = created_arc.read().unwrap();
        let db_store = environment.open_single("blocks", StoreOptions::create()).unwrap();
        let cache_store = environment.open_single("cache", StoreOptions::create()).unwrap();
        let mut instance = Blocks {
            environment: created_arc.clone(),
            db: db_store,
            chain_top: 0,
            cache: cache_store,
            cache_front: u64::max_value()
        };

        instance.chain_top = instance.last_sequence();
        instance.cache_front = instance.first_cached();

        instance
    }

    pub fn top(&self) -> u64 {
        self.chain_top
    }

    pub fn store(&mut self, block: RawBlock) -> bool {
        let block_sequence = block.sequence().unwrap();
        if self.contains(block_sequence) {
            return false;
        }

        if block_sequence == self.chain_top + 1 {
            // chain block
            if !self.check_map_size() {
                error!("failed to store block {}", block_sequence);
                return false;
            }
            {
                let guard = self.environment.read().unwrap();
                let mut writer = guard.write().unwrap();
                match self.db.put(&mut writer, &block, &Value::Blob(&block.data[..])) {
                    Err(e) => {
                        error!("failed to chain block: {}", e);
                        return false;
                    }
                    _ => {
                        match writer.commit() {
                            Err(e) => {
                                error!("failed to chain block: {}", e);
                                return false;
                            }
                            _ => {
                                self.chain_top = block_sequence;
                            }
                        }
                    }
                }
            }
            // 
            self.test_cached_blocks();
            return true;
        }

        // cache for further usage
        if !self.check_map_size() {
            error!("failed to cache block {}", block_sequence);
            return false;
        }
        let guard = self.environment.read().unwrap();
        let mut writer = guard.write().unwrap();
        match self.cache.put(&mut writer, &block, &Value::Blob(&block.data[..])) {
            Err(e) => {
                error!("failed to cache block: {}", e);
                return false;
            }
            _ => {
                match writer.commit() {
                    Err(e) => {
                        error!("failed to cache block: {}", e);
                        return false;
                    }
                    _ => {
                        if block_sequence < self.cache_front {
                            self.cache_front = block_sequence;
                        }
                    }
                }
            }
        }

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
                        error!("failed to get free space: {}", e);
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

    fn last_sequence(&self) -> u64 {
        let guard = self.environment.read().unwrap();
        let reader = guard.read().unwrap();
        match self.db.iter_start(&reader) {
            Err(_) => (),
            Ok(it) => {
                match it.last() {
                    None => (),
                    Some(res) => {
                        match res {
                            Err(_) => (),
                            Ok((k, _)) => {
                                let seq: u64 = deserialize_from(k).unwrap();
                                return seq;
                            }
                        }
                    }
                }
            }
        }

        0
    }

    fn first_cached(&self) -> u64 {
        let guard = self.environment.read().unwrap();
        let reader = guard.read().unwrap();
        match self.cache.iter_start(&reader) {
            Err(_) => (),
            Ok(mut it) => {
                match it.next() {
                    None => (),
                    Some(res) => {
                        match res {
                            Err(_) => (),
                            Ok((k, _)) => {
                                let seq: u64 = deserialize_from(k).unwrap();
                                return seq;
                            }
                        }
                    }
                }
            }
        }

        u64::max_value()
    }

    fn contains(&self, sequence: u64) -> bool {
        if sequence <= self.chain_top {
            return true;
        }
        if sequence < self.cache_front {
            return false;
        }
        if sequence == self.cache_front {
            return true;
        }

        // lookup through cache
        let mut key = [0u8; size_of::<u64>()];
        serialize_into(&mut key[..], &sequence).unwrap();
        let guard = self.environment.read().unwrap();
        let reader = guard.read().unwrap();
        match self.cache.get(&reader, &key) {
            Err(_) => false,
            Ok(None) => false,
            Ok(_) => true
        }
    }

    fn test_cached_blocks(&mut self) {
        let ready_to_chain = Vec::<RawBlock>::new();
        if self.chain_top + 1 == self.cache_front {
            let mut i = self.cache_front;
            loop {
                if self.contains(i) {
                    // todo load block from cache
                    // todo push block to ready_to_chain
                    i += 1;
                }
                else {
                    break;
                }
            }
            if !ready_to_chain.is_empty() {
                // todo remove all chained blocks from cache
                self.cache_front = self.first_cached(); 

                // chain all blocks from ready_to_chain
                for block in ready_to_chain {
                    // store() is not dangerous, it will subsequently call to test_cahced_blocks() wich immediately return 
                    // due to cache has already cleared from all theese blocks and at most one block behind
                    self.store(block);
                }
            }
        }
    }

    fn remove_from_cache(&mut self, block: RawBlock) {
        let block_sequence = block.sequence().unwrap();
        if block_sequence <= self.cache_front {
            return;
        }

        // todo try remove block

    }

}
