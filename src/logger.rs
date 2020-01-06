use log::{SetLoggerError, LevelFilter};

mod config;
use config::SharedConfig;

pub fn init(conf: SharedConfig) -> Result<(), SetLoggerError> {
	let data_guard = conf.read().unwrap();
	let lvl = data_guard.
	log::set_boxed_logger(Box::new(SimpleLogger)).map(|()| log::set_max_level(LevelFilter::Info))
}

fn init_logger(conf: SharedConfig) -> Log {

}