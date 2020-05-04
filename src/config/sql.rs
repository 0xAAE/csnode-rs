use super::try_parse;
use super::try_update;
use std::collections::HashMap;

pub struct Data {
    /// SQL server host name or ip address
    host: String,
    /// connection port 5432 by default
    port: u16,
    /// name of database
    name: String,
    /// username and password for access
    user: String,
    password: String,
}

impl Data {
    pub fn new() -> Data {
        Data {
            host: "localhost".to_string(),
            port: 5432,
            name: "roundinfo".to_string(),
            user: "postgres".to_string(),
            password: "postgres".to_string(),
        }
    }

    pub fn update(&mut self, prop: &HashMap<String, String>) -> bool {
        let mut updated = false;
        for (k, v) in prop.iter() {
            match k.as_str() {
                "host" => {
                    updated = try_update(&mut self.host, k, v) || updated;
                }
                "port" => {
                    updated = try_parse(&mut self.port, k, v) || updated;
                }
                "name" => {
                    updated = try_update(&mut self.name, k, v) || updated;
                }
                "user" => {
                    updated = try_update(&mut self.user, k, v) || updated;
                }
                "password" => {
                    updated = try_update(&mut self.password, k, v) || updated;
                }
                _ => (),
            }
        }
        updated
    }
}
