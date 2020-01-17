//use super::super::PublicKey;
use super::super::PUBLIC_KEY_SIZE;
//use super::super::bitflags;
use super::super::blake2s_simd::{blake2s, Hash};
//use std::mem::size_of;

use std::convert::TryInto;

bitflags! {
	pub struct Flags: u8 {
		const ZERO = 0;
		// network command
		const N = 0b0000_0001;
		// fragment of multifragmented packet
		const F = 0b0000_0010;
		// broadcast packet
		const B = 0b0000_0100;
		// compressed
		const Z = 0b0000_1000;
		// encrypted
		const E = 0b0001_0000;
		// signed
		const S = 0b0010_0000;
		// neighbors-only direct packet
		const D = 0b0100_0000;

		const FB = Self::F.bits | Self::B.bits;
		const NFB = Self::N.bits | Self::F.bits | Self::B.bits;
	}
}

#[test]
fn test_bitflags() {
	let h1 = Flags::N | Flags::D | Flags::Z;
    let h2 = Flags::B | Flags::F | Flags::Z;
    assert_eq!((h1 | h2), Flags::N | Flags::F | Flags::B | Flags::Z | Flags::D);   // union
	assert_eq!((h1 & h2), Flags::Z);     // intersection
	let h3 = Flags::N | Flags::F | Flags::B | Flags::D | Flags::Z | Flags::E | Flags::S;
    assert_eq!((h3 - h1), Flags::F | Flags::B | Flags::E | Flags::S);     // set difference
    assert_eq!(!h2, Flags::N | Flags::E | Flags::S | Flags::D);           // set complement
}

pub struct Header {
	// flags: Flags,
	// number: u16,
	// count: u16,
	// id: u64,
	// sender: Option<Box<PublicKey>>,
	// target: Option<Box<PublicKey>>,
	// hash: Hash
}

// ! https://docs.rs/tokio-byteorder/0.2.0/tokio_byteorder/
// ! https://stackoverflow.com/questions/29307474/how-can-i-convert-a-buffer-of-a-slice-of-bytes-u8-to-an-integer
// ! https://doc.rust-lang.org/std/primitive.u16.html#method.from_le_bytes

// fn read_public_key(input: &[u8]) -> Box<PublicKey> {
// 	let mut tmp: PublicKey = Default::default();
// 	tmp.copy_from_slice(input);
// 	Box::new(tmp)
// }

impl Header {
	// pub fn new(bytes: &[u8]) -> Option<Header> {
	// 	if bytes.len() == 0 {
	// 		return None;
	// 	}
	// 	// deduce header size from flags
	// 	let size = Header::valid_len(bytes[0]);
	// 	if size == 0 {
	// 		// illegal header value
	// 		return None;
	// 	}
	// 	if bytes.len() < size {
	// 		// data too small
	// 		return None;
	// 	}

	// 	let flags;
	// 	match Flags::from_bits(bytes[0]) {
	// 		None => return None,
	// 		Some(f) => flags = f
	// 	}
		
	// 	let mut pos: usize = 1; // just behind the flags
	// 	let mut number: u16 = 0;
	// 	let mut count: u16 = 1;
	// 	if flags.contains(Flags::F) {
	// 		number = u16::from_le_bytes(bytes[pos..].try_into().unwrap());
	// 		pos += size_of::<u16>();
	// 		count = u16::from_le_bytes(bytes[pos..].try_into().unwrap());
	// 		pos += size_of::<u16>();
	// 	}

	// 	let id = u64::from_le_bytes(bytes[pos..].try_into().unwrap());
	// 	pos += size_of::<u64>();

	// 	let mut sender: Option<Box<PublicKey>> = None;
	// 	if !flags.contains(Flags::N) {
	// 		sender = Some(read_public_key(&bytes[pos..]));
	// 		// let mut tmp: PublicKey = Default::default();
	// 		// tmp.copy_from_slice(&bytes[pos..]);
	// 		// sender = Some(Box::new(tmp));
	// 		pos += PUBLIC_KEY_SIZE;
	// 	}

	// 	let mut target: Option<Box<PublicKey>> = None;
	// 	if !flags.contains(Flags::B) && !flags.contains(Flags::D) {
	// 		target = Some(read_public_key(&bytes[pos..]));
	// 		//pos += PUBLIC_KEY_SIZE;
	// 	}

	// 	// todo calculate hash of bytes[0..size-1]
	// 	let hash = blake2s(&bytes[..size]);

	// 	Some(Header {
	// 		flags: flags,
	// 		number: number,
	// 		count: count,
	// 		id: id,
	// 		sender: sender,
	// 		target: target,
	// 		hash: hash
	// 	})
	// }

