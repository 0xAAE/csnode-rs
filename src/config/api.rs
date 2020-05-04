use super::try_parse;
use super::try_update;
use std::collections::HashMap;

pub struct Data {
    api_port: u16,                   // = 9090;
    ajax_port: u16,                  // = 8081;
    executor_port: u16,              // = 9080;
    apiexec_port: u16,               // = 9070;
    executor_send_timeout: u32,      // = 4000;
    executor_recv_timeout: u32,      // = 4000;
    server_send_timeout: u32,        // = 30000;
    server_recv_timeout: u32,        // = 30000;
    ajax_send_timeout: u32,          // = 30000;
    ajax_recv_timeout: u32,          // = 30000;
    executor_host: String,           //{ "localhost" };
    executor_launch_command: String, //{};
    executor_launch_delay: u32,      // = 100;
    executor_observer_delay: u32,    //= 100;
    executor_test_delay: u32,        // = 1000;
    executor_multi_instance: bool,   // = false;
    executor_commit_min: u32,        // = 1506;   // first commit with support of checking
    executor_commit_max: u32,        //{-1};      // unlimited range on the right
    executor_jps_command: String,    // = "jps";
}

impl Data {
    pub fn new() -> Data {
        Data {
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
            executor_jps_command: "jps".to_string(),
        }
    }

    pub fn update(&mut self, prop: &HashMap<String, String>) -> bool {
        let mut updated = false;
        for (k, v) in prop.iter() {
            match k.as_str() {
                "port" => {
                    updated = try_parse(&mut self.api_port, k, v) || updated;
                }
                "ajax_port" => {
                    updated = try_parse(&mut self.ajax_port, k, v) || updated;
                }
                "executor_port" => {
                    updated = try_parse(&mut self.executor_port, k, v) || updated;
                }
                "apiexec_port" => {
                    updated = try_parse(&mut self.apiexec_port, k, v) || updated;
                }
                "executor_send_timeout" => {
                    updated = try_parse(&mut self.executor_send_timeout, k, v) || updated;
                }
                "executor_receive_timeout" => {
                    updated = try_parse(&mut self.executor_recv_timeout, k, v) || updated;
                }
                "server_send_timeout" => {
                    updated = try_parse(&mut self.server_send_timeout, k, v) || updated;
                }
                "server_receive_timeout" => {
                    updated = try_parse(&mut self.server_recv_timeout, k, v) || updated;
                }
                "ajax_server_send_timeout" => {
                    updated = try_parse(&mut self.ajax_send_timeout, k, v) || updated;
                }
                "ajax_server_receive_timeout" => {
                    updated = try_parse(&mut self.ajax_recv_timeout, k, v) || updated;
                }
                "executor_ip" => {
                    updated = try_update(&mut self.executor_host, k, v) || updated;
                }
                "executor_command" => {
                    updated = try_update(&mut self.executor_launch_command, k, v) || updated;
                }
                "executor_run_delay" => {
                    updated = try_parse(&mut self.executor_launch_delay, k, v) || updated;
                }
                "executor_background_thread_delay" => {
                    updated = try_parse(&mut self.executor_observer_delay, k, v) || updated;
                }
                "executor_check_version_delay" => {
                    updated = try_parse(&mut self.executor_test_delay, k, v) || updated;
                }
                "executor_multi_instance" => {
                    updated = try_parse(&mut self.executor_multi_instance, k, v) || updated;
                }
                "executor_commit_min" => {
                    updated = try_parse(&mut self.executor_commit_min, k, v) || updated;
                }
                "executor_commit_max" => {
                    updated = try_parse(&mut self.executor_commit_max, k, v) || updated;
                }
                "jps_command" => {
                    updated = try_update(&mut self.executor_jps_command, k, v) || updated;
                }
                _ => (),
            }
        }
        updated
    }
}
