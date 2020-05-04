use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use log::{debug, warn};

// network submodules
use super::TEST_STOP_DELAY_SEC;
use super::packet::{Packet, MsgType};
// top-level modules
use super::super::config::SharedConfig;
use super::super::core_logic::{CoreLogic, SharedRound};
use super::SharedBlocks;

pub struct MessageProcessor {
    rx_msg: Receiver<Packet>,
    tx_send: Sender<Packet>,
    logic: CoreLogic
}

impl MessageProcessor {

    pub fn new(conf: SharedConfig, rx_msg: Receiver<Packet>, tx_send:Sender<Packet>, blocks: SharedBlocks, round: SharedRound) -> MessageProcessor {
        MessageProcessor {
            rx_msg,
            tx_send: tx_send.clone(),
            logic: CoreLogic::new(conf, tx_send, blocks, round)
        }
    }

    pub fn recv(&mut self) {
		match self.rx_msg.recv_timeout(Duration::from_secs(TEST_STOP_DELAY_SEC)) {
			Err(_) => (),
			Ok(mut p) => {
                match p.msg_type() {
                    None => {
                        warn!("unknown message, drop");
                    },
                    Some(mt) => {
                       if mt == MsgType::RequestedBlock {
                            // actually only particular packets use compression despite of
                            // Flags::Compressed value
                            if p.is_compressed() {
                                p = p.decompress();
                            }        
                        }

                        match p.round() {
                            None => {
                                warn!("malformed message, round not set, drop");
                            }
                            Some(r) => {
                                match p.address() {
                                    None => {
                                        warn!("unknown sender, drop");
                                    }
                                    Some(s) => {
                                        debug!("msg::{}", mt.to_string());
                                        self.logic.handle(s, mt, r, p.payload());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

}
