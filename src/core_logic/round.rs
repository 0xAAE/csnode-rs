use log::{info};

pub struct Round {
    current: u64
}

impl Round {

    pub fn new() -> Round {
        Round {
            current: 0
        }
    }

    pub fn current(&self) -> u64 {
        self.current
    }

    pub fn handle_table(&mut self, rnd: u64, _bytes: Option<&[u8]>) -> bool {
        self.current = rnd;
        info!("-------------------------- R-{} --------------------------", rnd);
        true
    }
}