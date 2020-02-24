use std::sync::mpsc::Sender;
use std::io::Write;
use std::collections::HashMap;
use std::sync::RwLock;
use std::mem::size_of_val;
use std::iter::IntoIterator;

use log::{debug, info, warn, error};

use super::config::SharedConfig;
use super::PublicKey;
use super::{NODE_VERSION, UUID_TESTNET};
use super::network::packet::{Flags, Packet};
use super::blockchain::SharedBlocks;
use super::core_logic::SharedRound;

extern crate bincode;
use bincode::{serialize_into, deserialize_from};
extern crate base58;
use base58::ToBase58; // [u8].to_base58()

type Command = super::network::packet::NghbrCmd;
type Message = super::network::packet::MsgType;

mod block_sync;
use block_sync::BlockSync;

#[derive(Default)]
struct PeerInfo {
    /// build numbder
    version: u16,
    /// blockchain UUID
    uuid: u64,
    /// tlast reported max stored block sequence
    pub sequence: u64,
    /// the last repported consensus round
    round: u64,
    /// requires to be persistent
    persistent: bool
}

pub struct Collaboration {
    tx_send: Sender<Packet>,
    sequence: u64,
    neighbours: RwLock<HashMap<PublicKey, PeerInfo>>,
    config: SharedConfig,
    blocks: SharedBlocks,
    round: SharedRound,
    sync: BlockSync
}

impl Collaboration {

