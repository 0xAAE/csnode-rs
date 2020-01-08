use super::TEST_STOP_DELAY_SEC;
use super::super::config::SharedConfig;
use log::{info, warn};

use std::net::UdpSocket;
use std::time::Duration;
use std::sync::mpsc::SyncSender;

pub struct FragmentReceiver {
	sock: UdpSocket,
	tx: SyncSender<Vec<u8>>
}

impl FragmentReceiver {
	pub fn new(conf: SharedConfig, tx: SyncSender<Vec<u8>>) -> FragmentReceiver {
		let data_guard = conf.read().unwrap();
		let host_input = &data_guard.host_input;
		let socket = UdpSocket::bind((host_input.ip, host_input.port)).unwrap();
		// timeout to block read from socket
		socket.set_read_timeout(Some(Duration::from_secs(TEST_STOP_DELAY_SEC))).unwrap();
		FragmentReceiver {
			sock: socket,
			tx: tx
		}
	}

	pub fn recv(&self, buf: &mut [u8]) -> usize {
		let cnt = match self.sock.recv_from(buf) {
			Ok((cnt, _)) => cnt,
			Err(_) => 0
		};
		if cnt > 0 {
			info!("fragment of {} bytes received", cnt);
			if self.tx.send(buf[..cnt].to_vec()).is_err() {
				warn!("failed to pass fragment to collector");
				return 0;
			}
		}
		cnt
	}
}
