extern crate log;
extern crate log4rs;

use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};
use log4rs::filter::threshold::ThresholdFilter;
//use log4rs::append::rolling_file::RollingFileAppender;

use super::config::SharedConfig;

pub fn init(conf: SharedConfig) {
	let data_guard = conf.read().unwrap();
	let lvl_core = data_guard.logger.min_level;
	let lvl_console = std::cmp::min(data_guard.logger.min_console_level, lvl_core);
	let lvl_file = std::cmp::min(data_guard.logger.min_file_level, lvl_core);

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}")))
        .build();

    let logfile = FileAppender::builder()
        // 2020-01-06 16:02:37 UTC - module(LEVEL): message
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S %Z)(utc)} - {M}(({l})): {m}{n}")))
        .build("log/log.txt")
        .unwrap();

    // let rolling_logfile = RollingFileAppender::builder().build("log/log_N.txt", Box::new(Policy::));

    let config = Config::builder()
        .appender(Appender::builder()
            .filter(Box::new(ThresholdFilter::new(lvl_console)))
            .build("stdout", Box::new(stdout)))
        .appender(Appender::builder()
            .filter(Box::new(ThresholdFilter::new(lvl_file)))
            .build("common", Box::new(logfile)))
        // .logger(Logger::builder()
        //     .appender("logcommon")
        //     .additive(false)
        //     .build("common", lvl_file)) // to enable: info!(target: "common", "message");
		.build(Root::builder()
			.appender("stdout")
			.appender("common")
			.build(std::cmp::max(lvl_console, lvl_file)))
        .unwrap();

    log4rs::init_config(config).unwrap();
    //log::set_boxed_logger(Box::new(log4rs::Logger::new(config))).unwrap();
}
