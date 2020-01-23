use super::TEST_STOP_DELAY_SEC;
use super::fragment::Fragment;
use super::packet::Packet;
use super::super::PublicKey;

use log::info;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::collections::{BTreeSet, HashMap};

#[derive(std::hash::Hash, std::cmp::Eq)]
struct PacketUnique {
	id: u64,
	sender: PublicKey
}

impl PartialEq for PacketUnique {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.sender == other.sender
    }
}

/// store uncompleted packets fragments: for every different packet accumulates all unique fragments;
/// when all fragments of some packet has got move it to completed
type PartialFragmentsCollection = HashMap<PacketUnique, BTreeSet<Fragment>>;

pub struct PacketCollector {
	rx: Receiver<Fragment>,
	partial: PartialFragmentsCollection,
	completed: Vec<Packet>
}

impl PacketCollector {

	pub fn new(rx: Receiver<Fragment>) -> PacketCollector {
		let partial = PartialFragmentsCollection::new();
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
						let frg: String = data.fragmentation().map_or("single".to_string(), |v| {
							format!("{} from {}", v.0, v.1)
						});
						info!("get fragment with payload of {} bytes, flags {:?}, {}", p.len(), data.flags(), frg);
					}
				}
			}
		}
	}
}
