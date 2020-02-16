use std::sync::mpsc::Sender;
use std::io::Write;

use log::{info, warn};

use super::config::SharedConfig;
use super::PublicKey;
use super::{NODE_VERSION, UUID_TESTNET};
use super::network::packet::{Flags, Packet};

extern crate bincode;
use bincode::serialize_into;

type Command = super::network::packet::NghbrCmd;

pub struct Collaboration {
    tx_send: Sender<Packet>
}

impl Collaboration {

    pub fn new(_conf: SharedConfig, tx_send: Sender<Packet>) -> Collaboration {
        Collaboration {
            tx_send: tx_send
        }
    }

    pub fn handle(&self, sender: &PublicKey, cmd: Command, bytes: Option<&[u8]>) {
        match cmd {
            Command::Error => self.handle_error(sender, bytes),
            Command::VersionRequest => self.handle_version_request(sender, bytes),
            Command::VersionReply => self.handle_version_reply(sender, bytes),
            Command::Ping => self.handle_ping(sender, bytes),
            Command::Pong => self.handle_pong(sender, bytes)
        };
    }

    fn handle_error(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {

    }

    fn handle_version_request(&self, sender: &PublicKey, _bytes: Option<&[u8]>) {
        // send version reply:
        let mut output: Vec<u8> = Vec::<u8>::new();
        match pack_version_reply(&mut output) {
            Err(_) => {
                warn!("Failed to serialize version reply");
            },
            Ok(_) => {
                match Packet::new(*sender, output) {
                    None => {
                        warn!("Failed create version reply packet");
                    },
                    Some(pack) => {
                        match self.tx_send.send(pack) {
                            Err(e) => {
                                warn!("Failed send version reply packet: {}", e);
                            },
                            Ok(_) => {
                                info!("-> version reply packet");
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_version_reply(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {
        
    }

    fn handle_ping(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {
        
    }

    fn handle_pong(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {
        
    }
}

fn pack_version_reply(output: &mut Vec<u8>) -> bincode::Result<()> {
    let cmd_len = 1 + 1 + 2 + 8 + 8 + 8;
    output.reserve(cmd_len);
    
    let flags = Flags::N;        
    let cmd = Command::VersionReply as u8;

    serialize_into(output.by_ref(), &flags.bits())?;
    serialize_into(output.by_ref(), &cmd)?;
    serialize_into(output.by_ref(), &NODE_VERSION)?;
    serialize_into(output.by_ref(), &UUID_TESTNET)?;
    
    let last_seq: u64 = 0;
    serialize_into(output.by_ref(), &last_seq)?;
    
    let cur_round: u64 = 0; 
    serialize_into(output.by_ref(), &cur_round)?;

    assert_eq!(cmd_len, output.len());

    Ok(())
}
