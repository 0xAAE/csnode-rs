use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use log::{debug, warn};

// network submodules
use super::TEST_STOP_DELAY_SEC;
use super::packet::Packet;
// top-level modules
use super::super::config::SharedConfig;
use super::super::collaboration::Collaboration;
use super::super::SharedBlocks;
use super::SharedRound;

pub struct CommandProcessor {
    rx_cmd: Receiver<Packet>,
    collaboration: Collaboration
}

impl CommandProcessor {

    pub fn new(conf: SharedConfig, rx_cmd: Receiver<Packet>, tx_send: Sender<Packet>, blocks: SharedBlocks, round: SharedRound) -> CommandProcessor {
        CommandProcessor {
            rx_cmd: rx_cmd,
            collaboration: Collaboration::new(conf, tx_send, blocks, round)
        }
    }

    pub fn recv(&mut self) {
		match self.rx_cmd.recv_timeout(Duration::from_secs(TEST_STOP_DELAY_SEC)) {
			Err(_) => (),
			Ok(p) => {
                match p.nghbr_cmd() {
                    None => {
                        warn!("unknown command, drop");
                    },
                    Some(v) => {
                        match p.address() {
                            None => {
                                warn!("cmd::{} has no sender, drop", v.to_string());
                            }
                            Some(s) => {
                                debug!("cmd::{}", v.to_string());
                                self.collaboration.handle(s, v, p.payload());
                            }
                        }
                    }
                };
            }
        }
    }

    pub fn ping_all(&self) {
        self.collaboration.ping_all();
    }

}
