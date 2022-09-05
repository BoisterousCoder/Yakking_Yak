use std::str;
use base64;
use serde::{Deserialize, Serialize};

pub fn decodeBase64(inTxt:&str) -> String{
	return str::from_utf8(base64::decode(inTxt).unwrap().as_slice()).unwrap().to_string()
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Address{
	pub name: String,
	pub deviceId: i32
}
impl Address{
	pub fn new(name:&str, deviceId:i32) -> Address{	
		return Address{
			name: name.to_string(),
			deviceId: deviceId
		}
	}
	pub fn asSendable(&self) -> String{
		format!("{}@{}", base64::encode(self.name.clone()), self.deviceId)
	}
	pub fn fromSendable(s:String) -> Address{
		let addrData:Vec<&str> = splitAndClean(&s, '@');
		Address::new(&decodeBase64(addrData[0]), addrData[1].parse().unwrap())
	}
}
pub fn splitAndClean(text:&str, split:char) -> Vec<&str>{
	text.split(split)
		.map(|seg| seg.trim())//remove whitespace
		.filter(|seg| !seg.is_empty())//remove empty segments
		.collect()
}