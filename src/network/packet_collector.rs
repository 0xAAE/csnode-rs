use super::TEST_STOP_DELAY_SEC;
use super::fragment::Fragment;

use log::info;
use std::sync::mpsc::Receiver;
use std::time::Duration;

pub struct PacketCollector {
	rx: Receiver<Fragment>
}

impl PacketCollector {

	pub fn new(rx: Receiver<Fragment>) -> PacketCollector {
		PacketCollector {
			rx: rx
		}
	}

	pub fn recv(&self) {
		match self.rx.recv_timeout(Duration::from_secs(TEST_STOP_DELAY_SEC)) {
			Err(_) => (),
			Ok(data) => {
				match data.payload() {
					None => {
						info!("get fragment with no payload");
					}
					Some(p) => {
						let frg: String = data.fragment().map_or("single".to_string(), |v| {
							return format!("{} from {}", v.0, v.1);
						});
						info!("get fragment with payload of {} bytes, flags {:?}, {}", p.len(), data.flags(), frg);
					}
				}
			}
		}
	}
}
