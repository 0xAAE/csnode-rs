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
		for (k, v) in prop.iter() {
			match k.as_str() {
				"port" => {
					match v.parse::<u16>() {
						Err(_) => {
							//println!("error in {} value: it must be one of {}", k, std::any::type_name::<u16>());
							if self.is_set {
								self.is_set = false;
								updated = true;
							}
						}
						Ok(val) => {
							if self.port != val {
								println!("{} is updated: {} -> {}", k, &self.port, &val);
								self.port = val;
								updated = true;
								self.is_set = self.port != 0;
							}
						}
					}
				}
				"ip" => {
					match v.parse::<IpAddr>() {
						Err(_) => {
							//println!("IP addres parse error"),
						}
						Ok(addr) => {
							if &self.ip != &addr {
								self.ip = addr;
								updated = true;
							}
						}
					};
				}
				_ => ()
			}
		}
		updated
	}
}
