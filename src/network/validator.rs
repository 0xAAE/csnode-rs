use super::packet::Packet;

pub struct Validator {

}

impl Validator {

    pub fn new() -> Validator {
        return Validator {

        }
    }

    pub fn validate(&self, _packet: &Packet) -> bool {
        // todo implement packet validation
        true
    }
}