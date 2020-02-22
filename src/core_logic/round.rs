use std::time::Instant;
use log::{info};

use num_format::{Locale, ToFormattedString};

pub struct Round {
    // the first round after start
    first: u64,
    // time point the first round started
    first_start: Instant,
    // current round
    current: u64,
    // time point current round started  
    current_start: Instant,
    // average round duration
    ave_duration: u64
}

impl Round {

    pub fn new() -> Round {
        let now = Instant::now();
        Round {
            first: 0,
            first_start: now,
            current: 0,
            current_start: now,
            ave_duration: 0
        }
    }

    pub fn current(&self) -> u64 {
        self.current
    }

    pub fn handle_table(&mut self, rnd: u64, _bytes: Option<&[u8]>) -> bool {
        if self.first == 0 {
            self.first = rnd;
        }
        self.current = rnd;
        let uptime_ms = self.first_start.elapsed().as_millis() as u64;
        let current_duration = self.current_start.elapsed();
        self.current_start = Instant::now();
        if self.current > self.first {
            self.ave_duration = uptime_ms / (self.current - self.first);
        }


        
        info!("-------------------------- R: {} --------------------------", rnd.to_formatted_string(&Locale::ru));
        info!("last round: {} ms, ave: {} ms, uptime: {}", current_duration.as_millis(), self.ave_duration, format_ms(uptime_ms));
        true
    }
}

fn format_ms(value: u64) -> String {
    let mut uptime = value / 1000;
    let ss = uptime % 60;
    uptime /= 60;
    let mm = uptime % 60;
    uptime /= 60;
    let hh = uptime % 24;
    uptime /= 24;
    format!("{}d {:02}:{:02}:{:02}", uptime, hh, mm, ss)
}

#[test]
fn test_format_ms() {
    let value = (((2 * 24 + 17) * 60 + 33) * 60 + 29) * 1000 + 345;
    let ms = value % 1000;
    let mut uptime = value / 1000;
    let ss = uptime % 60;
    uptime /= 60;
    let mm = uptime % 60;
    uptime /= 60;
    let hh = uptime % 24;
    uptime /= 24;
    assert_eq!(uptime, 2);
    assert_eq!(hh, 17);
    assert_eq!(mm, 33);
    assert_eq!(ss, 29);
    assert_eq!(ms, 345);
}