	fn valid_len(flags: u8) -> usize {
		match Flags::from_bits(flags) {
			Some(Flags::N) => 1, // flags only
			Some(Flags::B) => 41, // flags + id + sender
			Some(Flags::FB) => 45, // flags + idx/cnt + id + sender
			Some(Flags::F) => 77, // flags + idx/cnt + id + sender + target
			Some(Flags::ZERO) => 73, // flags + id + sender + target
			Some(Flags::NFB) => 5, // flags + idx/cnt
			Some(_) => 0, // not used
			None => 0 // disalowed
		}
	}
}

#[test]
fn test_valid_len() {
	let f_net = Flags::N.bits;
	let f_bcast = Flags::B.bits;
	let f_frag_bcast = Flags::FB.bits;
	let f_frag = Flags::F.bits;
	let f_zero = Flags::ZERO.bits;
	let f_frag_bcast_net = Flags::NFB.bits;

	assert_eq!(Header::valid_len(f_net), 1);
	assert_eq!(Header::valid_len(f_bcast), 41);
	assert_eq!(Header::valid_len(f_frag_bcast), 45);
	assert_eq!(Header::valid_len(f_frag), 77);
	assert_eq!(Header::valid_len(f_zero), 73);
	assert_eq!(Header::valid_len(f_frag_bcast_net), 5);

	let f_illegal_1 = Flags::N | Flags::B; 
	assert_eq!(Header::valid_len(f_illegal_1.bits), 0);

	let f_illegal_2 = Flags::D | Flags::B; 
	assert_eq!(Header::valid_len(f_illegal_2.bits), 0);

	assert_eq!(Header::valid_len(0xFF), 0);
}

/// Packet always has valid header, it is garanteed in method new(). There is no way to create Packet with invalid header from Vec<u8>
pub struct Fragment {
	bytes: Vec<u8>,
	// ensure bytes has enough length to safely parse header, is based on bytes[0] value (i.e. flags)
	header_size: usize,
	header_hash: Box<Hash>,
	hash: Box<Hash>
}

impl Fragment {

	pub fn new(input: Vec<u8>) -> Option<Fragment> {
		if input.len() == 0 {
			return None;
		}
		// deduce header size from flags
		let hdr_size = Header::valid_len(input[0]);
		if hdr_size == 0 {
			// illegal header flags
			return None;
		}
		if input.len() < hdr_size {
			// packet is broken
			return None;
		}

		match Flags::from_bits(input[0]) {
			None => return None,
			Some(_) => ()
		};
		let hhash = Box::new(blake2s(&input[ .. hdr_size]));
		let phash = Box::new(blake2s(&input[ .. ]));

		Some(Fragment {
			bytes: input,
			header_size: hdr_size,
			header_hash: hhash,
			hash: phash
		})
	}

	pub fn flags(&self) -> Flags {
		Flags::from_bits(self.bytes[0]).unwrap()
	}

	pub fn fragment(&self) -> Option<(u16, u16)> {
		if self.flags().contains(Flags::F) {
			let number = u16::from_le_bytes(self.bytes[1..].try_into().unwrap());
			let count = u16::from_le_bytes(self.bytes[3..].try_into().unwrap());
			return Some((number, count));
		}
		None
	}

	pub fn id(&self) -> u64 {
		let mut pos = 1; // flags
		if self.flags().contains(Flags::F) {
			pos += 4; // number + count
		}
		u64::from_le_bytes(self.bytes[pos..].try_into().unwrap())
	}

	pub fn sender(&self) -> Option<&[u8]> {
		let f = self.flags();
		if f.contains(Flags::N) {
			return None; // does not contain sender key
		}
		let mut pos = 1 + 8; // flags + id
		if f.contains(Flags::F) {
			pos += 4; // number + count
		}
		Some(&self.bytes[pos .. pos + PUBLIC_KEY_SIZE])
	}

	pub fn target(&self) -> Option<&[u8]> {
		let f = self.flags();
		if f.contains(Flags::B) || f.contains(Flags::D) {
			return None; // does not contain target key
		}
		let mut pos = 1 + 8; // flags + id
		if f.contains(Flags::F) {
			pos += 4; // number + count
		}
		if !f.contains(Flags::N) {
			pos += PUBLIC_KEY_SIZE; // sender
		}
		Some(&self.bytes[pos .. pos + PUBLIC_KEY_SIZE])
	}

	pub fn payload(&self) -> Option<&[u8]> {
		if self.bytes.len() > self.header_size {
			return Some(&self.bytes[self.header_size .. ]);
		}
		None
	}
}
