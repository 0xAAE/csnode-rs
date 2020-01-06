extern crate log;
extern crate log4rs;

//use log::{SetLoggerError, LevelFilter, Log};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};

use super::config::SharedConfig;

// pub fn init(conf: SharedConfig) -> Result<(), SetLoggerError> {
// 	let lvl;
// 	{
// 		let data_guard = conf.read().unwrap();
// 		lvl = data_guard.logger.min_level;
// 	}
// 	log::set_boxed_logger(Box::new(init_logger(conf))).map(|()| log::set_max_level(lvl));
// }

pub fn init(conf: SharedConfig) {
	let data_guard = conf.read().unwrap();
	let lvl_core = data_guard.logger.min_level;
	let lvl_console = std::cmp::max(data_guard.logger.min_console_level, lvl_core);
	let lvl_file = std::cmp::max(data_guard.logger.min_file_level, lvl_core);

	let stdout = ConsoleAppender::builder().build();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("log/log.txt")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        //.logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
        .logger(Logger::builder()
            .appender("logfile")
            .additive(false)
            .build("app::logfile", lvl_file))
		.build(Root::builder()
			.appender("stdout")
			.build(lvl_console))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
