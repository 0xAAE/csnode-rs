use std::sync::mpsc::Receiver;
use std::time::Duration;

use log::debug;

use super::TEST_STOP_DELAY_SEC;
use super::packet::Packet;

extern crate base58;
use base58::ToBase58; // [u8].to_base58()

extern crate csp2p_rs;
use csp2p_rs::{CSHost};

pub struct PacketSender {
    rx_send: Receiver<Packet>
}

impl PacketSender {
    pub fn new(rx_send: Receiver<Packet>) -> PacketSender {
        PacketSender {
            rx_send
        }
    }

    pub fn recv(&self) {
        match self.rx_send.recv_timeout(Duration::from_secs(TEST_STOP_DELAY_SEC)) {
			Err(_) => (),
			Ok(pack) => {
                match pack.address() {
                    None => {
                        debug!("-> broadcast packet");
                        CSHost::broadcast(pack.data());
                    }
                    Some(id) => {
                        CSHost::send_to(id, pack.data());
                        debug!("-> send packet to {}", id.to_base58());
                    }
                }
                // todo send packet to "raw"
            }
        }
    }
}