use super::TEST_STOP_DELAY_SEC;
use super::packet::Packet;

use log::{debug, info, warn};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};
use std::time::Duration;

extern crate csp2p_rs;
use csp2p_rs::RawPacket;

use super::validator::Validator;

pub struct PacketCollector {
	rx_raw: Receiver<RawPacket>,
	tx_cmd: SyncSender<Packet>,
	tx_msg: SyncSender<Packet>,
	validator: Validator
}

impl PacketCollector {

	pub fn new(rx_raw: Receiver<RawPacket>, tx_cmd: SyncSender<Packet>, tx_msg: SyncSender<Packet>) -> PacketCollector {
		PacketCollector {
			rx_raw: rx_raw,
			tx_cmd: tx_cmd,
			tx_msg: tx_msg,
			validator: Validator::new()
		}
	}

	pub fn recv(&self) {
		match self.rx_raw.recv_timeout(Duration::from_secs(TEST_STOP_DELAY_SEC)) {
			Err(_) => (),
			Ok(data) => {
				match Packet::new(data.0, data.1) {
					None => (),
					Some(pack) => {
						if !self.validator.validate(&pack) {
							warn!("packet rejected by validator, drop");
							return;
						}
						if pack.is_neigbour() {
							let cmd = match pack.nghbr_cmd() {
								None => "Unknown".to_string(),
								Some(v) => v.to_string()
							};
							debug!("<- cmd::{}: {} bytes", cmd, pack.payload().unwrap_or_default().len());
							match self.tx_cmd.try_send(pack) {
								Ok(_) => (),
								Err(TrySendError::Full(_)) => {
									info!("command queue is full, drop until someone is handled");
								},
								Err(TrySendError::Disconnected(_)) => {
									warn!("neighbourhood is disconnected")
								}
							};
						}
						else { // pack is message
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
							debug!("<- msg::{}[{}]: {} bytes", mt, r, plen);
							match self.tx_msg.try_send(pack) {
								Ok(_) => (),
								Err(TrySendError::Full(_)) => {
									info!("message queue is full, drop until someone is handled");
								},
								Err(TrySendError::Disconnected(_)) => {
									warn!("message processor is disconnected")
								}
							};
						}
					}
				}
			}
		}
	}
}
