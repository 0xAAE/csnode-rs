use std::sync::mpsc::Sender;
use std::io::Write;
use std::collections::HashMap;

use log::{debug, info, warn, error};

use super::config::SharedConfig;
use super::PublicKey;
use super::{NODE_VERSION, UUID_TESTNET};
use super::network::packet::{Flags, Packet};

extern crate bincode;
use bincode::{serialize_into, deserialize_from};
extern crate base58;
use base58::ToBase58; // [u8].to_base58()

type Command = super::network::packet::NghbrCmd;

#[derive(Default)]
struct PeerInfo {
    /// build numbder
    version: u16,
    /// blockchain UUID
    uuid: u64,
    /// tlast reported max stored block sequence
    sequence: u64,
    /// the last repported consensus round
    round: u64,
    /// requires to be persistent
    persistent: bool
}

pub struct Collaboration {
    tx_send: Sender<Packet>,
    sequence: u64,
    round: u64,
    neighbours: HashMap<PublicKey, PeerInfo>,
    config: SharedConfig
}

impl Collaboration {

    pub fn new(conf: SharedConfig, tx_send: Sender<Packet>) -> Collaboration {
        Collaboration {
            tx_send: tx_send,
            sequence: 0,
            round: 0,
            neighbours: HashMap::<PublicKey, PeerInfo>::new(),
            config: conf
        }
    }

    pub fn handle(&mut self, sender: &PublicKey, cmd: Command, bytes: Option<&[u8]>) {
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

    fn handle_version_reply(&mut self, sender: &PublicKey, bytes: Option<&[u8]>) {
        // try to unpack peer info
        if bytes.is_none() {
            warn!("malformed version reply: no payload");
            return;
        }
        let req_len = 2 + 8 + 8 + 8; // ver + uuid + seq + round
        let input = bytes.unwrap();
        if input.len() < req_len {
            warn!("inconsistent version reply payload, required {} bytes, actual {}", req_len, input.len());
            return;
        }
        /*
            PeerInfo info;
            ...
            tryToAddNew(sender, info);
        */
        match unpack_peer_info(input) {
            Err(e) => {
                warn!("failed to unpack remote peer info: {}", e);
                return;
            }
            Ok(peer_info) => {
                if !self.try_add_peer(sender, peer_info) {
                    debug!("new peer info rejected");
                }
                else {
                    info!("add new neighbour, now total {}", self.neighbours.len());
                }
            }
        };
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
                                debug!("transfer version request packet");
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_node_lost(&self, _node_id: &PublicKey) {
    }

    fn try_add_peer(&mut self, key: &PublicKey, peer_info: PeerInfo) -> bool {

        if self.neighbours.contains_key(key) {

            let max_neighbours;
            let mut min_version = NODE_VERSION;
            {
                let conf_guard = self.config.read().unwrap();
                max_neighbours = conf_guard.max_neighbours;
                let tmp = conf_guard.min_compatible_version;
                if tmp > 0 {
                    min_version = tmp as u16;
                }
            }

            // test compatibility
            if peer_info.version < min_version {
                debug!("peer version is incompatible: {} < {}, reject", peer_info.version, NODE_VERSION);
                return false;
            }
            if peer_info.uuid != UUID_TESTNET {
                debug!("peer blockchain is incompatible, reject");
                return false;
            }

            // test selfability
    
            if self.neighbours.len() >= max_neighbours {
                debug!("max allowed neighbors {} has reached, reject", max_neighbours);
                return false;
            }
        }

        match self.neighbours.insert(*key, peer_info) {
            None => (),
            Some(_old_info) => {
                info!("already known peer {} has found again", key[..].to_base58());
                // if peer_info.round != old_info.round || peer_info.sequence != old_info.sequence {
                //     debug!("peer updated")
                // }
            }
        };

        true
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

fn unpack_peer_info(input: &[u8]) -> bincode::Result<PeerInfo> {
    /*
        PeerInfo info;
        cs::IDataStream stream(pack.getMsgData(), pack.getMsgSize());
        stream >> info.nodeVersion;
        stream >> info.uuid;
        stream >> info.lastSeq;
        stream >> info.roundNumber;
        info.permanent = isPermanent(sender);
    */
    let mut peer_info: PeerInfo = Default::default();

    peer_info.version = deserialize_from(input)?;
    peer_info.uuid = deserialize_from(input)?;
    peer_info.sequence = deserialize_from(input)?;
    peer_info.round = deserialize_from(input)?;

    Ok(peer_info)
}
