use log::{debug, trace, info, warn};
use ini::Ini;
use std::fmt::Display;
use std::str::FromStr;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type SharedConfig = Arc<RwLock<Config>>;

mod endpoint;
mod api;
mod events;
mod sync;
mod sql;
mod conveyer;
mod logger;

pub struct Config {
	// [params]
	pub node_id: String,
	pub hosts_filename: String,
	bootstrap_type: String,
	ipv6: bool,
	pub min_compatible_version: u32,
	backward_compatible: bool,
	min_neighbours: usize,
	pub max_neighbours: usize,
	//connection_bandwidth: usize, // obsolete
	pub reload_delay_sec: u32, // observer_wait_time
	restrict_neighbours: bool,
	broadcast_percent: u32,
	// [start_node]
	start_node: endpoint::Data,
	// [host_input]
	pub host_input: endpoint::Data,
	// [pool_sync]
	pub sync: sync::Data,
	// [api]
	api: api::Data,
	// [conveyer]
	conveyer: conveyer::Data,
	// [event_report]
	events: events::Data,
	// [dbsql]
	sql: sql::Data,
	// logger
	pub logger: logger::Data,
	// source file to read
	ini_file: String
}

impl Config {

	pub fn new(file_name: &str) -> Config {
		let mut instance = Config {
			node_id: String::from("AAExXjedndkJZrtPpJSX3taw5JB4sjqx32xWWWDnsKUu"),
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
			logger: logger::Data::new(),
			ini_file: file_name.to_string()
		};

		instance.reload();
		instance
	}

	pub fn reload(&mut self) {
		if let Ok(ini) = Ini::load_from_file(&self.ini_file) {
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
					Some("Core") => {
						self.logger.update_core(prop);
					}
					Some("Sinks.Console") => {
						self.logger.update_console(prop);
					}
					Some("Sinks.File") => {
						self.logger.update_file(prop);
					}
					//Some("Sinks.Event") => {}
					Some(s) => {
						trace!("ignore {} section", s);
					},
					None => {}
				};
			}
		}
	}

	fn update(&mut self, prop: &HashMap<String, String>) -> bool {
		let mut updated = false;
		for (k, v) in prop.iter() {
			match k.as_str() {
				"node_id" => {
					updated = try_update(&mut self.node_id, k, v) || updated;
				}
				"hosts_filename" => {
					updated = try_update(&mut self.hosts_filename, k, v) || updated;
				}
				"bootstrap_type" => {
					updated = try_update(&mut self.bootstrap_type, k, v) || updated;
				}
				"ipv6" => {
					updated = try_parse(&mut self.ipv6, k, v) || updated;
				}
				"min_compatible_version" => {
					updated = try_parse(&mut self.min_compatible_version, k, v) || updated;
				}
				"compatible_version" => {
					updated = try_parse(&mut self.backward_compatible, k, v) || updated;
				}
				"min_neighbours" => {
					updated = try_parse(&mut self.min_neighbours, k, v) || updated;
				}
				"max_neighbours" => {
					updated = try_parse(&mut self.max_neighbours, k, v) || updated;
				}
				"restrict_neighbours" => {
					updated = try_parse(&mut self.restrict_neighbours, k, v) || updated;
				}
				"broadcast_filling_percents" => {
					let mut tmp: u32 = self.broadcast_percent;
					if try_parse(&mut tmp, k, v) {
						if tmp <= 100 {
							self.broadcast_percent = tmp;
							updated = true;
						}
						else {
							info!("Value of {} must be in range 0..100", k);
						}
					}
				}
				"observer_wait_time" => {
					updated = try_parse(&mut self.reload_delay_sec, k, v) || updated;
				}
				_ => ()
			}
		}
		updated
	}

}

#[test]
fn test_update_params() {
	let mut conf = Config::new("");
	let mut data = HashMap::<String, String>::new();
	data.insert("hosts_filename".to_string(), "nodes.txt".to_string());
	data.insert("bootstrap_type".to_string(), "start_node".to_string());
	data.insert("min_compatible_version".to_string(), "460".to_string());
	data.insert("compatible_version".to_string(), "true".to_string());

	assert_eq!(conf.update(&data), true);
	assert_eq!(conf.update(&data), false);
	assert_eq!(conf.update(&data), false);
}

fn try_parse<N: FromStr + PartialEq + Copy + Display>(param: &mut N, key: &str, val: &str) -> bool {
	match val.parse::<N>() {
		Err(_) => {
			warn!("error in {} value: it must be valid for {} type", key, std::any::type_name::<N>());
		}
		Ok(val) => {
			if param != &val {
				debug!("{} is updated: {} -> {}", key, &param, &val);
				*param = val;
				return true;
			}
		}
	}
	false
}

fn try_update(param: &mut String, key: &str, val: &str) -> bool {
	if param != val {
		debug!("{} is updated: {} -> {}", key, &param, val);
		*param = val.to_string();
		return true;
	}
	false
}
