extern crate log;
extern crate log4rs;

use log4rs::append::console::ConsoleAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};
use log4rs::filter::threshold::ThresholdFilter;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;

use super::config::SharedConfig;

pub fn init(conf: SharedConfig) {
	let data_guard = conf.read().unwrap();
	let lvl_core = data_guard.logger.min_level;
	let lvl_console = std::cmp::min(data_guard.logger.min_console_level, lvl_core);
	let lvl_file = std::cmp::min(data_guard.logger.min_file_level, lvl_core);

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}")))
        .build();

    let file_pattern = "{d(%Y-%m-%d %H:%M:%S %Z)(utc)} - {M}(({l})): {m}{n}";
    let window_size = 10; // log0, log1, log2, .., log10
    let fixed_window_roller = FixedWindowRoller::builder().build("log/log_{}.txt", window_size).unwrap();
    let size_limit = 50 * 1024 * 1024; // 50MB as max log file size to roll
    let size_trigger = SizeTrigger::new(size_limit);
    let compound_policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(fixed_window_roller));
    let rolling_logfile = RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(file_pattern)))
            .build("log/log_0.txt", Box::new(compound_policy)).unwrap();

    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(lvl_console)))
                .build("stdout", Box::new(stdout))
        )
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(lvl_file)))
                .build("common", Box::new(rolling_logfile))
        )
        // .logger(Logger::builder()
        //     .appender("common")
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
