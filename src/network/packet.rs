use super::super::PublicKey;
use super::super::bitflags;

bitflags! {
	struct Flags: u8 {
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
	flags: u8,
	number: u16,
	count: u16,
	id: u64,
	sender: Box<PublicKey>,
	target: Box<PublicKey>
}

impl Header {
	pub fn new(bytes: &[u8]) -> Option<Header> {
		if bytes.len() == 0 {
			return None;
		}
		// deduce header size from flags
		let size = Header::valid_len(bytes[0]);
		if size == 0 {
			// illegal header value
			return None;
		}

		None
	}

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
