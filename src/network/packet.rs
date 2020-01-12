#[macro_use]
extern crate bitflags;

use super::PublicKey;

bitflags! {
	struct Flags: u8 {
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
	}
}

pub struct Header {
	flags: u8,
	number: u16,
	count: u16,
	id: u64,
	sender: PublicKey,
	target: PublicKey
}

#[test]
fn test_bitflags() {
	let e1 = Flags::A | Flags::C;
    let e2 = Flags::B | Flags::C;
    assert_eq!((e1 | e2), Flags::ABC);   // union
    assert_eq!((e1 & e2), Flags::C);     // intersection
    assert_eq!((e1 - e2), Flags::A);     // set difference
    assert_eq!(!e2, Flags::A);           // set complement
}
