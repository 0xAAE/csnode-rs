use super::config::SharedConfig;
use super::PublicKey;
use super::{NODE_VERSION, UUID_TESTNET};
use super::network::packet::Flags;

extern crate bincode;
use bincode::serialize_into;

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
            Command::VersionRequest => {
                match self.handle_version_request(sender, bytes) {
                    Ok(_) => (),
                    Err(_) => ()
                }
            },
            Command::VersionReply => self.handle_version_reply(sender, bytes),
            Command::Ping => self.handle_ping(sender, bytes),
            Command::Pong => self.handle_pong(sender, bytes)
        };
    }

    fn handle_error(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {

    }

    fn handle_version_request(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) -> bincode::Result<()> {
        // send version reply:
        let cmd_len = 1 + 1 + 2 + 8 + 8 + 8;

        let mut output: Vec<u8> = Vec::<u8>::new();
        output.reserve(cmd_len);
        
        let flags = Flags::N;        
        let cmd = Command::VersionReply as u8;

        serialize_into(&mut output, &flags.bits())?;
        serialize_into(&mut output, &cmd)?;
        serialize_into(&mut output, &NODE_VERSION)?;
        serialize_into(&mut output, &UUID_TESTNET)?;
        
        let last_seq: u64 = 0;
        serialize_into(&mut output, &last_seq)?;
        
        let cur_round: u64 = 0; 
        serialize_into(&mut output, &cur_round)?;

        assert_eq!(cmd_len, output.len());

        Ok(())
    }

    fn handle_version_reply(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {
        
    }

    fn handle_ping(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {
        
    }

    fn handle_pong(&self, _sender: &PublicKey, _bytes: Option<&[u8]>) {
        
    }
}
