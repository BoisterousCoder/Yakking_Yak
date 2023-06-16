use crate::utils::{Address, splitAndClean};
use std::str;
use crate::KeyBundle::{KeyBundle, SecretKey};
use web_sys::console;
use x25519_dalek::PublicKey;
use serde::{Serialize, Deserialize};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};

//const NONCE_MSG:&str = "I like cheese cake";

#[derive(Serialize, Deserialize)]
pub struct Crypto{
	pub is_encrypting:bool,
	self_data:KeyBundle,
	other_people: Vec<KeyBundle>
}
impl Crypto{
	pub fn new(name:&str, deviceId:i32, randNum:u64) -> Crypto{
		return Crypto{
			self_data: KeyBundle::newSelfKeySet(Address::new(name, deviceId), randNum),
			other_people: vec!(),
			is_encrypting: false
		}
	}
	pub fn add_public_key(&mut self, addr:Address, public_key_data:[u8; 32]) {
		let public_key = PublicKey::from(public_key_data);
		self.other_people.push(KeyBundle{
			address:addr, 
			secret: SecretKey::Empty(),
			public_key:public_key
		});
	}
	pub fn addr(&self) -> Address{
		Address::new(&self.self_data.address.name, self.self_data.address.deviceId)
	}
	pub fn trust(&mut self, name:String) -> Option<&Address>{
		for mut person in &mut self.other_people {
			if person.address.name.eq(&name){
				person.secret = match &self.self_data.secret {
					SecretKey::Ephemeral(secret) => SecretKey::Shared(secret.diffie_hellman(&person.public_key)),
					SecretKey::Empty() => SecretKey::Empty(), //this code should never be called
					SecretKey::Shared(_) => SecretKey::Empty() //this code should never be called
				};
				return Some(&person.address);
			}
		}
		None
	}
	pub fn person(&self, forien:&Address) -> Option<&KeyBundle>{
		if forien.eq(&self.self_data.address) {
			return Some(&self.self_data);
		}
		for person in &self.other_people {
			if person.address.eq(&forien){
				return Some(person);
			}
		}
		return None;
	}
	pub fn relation(&self, forien:&Address) -> String {
		return match self.person(forien) {
			Some(key_bundle) => {
				match key_bundle.secret {
					SecretKey::Ephemeral(_) => "self".to_string(),
					SecretKey::Shared(_) => "trusted".to_string(),
					SecretKey::Empty() => "allowedTrust".to_string()
				}
			},
			None => "unknown".to_string()
		};
	}
	pub fn list_people(&self) -> String{
		let mut s = String::new();
		for person in &self.other_people {
			s = format!("{}{}@{},\n", s, person.address.name, person.address.deviceId);
		}
		return s;
	}
	pub fn encrypt(&self, text:String) -> String {
		let mut encrypted_text:String = "".to_string();
		for person in &self.other_people {
			if let SecretKey::Shared(secret) = &person.secret{
				let key = secret.as_bytes().clone();
				encrypted_text += &format!("{}.{};", person.address.asSendable(), crypt(&key, &text, true));
			}
		}
		return encrypted_text;
	}
	pub fn decrypt(&self, from:&Address, addressed_msg_data:String) -> String {
		let addressed_msgs:Vec<&str> = splitAndClean(&addressed_msg_data, ';');
		for addressed_msg in addressed_msgs{
			let addressed_msgSplit:Vec<&str> = splitAndClean(addressed_msg, '.');
			let address = Address::fromSendable(addressed_msgSplit[0].to_string());
			if address.name == self.self_data.address.name {
				for person in &self.other_people{
					if person.address.name == from.name {
						if let SecretKey::Shared(secret) = &person.secret{
							let key = secret.as_bytes();
							return crypt(&key, addressed_msgSplit[1], false);
						}
						return "has sent a secure message but you cannot read it as you do not trust them".to_string();
					}
				}
			}
		}
		"has sent a message secure but does not trust you".to_string()
	}
	pub fn public_key(&self) -> String{
		return base64::encode(self.self_data.public_key.as_bytes());
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