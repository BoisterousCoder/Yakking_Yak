#![allow(non_snake_case)]
use web_sys::console;
use wasm_bindgen::prelude::*;

mod utils;
mod encrypter;
mod serverhandlers;
mod KeyBundle;
//mod cmdhandler;

use crate::encrypter::Crypto;
use crate::serverhandlers::{ServerMsg, MsgContent};

const RANDOM_NUMBER:u64 = 1234567890; //TODO: fix the seed to its actually random

#[wasm_bindgen]
pub fn newState(name:&str, deviceId:i32) -> String{
	console::log_1(&"Initializing State..".into());
	let state = Crypto::new(name, deviceId, RANDOM_NUMBER);
	console::log_1(&"Returning Initial State..".into());
	return serde_json::to_string(&state).unwrap();
}

#[wasm_bindgen]
pub fn getJoin(_state:&str, group:&str) -> String{
	console::log_1(&"Rebuilding State..".into());
	let state:Crypto = serde_json::from_str(_state).unwrap();
	console::log_1(&"Building Join Message..".into());
	return ServerMsg::new(&state.addr(), MsgContent::Join(group.to_string())).toWritable();
}
// #[wasm_bindgen]
// pub fn onMsg(msg:&str, _state: &str) -> String{
// 	let mut state:encrypter::Crypto = serde_json::from_str(_state).unwrap();
// 	ServerMsg::fromServer(msg).handleSelf(&mut state);
// 	return serde_json::to_string(&state).unwrap();
// }