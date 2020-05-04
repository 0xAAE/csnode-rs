use std::sync::{Arc, RwLock};

pub mod blocks;
pub mod caches;
pub mod raw_block;
mod raw_block_tests;

pub type SharedBlocks = Arc<RwLock<blocks::Blocks>>;
pub type SharedCaches = Arc<RwLock<caches::Caches>>;

//use ruspiro_singleton::Singleton;
// pub static BLOCKS: Singleton<blocks::Blocks> = Singleton::new(blocks::Blocks::new());
// pub static CACHES: Singleton<caches::Caches> = Singleton::new(caches::Caches::new());