use super::config::SharedConfig;
use super::PublicKey;

type Command = super::network::packet::NghbrCmd;

pub struct Collaboration {
    
}

impl Collaboration {

    pub fn new(_conf: SharedConfig) -> Collaboration {
        Collaboration {}
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

    fn handle_version_request(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {
        // send version reply:
        /*
        formPacket(BaseFlags::NetworkMsg,
                    NetworkCommand::VersionReply,
                    NODE_VERSION,
                    node_->getBlockChain().uuid(),
                    node_->getBlockChain().getLastSeq(),
                    cs::Conveyer::instance().currentRoundNumber()),
                    receiver);        */
        
    }

    fn handle_version_reply(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {
        
    }

    fn handle_ping(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {
        
    }

    fn handle_pong(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {
        
    }
}
