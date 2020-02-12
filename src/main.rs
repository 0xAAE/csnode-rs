#[macro_use]
extern crate bitflags;

extern crate log;
use log::info;

extern crate blake2s_simd;

extern crate csp2p_rs;
use csp2p_rs::CSHost;
use csp2p_rs::NodeInfo;

extern crate base58;
use base58::FromBase58;

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
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

pub const PUBLIC_KEY_SIZE: usize = 32;
pub const HASH_SIZE: usize = 32;

pub type PublicKey = [u8; PUBLIC_KEY_SIZE];
//pub type Hash = [u8; HASH_SIZE]; // hash is defined in blake2s

static ZERO_PUBLIC_KEY: PublicKey = [0u8; PUBLIC_KEY_SIZE];

fn main() {
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
    
    // run network (which in its turn will start all necessary own threads)

    // get from config
    let node_id: String;
    let hosts_filename: String;
    {
        let conf_guard = conf.read().unwrap();
        node_id = conf_guard.node_id.clone();
        hosts_filename = conf_guard.hosts_filename.clone();
    }

    // init host with own id
    let bytes = node_id[..].from_base58().unwrap(); // base58 -> Vec<u8>
    let mut host = CSHost::new(&bytes[..]).unwrap();

    // init host entry points list
    let mut known_hosts = Vec::<NodeInfo>::new();
    parse_known_hosts_or_default(&mut known_hosts, &hosts_filename);
    host.add_known_hosts(known_hosts);
    host.start();

    // imitate other work: sleep too long and exit
    thread::sleep(time::Duration::from_secs(300));
    stop_flag.store(true, Ordering::SeqCst);
    config_observer.join().unwrap();
    
    host.stop();
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

fn parse_known_hosts_or_default(known_hosts: &mut Vec<NodeInfo>, hosts_filename: &String) {
    if hosts_filename.len() > 0 {
        match File::open(PathBuf::from(&hosts_filename)) {
            Err(e) => {
                println!("Failed to open file {}: {}", &hosts_filename, e);
                // add well known ru3 as entry point
                known_hosts.push(
                    NodeInfo {
                        id: "HBxj19cnpayn46GSqBGyKQXMaLThH4quuPt5gf8aFndg".from_base58().unwrap(),
                        ip: "195.133.147.58".to_string(),
                        port: 9000
                    }
                );
            }
            Ok(f) => {
                let reader = BufReader::new(f);
                for line in reader.lines() {
                    match line {
                        Err(_) => continue,
                        Ok(item) => {
                            let parts = item.split_whitespace().collect::<Vec<_>>();
                            if parts.len() != 2 {
                                println!("Malformed known_hosts record, must conform <ip:port id>, found {}", item);
                                continue;
                            }
                            let addr = parts[0].split(':').collect::<Vec<_>>();
                            if addr.len() != 2 {
                                println!("Malformed ip:port part, found {}", parts[0]);
                                continue;
                            }
                            // base58 -> Vec<u8>
                            let bytes: Vec<u8> = match parts[1].from_base58() {
                                Err(_) => {
                                    println!("Malformed id, must be a 32-byte key encoded base58, found {}", parts[1]);
                                    continue;
                                }
                                Ok(b) => b
                            };
                            known_hosts.push(
                                NodeInfo {
                                    id: bytes,
                                    ip: addr[0].to_string(),
                                    port: u16::from_str(addr[1]).unwrap_or(0)
                                }
                            );
                        }
                    }
                }
            }
        }
    }
}