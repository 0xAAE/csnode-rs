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
		// compressed data
		const C = 0b0000_0010;
		// signed data
		const S = 0b0000_0100;

		// signed neighbour command
		const NS = Self::N.bits | Self::S.bits;
		// compressed neighbour command
		const NC = Self::N.bits | Self::C.bits;
		// comressed signed message
		const CS = Self::C.bits | Self::S.bits;
	}
}

#[test]
fn test_bitflags() {
	let h1 = Flags::N | Flags::S;
    let h2 = Flags::C | Flags::S;
    assert_eq!((h1 | h2), Flags::N | Flags::S | Flags::C); // union
	assert_eq!((h1 & h2), Flags::S); // intersection
	let h3 = Flags::N | Flags::C | Flags::S;
    assert_eq!((h3 - h1), Flags::C); // set difference
	assert_eq!(!h2, Flags::N); // set complement
	assert_eq!(h1, Flags::NS);
	assert_eq!(h2, Flags::CS);
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

// copy of c++ enum
#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
pub enum NghbrCmd {
    Error = 1,
    VersionRequest,
    VersionReply,
    Ping,
    Pong
}

impl fmt::Display for NghbrCmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
		! self.is_neigbour()
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

	pub fn is_compressed(&self) -> bool {
		if self.data.len() == 0 {
			return false;
		}
		check_flag(self.data[0], Flags::C)
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

	pub fn nghbr_cmd(&self) -> Option<NghbrCmd> {
		if self.data.len() < 2 {
			return None;
		}
		if ! self.is_neigbour() {
			return None;
		}
		match NghbrCmd::try_from(self.data[1]) {
			Err(_) => None,
			Ok(cmd) => Some(cmd)
		}
	}

	pub fn round(&self) -> Option<u64> {
		if !self.is_message() {
			return None;
		}
		if self.data.len() < 10 {
			return None;
		}
		Some(u64::from_le_bytes(self.data[2..10].try_into().unwrap()))
	}

	pub fn payload(&self) -> Option<&[u8]> {
		if self.is_neigbour() {
			// neigbour pack: flags(1) + cmd(1) + payload
			if self.data.len() < 3 {
				return None;
			}
			return Some(&self.data[2..]);
		}
		// message pack: flags(1) + msg(1) + round(8) + payload 
		if self.data.len() < 11 {
			return None;
		}
		Some(&self.data[10..])
	}

	pub fn sender(&self) -> Option<&PublicKey> {
		if self.sender.is_some() {
			Some(self.sender.as_ref().unwrap())
		}
		else {
			None
		}
	}
}

fn check_flag(byte: u8, f: Flags) -> bool {
	match Flags::from_bits(byte) {
		None => false,
		Some(flags) => flags.contains(f)
	}
}
