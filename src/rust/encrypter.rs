use crate::utils::{Address, splitAndClean};
use std::str;
use crate::KeyBundle::{KeyBundle, SecretKey};
use x25519_dalek::PublicKey;
use serde::{Serialize, Deserialize};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce // Or `Aes128Gcm`
};

//use crate::store::{attemptFetchIdData, storeID, RawIdData};

//const NONCE_MSG:&str = "I like cheese cake";

#[derive(Serialize, Deserialize)]
pub struct Crypto{
	selfData:KeyBundle,
	otherPeople: Vec<KeyBundle>
}
impl Crypto{
	pub fn new(name:&str, deviceId:i32, randNum:u64) -> Crypto{
		return Crypto{
			selfData: KeyBundle::newSelfKeySet(Address::new(name, deviceId), randNum),
			otherPeople: vec!(),
		}
	}
	pub fn addPublicKey(&mut self, addr:Address, publicKeyData:[u8; 32]) {
		let publicKey = PublicKey::from(publicKeyData);
		self.otherPeople.push(KeyBundle{
			address:addr, 
			secret: SecretKey::Empty(),
			publicKey:publicKey
		});
	}
	pub fn addr(&self) -> Address{
		Address::new(&self.selfData.address.name, self.selfData.address.deviceId)
	}
	pub fn trust(&mut self, name:String) -> Option<&Address>{
		for mut person in &mut self.otherPeople {
			if person.address.name.eq(&name){
				person.secret = match &self.selfData.secret {
					SecretKey::Ephemeral(secret) => SecretKey::Shared(secret.diffie_hellman(&person.publicKey)),
					SecretKey::Empty() => SecretKey::Empty(), //this code should never be called
					SecretKey::Shared(_) => SecretKey::Empty() //this code should never be called
				};
				return Some(&person.address);
			}
		}
		None
	}
	pub fn person(&self, name:String) -> Option<&KeyBundle>{
		for person in &self.otherPeople {
			if person.address.name.eq(&name){
				return Some(person);
			}
		}
		return None;
	}
	pub fn listPeople(&self) -> String{
		let mut s = String::new();
		for person in &self.otherPeople {
			s = format!("{}{}@{},\n", s, person.address.name, person.address.deviceId);
		}
		return s;
	}
	pub fn encrypt(&self, text:String) -> String {
		let mut encryptedText:String = "".to_string();
		for person in &self.otherPeople {
			if let SecretKey::Shared(secret) = &person.secret{
				let key = secret.as_bytes().clone();
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
			if address.name == self.selfData.address.name {
				for person in &self.otherPeople{
					if person.address.name == from.name {
						if let SecretKey::Shared(secret) = &person.secret{
							let key = secret.as_bytes();
							return crypt(&key, addressedMsgSplit[1], false);
						}
						return "has sent a secure message but you cannot read it as you do not trust them".to_string();
					}
				}
			}
		}
		"has sent a message secure but does not trust you".to_string()
	}
	pub fn publicKey(&self) -> String{
		return base64::encode(self.selfData.publicKey.as_bytes());
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