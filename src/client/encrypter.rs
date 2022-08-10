//use std::time::SystemTime;
//use base64;
use crate::utils::{ConnectionData, Address, splitAndClean};
use rand_core::OsRng;
use std::str;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce // Or `Aes128Gcm`
};

//use crate::store::{attemptFetchIdData, storeID, RawIdData};

const DEVICE_ID:i32 = 12345;
//const NONCE_MSG:&str = "I like cheese cake";

pub struct Crypto{
	pub connData: ConnectionData,
	pub publicKey: PublicKey,
	otherPeople: Vec<KeyBundle>,
	secretKey:EphemeralSecret,
	deviceId: i32
}
impl Crypto{
	pub fn new(connData:ConnectionData) -> Crypto{
		
		let secretKey = EphemeralSecret::new(OsRng);
		//println!("{}", secretKey.as_bytes());
		return Crypto{
			connData: connData,
			publicKey: PublicKey::from(&secretKey),
			secretKey: secretKey,
			otherPeople: vec!(),
			deviceId: DEVICE_ID
		}
	}
	pub fn addPublicKey(&mut self, addr:Address, publicKeyData:[u8; 32]) {
		let publicKey = PublicKey::from(publicKeyData);
		self.otherPeople.push(KeyBundle{
			address:addr, 
			sharedSecret: None,
			publicKey:publicKey
		});
	}
	pub fn addr(&self) -> Address{
		Address::new(&self.connData.name, self.deviceId)
	}
	pub fn trust(&mut self, name:String) -> Option<&Address>{
		for mut person in &mut self.otherPeople {
			if person.address.name.eq(&name){
				person.sharedSecret = Some(self.secretKey.diffie_hellman(&person.publicKey));
				return Some(&person.address);
			}
		}
		None
	}
	pub fn isTrusting(&self) -> bool{
		for person in &self.otherPeople {
			if person.isTrusting(){
				return true;
			}
		}
		return false;
	}
	pub fn listPeople(&self){
		for person in &self.otherPeople {
			println!("{}@{}", person.address.name, person.address.deviceId);
		}
	}
	pub fn encrypt(&self, text:String) -> String {
		let mut encryptedText:String = "".to_string();
		for person in &self.otherPeople {
			if person.isTrusting() {
				//TODO: Encrypt the text with the shared key
				let key = person.sharedSecret.as_ref().unwrap().as_bytes().clone();
				encryptedText += &format!("{}.{};", person.address.asSendable(), crypt(&key, &text, true));
			}
		}
		return encryptedText;
	}
	pub fn decrypt(&self, from:&Address, addressedMsgData:String) -> String {
		let addressedMsgs:Vec<&str> = splitAndClean(&addressedMsgData, ';');
		for addressedMsg in addressedMsgs{
			let addressedMsgSplit:Vec<&str> = splitAndClean(addressedMsg, '.');
			let address = Address::fromSendable(addressedMsgSplit[0].to_string());
			if address.name == self.connData.name {
				for person in &self.otherPeople{
					if person.address.name == from.name {
						if person.isTrusting(){
							//TODO Actually decrypt the data here
							let key = person.sharedSecret.as_ref().unwrap().as_bytes().clone();
							return crypt(&key, addressedMsgSplit[1], false);
						}
						return "has sent a secure message but you cannot read it as you do not trust them".to_string();
					}
				}
			}
		}
		"has sent a message secure but does not trust you".to_string()
	}
}

fn crypt(key:&[u8; 32], data:&str, isEncrypting:bool) -> String{
	let cipher = Aes256Gcm::new_from_slice(key).unwrap();
	//let nonce = Nonce::from_slice(NONCE_MSG.as_bytes().as_ref());
	let nonce = Nonce::from_slice(b"unique nonce");
	let dataBytes = data.as_bytes().as_ref();
	return if isEncrypting {
		base64::encode(cipher.encrypt(nonce, dataBytes).unwrap())
	}else{
		let bytes = base64::decode(dataBytes).unwrap();
		let resBytes = cipher.decrypt(nonce, bytes.as_ref()).unwrap();
		str::from_utf8(&resBytes).unwrap().to_string()
	}
}

pub struct KeyBundle{
	pub publicKey: PublicKey,
	pub sharedSecret: Option<SharedSecret>,
	pub address: Address
}
impl KeyBundle{
	pub fn isTrusting(&self) -> bool{
		return self.sharedSecret.is_some();
	}
}