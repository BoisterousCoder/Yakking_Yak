#![allow(non_snake_case)]
use web_sys::console;
use wasm_bindgen::prelude::*;

mod utils;
mod encrypter;
mod msg;
mod KeyBundle;
mod handler;

use crate::encrypter::Crypto;
use crate::msg::{YakMsg, MsgContent};

const RANDOM_NUMBER:u64 = 1234567890; //TODO: fix the seed to its actually random

#[wasm_bindgen]
pub fn newState(name:&str, deviceId:i32) -> String{
	console::log_1(&"Initializing State..".into());
	let state = Crypto::new(name, deviceId, RANDOM_NUMBER);
	console::log_1(&"Returning Initial State..".into());
	return serde_json::to_string(&state).unwrap();
}
/*
function here are defined such that the starting keyword tells you what the return
onExample, returns a message to send
handleExample, returns a modified state
getExample, returns a string containing something to display to the user
*/

#[wasm_bindgen]
pub fn onJoin(_state:&str, group:&str) -> String{
	let state:Crypto = serde_json::from_str(_state).unwrap();
	let content = MsgContent::Join(group.to_string());
	let msg =  YakMsg::new(&state.addr(), content);
	return msg.toWritable();
}
#[wasm_bindgen]
pub fn onAllowTrust(_state:&str) -> String{
	let state:Crypto = serde_json::from_str(_state).unwrap();
	let content = MsgContent::PublicKey(state.publicKey());
	let msg =  YakMsg::new(&state.addr(), content);
	return msg.toWritable();
}

#[wasm_bindgen]
pub fn onBroadcast(_state:&str, text:&str) -> String{
	let state:Crypto = serde_json::from_str(_state).unwrap();
	let content = MsgContent::InsecureText(text.to_string());
	let msg =  YakMsg::new(&state.addr(), content);
	return msg.toWritable();
}
#[wasm_bindgen]
pub fn onSend(_state:&str, text:&str) -> String{
	let state:Crypto = serde_json::from_str(_state).unwrap();
	let content = MsgContent::SecureText(state.encrypt(text.to_string()));
	let msg =  YakMsg::new(&state.addr(), content);
	return msg.toWritable();
}
#[wasm_bindgen]
pub fn onTrust(_state:&str, name:&str) -> String{
	let state:encrypter::Crypto = serde_json::from_str(_state).unwrap();
	let content = match state.person(name.to_string()) {
		Some(person) => Some(MsgContent::Trust(person.address.clone())),
		None => None
	};
	return match content {
		Some(x) => YakMsg::new(&state.addr(), x).toWritable(),
		None => "".to_string()
	};
}
#[wasm_bindgen]
pub fn getList(_state:&str) -> String{
	let state:Crypto = serde_json::from_str(_state).unwrap();
	return state.listPeople();
}
#[wasm_bindgen]
pub fn getDisplay(_state:&str, msg:&str) -> String{
	let state:Crypto = serde_json::from_str(_state).unwrap();
	return YakMsg::fromServer(msg).display(&state);
}
#[wasm_bindgen]
pub fn getRelation(_state:&str, name:&str) -> String{
	let state:Crypto = serde_json::from_str(_state).unwrap();
	return state.relation(name.to_string());
}

#[wasm_bindgen]
pub fn handleIncoming(_state: &str, msg:&str) -> String{
	let mut state:encrypter::Crypto = serde_json::from_str(_state).unwrap();
	YakMsg::fromServer(msg).handleSelf(&mut state);
	return serde_json::to_string(&state).unwrap();
}
#[wasm_bindgen]
pub fn handleTrust(_state: &str, name:&str) -> String{
	let mut state:Crypto = serde_json::from_str(_state).unwrap();
	
	return match state.trust(name.to_string()) {
		Some(_) => serde_json::to_string(&state).unwrap(),
		None => "".to_string()
	};
}