
use std::str;
use base64;
use serde::{Deserialize, Serialize};


#[cfg(target_arch = "wasm32")]
use web_sys::console;

pub fn decode_base64(text:&str) -> String{
	log(&format!("Decoding base64 ({})", text));
	#[allow(deprecated)]
	return str::from_utf8(base64::decode(text).unwrap().as_slice()).unwrap().to_string()
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Address{
	pub name: String,
	pub device_id: [u8; 32]
}
impl Address{
	pub fn new(name:&str, device_id:[u8; 32]) -> Address{	
		return Address{
			name: name.to_string(),
			device_id
		}
	}
	#[allow(deprecated)]
	pub fn as_sendable(&self) -> String{
		format!("{}@{}", base64::encode(self.name.clone()), base64::encode(self.device_id))
	}
	pub fn from_sendable(s:String) -> Address{
		let addr_data:Vec<&str> = split_and_clean(&s, '@');
		#[allow(deprecated)]
		let device_byte_vec = base64::decode(addr_data[1]).unwrap();
		let mut device_id = [0u8; 32];
		let mut i = 0;
		for byte in device_byte_vec{
			device_id[i] = byte;
			i += 1;
			if i == device_id.len() {
				break;
			}
		}
		Address::new(&decode_base64(addr_data[0]), device_id)
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