// use magic_crypt::MagicCryptTrait;
// use magic_crypt::new_magic_crypt;
// use std::fs;
// use std::fs::File;
// use std::io::prelude::*;
// use base64;
// use serde::{Deserialize, Serialize};
// use crate::utils::*;

// #[derive(Serialize, Deserialize)]
// pub struct KeyPairData{
// 	pub public: String,
// 	pub private:String
// }
// #[derive(Serialize, Deserialize)]
// pub struct RawIdData{
// 	pub id: KeyPairData,
// 	pub reg: u32
// }

// pub fn attemptFetchIdData(connData:ConnectionData) -> Option<RawIdData>{
// 	if isStoreExisting(connData.get_fileName()) {
// 		let dataStr = read(connData);
// 		Some(serde_json::from_str(dataStr.as_str()).unwrap())
// 	}else {
// 		None
// 	}
// }

// pub fn storeID(connData:ConnectionData, registration:u32, publicKeyData:Vec<u8>, privateKeyData:Vec<u8>){
// 	let idData = RawIdData {
// 		id: KeyPairData{
// 			public: base64::encode(publicKeyData),
// 			private: base64::encode(privateKeyData)
// 		},
// 		reg: registration
// 	};
// 	write(connData, serde_json::to_string(&idData).unwrap());
// }

// fn isStoreExisting(file:String) -> bool{
// 	return fs::metadata(file).is_ok();
// }
// fn write(connData:ConnectionData, text:String){
// 	let mut file = File::create(connData.get_fileName()).unwrap();
// 	let key = new_magic_crypt!(connData.password, 256);
// 	let data = key.encrypt_str_to_base64(text);
	
// 	file.write_all(data.as_bytes());
// 	file.sync_all();
// }
// fn read(connData:ConnectionData) -> String{
// 	let mut file = File::open(connData.get_fileName()).unwrap();
// 	let mut data = String::new();
// 	file.read_to_string(&mut data);
// 	let key = new_magic_crypt!(connData.password, 256);

// 	return key.decrypt_base64_to_string(&data).unwrap();
// }