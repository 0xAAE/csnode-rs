use std::sync::mpsc::Sender;
use std::io::Write;

use log::{debug, warn, error};

use super::config::SharedConfig;
use super::PublicKey;
use super::{NODE_VERSION, UUID_TESTNET};
use super::network::packet::{Flags, Packet};

extern crate bincode;
use bincode::serialize_into;

type Command = super::network::packet::NghbrCmd;

pub struct Collaboration {
    tx_send: Sender<Packet>,
    sequence: u64,
    round: u64
}

impl Collaboration {

    pub fn new(_conf: SharedConfig, tx_send: Sender<Packet>) -> Collaboration {
        Collaboration {
            tx_send: tx_send,
            sequence: 0,
            round: 0
        }
    }

    pub fn handle(&self, sender: &PublicKey, cmd: Command, bytes: Option<&[u8]>) {
        match cmd {
            Command::Error => self.handle_error(sender, bytes),
            Command::VersionRequest => self.handle_version_request(sender, bytes),
            Command::VersionReply => self.handle_version_reply(sender, bytes),
            Command::Ping => self.handle_ping(sender, bytes),
            Command::Pong => self.handle_pong(sender, bytes),
            Command::NodeFound => self.handle_node_found(sender),
            Command::NodeLost => self.handle_node_lost(sender)
        };
    }

    fn handle_error(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {

    }

    fn handle_version_request(&self, sender: &PublicKey, _bytes: Option<&[u8]>) {
        // send version reply:
        let mut output: Vec<u8> = Vec::<u8>::new();
        match pack_version_reply(&mut output, self.sequence, self.round) {
            Err(_) => {
                error!("failed to serialize version reply");
            },
            Ok(_) => {
                match Packet::new(*sender, output) {
                    None => {
                        error!("failed create version reply packet");
                    },
                    Some(pack) => {
                        match self.tx_send.send(pack) {
                            Err(e) => {
                                warn!("failed send version reply packet: {}", e);
                            },
                            Ok(_) => {
                                debug!("transfer version reply packet");
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_version_reply(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {
        
    }

    fn handle_ping(&self, sender: &PublicKey, _bytes: Option<&[u8]>) {
        // send pong:
        let mut output: Vec<u8> = Vec::<u8>::new();
        match pack_pong(&mut output, self.sequence, self.round) {
            Err(_) => {
                error!("failed to serialize pong");
            },
            Ok(_) => {
                match Packet::new(*sender, output) {
                    None => {
                        error!("failed create pong packet");
                    },
                    Some(pack) => {
                        match self.tx_send.send(pack) {
                            Err(e) => {
                                warn!("failed send pong packet: {}", e);
                            },
                            Ok(_) => {
                                debug!("transfer pong packet");
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_pong(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {
        
    }

    fn handle_node_found(&self, node_id: &PublicKey) {
        // send version request:
        let mut output: Vec<u8> = Vec::<u8>::new();
        match pack_version_request(&mut output) {
            Err(_) => {
                error!("failed to serialize version request");
            },
            Ok(_) => {
                match Packet::new(*node_id, output) {
                    None => {
                        error!("failed create version request packet");
                    },
                    Some(pack) => {
                        match self.tx_send.send(pack) {
                            Err(e) => {
                                warn!("failed send version request packet: {}", e);
                            },
                            Ok(_) => {
                                debug!("create version request packet");
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_node_lost(&self, _node_id: &PublicKey) {
    }
}

fn pack_version_request(output: &mut Vec<u8>) -> bincode::Result<()> {
    let cmd_len = 1 + 1; // flags + cmd
    output.reserve(cmd_len);
    serialize_into(output.by_ref(), &Flags::N.bits())?;
    serialize_into(output.by_ref(), &(Command::VersionReply as u8))?;

    Ok(())
}

fn pack_version_reply(output: &mut Vec<u8>, sequence: u64, round: u64) -> bincode::Result<()> {
    let cmd_len = 1 + 1 + 2 + 8 + 8 + 8; // flags + cmd + version + uuid + sequence + round
    output.reserve(cmd_len);

    serialize_into(output.by_ref(), &Flags::N.bits())?;
    serialize_into(output.by_ref(), &(Command::VersionReply as u8))?;
    serialize_into(output.by_ref(), &NODE_VERSION)?;
    serialize_into(output.by_ref(), &UUID_TESTNET)?;
    serialize_into(output.by_ref(), &sequence)?;
    serialize_into(output.by_ref(), &round)?;

    assert_eq!(cmd_len, output.len());

    Ok(())
}

fn pack_ping(output: &mut Vec<u8>) -> bincode::Result<()> {
    let cmd_len = 1 + 1; // flags + cmd
    output.reserve(cmd_len);
    serialize_into(output.by_ref(), &Flags::N.bits())?;
    serialize_into(output.by_ref(), &(Command::Ping as u8))?;

    Ok(())
}

fn pack_pong(output: &mut Vec<u8>, sequence: u64, round: u64) -> bincode::Result<()> {
    let cmd_len = 1 + 1 + 8 + 8; // flags + cmd + sequence + round
    output.reserve(cmd_len);

    serialize_into(output.by_ref(), &Flags::N.bits())?;
    serialize_into(output.by_ref(), &(Command::Pong as u8))?;
    serialize_into(output.by_ref(), &sequence)?;
    serialize_into(output.by_ref(), &round)?;

    Ok(())    
}
