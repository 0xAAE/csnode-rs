use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use ini::Ini;

pub struct Config {
	pool_sync_config: PoolSyncData,
	api_config: ApiData,
	conveyer_config: ConveyerData,
	events_report_config: EventsReportData,
	db_sql_config: DbSQLData
}

struct EndpointData {
    is_set: bool, // = false;
    port: u16, // = 0;
    ip: std::net::IpAddr // {};
}

pub struct PoolSyncData {
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

pub struct ApiData {
    api_port: u16, // = 9090;
    ajax_port: u16, // = 8081;
    executor_port: u16, // = 9080;
    apiexec_port: u16, // = 9070;
    executor_send_timeout: u32, // = 4000;
    executor_recv_timeout: u32, // = 4000;
    server_send_timeout: u32, // = 30000;
    server_recv_timeout: u32, // = 30000;
    ajax_send_timeout: u32, // = 30000;
    ajax_recv_timeout: u32, // = 30000;
    executor_host: String, //{ "localhost" };
    executor_launch_command: String, //{};
    executor_launch_delay: u32, // = 100;
    executor_observer_delay: u32, //= 100;
    executor_test_delay: u32, // = 1000;
    executor_multi_instance: bool, // = false;
    executor_commit_min: u32, // = 1506;   // first commit with support of checking
    executor_commit_max: u32, //{-1};      // unlimited range on the right
    executor_jps_command: String // = "jps";
}

pub struct ConveyerData {
    send_cache_delay: usize, // = DEFAULT_CONVEYER_SEND_CACHE_VALUE;
    max_packet_resends: usize, // = DEFAULT_CONVEYER_MAX_RESENDS_SEND_CACHE;
    packet_ttl: usize // = DEFAULT_CONVEYER_MAX_PACKET_LIFETIME;
}

struct EventsReportData {
    /// event reports collector address
    collector_ep: EndpointData, //
    /// general on/off
    on: bool, // = false;
    /// report filters, only actual if on is true
    /// report every liar in consensus
    consensus_liar: bool, // = false;
    /// report every silent trusted node in consensus
    consensus_silent: bool, // = false;
    /// report consensus is not achieved
    consensus_failed: bool, // = true;
    /// report every liar in smart contracts consensus
    contracts_liar: bool, // = false;
    /// report every silent trusted node in smart contracts consensus
    contracts_silent: bool, // = false;
    /// report smart contracts consensus is not achieved
    contracts_failed: bool, // = true;
    /// report put node into gray list
    add_to_gray_list: bool, // = true;
    /// report remove node from gray list
    erase_from_gray_list: bool, // = false;
    /// basic transaction is rejected by final consensus
    reject_transaction: bool, // = true;
    /// contract-related transaction is rejected just after execution, before consensus started
    reject_contract_execution: bool, // = true;
    /// contract-related transaction is rejected by final basic consensus
    reject_contract_consensus: bool, // = true;
    /// invalid block detected by node
    alarm_invalid_block: bool, // = true;
    /// big bang occurred
    big_bang: bool // = false;
}

struct DbSQLData {
    /// SQL server host name or ip address
    host: String, // { "localhost" };
    /// connection port 5432 by default
    port: u16, // = 5432;
    /// name of database
    name: String, // { "roundinfo" };
    /// username and password for access
    user: String, // { "postgres" };
    password: String // { "postgres" };
}

impl Config {

	pub fn new(file_name: &str) -> Config {
		let default_packet_ttl = 10;

		let mut instance = Config {
			pool_sync_config: PoolSyncData {
				single_block_reply: true,
				fast_mode: false,
				max_block_request: 25,
				request_round_delay: 20,
				max_neighbour_req_count: 10,
				update_required_blocks_delay: 350
			},
			api_config: ApiData {
				api_port: 9090,
				ajax_port: 8081,
				executor_port: 9080,
				apiexec_port: 9070,
				executor_send_timeout: 4_000,
				executor_recv_timeout: 4_000,
				server_send_timeout: 30_000,
				server_recv_timeout: 30_000,
				ajax_send_timeout: 30_000,
				ajax_recv_timeout: 30_000,
				executor_host: "localhost".to_string(),
				executor_launch_command: String::new(),
				executor_launch_delay: 100,
				executor_observer_delay: 100,
				executor_test_delay: 1_000,
				executor_multi_instance: false,
				// first commit with support of checking:
				executor_commit_min: 1506,
				// unlimited range on the right
				executor_commit_max: std::u32::MAX,
				executor_jps_command: "jps".to_string()			
			},
			conveyer_config: ConveyerData {
				send_cache_delay: 10,
				packet_ttl: default_packet_ttl,
				// 1 <= ttl <= 10
				max_packet_resends: std::cmp::max(1, std::cmp::min(default_packet_ttl / 2, 10))
			},
			events_report_config: EventsReportData {
					collector_ep: EndpointData {
						is_set: false,
						port: 0,
						ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
					},
					on: false,
					consensus_liar: false,
					consensus_silent: false,
					consensus_failed: true,
					contracts_liar: false,
					contracts_silent: false,
					contracts_failed: true,
					add_to_gray_list: true,
					erase_from_gray_list: false,
					reject_transaction: true,
					reject_contract_execution: true,
					reject_contract_consensus: true,
					alarm_invalid_block: true,
					big_bang: false
			},
			db_sql_config: DbSQLData {
					host: "localhost".to_string(),
					port: 5432,
					name: "roundinfo".to_string(),
					user: "postgres".to_string(),
					password: "postgres".to_string()
			}
		};

		let params = "params".to_string();

		let ini = Ini::load_from_file(file_name).unwrap();
		for (sec, prop) in ini.iter() {
			println!("\nSection: {:?}", *sec);
			match sec.as_ref().map(String::as_str) {
				Some("params") => {
					for (k, v) in prop.iter() {
						match k.as_str() {
							"node_type" => {}
							"hosts_filename" => {}
							"bootstrap_type" => {}
							"ipv6" => {}
							"min_compatible_version" => {}
							"compatible_version" => {}
							"min_neighbours" => {}
							"max_neighbours" => {}
							"restrict_neighbours" => {}
							"broadcast_filling_percents" => {}
							_ => {}
						}
						println!("{} = {}", *k, *v);
					}
				}
				Some("start_node") => {}
				Some("host_input") => {}
				Some("api") => {}
				Some("pool_sync") => {}
				Some("Core") => {}
				// Some("Sinks.Console") => {}
				// Some("Sinks.File") => {}
				// Some("Sinks.Event") => {}
				Some("event_report") => {}
				Some(_) => {}
				None => {}
			};
		}

		instance
	}


}