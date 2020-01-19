use super::TEST_STOP_DELAY_SEC;
use super::fragment::Fragment;
use super::packet::Packet;
use super::super::blake2s_simd::Hash;

use log::info;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::collections::{BTreeSet, HashMap};

type FragmentsCollection = HashMap<Hash, BTreeSet<Fragment>>;

pub struct PacketCollector {
	rx: Receiver<Fragment>,
	partial: HashMap<Hash, BTreeSet<Fragment>>,
	completed: Vec<Packet>
}

impl PacketCollector {

	pub fn new(rx: Receiver<Fragment>) -> PacketCollector {
		let partial = FragmentsCollection::new();
		let completed = Vec::<Packet>::new();
		PacketCollector {
			rx: rx,
			partial: partial,
			completed: completed
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
							format!("{} from {}", v.0, v.1)
						});
						info!("get fragment with payload of {} bytes, flags {:?}, {}", p.len(), data.flags(), frg);
					}
				}
			}
		}
	}
}
