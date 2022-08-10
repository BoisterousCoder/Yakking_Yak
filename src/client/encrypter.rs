//use std::time::SystemTime;
//use base64;
use crate::utils::{ConnectionData, decodeBase64, Address, splitAndClean};
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};
//use crate::store::{attemptFetchIdData, storeID, RawIdData};

const DEVICE_ID:i32 = 12345;

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
			if(person.isTrusting()){
				//TODO: Encrypt the text with the shared key
				encryptedText += &format!("{}.{};", person.address.asSendable(), base64::encode(text.clone()));
			}
		}
		return encryptedText;
	}
	pub fn decrypt(&self, from:&Address, addressedMsgData:String) -> String {
		let addressedMsgs:Vec<&str> = splitAndClean(&addressedMsgData, ';');
		for addressedMsg in addressedMsgs{
			let addressedMsgSplit:Vec<&str> = splitAndClean(addressedMsg, '.');
			let address = Address::fromSendable(addressedMsgSplit[0].to_string());
			if(address.name == self.connData.name){
				for person in &self.otherPeople{
					if(person.address.name == from.name){
						if person.isTrusting(){
							//TODO Actually decrypt the data here
							return decodeBase64(addressedMsgSplit[1]);
						}
						return "has sent a secure message but you cannot read it as you do not trust them".to_string();
					}
				}
			}
		}
		"has sent a message secure but does not trust you".to_string()
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