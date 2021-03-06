use std::sync::mpsc::Sender;

use log::{debug, info};

use super::config::SharedConfig;
use super::PublicKey;
use super::network::packet::{Packet, MsgType};

mod round;
use round::Round;

pub struct CoreLogic {
    tx_send: Sender<Packet>,
    config: SharedConfig,
    round: Round
}

impl CoreLogic {
    pub fn new(conf: SharedConfig, tx_send: Sender<Packet>) -> CoreLogic {
        CoreLogic {
            tx_send: tx_send,
            config: conf,
            round: Round::new()
        }
    }

    pub fn handle(&mut self, sender: &PublicKey, msg: MsgType, rnd: u64, bytes: Option<&[u8]>) {
        if !self.test_packet_round(rnd, &msg) {
            return;
        }
        match msg {
            MsgType::BootstrapTable => self.handle_bootstrap_table(sender, rnd, bytes),
            // MsgType::Transactions,
            // MsgType::FirstTransaction,
            // MsgType::NewBlock,
            // MsgType::BlockHash,
            // MsgType::BlockRequest,
            // MsgType::RequestedBlock,
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
            // MsgType::NewCharacteristic,
            // MsgType::WriterNotification,
            // MsgType::FirstSmartStage,
            // MsgType::SecondSmartStage,
            MsgType::RoundTable => self.handle_round_table(sender, rnd, bytes),
            // MsgType::ThirdSmartStage,
            // MsgType::SmartFirstStageRequest,
            // MsgType::SmartSecondStageRequest,
            // MsgType::SmartThirdStageRequest,
            // MsgType::HashReply,
            // MsgType::RejectedContracts,
            // MsgType::RoundPackRequest,
            // MsgType::StateRequest,
            // MsgType::StateReply,
            // MsgType::Utility,
            // MsgType::EmptyRoundPack,
            // MsgType::BlockAlarm,
            // MsgType::EventReport,
            MsgType::NodeStopRequest => self.handle_stop_request(sender, rnd, bytes),
            _ => debug!("{} handler is not implemented yet", msg.to_string())
        }
    }

    fn test_packet_round(&self, rnd: u64, msg: &MsgType) -> bool {
        let cur = self.round.current();
        match msg {
            MsgType::BootstrapTable => { rnd >= cur },
            _ => rnd > cur
        }
    }

    fn handle_bootstrap_table(&self, _sender: &PublicKey, _rnd: u64, _bytes: Option<&[u8]>) {

    }

    fn handle_transaction_packet(&self, _sender: &PublicKey, _rnd: u64, _bytes: Option<&[u8]>) {
        
    }

    fn handle_round_table(&mut self, _sender: &PublicKey, rnd: u64, bytes: Option<&[u8]>) {
        if !self.round.handle_table(rnd, bytes) {
            info!("failed to handle round table")
        }
    }

    fn handle_stop_request(&self, _sender: &PublicKey, _rnd: u64, _bytes: Option<&[u8]>) {
        
    }
}
