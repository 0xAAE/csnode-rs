use std::sync::mpsc::Sender;
use std::mem::size_of;

use log::{debug, info, warn};
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
            tx_send,
            config: conf,
            round,
            blocks
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
            MsgType::RoundTableRequest
            | MsgType::RoundTableReply
            | MsgType::TransactionPacket
            | MsgType::TransactionsPacketReply
            | MsgType::TransactionsPacketRequest
            | MsgType::BlockRequest
            | MsgType::RequestedBlock
            | MsgType::StateRequest
            | MsgType::StateReply
            | MsgType::EmptyRoundPack
            | MsgType::BlockAlarm
            | MsgType::EventReport => true,
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
                let mut input = data[size_of::<u64>()..].to_vec();
                let mut first = 0u64;
                let mut last = 0u64;
                let mut failed = Vec::<u64>::new();
                for i in 0..count {
                    match RawBlock::new_from_stream(input) {
                        None => {
                            info!("failed to extract block from data");
                            return;
                        },
                        Some(result) => {
                            input = result.1;
                            let block = result.0;
                            let seq = block.sequence().unwrap();
                            if i == 0 {
                                first = seq;
                            }
                            else if i + 1 == count {
                                last = seq;
                            }
                            let mut b = self.blocks.write().unwrap();
                            if !b.store(block) {
                                failed.push(seq);
                                warn!("failed to store block {} to blockchain", seq);

                            }
                        }
                    }
                }
                let cnt_ok = 1 + last - first - failed.len() as u64;
                if last > 0 {
                    if failed.is_empty() {
                        info!("stored {} blocks form {}..{}", cnt_ok, first, last);
                    }
                    else {
                        warn!("only {} blocks from {}..{} stred, {} failed", cnt_ok, first, last, failed.len());
                    }
                }
            }
        }
    }
}
