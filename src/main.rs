#[macro_use]
extern crate bitflags;
extern crate log;

use log::info;

mod config;
use config::SharedConfig;

mod logger;
mod network;

use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::spawn;
use std::thread::JoinHandle;
use std::time;

pub const PUBLIC_KEY_SIZE: usize = 32;

pub type PublicKey = [u8; PUBLIC_KEY_SIZE];

fn main() {
    println!("Hello, world!");
    let mut file_name = "config.ini".to_string();
    for arg in std::env::args().skip(1) {
        if arg.starts_with("--config=") {
            match arg.split("=").skip(1).next() {
                Some(v) => {
                    file_name = v.to_string();
                }
                None => {
                    println!("Incorrect option value for --config, must be --config=<file name>");
                }
            };
        }
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let conf: SharedConfig = Arc::new(RwLock::new(config::Config::new(&file_name)));
    
    // init logger
    logger::init(conf.clone());
    // run config observer thread:
    let config_observer = start_config_observer_thread(conf.clone(), stop_flag.clone());
    // run network thread (which in its turn will start all necessary own threads)
    let network = start_network_thread(conf.clone(), stop_flag.clone());

    // imitate other work: sleep too long and exit
    thread::sleep(time::Duration::from_secs(30));
    stop_flag.store(true, Ordering::SeqCst);
    config_observer.join().unwrap();
    network.join().unwrap();
}

fn start_config_observer_thread(config: SharedConfig, stop_flag: Arc<AtomicBool>) -> JoinHandle<()> {
    info!("Start logger");
    let handle = spawn(move || {
        info!("Logger started");
        let mut wait_sec;
        loop {
            // get wait seconds
            {
                let data_guard = config.read().unwrap();
                wait_sec = data_guard.reload_delay_sec;
            }
            // test exit flag before pause
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
            let pause = time::Duration::from_secs(wait_sec.into());
            thread::sleep(pause);
            // test exit flag before reload config
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
            // reload configuration parameters
            {
                let mut data_guard = config.write().unwrap();
                data_guard.reload();
            }
        }
    });
    handle
}

fn start_network_thread(config: SharedConfig, stop_flag: Arc<AtomicBool>) -> JoinHandle<()> {
    info!("Start network");
    let handle = spawn(move || {
        let net = network::Network::new(config);
        info!("Network started");
        loop {
            thread::sleep(time::Duration::from_secs(2));
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Trying to stop network");
		net.stop();
        info!("Network stopped");
    });
    handle
}
