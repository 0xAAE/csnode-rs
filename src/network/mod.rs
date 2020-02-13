use super::config::SharedConfig;
use std::thread;
use std::thread::JoinHandle;
use std::thread::spawn;
use std::time;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::{SyncSender, Receiver};
use log::info;

extern crate base58;
use base58::FromBase58;

extern crate csp2p_rs;
use csp2p_rs::CSHost;
use csp2p_rs::NodeInfo;

const TEST_STOP_DELAY_SEC: u64 = 2;
const MAX_PACKET_QUEUE: usize = 1024;

//mod fragment;
mod packet;
//mod fragment_receiver;
mod packet_collector;

use packet::Packet;

pub struct Network {
	collect_thread:		JoinHandle<()>,
	dispatch_thread:	JoinHandle<()>,
	prepare_thread:		JoinHandle<()>,
	send_thread:		JoinHandle<()>,
	stop_flag: Arc<AtomicBool>
}

impl Network {
	pub fn new(conf: SharedConfig) -> Box<Network> {
		let stop_flag_instance = Arc::new(AtomicBool::new(false));
		// todo move queue limitation to <pack_collector -> pack_handler> channel
		let (tx_packet, rx_packet) = sync_channel::<Vec<u8>>(MAX_PACKET_QUEUE);

		// start p2p-compat CSHost

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
		let mut host = CSHost::new(&bytes[..], tx_packet).unwrap();
	
		// init host entry points list
		let mut known_hosts = Vec::<NodeInfo>::new();
		parse_known_hosts_or_default(&mut known_hosts, &hosts_filename);
		host.add_known_hosts(known_hosts);
		host.start();
		
		let instance = Box::new(Network {
			stop_flag: stop_flag_instance.clone(),
			collect_thread: start_collect(conf.clone(), stop_flag_instance.clone(), rx_packet),
			dispatch_thread: start_dispatch(conf.clone(), stop_flag_instance.clone()),
			prepare_thread: start_prepare(conf.clone(), stop_flag_instance.clone()),
			send_thread: start_send(conf.clone(), stop_flag_instance.clone())
		});
		instance
	}

	pub fn stop(self) {
		self.stop_flag.store(true, Ordering::SeqCst);
		self.collect_thread.join().expect("Failed to stop packet collector");
		self.dispatch_thread.join().expect("Failed to stop packet dispatcher");
		self.prepare_thread.join().expect("Failed to stop send preparator");
		self.send_thread.join().expect("Failed to stop fragment sender");
	}
}

fn start_collect(_conf: SharedConfig, stop_flag: Arc<AtomicBool>, rx: Receiver<Packet>) -> JoinHandle<()> {
	info!("Start packet collector");
	let handle = spawn(move || {
		info!("Packet collector started");
		let packet_collector = packet_collector::PacketCollector::new(rx);
        loop {
			packet_collector.recv();
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Packet collector stopped");
	});
	handle
}

fn start_dispatch(_conf: SharedConfig, stop_flag: Arc<AtomicBool>) -> JoinHandle<()> {
	info!("Start packet dispatcher");
	let handle = spawn(move || {
		info!("Packet dispatcher started");
        loop {
            thread::sleep(time::Duration::from_secs(TEST_STOP_DELAY_SEC));
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Packet dispatcher stopped");
	});
	handle
}

fn start_prepare(_conf: SharedConfig, stop_flag: Arc<AtomicBool>) -> JoinHandle<()> {
	info!("Start send preparator");
	let handle = spawn(move || {
		info!("Send preparator started");
        loop {
            thread::sleep(time::Duration::from_secs(TEST_STOP_DELAY_SEC));
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Send preparator stopped");
	});
	handle
}

fn start_send(_conf: SharedConfig, stop_flag: Arc<AtomicBool>) -> JoinHandle<()> {
	info!("Start fragment sender");
	let handle = spawn(move || {
		info!("Fragment sender started");
        loop {
            thread::sleep(time::Duration::from_secs(TEST_STOP_DELAY_SEC));
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Fragment sender stopped");
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
