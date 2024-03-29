#![cfg(target_arch = "wasm32")]
use std::string::String;

use web_sys::{window, Document, Element};
use wasm_bindgen::{prelude::*, JsValue};
use js_sys::Function;
use std::format;

mod client;
mod client_web;

use crate::client::store::Crypto;
use crate::client::serverhandlers::{ServerMsg, MsgContent};
use crate::client::utils::log;

const SEED:u64 = 1234567890; //TODO: fix the seed to its actually random
const PROXY_SEED:u64 = 0987654321; //TODO: fix the seed to its actually random
const DEVICE_ID:[u8; 32] = [1u8; 32];//TODO: Make this useful
const PASSWORD:&str = "ABCDE";

#[wasm_bindgen(start)]
pub fn startListeners(){
	log(&"Adding Javascript Listeners.." );
	let dom = get_dom();

	let msg_form = dom.get_element_by_id("msgForm").unwrap();
	createListener(msg_form, "submit", "value", "msgInput", true, true);

	let group_form = dom.get_element_by_id("groupForm").unwrap();
	createListener(group_form, "submit", "value", "groupInput", true, false);

	let is_encrypting_button = dom.get_element_by_id("isEncrypting").unwrap();
	createListener(is_encrypting_button, "click", "checked", "isEncrypting", false, false);
}

#[wasm_bindgen]
pub fn createState(name:&str) -> String{
	log(&"Displaying Name.." );
	let dom:Document = get_dom();
	dom.get_element_by_id("name").unwrap().set_inner_html(name);

	log(&format!("Initializing State for {}..", name) );
	let state = Crypto::new(name, PASSWORD, DEVICE_ID, SEED, PROXY_SEED);
	log("Returning State.." );
	
	return serde_json::to_string(&state).unwrap();
}

#[wasm_bindgen]
pub fn handleEvent(_state:&str, send_msg:Function, event_name:&str, value:&str) -> String{
	log("Handling Event.." );
	let mut state:Crypto = serde_json::from_str(_state).unwrap();

	match event_name.split('-').next().unwrap() {
		"msg" => {
			log(&format!("Recieved Msg {}", &value) );
			if let Some(msg) = ServerMsg::from_server(value, &mut state){
				update_msgs(&state);

				if let MsgContent::Join(_) = msg.content {
					let content_to_send = MsgContent::PublicKey(state.public_key());
					let msg_to_send = ServerMsg::new(&state.get_address(), content_to_send);
					send_msg.call1(&JsValue::null(), &msg_to_send.to_string(&state).into());
				}
			}
		},
		"msgInput" => {
			log("Sent Message");
			let content = match state.is_encrypting {
				true => {
					log(&value);
					// log(&state.encrypt(value.to_string()) );
					MsgContent::SecureText(state.encrypt(value.to_string()))
				},
				false => MsgContent::InsecureText(value.to_string())
			};
			let msg =  ServerMsg::new(&state.get_address(), content);
			send_msg.call1(&JsValue::null(), &msg.to_string(&state).into());
		},
		"groupInput" => {
			log("Group Change" );
			let content = MsgContent::Join(value.to_string());
			let msg =  ServerMsg::new(&state.get_address(), content);
			send_msg.call1(&JsValue::null(), &msg.to_string(&state).into());
			state.new_group(SEED, PROXY_SEED);
		},
		"isEncrypting" => {
			log(&format!("isEncrypting Changed to {}", value) );
			state.is_encrypting = match value {
				"true" => true,
				_ => false
			};
		},
		"clickName" =>{
			log(&format!("Clicked on user {}", value) );
			let content = match state.trust(value.to_string()) {
				Some(forein) => Some(MsgContent::Trust(forein.clone())),
				None => None
			};
			if content.is_some() {
				let msg = ServerMsg::new(&state.get_address(), content.unwrap());
				send_msg.call1(&JsValue::null(), &msg.to_string(&state).into());
			}
			log(&serde_json::to_string(&state).unwrap());
		}
		_ => log("Unknown Event Occured"),
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
fn update_msgs(state:&Crypto){
	let func_body = "
		document.getElementById('messages').innerHTML = '';
	";
	let empty_func = Function::new_no_args(&func_body);
	empty_func.call0(&JsValue::null());

	for msg in state.get_msgs(){
        display_msg(&msg, state);
    }
}
fn display_msg(msg:&ServerMsg, state:&Crypto){
	if let Some(display_msg) = msg.display(state){
		log(&display_msg.clone() );
		let func_body = format!("
			let item = document.createElement('li');
			item.innerHTML = '{}';
			item.addEventListener('click', (event) => {{
				let name = '{}';
				console.log('name is '+ name);
				window.rustState = window.rust.handleEvent(window.rustState, window.sendToServer, 'clickName', name);
			}});
			document.getElementById('messages').prepend(item);
			window.scrollTo(0, document.body.scrollHeight);
		", display_msg, msg.from.name);
		let display_func = Function::new_no_args(&func_body);
		display_func.call0(&JsValue::null());
	}
}