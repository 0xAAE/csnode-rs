use super::config::SharedConfig;
use std::thread::{JoinHandle, spawn};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender, Sender};
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;
use log::info;

extern crate base58;
use base58::FromBase58;

extern crate csp2p_rs;
use csp2p_rs::{CSHost, NodeInfo, NodeId, RawPacket};

pub const TEST_STOP_DELAY_SEC: u64 = 2;
const MAX_MSG_QUEUE: usize = 1024;
const MAX_CMD_QUEUE: usize = 1024;

pub mod packet;
use packet::Packet;

//mod fragment_receiver;

mod packet_collector;
mod command_processor;
mod message_processor;
mod packet_sender;
mod validator;

pub struct Network {
	collect_thread:		JoinHandle<()>,
	neighbours_thread: 	JoinHandle<()>,
	processor_thread:	JoinHandle<()>,
	sender_thread:		JoinHandle<()>,
    stop_flag:          Arc<AtomicBool>,
    host:               CSHost
}

impl Network {
	pub fn new(conf: SharedConfig) -> Box<Network> {
		let stop_flag_instance = Arc::new(AtomicBool::new(false));
        // p2p-compat -> packet_collector channel, fully async:
        let (tx_raw, rx_raw) = channel::<RawPacket>();
        // packet_collector -> neighbourhood channel, may drop excess commands
        let (tx_cmd, rx_cmd) = sync_channel::<Packet>(MAX_CMD_QUEUE);
        // packet_collector -> msg_processor channel, may drop excess messages
        let (tx_msg, rx_msg) = sync_channel::<Packet>(MAX_MSG_QUEUE);
        // neighbourhood, msg_processor -> packet_sender
        let (tx_send, rx_send) = channel::<Packet>();

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
		let mut host = CSHost::new(&bytes[..], tx_raw).unwrap();
	
		// init host entry points list
		let mut known_hosts = Vec::<NodeInfo>::new();
		parse_known_hosts_or_default(&mut known_hosts, &hosts_filename);
		host.add_known_hosts(known_hosts);
		host.start();
		
		let instance = Box::new(
            Network {
                stop_flag: stop_flag_instance.clone(),
                collect_thread: start_collect(conf.clone(), stop_flag_instance.clone(), rx_raw, tx_cmd, tx_msg),
                neighbours_thread: start_neighbourhood(conf.clone(), stop_flag_instance.clone(), rx_cmd, tx_send.clone()),
                processor_thread: start_msg_processor(conf.clone(), stop_flag_instance.clone(), rx_msg, tx_send),
                sender_thread: start_sender(conf.clone(), stop_flag_instance.clone(), rx_send),
                host: host
            });
		instance
	}

	pub fn stop(mut self) {
        self.host.stop();
		self.stop_flag.store(true, Ordering::SeqCst);
		self.collect_thread.join().expect("Failed to stop packet collector");
		self.neighbours_thread.join().expect("Failed to stop neihbourhood");
		self.processor_thread.join().expect("Failed to stop message processor");
		self.sender_thread.join().expect("Failed to stop fragment sender");
	}
}

fn start_collect(_conf: SharedConfig, stop_flag: Arc<AtomicBool>,
        rx_raw: Receiver<(NodeId, Vec<u8>)>,
        tx_cmd: SyncSender<Packet>,
        tx_msg: SyncSender<Packet>) -> JoinHandle<()> {
	info!("Start packet collector");
	let handle = spawn(move || {
		info!("Packet collector started");
		let packet_collector = packet_collector::PacketCollector::new(rx_raw, tx_cmd, tx_msg);
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

fn start_neighbourhood(conf: SharedConfig, stop_flag: Arc<AtomicBool>, rx_cmd: Receiver<Packet>, tx_send: Sender<Packet>) -> JoinHandle<()> {
	info!("Start neighbourhood service");
	let handle = spawn(move || {
        info!("Neighbourhood started");
        let mut neighbourhood = command_processor::CommandProcessor::new(conf.clone(), rx_cmd, tx_send);
        loop {
            neighbourhood.recv();
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Neighbourhood stopped");
	});
	handle
}

fn start_msg_processor(_conf: SharedConfig, stop_flag: Arc<AtomicBool>, rx_msg: Receiver<Packet>, tx_send: Sender<Packet>) -> JoinHandle<()> {
	info!("Start message processor");
	let handle = spawn(move || {
        info!("Message processor started");
        let mut msg_processor = message_processor::MessageProcessor::new(_conf.clone(), rx_msg, tx_send);
        loop {
            msg_processor.recv();
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Message processor stopped");
	});
	handle
}

fn start_sender(_conf: SharedConfig, stop_flag: Arc<AtomicBool>, rx_send: Receiver<Packet>) -> JoinHandle<()> {
	info!("Start packet sender");
	let handle = spawn(move || {
        info!("Packet sender started");
        let packet_sender = packet_sender::PacketSender::new(rx_send);
        loop {
            packet_sender.recv();
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Packet sender stopped");
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
