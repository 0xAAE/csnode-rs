use super::config::SharedConfig;
use std::thread;
use std::thread::JoinHandle;
use std::thread::spawn;
use std::time;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::{SyncSender, Receiver};
use log::info;

const TEST_STOP_DELAY_SEC: u64 = 2;
const MAX_FRAGMENT_SIZE: usize = 1024;
const MAX_FRAGMENT_QUEUE: usize = 1024;

mod fragment_receiver;
mod packet_collector;

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
		let (tx_fragment, rx_fragment) = sync_channel::<Vec<u8>>(MAX_FRAGMENT_QUEUE);
		let instance = Box::new(Network {
			stop_flag: stop_flag_instance.clone(),
			recv_thread: start_recv(conf.clone(), stop_flag_instance.clone(), tx_fragment),
			collect_thread: start_collect(conf.clone(), stop_flag_instance.clone(), rx_fragment),
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

fn start_recv(conf: SharedConfig, stop_flag: Arc<AtomicBool>, tx: SyncSender<Vec<u8>>) -> JoinHandle<()> {
	info!("Start fragment receiver");
	let handle = spawn(move || {
		info!("Fragment receiver started");
		let fragment_receiver = fragment_receiver::FragmentReceiver::new(conf, tx);
		let mut buf = [0u8; MAX_FRAGMENT_SIZE];
		let mut total_bytes = 0;
        loop {
			let cnt = fragment_receiver.recv(&mut buf);
			if cnt > 0 {
				total_bytes = total_bytes + cnt;
			}
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
        }
        info!("Fragment receiver stopped");
	});
	handle
}

fn start_collect(_conf: SharedConfig, stop_flag: Arc<AtomicBool>, rx: Receiver<Vec<u8>>) -> JoinHandle<()> {
	info!("Start packet collector");
	let handle = spawn(move || {
		info!("Packet collector started");
		let packet_collector = packet_collector::PacketCollector::new(rx);
        loop {
			packet_collector.recv();
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
