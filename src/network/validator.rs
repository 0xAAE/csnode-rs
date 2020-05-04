use super::packet::Packet;

pub struct Validator {

}

impl Validator {

    pub fn new() -> Validator {
        Validator {
        }
    }

    pub fn validate(&self, packet: &Packet) -> bool {
        // wellformed message has to contain round
        packet.is_neigbour() || packet.round().is_some()
    }
}