use super::fragment::{Flags, Fragment};
//use super::super::PUBLIC_KEY_SIZE;
use super::super::PublicKey;
use super::super::blake2s_simd::Hash;

use std::collections::BTreeSet;

pub struct Packet {
	flags: Flags,
	count_fragments: u16,
	id: u64,
	sender: Option<Box<PublicKey>>,
	target: Option<Box<PublicKey>>,
	hash: Hash,
	payload: Vec<u8>
}

impl Packet {

	pub fn new(fragments: BTreeSet<Fragment>) -> Option<Packet> {
		None
	}

}
