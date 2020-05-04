use super::config::SharedConfig;
use std::thread::{JoinHandle, spawn};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender}; // , Sender
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;
use std::time::Instant;
use log::info;

use base58::FromBase58;

use csp2p_rs::{CSHost, NodeInfo, NodeId, RawPacket};

pub const TEST_STOP_DELAY_SEC: u64 = 2;
const PING_NEIGHBOURS_DELAY_MS: u64 = 1900;
const MAX_MSG_QUEUE: usize = 1024;
const MAX_CMD_QUEUE: usize = 1024;

pub mod packet;
use packet::Packet;
use super::SharedBlocks;

mod packet_collector;
mod command_processor;
use command_processor::CommandProcessor;
mod message_processor;
use message_processor::MessageProcessor;
mod packet_sender;
mod validator;

use super::core_logic::{SharedRound, CoreLogic};

pub struct Network {
	collect_thread:		JoinHandle<()>,
	neighbours_thread: 	JoinHandle<()>,
	processor_thread:	JoinHandle<()>,
	sender_thread:		JoinHandle<()>,
    stop_flag:          Arc<AtomicBool>,
    host:               CSHost
}

impl Network {
	pub fn new(conf: SharedConfig, blocks: SharedBlocks) -> Box<Network> {
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
        
        let round = CoreLogic::new_shared_round();
        let msg_processor = MessageProcessor::new(conf.clone(), rx_msg, tx_send.clone(), blocks.clone(), round.clone());
        let neighbourhood = CommandProcessor::new(conf.clone(), rx_cmd, tx_send, blocks, round);

		Box::new(
            Network {
                stop_flag: stop_flag_instance.clone(),
                collect_thread: start_collect(conf.clone(), stop_flag_instance.clone(), rx_raw, tx_cmd, tx_msg),
                neighbours_thread: start_neighbourhood(stop_flag_instance.clone(), neighbourhood),
                processor_thread: start_msg_processor(stop_flag_instance.clone(), msg_processor),
                sender_thread: start_sender(conf, stop_flag_instance, rx_send),
                host
            }
        )
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
	spawn(move || {
		info!("Packet collector started");
		let packet_collector = packet_collector::PacketCollector::new(rx_raw, tx_cmd, tx_msg);
        loop {
			packet_collector.recv();
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Packet collector stopped");
	})
}

fn start_neighbourhood(stop_flag: Arc<AtomicBool>, mut neighbourhood: CommandProcessor) -> JoinHandle<()> {
	info!("Start neighbourhood service");
	spawn(move || {
        info!("Neighbourhood started");
        
        let mut prev_ping = Instant::now();
        loop {
            let ping_pause = prev_ping.elapsed();
            if ping_pause.as_millis() as u64 >= PING_NEIGHBOURS_DELAY_MS {
                neighbourhood.ping_all();
                prev_ping = Instant::now();
            }
            neighbourhood.recv();
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Neighbourhood stopped");
	})
}

fn start_msg_processor(stop_flag: Arc<AtomicBool>, mut msg_processor: MessageProcessor) -> JoinHandle<()> {
	info!("Start message processor");
	spawn(move || {
        info!("Message processor started");
        loop {
            msg_processor.recv();
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Message processor stopped");
	})
}

fn start_sender(_conf: SharedConfig, stop_flag: Arc<AtomicBool>, rx_send: Receiver<Packet>) -> JoinHandle<()> {
	info!("Start packet sender");
	spawn(move || {
        info!("Packet sender started");
        let packet_sender = packet_sender::PacketSender::new(rx_send);
        loop {
            packet_sender.recv();
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Packet sender stopped");
	})
}

fn parse_known_hosts_or_default(known_hosts: &mut Vec<NodeInfo>, hosts_filename: &str) {
    if !hosts_filename.is_empty() {
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
