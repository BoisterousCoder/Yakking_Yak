#![cfg(target_arch = "wasm32")]
use std::fmt::format;
use std::{string::String, thread::Thread, time::Duration, option::Option};

use web_sys::{console, window, Document, Event, Element};
use wasm_bindgen::{prelude::*, JsValue};
use js_sys::{Function, Array};
use std::{thread, format};

mod utils;
mod encrypter;
mod serverhandlers;
mod KeyBundle;

use crate::encrypter::Crypto;
use crate::serverhandlers::{ServerMsg, MsgContent};

const RANDOM_NUMBER:u64 = 1234567890; //TODO: fix the seed to its actually random
const DEVICE_ID:i32 = 12345;//TODO: Make this useful
const SLEEP_TIME:u64 = 20;

#[wasm_bindgen(start)]
pub fn startListeners(){
	console::log_1(&"Adding Javascript Listeners..".into());
	let dom = get_dom();

	let msg_form = dom.get_element_by_id("msgForm").unwrap();
	createListener(msg_form, "submit", "value", "msgInput", true, true);

	let group_form = dom.get_element_by_id("groupForm").unwrap();
	createListener(group_form, "submit", "value", "groupInput", true, false);

	let allow_trust_button = dom.get_element_by_id("allowTrust").unwrap();
	createListener(allow_trust_button, "click", "value", "allowTrust", false, false);

	let is_encrypting_button = dom.get_element_by_id("isEncrypting").unwrap();
	createListener(is_encrypting_button, "click", "checked", "isEncrypting", false, false);
}

#[wasm_bindgen]
pub fn createState(name:&str) -> String{
	console::log_1(&"Displaying Name..".into());
	let dom:Document = get_dom();
	dom.get_element_by_id("name").unwrap().set_inner_html(name);

	console::log_1(&format!("Initializing State for {}..", name).into());
	let state = Crypto::new(name, DEVICE_ID, RANDOM_NUMBER);
	console::log_1(&"Returning State..".into());
	
	return serde_json::to_string(&state).unwrap();
}

#[wasm_bindgen]
pub fn handleEvent(_state:&str, send_msg:Function, event_name:&str, value:&str) -> String{
	console::log_1(&"Handling Event..".into());
	let mut state:encrypter::Crypto = serde_json::from_str(_state).unwrap();

	match event_name.split('-').next().unwrap() {
		"msg" => {
			console::log_1(&format!("Recieved Msg {}", &value).into());
			let msg = ServerMsg::fromServer(value);
			msg.handleSelf(&mut state);
			display_msg(msg, &state);
		},
		"msgInput" => {
			console::log_1(&"Sent Message".into());
			let content = match state.is_encrypting {
				true => {
					console::log_1(&value.into());
					console::log_1(&state.encrypt(value.to_string()).into());
					MsgContent::SecureText(state.encrypt(value.to_string()))
				},
				false => MsgContent::InsecureText(value.to_string())
			};
			let msg =  ServerMsg::new(&state.addr(), content);
			send_msg.call1(&JsValue::null(), &msg.toWritable().into());
		},
		"groupInput" => {
			console::log_1(&"Group Change".into());
			let content = MsgContent::Join(value.to_string());
			let msg =  ServerMsg::new(&state.addr(), content);
			send_msg.call1(&JsValue::null(), &msg.toWritable().into());
		},
		"isEncrypting" => {
			console::log_1(&format!("isEncrypting Changed to {}", value).into());
			state.is_encrypting = match value {
				"true" => true,
				_ => false
			};
		},
		"allowTrust" => {
			console::log_1(&"allowTrust Clicked".into());
			let content = MsgContent::PublicKey(state.public_key());
			let msg =  ServerMsg::new(&state.addr(), content);
			send_msg.call1(&JsValue::null(), &msg.toWritable().into());
		},
		"clickName" =>{
			console::log_1(&format!("Clicked on user {}", value).into());
			let content = match state.trust(value.to_string()) {
				Some(forein) => Some(MsgContent::Trust(forein.clone())),
				None => None
			};
			// let content = match state.person(value.to_string()) {
			// 	Some(person) => Some(MsgContent::Trust(person.address.clone())),
			// 	None => None
			// };
			if content.is_some() {
				let msg = ServerMsg::new(&state.addr(), content.unwrap());
				send_msg.call1(&JsValue::null(), &msg.toWritable().into());
			}
		}
		_ => console::log_1(&"Unknown Event Occured".into()),
	}
	
	return serde_json::to_string(&state).unwrap();
}

fn get_dom() -> Document {
	return window().unwrap().document().unwrap();
}

