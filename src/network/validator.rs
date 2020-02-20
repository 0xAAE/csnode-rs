use super::packet::Packet;

pub struct Validator {

}

impl Validator {

    pub fn new() -> Validator {
        return Validator {

        }
    }

    pub fn validate(&self, packet: &Packet) -> bool {
        if !packet.is_neigbour() {
            if packet.round().is_none() {
                // malformed message: does not contain round
                return false;
            }
        }
        true
    }
}