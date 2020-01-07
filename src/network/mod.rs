use super::config::SharedConfig;
use std::thread;
use std::thread::JoinHandle;
use std::thread::spawn;
use std::time;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use log::info;

const TEST_STOP_DELAY_SEC: u64 = 2;

pub struct Network {
	recv_thread:		JoinHandle<()>,
	collect_thread:		JoinHandle<()>,
	dispatch_thread:	JoinHandle<()>,
	prepare_thread:		JoinHandle<()>,
	send_thread:		JoinHandle<()>,
	stop_flag: Arc<AtomicBool>
}

impl Network {
	pub fn new(conf: SharedConfig) -> Box<Network> {
		let stop_flag_instance = Arc::new(AtomicBool::new(false)); 
		let instance = Box::new(Network {
			stop_flag: stop_flag_instance.clone(),
			recv_thread: start_recv(conf.clone(), stop_flag_instance.clone()),
			collect_thread: start_collect(conf.clone(), stop_flag_instance.clone()),
			dispatch_thread: start_dispatch(conf.clone(), stop_flag_instance.clone()),
			prepare_thread: start_prepare(conf.clone(), stop_flag_instance.clone()),
			send_thread: start_send(conf.clone(), stop_flag_instance.clone())
		});
		instance
	}

	pub fn stop(self) {
		self.stop_flag.store(true, Ordering::SeqCst);
		self.recv_thread.join().expect("Failed to stop fragment receiver");
		self.collect_thread.join().expect("Failed to stop packet collector");
		self.dispatch_thread.join().expect("Failed to stop packet dispatcher");
		self.prepare_thread.join().expect("Failed to stop send preparator");
		self.send_thread.join().expect("Failed to stop fragment sender");
	}
}

fn start_recv(_conf: SharedConfig, stop_flag: Arc<AtomicBool>) -> JoinHandle<()> {
	info!("Start fragment receiver");
	let handle = spawn(move || {
		info!("Fragment receiver started");
        loop {
            thread::sleep(time::Duration::from_secs(TEST_STOP_DELAY_SEC));
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Fragment receiver stopped");
	});
	handle
}

fn start_collect(_conf: SharedConfig, stop_flag: Arc<AtomicBool>) -> JoinHandle<()> {
	info!("Start packet collector");
	let handle = spawn(move || {
		info!("Packet collector started");
        loop {
            thread::sleep(time::Duration::from_secs(TEST_STOP_DELAY_SEC));
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Packet collector stopped");
	});
	handle
}

fn start_dispatch(_conf: SharedConfig, stop_flag: Arc<AtomicBool>) -> JoinHandle<()> {
	info!("Start packet dispatcher");
	let handle = spawn(move || {
		info!("Packet dispatcher started");
        loop {
            thread::sleep(time::Duration::from_secs(TEST_STOP_DELAY_SEC));
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Packet dispatcher stopped");
	});
	handle
}

fn start_prepare(_conf: SharedConfig, stop_flag: Arc<AtomicBool>) -> JoinHandle<()> {
	info!("Start send preparator");
	let handle = spawn(move || {
		info!("Send preparator started");
        loop {
            thread::sleep(time::Duration::from_secs(TEST_STOP_DELAY_SEC));
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Send preparator stopped");
	});
	handle
}

fn start_send(_conf: SharedConfig, stop_flag: Arc<AtomicBool>) -> JoinHandle<()> {
	info!("Start fragment sender");
	let handle = spawn(move || {
		info!("Fragment sender started");
        loop {
            thread::sleep(time::Duration::from_secs(TEST_STOP_DELAY_SEC));
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Fragment sender stopped");
	});
	handle
}
