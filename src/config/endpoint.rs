use super::try_parse;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub struct Data {
    pub is_set: bool, // = false;
    port: u16, // = 0;
    ip: std::net::IpAddr // {};
}

impl Data {

	pub fn new() -> Data {
		Data {
			is_set: false,
			port: 0,
			ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
		}
	}

	pub fn update(&mut self, prop: &HashMap<String, String>) -> bool {
		let mut updated = false;
		self.is_set = false;
		self.port = 0;
		for (k, v) in prop.iter() {
			match k.as_str() {
				"port" => {
					updated = updated || try_parse(&mut self.port, k, v);
					if self.port != 0 {
						self.is_set = true;
					}
				}
				"ip" => {
					match v.parse::<IpAddr>() {
						Err(_) => println!("IP addres parse error"),
						Ok(addr) => {
							if &self.ip != &addr {
								self.ip = addr;
								updated = true;
							}
						}
					};
				}
				_ => {
					println!("Ignore unknown parameter {}", k);
				}
			}
		}
		updated
	}
}