fn createListener(ele:Element, event_name:&str, property:&str, input_name:&str, prevent_default:bool, is_resetting:bool){
	let func_body = format!("
		let id = '{}';
		let input = document.getElementById(id);
		let value = input.{};
		if({}){{
			e.preventDefault();
		}}
		if (value != undefined) {{
			window.rustState = window.rust.handleEvent(window.rustState, window.sendToServer, id, ''+value);
			if({}){{
				input.value = '';
			}};
		}}
	", input_name, property, prevent_default.to_string(), is_resetting.to_string());
	let add_event_result = Function::new_with_args("e", &func_body);
	ele.add_event_listener_with_callback(event_name, &add_event_result);
}
fn display_msg(msg:ServerMsg, state:&Crypto){
	let dom = get_dom();
	let display_msg = msg.display(state);
	console::log_1(&display_msg.clone().into());
	let func_body = format!("
		let item = document.createElement('li');
		item.innerHTML = '{}';
		item.addEventListener('click', (event) => {{
			let name = '{}';
			console.log('name is '+ name);
			window.rustState = window.rust.handleEvent(window.rustState, window.sendToServer, 'clickName', name);
			
			// if(rust.getRelation(state, name) == 'allowedTrust'){{
			// 	state = rust.handleTrust(state, name);
			// 	sendToServer(rust.onTrust(state, name));
			// }}
		}});
		document.getElementById('messages').prepend(item);
		window.scrollTo(0, document.body.scrollHeight);
	", display_msg, msg.from.name);
	let display_func = Function::new_no_args(&func_body);
	display_func.call0(&JsValue::null());
}

/*
function here are defiobjectned such that the starting keyword tells you what the return
onExample, returns a message to send
handleExample, returns a modified state
getExample, returns a string containing something to display to the user
*/

// #[wasm_bindgen]
// pub fn onJoin(state:Crypto, group:&str) -> String{
// 	let content = MsgContent::Join(group.to_string());
// 	let msg =  ServerMsg::new(&state.addr(), content);
// 	return msg.toWritable();
// 	let msg =  ServerMsg::new(&state.addr(), content);
// 	return msg.toWritable();
// }
// #[wasm_bindgen]
// pub fn onAllowTrust(state:Crypto) -> String{
// 	let content = MsgContent::PublicKey(state.publicKey());
// 	let msg =  ServerMsg::new(&state.addr(), content);
// 	return msg.toWritable();
// }

// #[wasm_bindgen]
// pub fn onBroadcast(state:Crypto, text:&str) -> String{
// 	let content = MsgContent::InsecureText(text.to_string());
// 	let msg =  ServerMsg::new(&state.addr(), content);
// 	return msg.toWritable();
// }
// #[wasm_bindgen]
// pub fn onSend(state:Crypto, text:&str) -> String{
// 	let state:Crypto = serde_json::from_str(_state).unwrap();
// 	let content = MsgContent::SecureText(state.encrypt(text.to_string()));
// 	let msg =  ServerMsg::new(&state.addr(), content);
// 	return msg.toWritable();
// }
// #[wasm_bindgen]
// pub fn onTrust(state:Crypto, name:&str) -> String{
// 	let state:encrypter::Crypto = serde_json::from_str(_state).unwrap();
// 	let content = match state.person(name.to_string()) {
// 		Some(person) => Some(MsgContent::Trust(person.address.clone())),
// 		None => None
// 	};
// 	return match content {
// 		Some(x) => ServerMsg::new(&state.addr(), x).toWritable(),
// 		None => "".to_string()
// 	};
// }
// #[wasm_bindgen]
// pub fn getList(_state:&str) -> String{
// 	let state:Crypto = serde_json::from_str(_state).unwrap();
// 	return state.listPeople();
// }
// #[wasm_bindgen]
// pub fn getDisplay(_state:&str, msg:&str) -> String{
// 	let state:Crypto = serde_json::from_str(_state).unwrap();
// 	return ServerMsg::fromServer(msg).display(&state);
// }
// #[wasm_bindgen]
// pub fn getRelation(_state:&str, name:&str) -> String{
// 	let state:Crypto = serde_json::from_str(_state).unwrap();
// 	return state.relation(name.to_string());
// }

// #[wasm_bindgen]
// pub fn handleIncoming(_state: &str, msg:&str) -> String{
// 	let mut state:encrypter::Crypto = serde_json::from_str(_state).unwrap();
// 	ServerMsg::fromServer(msg).handleSelf(&mut state);
// 	return serde_json::to_string(&state).unwrap();
// }
// #[wasm_bindgen]
// pub fn handleTrust(_state: &str, name:&str) -> String{
// 	let mut state:Crypto = serde_json::from_str(_state).unwrap();
	
// 	return match state.trust(name.to_string()) {
// 		Some(_) => serde_json::to_string(&state).unwrap(),
// 		None => "".to_string()
// 	};
// }