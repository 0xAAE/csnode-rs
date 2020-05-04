use super::raw_block::RawBlock;

use rkv::{Manager, Rkv, SingleStore, Value, StoreOptions, EnvironmentBuilder, EnvironmentFlags}; // , StoreError
use bincode::{deserialize_from, serialize_into};

use std::fs;
use std::path::Path;
use std::sync::{RwLock, Arc};
use std::mem::size_of; //{size_of_val, size_of};
// use std::io::Write;
use log::{info, error};

static START_BLOCKCHAIN_SIZE: usize = 1024 * 1024 * 1024; // 1G
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
            // test if next cached block is
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
        let mut ready_to_chain = Vec::<RawBlock>::new();
        let mut next_req = self.chain_top + 1; 
        while next_req == self.cache_front {
            // pop_from_cach implicitly moves cache_front to next value:
            match self.pop_from_cache() {
                None => {
                    error!("panic! failed pop next block {}", next_req);
                    break;
                },
                Some(block) => {
                    ready_to_chain.push(block);
                    next_req += 1;
                }
            }
        }
        if !ready_to_chain.is_empty() {
            // chain all blocks from ready_to_chain
            for block in ready_to_chain {
                // store() is not dangerous, it will subsequently call to test_cahced_blocks() wich immediately return 
                // due to cache has already cleared from all theese blocks and at most one block behind
                self.store(block);
                // todo log info
            }
        }
    }

    // remove first cached block from cache and return it
    fn pop_from_cache(&mut self) -> Option<RawBlock> {
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
                            Ok((_, v)) => {
                                match v {
                                    None => (),
                                    Some(value) => {
                                        if let Value::Blob(bytes) = value {
                                            let ret = RawBlock::new_from_bytes(bytes);
                                            // re-assign cache_front
                                            match it.next() {
                                                None => {
                                                    self.cache_front = u64::max_value();
                                                },
                                                Some(res) => {
                                                    match res {
                                                        Err(_) => {
                                                            self.cache_front = u64::max_value();
                                                        }
                                                        Ok((k, _)) => {
                                                            self.cache_front = deserialize_from(k).unwrap();
                                                        }
                                                    }
                                                }
                                            }
                                            return ret;       
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

}
