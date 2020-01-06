use log::LevelFilter;
use log::debug;
use std::collections::HashMap;

pub struct Data {
	/// min core level
	pub min_level: LevelFilter,
	/// min output to console level
	pub min_console_level: LevelFilter,
	/// min output to file level
	pub min_file_level: LevelFilter
}

impl Data {

	pub fn new() -> Data {
		Data {
			min_level: LevelFilter::Debug,
			min_console_level: LevelFilter::Info,
			min_file_level: LevelFilter::Debug
		}
	}

	pub fn update_core(&mut self, prop: &HashMap<String, String>) -> bool {
		let mut updated = false;
		for (k, v) in prop.iter() {
			match k.as_str() {
				"Filter" => {
					updated = Data::update_level(&mut self.min_level, v);
				}
				_ => ()
			}
		}
		updated
	}

	pub fn update_console(&mut self, prop: &HashMap<String, String>) -> bool {
		let mut updated = false;
		for (k, v) in prop.iter() {
			match k.as_str() {
				"Filter" => {
					updated = Data::update_level(&mut self.min_console_level, v);
				}
				_ => ()
			}
		}
		updated
	}

	pub fn update_file(&mut self, prop: &HashMap<String, String>) -> bool {
		let mut updated = false;
		for (k, v) in prop.iter() {
			match k.as_str() {
				"Filter" => {
					updated = Data::update_level(&mut self.min_file_level, v);
				}
				_ => ()
			}
		}
		updated
	}

	fn update_level(param: &mut LevelFilter, value: &str) -> bool {
		let mut updated = false;
		if let Some(lvl) = Data::try_parse_level(value) {
			if lvl != *param {
				debug!("filter is updated: {} -> {}", *param, &lvl);
				*param = lvl;
				updated = true;				
			}
		}
		updated
	}

	fn try_parse_level(v: &str) -> Option<LevelFilter> {
		let param = "%severity%";
		if let Some(index) = v.to_lowercase().find(param) {
			let value = v[index + param.len()..].to_lowercase();
			if value.contains("trace") {
				return Some(LevelFilter::Trace);
			}
			if value.contains("debug") {
				return Some(LevelFilter::Debug);
			}
			if value.contains("info") {
				return Some(LevelFilter::Info);
			}
			if value.contains("warning") {
				return Some(LevelFilter::Warn);
			}
			if value.contains("error") {
				return Some(LevelFilter::Error);
			}
		}
		return None;
	}
}

#[test]
fn test_try_parse_level() {
	assert_eq!(Data::try_parse_level("%Severity% >= debug"), Some(LevelFilter::Debug));
	assert_eq!(Data::try_parse_level("%Severity% >= trace"), Some(LevelFilter::Trace));
	assert_eq!(Data::try_parse_level("%Severity% >= info"), Some(LevelFilter::Info));
	assert_eq!(Data::try_parse_level("%Severity% >= warning"), Some(LevelFilter::Warn));
	assert_eq!(Data::try_parse_level("%Severity% >= error"), Some(LevelFilter::Error));
	assert_eq!(Data::try_parse_level("%Severity%>=error"), Some(LevelFilter::Error));
	assert_eq!(Data::try_parse_level(" %Severity% >= error "), Some(LevelFilter::Error));
	assert_eq!(Data::try_parse_level("%Severity% >= err"), None);
	assert_eq!(Data::try_parse_level("Severity >= error"), None);
	assert_eq!(Data::try_parse_level("debug"), None);
}
