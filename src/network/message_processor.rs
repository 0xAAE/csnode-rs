use std::sync::mpsc::Receiver;
use std::time::Duration;

use log::info;

use super::TEST_STOP_DELAY_SEC;
use super::packet::Packet;

pub struct MessageProcessor {
	rx_msg: Receiver<Packet>
}

impl MessageProcessor {

    pub fn new(rx_msg: Receiver<Packet>) -> MessageProcessor {
        MessageProcessor {
            rx_msg: rx_msg
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
                info!("<- msg::{}", mt)
            }
        }
    }

}
