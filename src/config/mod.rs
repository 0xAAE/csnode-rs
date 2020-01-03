use ini::Ini;
use std::fmt::Display;
use std::str::FromStr;
use std::collections::HashMap;

mod endpoint;
mod api;
mod events;
mod sync;
mod sql;
mod conveyer;

pub struct Config {
	// [params]
	// node_type: String, // obsolete
	hosts_filename: String,
	bootstrap_type: String,
	ipv6: bool,
	min_compatible_version: u32,
	backward_compatible: bool,
	min_neighbours: usize,
	max_neighbours: usize,
	//connection_bandwidth: usize, // obsolete
	pub reload_delay_sec: u32, // observer_wait_time
	restrict_neighbours: bool,
	broadcast_percent: u32,
	// [start_node]
	start_node: endpoint::Data,
	// [host_input]
	host_input: endpoint::Data,
	// [pool_sync]
	sync: sync::Data,
	// [api]
	api: api::Data,
	// [conveyer]
	conveyer: conveyer::Data,
	// [event_report]
	events: events::Data,
	// [dbsql]
	sql: sql::Data,
	// source file to read
	ini_file: String
}

impl Config {

	pub fn new(file_name: &str) -> Config {
		let mut instance = Config {
			//node_type: String::from("client"),
			hosts_filename: String::new(),
			bootstrap_type: String::from("start_node"),
			ipv6: false,
			min_compatible_version: 0,
			backward_compatible: false,
			min_neighbours: 5,
			max_neighbours: 8,
			restrict_neighbours: true,
			broadcast_percent: 100,
			reload_delay_sec: 10, //5 * 60,
			start_node: endpoint::Data::new(),
			host_input: endpoint::Data::new(),
			sync: sync::Data::new(),
			api: api::Data::new(),
			conveyer: conveyer::Data::new(),
			events: events::Data::new(),
			sql: sql::Data::new(),

			ini_file: file_name.to_string()
		};

		instance.reload();
		instance
	}

	pub fn reload(&mut self) {
		let ini = Ini::load_from_file(&self.ini_file).unwrap();
		for (sec, prop) in ini.iter() {
			match sec.as_ref().map(String::as_str) {
				Some("params") => {
					self.update(prop);
				}
				Some("start_node") => {
					self.start_node.update(prop);
				}
				Some("host_input") => {
					self.host_input.update(prop);
				}
				Some("api") => {
					self.api.update(prop);
				}
				Some("conveyer") => {
					self.conveyer.update(prop);
				}
				Some("pool_sync") => {
					self.sync.update(prop);
				}
				Some("event_report") => {
					self.events.update(prop);
				}
				Some("dbsql") => {
					self.sql.update(prop);
				}
				//Some("Core") => {},
				//Some("Sinks.Console") => {}
				//Some("Sinks.File") => {}
				//Some("Sinks.Event") => {}
				Some(val) => {
					println!("ignore {} section", val);
				},
				None => {}
			};
		}
	}

	fn update(&mut self, prop: &HashMap<String, String>) -> bool {
		let mut updated = false;
		for (k, v) in prop.iter() {
			match k.as_str() {
				"node_type" => {
					//println!("node_type is obsolete and ignored");
				}
				"hosts_filename" => {
					updated = updated || try_update(&mut self.hosts_filename, k, v);
				}
				"bootstrap_type" => {
					updated = updated || try_update(&mut self.bootstrap_type, k, v);
				}
				"ipv6" => {
					updated = updated || try_parse(&mut self.ipv6, k, v);
				}
				"min_compatible_version" => {
					updated = updated || try_parse(&mut self.min_compatible_version, k, v);
				}
				"compatible_version" => {
					updated = updated || try_parse(&mut self.backward_compatible, k, v);
				}
				"min_neighbours" => {
					updated = updated || try_parse(&mut self.min_neighbours, k, v);
				}
				"max_neighbours" => {
					updated = updated || try_parse(&mut self.max_neighbours, k, v);
				}
				"restrict_neighbours" => {
					updated = updated || try_parse(&mut self.restrict_neighbours, k, v);
				}
				"broadcast_filling_percents" => {
					let mut tmp: u32 = 0;
					if try_parse(&mut tmp, k, v) {
						if tmp <= 100 {
							self.broadcast_percent = tmp;
							updated = true;
						}
						else {
							println!("Value of {} must be in range 0..100", k);
						}
					}
				}
				"observer_wait_time" => {
					updated = updated || try_parse(&mut self.reload_delay_sec, k, v);
				}
				_ => ()
			}
		}
		updated
	}
}

fn try_parse<N: FromStr + PartialEq + Copy + Display>(param: &mut N, key: &String, val: &String) -> bool {
	match val.parse::<N>() {
		Err(_) => {
			println!("error in {} value: it must be one of {}", key, std::any::type_name::<N>());
		}
		Ok(val) => {
			if param != &val {
				println!("{} is updated: {} -> {}", key, &param, &val);
				*param = val;
				return true;
			}
		}
	}
	false
}

fn try_update(param: &mut String, key: &String, val: &String) -> bool {
	if param != val {
		println!("{} is updated: {} -> {}", key, &param, val);
		*param = val.to_string();
		return true;
	}
	false
}
