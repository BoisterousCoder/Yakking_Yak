use crate::encrypter::Crypto;
use crate::utils::{decodeBase64, Address, splitAndClean};
use std::str;
use std::convert::TryInto;
use base64;

const INSECURE_LABEL:&str = "i";
const SECURE_LABEL:&str = "s";
const JOIN_LABEL:&str = "j";
const LEAVE_LABEL:&str = "l";
const PUBLIC_KEY_LABEL:&str = "p";
const TRUST_LABEL:&str = "t";
const BLANK_LABEL:&str = "_";

#[derive(Clone, Debug)]
pub enum MsgContent{
	InsecureText(String),
	SecureText(String),
	Join(String),
	PublicKey(String),
	Leave(String),
	Trust(Address),
	Blank()
}

#[derive(Clone, Debug)]
pub struct ServerMsg{
	pub from:Address,
	pub content:MsgContent
}
impl ServerMsg{
	pub fn new(from:&Address, content:MsgContent) -> ServerMsg{
		return ServerMsg{
			from: from.clone(),
			content
		}
	}
	pub fn fromServer(txt:&str) -> ServerMsg{
		//let txt = str::from_utf8(data).unwrap();
		//println!("{}", txt); //Uncomment if you want to see raw data
		let segments: Vec<&str> = splitAndClean(txt, '*');
		let addrSegments: Vec<&str> = splitAndClean(segments[0], '@');
		let contentData = decodeBase64(segments[2]);
		let name = decodeBase64(addrSegments[0]);
		let device_id = addrSegments[1].parse().unwrap();
		let content = match segments[1] {
			INSECURE_LABEL => MsgContent::InsecureText(contentData),
			SECURE_LABEL => MsgContent::SecureText(contentData),
			JOIN_LABEL => MsgContent::Join(contentData),
			PUBLIC_KEY_LABEL => MsgContent::PublicKey(contentData),
			TRUST_LABEL => MsgContent::Trust(Address::fromSendable(contentData)),
			LEAVE_LABEL => MsgContent::Leave(contentData),
			BLANK_LABEL => MsgContent::Blank(),
			&_ => MsgContent::Blank()
		};
		return ServerMsg{
			from: Address::new(&name, device_id), 
			content
		}
	}
	pub fn toString(&self) -> String{
		let (kind, body):(&str, String) = match &self.content {
			MsgContent::PublicKey(public_key) => (PUBLIC_KEY_LABEL, public_key.to_string()),
			MsgContent::SecureText(text) => (SECURE_LABEL, text.to_string()),
			MsgContent::InsecureText(txt) => (INSECURE_LABEL, txt.to_string()),
			MsgContent::Join(group) => (JOIN_LABEL, group.to_string()),
			MsgContent::Leave(group) => (LEAVE_LABEL, group.to_string()),
			MsgContent::Trust(addr) => (TRUST_LABEL, addr.asSendable()),
			MsgContent::Blank() => (BLANK_LABEL, String::from("_"))
		};
		return format!("*{}*{}*{}*", self.from.asSendable(), kind, base64::encode(body.as_bytes()))
	}
	pub fn display(&self, state:&Crypto) -> Option<String>{
		let msg_data = match &self.content {
			MsgContent::PublicKey(pub_key) => {
				if state.person_from_pub_key(pub_key).is_some() {
					None
				}else if self.from != state.get_address(){
					Some(("is alllowing people to trust them".to_string(), PUBLIC_KEY_LABEL))
				}else{
					None
				}
			},
			MsgContent::SecureText(txt) => {
				if self.from == state.get_address() {
					None
				}else {
					Some((state.decrypt(&self.from, txt.to_string()), SECURE_LABEL))
				}
			},
			MsgContent::InsecureText(txt) => Some((txt.to_string(), INSECURE_LABEL)),
			MsgContent::Join(_) => Some(("went online".to_string(), JOIN_LABEL)),
			MsgContent::Leave(_) => Some(("went offline".to_string(), LEAVE_LABEL)),
			MsgContent::Trust(addr) => {
				let relation = state.relation(&addr);
				Some((format!("is trusting <span class=\"{}\">{}</span>", relation, addr.name), TRUST_LABEL))
			},
			MsgContent::Blank() => Some(("Error Parsing Text".to_string(), BLANK_LABEL))
		};
		return match msg_data {
			Some((content, label)) => {
				let relation = state.relation(&self.from);
				Some(format!("<span class=\"{}\">({}) {}</span> {}", relation, label, self.from.name, content.replace("\r", "")))
			},
			None => None
		}
		
	}
	pub fn toWritable(self) -> String {
		self.toString()
	}
	pub fn handleSelf(&self, state:&mut Crypto){
		if self.from != state.addr(){
			if let MsgContent::PublicKey(data) = &self.content {
				state.add_public_key(self.from.clone(), decodeToPublicKeyBytes(data.clone()));
			}
		}
	}
}

fn decodeToPublicKeyBytes(s:String) -> [u8; 32]{
	let data = base64::decode(s).unwrap();
	let slice = data.as_slice();
	return match slice.try_into() {
        Ok(bytes) => bytes,
        Err(_) => panic!("Expected a Vec of length {} but it was {}", 32, data.len()),
    };
}