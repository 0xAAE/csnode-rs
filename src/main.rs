mod config;

use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::spawn;
use std::time;

#[derive(Clone)]
struct SharedConfig(Arc<RwLock<config::Config>>);

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
    let conf = SharedConfig(Arc::new(RwLock::new(config::Config::new(&file_name))));
    // observer thread data:
    let observer_stop = stop_flag.clone();
    let observer_data = conf.clone();
    let config_observer = spawn(move || {
        let mut wait_sec;
        loop {
            // get wait seconds
            {
                let data_guard = observer_data.0.read().unwrap();
                wait_sec = data_guard.reload_delay_sec;
            }
            let pause = time::Duration::from_secs(wait_sec.into());
            thread::sleep(pause);
            if observer_stop.load(Ordering::SeqCst) {
                break;
            }
            // reload configuration parameters
            {
                let mut data_guard = observer_data.0.write().unwrap();
                data_guard.reload();
            }
        }
        ()
    });

    // sleep too long and exit
    thread::sleep(time::Duration::from_secs(20 * 60));
    stop_flag.store(true, Ordering::SeqCst);
    config_observer.join().unwrap();
}
