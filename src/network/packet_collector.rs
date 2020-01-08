use super::TEST_STOP_DELAY_SEC;

use log::info;

use std::sync::mpsc::Receiver;
use std::time::Duration;

pub struct PacketCollector {
	rx: Receiver<Vec<u8>>
}

impl PacketCollector {

	pub fn new(rx: Receiver<Vec<u8>>) -> PacketCollector {
		PacketCollector {
			rx: rx
		}
	}

	pub fn recv(&self) {
		match self.rx.recv_timeout(Duration::from_secs(TEST_STOP_DELAY_SEC)) {
			Err(_) => (),
			Ok(data) => {
				info!("get fragment of {} bytes", data.len());
			}
		}
	}
}
