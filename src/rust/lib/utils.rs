
use std::str;
use base64;
use serde::{Deserialize, Serialize};


#[cfg(target_arch = "wasm32")]
use web_sys::console;

pub fn decodeBase64(text:&str) -> String{
	log(&format!("Decoding base64 ({})", text));
	return str::from_utf8(base64::decode(text).unwrap().as_slice()).unwrap().to_string()
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Address{
	pub name: String,
	pub device_id: i32
}
impl Address{
	pub fn new(name:&str, device_id:i32) -> Address{	
		return Address{
			name: name.to_string(),
			device_id
		}
	}
	pub fn as_sendable(&self) -> String{
		format!("{}@{}", base64::encode(self.name.clone()), self.device_id)
	}
	pub fn from_sendable(s:String) -> Address{
		let addr_data:Vec<&str> = split_and_clean(&s, '@');
		Address::new(&decodeBase64(addr_data[0]), addr_data[1].parse().unwrap())
	}
}
pub fn split_and_clean(text:&str, split:char) -> Vec<&str>{
	text.split(split)
		.map(|seg| seg.trim())//remove whitespace
		.filter(|seg| !seg.is_empty())//remove empty segments
		.collect()
}
pub fn log(text:&str){
	#[cfg(target_arch = "wasm32")]
	console::log_1(&text.to_string().into());
	#[cfg(not(target_arch = "wasm32"))]
	println!("Logged: {}", text);
}