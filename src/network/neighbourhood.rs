use std::sync::mpsc::Receiver;
use std::time::Duration;

use log::info;

use super::TEST_STOP_DELAY_SEC;
use super::packet::Packet;

pub struct Neighbourhood {
	rx_cmd: Receiver<Packet>
}

impl Neighbourhood {

    pub fn new(rx_cmd: Receiver<Packet>) -> Neighbourhood {
        Neighbourhood {
            rx_cmd: rx_cmd
        }
    }

    pub fn recv(&self) {
		match self.rx_cmd.recv_timeout(Duration::from_secs(TEST_STOP_DELAY_SEC)) {
			Err(_) => (),
			Ok(p) => {
                let cmd = match p.nghbr_cmd() {
                    None => "Unknown".to_string(),
                    Some(v) => v.to_string()
                };
                info!("<- cmd::{}", cmd);
            }
        }
    }

}
