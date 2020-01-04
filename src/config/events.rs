use super::endpoint;
use super::try_parse;
use std::collections::HashMap;

pub struct Data {
    /// event reports collector address
    collector_ep: endpoint::Data, //
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

impl Data {

	pub fn new() -> Data {

		Data {
			collector_ep: endpoint::Data::new(),
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
		}
	}

    pub fn update(&mut self, prop: &HashMap<String, String>) -> bool {
        let mut updated = self.collector_ep.update(prop);
        if self.on != self.collector_ep.is_set {
            self.on = self.collector_ep.is_set;
            updated = true;
        }
        if !self.on {
            return updated;
        }

		for (k, v) in prop.iter() {
			match k.as_str() {
				"consensus_liar" => {
					updated = try_parse(&mut self.consensus_liar, k, v) || updated;
				}
				"consensus_silent" => {
					updated = try_parse(&mut self.consensus_silent, k, v) || updated;
				}
				"consensus_failed" => {
					updated = try_parse(&mut self.consensus_failed, k, v) || updated;
				}
				"contracts_liar" => {
					updated = try_parse(&mut self.contracts_liar, k, v) || updated;
				}
				"contracts_silent" => {
					updated = try_parse(&mut self.contracts_silent, k, v) || updated;
				}
				"contracts_failed" => {
					updated = try_parse(&mut self.contracts_failed, k, v) || updated;
				}
				"add_gray_list" => {
					updated = try_parse(&mut self.add_to_gray_list, k, v) || updated;
				}
				"erase_gray_list" => {
					updated = try_parse(&mut self.erase_from_gray_list, k, v) || updated;
				}
				"reject_transaction" => {
					updated = try_parse(&mut self.reject_transaction, k, v) || updated;
				}
				"reject_contract_execution" => {
					updated = try_parse(&mut self.reject_contract_execution, k, v) || updated;
				}
				"reject_contract_consensus" => {
					updated = try_parse(&mut self.reject_contract_consensus, k, v) || updated;
				}
				"alarm_invalid_block" => {
					updated = try_parse(&mut self.alarm_invalid_block, k, v) || updated;
				}
				"big_bang" => {
					updated = try_parse(&mut self.big_bang, k, v) || updated;
				}
				_ => ()
			}
        }
        updated
	}
}