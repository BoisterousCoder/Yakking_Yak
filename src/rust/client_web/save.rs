#![cfg(target_arch = "wasm32")]
use crate::client::utils::Address;
use crate::client::ratchet::Ratchet;
use crate::client::forein_agent::ForeinAgent;
use crate::ServerMsg;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GroupSave {
    pub group:String,
    pub addr:Address,
    pub proxy_ratchet:Ratchet,
	pub agents: Vec<ForeinAgent>,
	pub msgs: Vec<ServerMsg>
}
