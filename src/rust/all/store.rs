use std::str;

use x25519_dalek::PublicKey;
use serde::{Serialize, Deserialize};

#[cfg(not(target_arch = "wasm32"))]
use crate::gtk::save::GroupSave;
#[cfg(target_arch = "wasm32")]
use crate::web::save::GroupSave;

use super::utils::{Address, log, split_and_clean};
use super::ratchet::Ratchet;
use super::forein_agent::ForeinAgent;
use super::key_bundle::{KeyBundle, SecretKey};
use super::serverhandlers::{ServerMsg, SecureMsgIdentifier, MsgContent};

const SALT_STRING:&str = "This is a temporary salt until I figure out what to put here";

#[derive(Serialize, Deserialize)]
pub struct Crypto{
	pub is_encrypting:bool,
	pub password:String,
	self_data:KeyBundle,
	proxy_ratchet:Ratchet,
	agents: Vec<ForeinAgent>,
	msgs: Vec<ServerMsg>
}
impl Crypto{
	pub fn new(name:&str, password:&str, device_id:[u8; 32], seed:u64, proxy_seed:u64) -> Crypto{
		let self_addr = Address::new(name, device_id);
		let proxy = KeyBundle::new_self_key_set(self_addr.clone(), proxy_seed);
		if let SecretKey::Ephemeral(proxy_private) = proxy.secret{
			let self_data = KeyBundle::new_self_key_set(self_addr, seed);
			let shared_secret= proxy_private.diffie_hellman(&self_data.public_key);
			let salt = SALT_STRING.as_bytes().to_vec();
			let proxy_ratchet = Ratchet::new(&shared_secret, false, salt);

			let mut state = Crypto{
				self_data,
				password:password.to_string(),
				proxy_ratchet,
				agents: vec!(),
				is_encrypting: false,
				msgs: vec!()
			};
			state.add_public_key(proxy.address.clone(), proxy.public_key.as_bytes().clone());
			state.trust(proxy.address.name.to_string());

			return state;
		}else{
			panic!("Unreachable!!!");
		};
		
	}
	pub fn add_public_key(&mut self, addr:Address, public_key_data:[u8; 32]) -> bool{
		log(&format!("Adding Public key: {} -- {}", addr.name, public_key_data.len()));
		let public_key = PublicKey::from(public_key_data);
		for agent in &self.agents {
			if &agent.keys.address == &addr{
				log("Key already in list");
				return false;
			}
		}
		self.agents.push(ForeinAgent{
			to_ratchet:None,
			from_ratchet:None,
			is_online: true,
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

						agent.keys.secret = SecretKey::Shared(shared_secret);
					},
					_ => panic!("This code should never be called! You must have an Ephemeral secret key.")
				};
				
				return Some(&agent.keys.address);
			}
		}
		None
	}
	pub fn add_msg(&mut self, msg:ServerMsg){
		self.msgs.push(msg);
		self.msgs.sort_by(|a, b| a.time_stamp.cmp(&b.time_stamp))
	}
	pub fn get_msgs(&self) -> Vec<ServerMsg>{
		return self.msgs.clone();
	}
	pub fn new_group(&mut self, seed:u64, proxy_seed:u64){
		let addr = self.get_address();
		let new_state = Self::new(&addr.name, &self.password, addr.device_id, seed, proxy_seed);
		*self = new_state;
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
		for agent in &self.agents {
			#[allow(deprecated)]
			if base64::encode(agent.keys.public_key.as_bytes()) == key.to_string(){
				return Some(agent);
			}
		}
		return None;
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
	pub fn get_agents(&self) -> Vec<ForeinAgent>{
		return  self.agents.clone();
	}
	pub fn encrypt(&mut self, text:String) -> Vec<SecureMsgIdentifier> {
		let mut msg_ids = vec![];
		for agent in &mut self.agents {
			if let SecretKey::Shared(_) = &agent.keys.secret{
				if let Some(ratchet) = &mut agent.to_ratchet{
					let msg_id = ratchet.len();
					let payload_raw = text.as_bytes();
					msg_ids.push(ratchet.process_payload(&agent.keys.address, msg_id, &payload_raw));
				}
			}
		}
		return msg_ids;
	}
	pub fn decrypt(&mut self, from:&Address, addressed_msg_data:String) -> Option<SecureMsgIdentifier> {
		let addressed_msgs:Vec<&str> = split_and_clean(&addressed_msg_data, ';');
		for addressed_msg in addressed_msgs{
			let addressed_msg_split:Vec<&str> = split_and_clean(addressed_msg, '*');
			let address = Address::from_sendable(addressed_msg_split[0].to_string());
			let msg_id = addressed_msg_split[1].parse::<usize>().expect("Message ID is not an unsigned int!");
			#[allow(deprecated)]
			let payload = base64::decode(addressed_msg_split[2]).expect("recived invalid base64 data");

			log(&format!("Decrypting...\n Messages:{}\n My addr: {}\n From: {}\n To: {}",
				addressed_msg_data,
				self.get_address().as_sendable(),
				from.as_sendable(),
				address.as_sendable()
			));

			if from == &self.get_address() {
				log("Poxy message found!");
				return Some(self.proxy_ratchet.process_payload(&self.self_data.address, msg_id, payload.as_slice()));
			}else if address == self.self_data.address {
				for agent in &mut self.agents{
					if &agent.keys.address == from {
						if let SecretKey::Shared(_) = &mut agent.keys.secret{
							if let Some(ratchet) = &mut agent.from_ratchet{
								return Some(ratchet.process_payload(&agent.keys.address, msg_id, payload.as_slice()));
							}
						}
						return None;
					}
				}
			}
		}
		None
	}
	pub fn public_key(&self) -> String{
		#[allow(deprecated)]
		return base64::encode(self.self_data.public_key.as_bytes());
	}
	pub fn get_address(&self) -> Address{
		return self.self_data.address.clone();
	}
	pub fn get_encrypted_msg(&self, id:&SecureMsgIdentifier) -> Option<Vec<u8>>{
		for agent in &self.agents {
			if agent.keys.address.eq(&id.address) {
				log(&format!("Getting Encrypted msg.. \n to:{}\n from:{}\n is_sender:{}\n Msg ID:{}", 
					agent.keys.address.as_sendable(), 
					self.self_data.address.as_sendable(),
					agent.keys.address == self.self_data.address, 
					id.ord));
				return if agent.keys.address == self.self_data.address && !id.is_sender {
					Some(self.proxy_ratchet.get_msg(id.ord).expect("self message not decryped yet"))
				}else if id.is_sender{
					Some(agent.to_ratchet.as_ref().unwrap().get_msg(id.ord).expect("to message not decryped yet"))
				}else{
					Some(agent.from_ratchet.as_ref().unwrap().get_msg(id.ord).expect("from message not decryped yet"))
				};
			}
		}
		log("Could not find ratchet source!");
		None
	}
	pub fn group_as_save(&self, group_name:&str) -> GroupSave{
		let mut save = GroupSave{
			addr:self.self_data.address.clone(),
			group:group_name.to_string(),
    		proxy_ratchet:self.proxy_ratchet.clone(),
			agents: self.agents.clone(),
			msgs: self.msgs.clone()
		};
		for agent in &mut save.agents {
			agent.is_online = false;
		}
		save.msgs = save.msgs.iter().filter_map(|msg|{
			match msg.content {
				MsgContent::Join(_) => None,
				MsgContent::Leave(_) => None,
				_ => Some(msg.clone())
			}
		}).collect();
		return save;
	}
	pub fn load_group_save(&mut self, save:GroupSave){
		self.msgs = save.msgs;
		self.agents = save.agents;
		self.proxy_ratchet = save.proxy_ratchet;
	}
	pub fn set_is_online(&mut self, addr:&Address, is_online:bool){
		for agent in self.agents.iter_mut(){
			if &agent.keys.address == addr {
				agent.is_online = is_online;
			}
		}
	}
}