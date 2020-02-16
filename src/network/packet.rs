use super::super::{PublicKey, PUBLIC_KEY_SIZE};
//use super::super::bitflags;
//use super::super::blake2s_simd::Hash;
use std::convert::{TryInto, TryFrom};
use std::fmt;

extern crate csp2p_rs;
use csp2p_rs::{NodeId, NODE_ID_SIZE};

extern crate num_enum;
use num_enum::TryFromPrimitive;

bitflags! {
	pub struct Flags: u8 {
		const ZERO = 0;
		// neighbour command
		const N = 0b0000_0001;
		// message & Flags::M)
		const M = 0b0000_0010;
		// signed
		const S = 0b0000_0100;

		// signed neighbour command
		const NS = Self::N.bits | Self::S.bits;
		// signed message
		const MS = Self::M.bits | Self::S.bits;
	}
}

#[test]
fn test_bitflags() {
	let h1 = Flags::N | Flags::S;
    let h2 = Flags::M | Flags::S;
    assert_eq!((h1 | h2), Flags::N | Flags::S); // union
	assert_eq!((h1 & h2), Flags::S); // intersection
	let h3 = Flags::N | Flags::M | Flags::S;
    assert_eq!((h3 - h1), Flags::M); // set difference
	assert_eq!(!h2, Flags::N); // set complement
	assert_eq!(h1, Flags::NS);
	assert_eq!(h2, Flags::MS);
}

// copy of c++ enum
#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum MsgType {
    BootstrapTable,
    Transactions,
    FirstTransaction,
    NewBlock,
    BlockHash,
    BlockRequest,
    RequestedBlock,
    FirstStage,
    SecondStage,
    ThirdStage,
    FirstStageRequest,
    SecondStageRequest,
    ThirdStageRequest,
    RoundTableRequest,
    RoundTableReply,
    TransactionPacket,
    TransactionsPacketRequest,
    TransactionsPacketReply,
    NewCharacteristic,
    WriterNotification,
    FirstSmartStage,
    SecondSmartStage,
    RoundTable = 22,
    ThirdSmartStage,
    SmartFirstStageRequest,
    SmartSecondStageRequest,
    SmartThirdStageRequest,
    HashReply,
    RejectedContracts,
    RoundPackRequest,
    StateRequest,
    StateReply,
    Utility,
    EmptyRoundPack,
    BlockAlarm,
    EventReport,
    NodeStopRequest = 255
}

impl fmt::Display for MsgType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //write!(f, "{:?}", self)
        // or, alternatively:
        fmt::Debug::fmt(self, f)
    }
}

pub struct Packet {
	sender: Option<Box<PublicKey>>,
	data: Vec<u8>
}

impl Packet {

	pub fn new(id: NodeId, bytes: Vec<u8>) -> Option<Packet> {
		if bytes.is_empty() {
			return None;
		}
		let sender: Option<Box<PublicKey>>;
		if NODE_ID_SIZE == PUBLIC_KEY_SIZE {
			sender = Some(Box::new(id));
		}
		else {
			sender = None;
		}
		Some(Packet {
			sender: sender,
			data: bytes
		})
	}

	pub fn is_message(&self) -> bool {
		if self.data.len() == 0 {
			return false;
		}
		check_flag(self.data[0], Flags::M)
	}

	pub fn is_neigbour(&self) -> bool {
		if self.data.len() == 0 {
			return false;
		}
		check_flag(self.data[0], Flags::N)
	}

	pub fn is_signed(&self) -> bool {
		if self.data.len() == 0 {
			return false;
		}
		check_flag(self.data[0], Flags::S)
	}

	pub fn msg_type(&self) -> Option<MsgType> {
		if self.data.len() < 2 {
			return None;
		}
		if !self.is_message() {
			return None;
		}
		match MsgType::try_from(self.data[1]) {
			Err(_) => None,
			Ok(m) => Some(m)
		}
	}

	// round() -> usize
	pub fn round(&self) -> Option<u64> {
		if !self.is_message() {
			return None;
		}
		if self.data.len() < 10 {
			return None;
		}
		Some(u64::from_le_bytes(self.data[2..].try_into().unwrap()))
	}

	pub fn payload(&self) -> Option<&[u8]> {
		if self.is_neigbour() {
			// neigbour pack: 1 byte + payload
			if self.data.len() < 2 {
				return None;
			}
			return Some(&self.data[1..]);
		}
		// message pack: 1 byte + 1 byte + 8 bytes + payload 
		if self.data.len() < 11 {
			return None;
		}
		Some(&self.data[10..])
	}
}

fn check_flag(byte: u8, f: Flags) -> bool {
	match Flags::from_bits(byte) {
		None => false,
		Some(f) => f.contains(f)
	}
}