    pub fn new(conf: SharedConfig, tx_send: Sender<Packet>, blocks: SharedBlocks, round: SharedRound) -> Collaboration {
        Collaboration {
            tx_send: tx_send,
            sequence: 0,
            neighbours: RwLock::new(HashMap::<PublicKey, PeerInfo>::new()),
            config: conf,
            blocks: blocks,
            round: round,
            sync: BlockSync::new()
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

    pub fn ping_all(&self) {
        // send ping packet to all neigbours
        let all = self.neighbours.read().unwrap();
        for item in all.keys() {
            let mut output: Vec<u8> = Vec::<u8>::new();
            match pack_ping(&mut output) {
                Err(_) => {
                    error!("failed to serialize ping info");
                },
                Ok(_) => {
                    match Packet::new(*item, output) {
                        None => {
                            error!("failed create ping packet");
                        },
                        Some(pack) => {
                            match self.tx_send.send(pack) {
                                Err(e) => {
                                    warn!("failed send ping packet to {}: {}", item.to_base58(), e);
                                },
                                Ok(_) => {
                                    debug!("transfer ping packet to {}", item.to_base58());
                                }
                            }
                        }
                    }
                }
            }
        }
        // test if block sync required
        let current_round;
        {
            current_round = self.round.read().unwrap().current();
        }
        let blocks_top;
        {
            blocks_top = self.blocks.read().unwrap().top();
        }
        if current_round > blocks_top && current_round - blocks_top > 1 {
            let max_allowed_request;
            {
                max_allowed_request = self.config.read().unwrap().sync.max_block_request as u64;
            }
            let begin = blocks_top + 1;
            let end = begin + std::cmp::min(begin + max_allowed_request + 1, current_round);
            let mut bytes = Vec::<u8>::new();
            serialize_into(&mut bytes, &0u8).unwrap();                              // no flags
            serialize_into(&mut bytes, &(Message::BlockRequest as u8)).unwrap();    // message
            serialize_into(&mut bytes, &current_round).unwrap();                    // round
            for s in begin..end {
                serialize_into(&mut bytes, &s).unwrap();                            // requested blocks sequences
            }

            for (node_id, peer) in all.iter() {
                if peer.sequence >= end {
                    match Packet::new(*node_id, bytes) {
                        None => {
                            error!("failed create packet to request blocks");
                        },
                        Some(pack) => {
                            match self.tx_send.send(pack) {
                                Err(e) => {
                                    warn!("failed transfer request for blocks to {}: {}", node_id.to_base58(), e);
                                },
                                Ok(_) => {
                                    debug!("transfer request for blocks to {}", node_id.to_base58());
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    fn handle_error(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {

    }

    fn handle_version_request(&self, sender: &PublicKey, _bytes: Option<&[u8]>) {
        let current_round;
        {
            current_round = self.round.read().unwrap().current();
        }
        // send version reply:
        let mut output: Vec<u8> = Vec::<u8>::new();
        match pack_version_reply(&mut output, self.sequence, current_round) {
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
            },
            Ok(peer_info) => {
                if !self.try_add_peer(sender, peer_info) {
                    debug!("new peer info rejected");
                }
                else {
                    let guard = self.neighbours.read().unwrap();
                    info!("add new neighbour, now total {}", guard.len());
                }
            }
        };
    }

    fn handle_ping(&self, sender: &PublicKey, _bytes: Option<&[u8]>) {
        let current_round;
        {
            current_round = self.round.read().unwrap().current();
        }
        // send pong:
        let mut output: Vec<u8> = Vec::<u8>::new();
        match pack_pong(&mut output, self.sequence, current_round) {
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

    fn handle_pong(&mut self, sender: &PublicKey, bytes: Option<&[u8]>) {
        if bytes.is_none() {
            warn!("malformed pong packet, no payload attached");
            return;
        }
        let req_len = 8 + 8; // sequence + round
        let input = bytes.unwrap();
        if input.len() < req_len {
            warn!("inconsistent pong payload, required {} bytes, actual {}", req_len, input.len());
            return;
        }
        match unpack_peer_update(input) {
            Err(e) => {
                warn!("faile to unpack remote peer update: {}", e);
                return;
            }
            Ok(data) => {
                if !self.try_update_peer(sender, &data) {
                    debug!("{} is not updated", sender.to_base58());
                }
                else {
                    let s: String;
                    if data.1 >= data.0 {
                        s = format!("+{}", &data.1 - &data.0);
                    }
                    else {
                        s = format!("-{}", &data.0 - &data.1);
                    }
                    debug!("{}: S {}, R {}, {}", sender.to_base58(), data.0, data.1, s);
                }
            }
        }
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

    fn handle_node_lost(&mut self, node_id: &PublicKey) {
        {
            let guard = self.neighbours.read().unwrap();
            if !guard.contains_key(node_id) {
                return;
            }
        }
        let lost_peer;
        {
            let mut guard = self.neighbours.write().unwrap();
            lost_peer = guard.remove_entry(node_id);
        }
        match lost_peer {
            None => (),
            Some(item) => {
                if item.1.persistent {
                    // send version request
                    self.handle_node_found(&item.0);
                }
            }
        }
    }

    fn try_update_peer(&mut self, key: &PublicKey, data: &(u64, u64)) -> bool {
        {
            let guard = self.neighbours.read().unwrap();
            if ! guard.contains_key(key) {
                return false;
            }
        }

        let mut guard = self.neighbours.write().unwrap();
        let info = guard.get_mut(key).unwrap();
        let mut updated = false;
        if info.sequence < data.0 {
            info.sequence = data.0;
            updated = true;
        }
        if info.round < data.1 {
            info.round = data.1;
            updated = true;
        }
        updated
    }

    fn try_add_peer(&mut self, key: &PublicKey, peer_info: PeerInfo) -> bool {

        let exists: bool;
        let count: usize;
        {
            let guard = self.neighbours.read().unwrap();
            exists = guard.contains_key(key);
            count = guard.len();
        }

        if exists {
            // read config
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
    
            if count >= max_neighbours {
                debug!("max allowed neighbors {} has reached, reject", max_neighbours);
                return false;
            }
        }

        let mut guard = self.neighbours.write().unwrap();
        match guard.insert(*key, peer_info) {
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

#[test]
fn sequencial_serialization() {
    let mut output = Vec::<u8>::new();
    let cmd_len = 1 + 1; // flags + cmd
    output.reserve(cmd_len);
    assert_eq!(serialize_into(output.by_ref(), &Flags::N.bits()).unwrap(),());
    assert_eq!(serialize_into(output.by_ref(), &(Command::VersionRequest as u8)).unwrap(), ());
    let data = [1u8, 2u8, 3u8, 4u8, 5u8];
    for d in &data {
        assert_eq!(serialize_into(output.by_ref(), d).unwrap(), ());
    }
    assert_eq!(output.len(), 7);
}

#[test]
fn sequencial_deserialization() {
    let data = [1u8, 2u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 16u8, 0u8, 0u8, 0u8];
    let input = &data[..];

    let mut pos = 0;
    let v1: u8 = deserialize_from(input).unwrap();
    assert_eq!(v1, 1u8);
    pos += size_of_val(&v1);
    let v2: u8 = deserialize_from(&input[pos..]).unwrap();
    assert_eq!(v2, 2u8);
    pos += size_of_val(&v2);
    let v255: u64 = deserialize_from(&input[pos..]).unwrap();
    assert_eq!(v255, 255);
    pos += size_of_val(&v255);
    let v16: u32 = deserialize_from(&input[pos..]).unwrap();
    assert_eq!(v16, 16);
}

fn pack_version_request(output: &mut Vec<u8>) -> bincode::Result<()> {
    let cmd_len = 1 + 1; // flags + cmd
    output.reserve(cmd_len);
    serialize_into(output.by_ref(), &Flags::N.bits())?;
    serialize_into(output.by_ref(), &(Command::VersionRequest as u8))?;

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
    let mut p = size_of_val(&peer_info.version);
    peer_info.uuid = deserialize_from(&input[p..])?;
    p += size_of_val(&peer_info.uuid);
    peer_info.sequence = deserialize_from(&input[p..])?;
    p += size_of_val(&peer_info.sequence);
    peer_info.round = deserialize_from(&input[p..])?;
    p += size_of_val(&peer_info.round);
    
    assert_eq!(p, input.len());
    Ok(peer_info)
}

fn unpack_peer_update(input: &[u8]) -> bincode::Result<(u64, u64)> {
    /*
        PeerInfo& info = neighbour->second;
        cs::IDataStream stream(pack.getMsgData(), pack.getMsgSize());
        stream >> info.lastSeq;
        stream >> info.roundNumber;
    */
    let seq: u64 = deserialize_from(input)?;
    let mut p = size_of_val(&seq);
    let round: u64 = deserialize_from(&input[p..])?;
    p += size_of_val(&round);

    assert_eq!(p, input.len());
    Ok((seq, round))
}
