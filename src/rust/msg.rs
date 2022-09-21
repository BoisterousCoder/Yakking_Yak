use crate::encrypter::Crypto;
//use crate::log;
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
pub struct YakMsg{
	pub from:Address,
	pub content:MsgContent
}
impl YakMsg{
	pub fn new(from:&Address, content:MsgContent) -> YakMsg{
		return YakMsg{
			from: from.clone(),
			content: content
		}
	}
	pub fn fromServer(txt:&str) -> YakMsg{
		//let txt = str::from_utf8(data).unwrap();
		//println!("{}", txt); //Uncomment if you want to see raw data
		let segments: Vec<&str> = splitAndClean(txt, '*');
		let addrSegments: Vec<&str> = splitAndClean(segments[0], '@');
		let contentData = decodeBase64(segments[2]);
		let name = decodeBase64(addrSegments[0]);
		let deviceId = addrSegments[1].parse().unwrap();
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
		return YakMsg{
			from: Address::new(&name, deviceId), 
			content: content
		}
	}
	pub fn toString(&self) -> String{
		let (kind, body):(&str, String) = match &self.content {
			MsgContent::PublicKey(publicKey) => (PUBLIC_KEY_LABEL, publicKey.to_string()),
			MsgContent::SecureText(text) => (SECURE_LABEL, text.to_string()),
			MsgContent::InsecureText(txt) => (INSECURE_LABEL, txt.to_string()),
			MsgContent::Join(group) => (JOIN_LABEL, group.to_string()),
			MsgContent::Leave(group) => (LEAVE_LABEL, group.to_string()),
			MsgContent::Trust(addr) => (TRUST_LABEL, addr.asSendable()),
			MsgContent::Blank() => (BLANK_LABEL, String::from("_"))
		};
		return format!("*{}*{}*{}*", self.from.asSendable(), kind, base64::encode(body.as_bytes()))
	}
	pub fn display(&self, state:&Crypto) -> String{
		let (content, label) = match &self.content {
			MsgContent::PublicKey(_) => ("is alllowing people to trust them".to_string(), PUBLIC_KEY_LABEL),
			MsgContent::SecureText(txt) => (state.decrypt(&self.from, txt.to_string()), SECURE_LABEL),
			MsgContent::InsecureText(txt) => (txt.to_string(), INSECURE_LABEL),
			MsgContent::Join(_) => ("went online".to_string(), JOIN_LABEL),
			MsgContent::Leave(_) => ("went offline".to_string(), LEAVE_LABEL),
			MsgContent::Trust(addr) => (format!("is trusting {}", addr.name), TRUST_LABEL),
			MsgContent::Blank() => ("Error Parsing Text".to_string(), BLANK_LABEL)
		};
		let relation = state.relation((&self.from.name).to_string());
		//TODO: add 
		return format!("<span class=\"{}\">*{}*\\{}</span> {}", relation, self.from.name, label, content.replace("\r", ""));
	}
	pub fn toWritable(self) -> String {
		self.toString()
	}
	pub fn handleSelf(&self, state:&mut Crypto){
		if self.from != state.addr(){
			if let MsgContent::PublicKey(data) = &self.content {
				state.addPublicKey(self.from.clone(), decodeToPublicKeyBytes(data.clone()));
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