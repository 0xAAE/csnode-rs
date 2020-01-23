use super::fragment::{Flags, Fragment};
//use super::super::PUBLIC_KEY_SIZE;
use super::super::PublicKey;
use super::super::ZERO_PUBLIC_KEY;
//use super::super::blake2s_simd::Hash;
use std::convert::TryInto;

use std::collections::BTreeSet;

pub struct Packet {
	flags: Flags,
	count_fragments: u16,
	id: u64,
	sender: Option<Box<PublicKey>>,
	target: Option<Box<PublicKey>>,
	payload: Vec<u8>
}

impl Packet {

	pub fn new(fragments: BTreeSet<Fragment>) -> Option<Packet> {
		if fragments.is_empty() {
			return None;
		}
		let first = fragments.iter().nth(0).unwrap();
		let flags = first.flags();
		let count = match first.fragmentation() {
			None => 1,
			Some(frg) => frg.1
		};
		let id = match first.id() {
			None => 0,
			Some(v) => v
		};
		let sender: Option<Box<PublicKey>> = match first.sender() {
			None => None,
			Some(s) => Some(Box::new(s.try_into().unwrap_or(ZERO_PUBLIC_KEY)))
		};
		let target: Option<Box<PublicKey>> = match first.target() {
			None => None,
			Some(t) => Some(Box::new(t.try_into().unwrap_or(ZERO_PUBLIC_KEY)))
		};
		// calc payload size
		let empty: &[u8] = &[];
		let mut payload_len = fragments.iter().fold(0, |v, frg| v + frg.payload().unwrap_or(empty).len());
		// todo add payload bytes
		let mut payload = Vec::<u8>::with_capacity(payload_len);
		for f in fragments.iter() {
			payload.extend(f.payload().unwrap_or(empty));
		}
		assert_eq!(payload.len(), payload_len);
		
		Some(Packet {
			flags: flags,
			count_fragments: count,
			id: id,
			sender: sender,
			target: target,
			payload: payload
		})
	}

}

impl std::hash::Hash for Packet {

	// identified by (id, sender)
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state);
		if self.sender != None {
			self.sender.hash(state);
		}
	}
	
}
