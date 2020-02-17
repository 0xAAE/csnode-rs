use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use log::debug;

// network submodules
use super::TEST_STOP_DELAY_SEC;
use super::packet::Packet;
// top-level modules
use super::super::config::SharedConfig;
//use super::super::collaboration::Collaboration;

pub struct MessageProcessor {
    rx_msg: Receiver<Packet>,
    tx_send: Sender<Packet>
}

impl MessageProcessor {

    pub fn new(_conf: SharedConfig, rx_msg: Receiver<Packet>, tx_send:Sender<Packet>) -> MessageProcessor {
        MessageProcessor {
            rx_msg: rx_msg,
            tx_send: tx_send
        }
    }

    pub fn recv(&self) {
		match self.rx_msg.recv_timeout(Duration::from_secs(TEST_STOP_DELAY_SEC)) {
			Err(_) => (),
			Ok(p) => {
                let mt = match p.msg_type() {
                    None => "Unknown".to_string(),
                    Some(v) => v.to_string()
                };
                debug!("msg::{}", mt)
            }
        }
    }

}
