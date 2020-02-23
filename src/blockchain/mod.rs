use std::sync::{Arc, RwLock};

pub mod blocks;
pub mod caches;
mod raw_block;

//extern crate ruspiro_singleton;
//use ruspiro_singleton::Singleton;

pub type SharedBlocks = Arc<RwLock<blocks::Blocks>>;
pub type SharedCaches = Arc<RwLock<caches::Caches>>;

// pub static BLOCKS: Singleton<blocks::Blocks> = Singleton::new(blocks::Blocks::new());
// pub static CACHES: Singleton<caches::Caches> = Singleton::new(caches::Caches::new());
