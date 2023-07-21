#![cfg(target_arch = "wasm32")]

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GroupSave {
    pub group:String,
    pub addr:Address,
    pub proxy_ratchet:Ratchet,
	pub agents: Vec<ForeinAgent>,
	pub msgs: Vec<ServerMsg>
}
