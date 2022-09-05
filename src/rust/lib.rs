#![allow(non_snake_case)]
use web_sys::console;

use wasm_bindgen::prelude::*;

mod utils;
mod encrypter;
mod serverhandlers;
mod KeyBundle;
//mod cmdhandler;

//use serverhandlers::*;


#[wasm_bindgen]
pub fn onConnect(name:&str, deviceId:i32, group:&str) -> String{
	console::log_1(&"Initializing State..".into());
	let state = encrypter::Crypto::new(name, deviceId);
	//send(&ServerMsg::new(&state.addr(), MsgContent::Join(group.to_string())).toWritable());
	console::log_1(&"Returning Initial State..".into());
	return serde_json::to_string(&state).unwrap();
}
// #[wasm_bindgen]
// pub fn onMsg(msg:&str, _state: &str) -> String{
// 	let mut state:encrypter::Crypto = serde_json::from_str(_state).unwrap();
// 	ServerMsg::fromServer(msg).handleSelf(&mut state);
// 	return serde_json::to_string(&state).unwrap();
// }