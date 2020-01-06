extern crate log;
use log::info;

mod config;
use config::SharedConfig;

mod logger;

use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::spawn;
use std::time;

fn main() {
    println!("Hello, world!");

    let mut file_name = "config.ini".to_string();
    for arg in std::env::args().skip(1) {
        if arg.starts_with("--config=") {
            match arg.split("=").skip(1).next() {
                Some(v) => {
                    file_name = v.to_string();
                }
                None => {
                    println!("Incorrect option value for --config, must be --config=<file name>");
                }
            };
        }
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let conf: SharedConfig = Arc::new(RwLock::new(config::Config::new(&file_name)));
    
    // init logger
    logger::init(conf.clone());
    info!("Logger started");
    // run observer thread:
    let config_observer = start_config_observer_thread(conf.clone(), stop_flag.clone());

    // imitate other work: sleep too long and exit
    thread::sleep(time::Duration::from_secs(20 * 60));
    stop_flag.store(true, Ordering::SeqCst);
    config_observer.join().unwrap();
}

fn start_config_observer_thread(config: SharedConfig, stop_flag: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
    let handle = spawn(move || {
        let mut wait_sec;
        loop {
            // get wait seconds
            {
                let data_guard = config.read().unwrap();
                wait_sec = data_guard.reload_delay_sec;
            }
            // test exit flag before pause
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
            let pause = time::Duration::from_secs(wait_sec.into());
            thread::sleep(pause);
            // test exit flag before reload config
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
            // reload configuration parameters
            {
                let mut data_guard = config.write().unwrap();
                data_guard.reload();
            }
        }
    });
    handle
}

//fn start_logger_thread()