use crate::utils::{Address, split_and_clean, log};
use crate::ratchet::Ratchet;
use crate::ForeinAgent::ForeinAgent;
use std::str;
use std::collections::HashMap;
use crate::KeyBundle::{KeyBundle, SecretKey};
use web_sys::console;
use x25519_dalek::PublicKey;
use serde::{Serialize, Deserialize};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};

const SALT_STRING:&str = "This is a temporary salt until I figure out what to put here";

#[derive(Serialize, Deserialize)]
pub struct Crypto{
	pub is_encrypting:bool,
	self_data:KeyBundle,
	agents: Vec<ForeinAgent>,
	//ratchets: HashMap<[u8; 32], (Ratchet, Ratchet)>
}
impl Crypto{
	pub fn new(name:&str, deviceId:i32, randNum:u64) -> Crypto{
		return Crypto{
			self_data: KeyBundle::newSelfKeySet(Address::new(name, deviceId), randNum),
			agents: vec!(),
			//ratchets: HashMap::new(),
			is_encrypting: false
		}
	}
	pub fn add_public_key(&mut self, addr:Address, public_key_data:[u8; 32]) -> bool{
		let public_key = PublicKey::from(public_key_data);
		for agent in &self.agents {
			if &agent.keys.public_key == &public_key || &agent.keys.address == &addr{
				return false;
			}
		}
		self.agents.push(ForeinAgent{
			to_ratchet:None,
			from_ratchet:None,
			keys:KeyBundle{
				address:addr, 
				secret: SecretKey::Empty(),
				public_key
			}
		});
		return true;
	}
	//TODO:Make this address and not name
	pub fn trust(&mut self, name:String) -> Option<&Address>{
		for mut agent in &mut self.agents {
			if agent.keys.address.name.eq(&name){
				match &self.self_data.secret {
					SecretKey::Ephemeral(secret) => {
						let shared_secret = secret.diffie_hellman(&agent.keys.public_key);
						let salt = SALT_STRING.as_bytes().to_vec();
						agent.to_ratchet = Some(Ratchet::new(&shared_secret, true, salt.clone()));
						agent.from_ratchet = Some(Ratchet::new(&shared_secret, false, salt));

						//self.ratchets.insert(shared_secret.as_bytes().clone(), (to_rachet, from_rachet));
						agent.keys.secret = SecretKey::Shared(shared_secret);
					},
					SecretKey::Empty() => panic!("This code should never be called! You must have an Ephemeral secret key."),
					SecretKey::Shared(_) => panic!("This code should never be called! You must have an Ephemeral secret key.")
				};
				
				return Some(&agent.keys.address);
			}
		}
		None
	}
	fn keys(&self, forien:&Address) -> Option<&KeyBundle>{
		if forien.eq(&self.self_data.address) {
			return Some(&self.self_data);
		}
		for agent in &self.agents {
			if agent.keys.address.eq(&forien){
				return Some(&agent.keys);
			}
		}
		return None;
	}	
	pub fn agent_from_pub_key(&self, key:&str) -> Option<&ForeinAgent>{
		// if self.public_key() == key.to_string() {
			// return Some(&self.self_data);
		// }else {
		for agent in &self.agents {
			if base64::encode(agent.keys.public_key.as_bytes()) == key.to_string(){
				return Some(agent);
			}
		}
		return None;
		// }
	}
	pub fn relation(&self, forien:&Address) -> String {
		return match self.keys(forien) {
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
		for agent in &self.agents {
			s = format!("{}{}@{},\n", s, agent.keys.address.name, agent.keys.address.deviceId);
		}
		return s;
	}
	pub fn encrypt(&mut self, text:String) -> String {
		let mut encrypted_text:String = "".to_string();
		for agent in &mut self.agents {
			if let SecretKey::Shared(secret) = &agent.keys.secret{
				let key = secret.as_bytes();
				if let Some(ratchet) = &mut agent.to_ratchet{
					let msg_id = ratchet.len();
					let payload = ratchet.process_payload(msg_id, &text);
					encrypted_text += &format!("{}*{}*{};", agent.keys.address.asSendable(), msg_id, payload);
				}
			}
		}
		// log(&encrypted_text);
		return encrypted_text;
	}
	pub fn decrypt(&mut self, from:&Address, addressed_msg_data:String) -> String {
		let addressed_msgs:Vec<&str> = split_and_clean(&addressed_msg_data, ';');
		for addressed_msg in addressed_msgs{
			let addressed_msg_split:Vec<&str> = split_and_clean(addressed_msg, '*');
			let address = Address::fromSendable(addressed_msg_split[0].to_string());
			let msg_id = addressed_msg_split[1].parse::<usize>().expect("Message ID is not an unsigned int!");
			// log(&addressed_msg);
			if address == self.self_data.address {
				for agent in &mut self.agents{
					if &agent.keys.address == from {
						if let SecretKey::Shared(secret) = &mut agent.keys.secret{
							let key = secret.as_bytes();
							if let Some(ratchet) = &mut agent.from_ratchet{
								return ratchet.process_payload(msg_id, addressed_msg_split[2])
							}
						}
						return "has sent a secure message but you cannot read it as you do not trust them".to_string();
					}
				}
			}
		}
		"has sent a secure message but does not trust you".to_string()
	}
	pub fn public_key(&self) -> String{
		return base64::encode(self.self_data.public_key.as_bytes());
	}
	pub fn get_address(&self) -> Address{
		return self.self_data.address.clone();
	}
}