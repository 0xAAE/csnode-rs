use std::sync::mpsc::Sender;

use log::{debug, info};
use base58::ToBase58; // [u8].to_base58()
use bincode::deserialize_from;

use super::config::SharedConfig;
use super::PublicKey;
use super::network::packet::{Packet, MsgType};
use super::SharedBlocks;

mod round;
pub use round::SharedRound;
use round::Round;

use super::blockchain::raw_block::RawBlock;

pub struct CoreLogic {
    tx_send: Sender<Packet>,
    config: SharedConfig,
    round: SharedRound,
    blocks: SharedBlocks
}

impl CoreLogic {

    pub fn new_shared_round() -> SharedRound {
        Round::new_shared()
    }

    pub fn new(conf: SharedConfig, tx_send: Sender<Packet>, blocks: SharedBlocks, round: SharedRound) -> CoreLogic {
        CoreLogic {
            tx_send: tx_send,
            config: conf,
            round: round,
            blocks: blocks
        }
    }

    pub fn handle(&mut self, sender: &PublicKey, msg: MsgType, rnd: u64, bytes: Option<&[u8]>) {
        if !self.test_packet_round(rnd, &msg) {
            return;
        }
        match msg {
            MsgType::BootstrapTable => self.handle_bootstrap_table(sender, rnd, bytes),
            MsgType::Transactions => {
                info!("obsolete MsgType::Transactions received")
            },
            MsgType::FirstTransaction => {
                info!("obsolete MsgType::FirstTransaction received")
            },
            MsgType::NewBlock => {
                info!("obsolete MsgType::NewBlock received")
            },
            // MsgType::BlockHash,
            // MsgType::BlockRequest,
            MsgType::RequestedBlock => self.handle_requested_blocks(sender, bytes),
            // MsgType::FirstStage,
            // MsgType::SecondStage,
            // MsgType::ThirdStage,
            // MsgType::FirstStageRequest,
            // MsgType::SecondStageRequest,
            // MsgType::ThirdStageRequest,
            // MsgType::RoundTableRequest,
            // MsgType::RoundTableReply,
            MsgType::TransactionPacket => self.handle_transaction_packet(sender, rnd, bytes),
            // MsgType::TransactionsPacketRequest,
            // MsgType::TransactionsPacketReply,
            MsgType::NewCharacteristic => {
                info!("obsolete MsgType::NewCharacteristic received")
            },
            MsgType::WriterNotification => {
                info!("obsolete MsgType::WriterNotification received")
            },
            // MsgType::FirstSmartStage,
            // MsgType::SecondSmartStage,
            MsgType::RoundTable => self.handle_round_table(sender, rnd, bytes),
            // MsgType::ThirdSmartStage,
            // MsgType::SmartFirstStageRequest,ply
            // MsgType::EmptyRoundPack,
            // MsgType::BlockAlarm,
            // MsgType::EventReport,
            MsgType::NodeStopRequest => self.handle_stop_request(sender, rnd, bytes),
            _ => debug!("{} handler is not implemented yet", msg.to_string())
        }
    }

    fn test_packet_round(&self, rnd: u64, msg: &MsgType) -> bool {
        match msg {
            // some packets are allowed from any round number:
            MsgType::RoundTableRequest => true,
            MsgType::RoundTableReply => true,
            MsgType::TransactionPacket => true,
            MsgType::TransactionsPacketReply => true,
            MsgType::TransactionsPacketRequest => true,
            MsgType::BlockRequest => true,
            MsgType::RequestedBlock => true,
            MsgType::StateRequest => true,
            MsgType::StateReply => true,
            MsgType::EmptyRoundPack => true,
            MsgType::BlockAlarm => true,
            MsgType::EventReport => true,
            // most of packets are allowed only from current or outrunning round:
            _ => {
                rnd >= self.round.read().unwrap().current()
            }
        }
    }

    fn handle_bootstrap_table(&self, _sender: &PublicKey, _rnd: u64, _bytes: Option<&[u8]>) {

    }

    fn handle_transaction_packet(&self, _sender: &PublicKey, _rnd: u64, _bytes: Option<&[u8]>) {
        
    }

    fn handle_round_table(&mut self, _sender: &PublicKey, rnd: u64, bytes: Option<&[u8]>) {
        let mut guard = self.round.write().unwrap();
        if !guard.handle_table(rnd, bytes) {
            info!("failed to handle round table")
        }
    }

    fn handle_stop_request(&self, _sender: &PublicKey, _rnd: u64, _bytes: Option<&[u8]>) {
        
    }

    fn handle_requested_blocks(&self, sender: &PublicKey, bytes: Option<&[u8]>) {
        match bytes {
            None => {
                info!("get requested blocks from {}", sender.to_base58());
            }
            Some(data) => {
                let count: u64 = deserialize_from(data).unwrap();
                let mut input = data[8..].to_vec();
                for _ in 0..count {
                    match RawBlock::new(input) {
                        None => {
                            info!("failed to extract block from data");
                            return;
                        },
                        Some(result) => {
                            input = result.1;
                            let block = result.0;
                            info!("successfully parsed block {}", block.sequence().unwrap());
                        }
                    }
                }
            }
        }
    }
}
