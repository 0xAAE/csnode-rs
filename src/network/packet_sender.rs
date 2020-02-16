use std::sync::mpsc::Receiver;
use std::time::Duration;

use log::info;

use super::TEST_STOP_DELAY_SEC;
use super::packet::Packet;

pub struct PacketSender {
    rx_send: Receiver<Packet>
}

impl PacketSender {
    pub fn new(rx_send: Receiver<Packet>) -> PacketSender {
        PacketSender {
            rx_send: rx_send
        }
    }

    pub fn recv(&self) {
        match self.rx_send.recv_timeout(Duration::from_secs(TEST_STOP_DELAY_SEC)) {
			Err(_) => (),
			Ok(pack) => {
                // todo send packet to "raw"
                info!("-> packet");
            }
        }
    }
}