use std::collections::HashMap;
use super::try_parse;

pub struct Data {
	/// true: sendBlockRequest one pool at a time; false: equal to number of pools requested
	single_block_reply: bool, // = true;                
	/// true: is silent mode synchro (sync up to the current round); false: normal mode      
	fast_mode: bool, // = false;                    
	/// max block count in one request: cannot be 0    
	max_block_request: u8, // = 25;                 
	/// round count to repeat request, 0 = never  
	request_round_delay: u8, // = 20;           
	/// max packet count to connect to another neighbor, 0 = never
	max_neighbour_req_count: u8, // = 10;          
	/// delay between updates of required block sequences, 0 = never, 1 = once per round, other value = delay in msec 
    update_required_blocks_delay: u16 // = 350;  
}

impl Data {

	pub fn new() -> Data {
		Data {
			single_block_reply: true,
			fast_mode: false,
			max_block_request: 25,
			request_round_delay: 20,
			max_neighbour_req_count: 10,
			update_required_blocks_delay: 350
		}
	}

	pub fn update(&mut self, prop: &HashMap<String, String>) -> bool {
		let mut updated = false;
		for (k, v) in prop.iter() {
			match k.as_str() {
				"one_reply_block" => {
					updated = try_parse(&mut self.single_block_reply, k, v) || updated;
				}
				"fast_mode" => {
					updated = try_parse(&mut self.fast_mode, k, v) || updated;
				}
				"block_pools_count" => {
					updated = try_parse(&mut self.max_block_request, k, v) || updated;
				}
				"request_repeat_round_count" => {
					updated = try_parse(&mut self.request_round_delay, k, v) || updated;
				}
				"neighbour_packets_count" => {
					updated = try_parse(&mut self.max_neighbour_req_count, k, v) || updated;
				}
				"sequences_verification_frequency" => {
					updated = try_parse(&mut self.update_required_blocks_delay, k, v) || updated;
				}
				_ => ()
			}
		}
		updated
	}
}
