
use std::{str, collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};
use base64;
use k256::ecdsa::{Signature, VerifyingKey, signature::Verifier};
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
	pub public_key: Vec<u8>
}
impl Address{
	pub fn new(name:&str, public_key:Vec<u8>) -> Address{	
		return Address{
			name: name.to_string(),
			public_key
		}
	}
	#[allow(deprecated)]
	pub fn as_sendable(&self) -> String{
		format!("{}@{}", base64::encode(self.name.clone()), base64::encode(self.public_key.as_slice()))
	}
	pub fn from_sendable(s:String) -> Address{
		let addr_data:Vec<&str> = split_and_clean(&s, '@');
		#[allow(deprecated)]
		let key_bytes = base64::decode(addr_data[1]).unwrap();
		Address::new(&decode_base64(addr_data[0]), key_bytes)
	}
	pub fn name(&self) -> String{
		self.name.clone()
	}
	pub fn verify(&self, signature_str:&str, msg:&str)->bool{
		#[allow(deprecated)]
		let sig_bytes= base64::decode(signature_str).unwrap();
		let signature = Signature::from_bytes(sig_bytes.as_slice().into()).unwrap();
		let msg_bytes= msg.as_bytes();
		let key = VerifyingKey::from_sec1_bytes(&self.public_key).unwrap();
		return key.verify(msg_bytes, &signature).is_ok();
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

pub fn calc_hash(text:&str) -> String{
    let mut hasher = DefaultHasher::new();
    text.to_string().hash(&mut hasher);
	#[allow(deprecated)]
    base64::encode(hasher.finish().to_be_bytes())
}