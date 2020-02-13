use super::TEST_STOP_DELAY_SEC;
use super::packet::Packet;

use log::{info, warn};
use std::sync::mpsc::Receiver;
use std::time::Duration;

extern crate csp2p_rs;
use csp2p_rs::NodeId;

pub struct PacketCollector {
	rx: Receiver<(NodeId, Vec<u8>)>
}

impl PacketCollector {

	pub fn new(receiver: Receiver<(NodeId, Vec<u8>)>) -> PacketCollector {
		PacketCollector {
			rx: receiver
		}
	}

	pub fn recv(&self) {
		match self.rx.recv_timeout(Duration::from_secs(TEST_STOP_DELAY_SEC)) {
			Err(_) => (),
			Ok(data) => {
				match Packet::new(data.0, data.1) {
					None => (),
					Some(pack) => {
						if pack.is_neigbour() {
							info!("get neigbour packet, payload size: {}", pack.payload().unwrap_or_default().len());
						}
						else if pack.is_message() {
							let mt = match pack.msg_type() {
								None => "Unknown".to_string(),
								Some(v) => v.to_string()
							};
							let r = match pack.round() {
								None => "Unset".to_string(),
								Some(v) => v.to_string()
							};
							let plen = match pack.payload() {
								None => "None".to_string(),
								Some(v) => v.len().to_string()
							};
							info!("get message packet {} from round {} with payload of {} bytes", mt, r, plen);
						}
						else {
							warn!("get strange packet, neither neigbour, nor message");
						}
					}
				}
			}
		}
	}
}
