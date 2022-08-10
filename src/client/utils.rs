use std::{io, str};
use base64;

//const FILE_EXT:&str = "keys";

pub fn getUserIn(prompt:String) -> String{
	let mut line = String::new();
	println!("{}", prompt);
	io::stdin().read_line(&mut line).unwrap();
	return line.replace("\r", "").replace("\n", "");
}
#[derive(Clone)]
pub struct ConnectionData{
	pub server:String,
	pub group:String,
	pub name:String,
	pub password:String
}
impl ConnectionData{
	pub fn get_ip(&self) -> String {
		return format!("http://{}/ws/", self.server)
	}
	// pub fn get_fileName(&self) -> String {
	// 	return format!("{}@{}.{}", self.name, self.get_storeName(), FILE_EXT);
	// }
}
pub fn decodeBase64(inTxt:&str) -> String{
	return str::from_utf8(base64::decode(inTxt).unwrap().as_slice()).unwrap().to_string()
}
#[derive(Clone, Debug)]
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