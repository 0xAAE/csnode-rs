use super::try_parse;
use std::collections::HashMap;

pub struct Data {
    send_cache_delay: usize,
    max_packet_resends: usize,
    packet_ttl: usize,
}

impl Data {
    pub fn new() -> Data {
        let default_packet_ttl = 10;

        Data {
            send_cache_delay: 10,
            packet_ttl: default_packet_ttl,
            // 1 <= ttl <= 10
            max_packet_resends: std::cmp::max(1, std::cmp::min(default_packet_ttl / 2, 10)),
        }
    }

    pub fn update(&mut self, prop: &HashMap<String, String>) -> bool {
        let mut updated = false;
        for (k, v) in prop.iter() {
            match k.as_str() {
                "send_cache_value" => {
                    updated = try_parse(&mut self.send_cache_delay, k, v) || updated;
                }
                "max_resends_send_cache" => {
                    updated = try_parse(&mut self.max_packet_resends, k, v) || updated;
                }
                "max_packet_life_time" => {
                    updated = try_parse(&mut self.packet_ttl, k, v) || updated;
                }
                _ => (),
            }
        }
        updated
    }
}
