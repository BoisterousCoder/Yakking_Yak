use crate::encrypter::Crypto;
use crate::utils::{decodeBase64, Address, splitAndClean};
use std::str;
use actix::io::SinkWrite;
use std::convert::TryInto;
use awc::{
	ws::{Codec, Message},
	BoxedSocket
};
use actix_codec::Framed;
use futures::stream::SplitSink;
use x25519_dalek::PublicKey;
use base64;

const INSECURE_LABEL:&str = "i";
const SECURE_LABEL:&str = "s";
const JOIN_LABEL:&str = "j";
const LEAVE_LABEL:&str = "l";
const PUBLIC_KEY_LABEL:&str = "p";
const TRUST_LABEL:&str = "t";
const BLANK_LABEL:&str = "_";

pub fn onStart(websocket:&mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>, state:&mut Crypto){
	websocket.write(ServerMsg::new(&state.addr(), MsgContent::Join(state.connData.group.clone())).toWritable());
}

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
			content: content
		}
	}
	pub fn fromServer(data:&Vec<u8>) -> ServerMsg{
		let txt = str::from_utf8(data).unwrap();
		println!("{}", txt); //Uncomment if you want to see raw data
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
		return ServerMsg{
			from: Address::new(&name, deviceId), 
			content: content
		}
	}
	pub fn toString(self) -> String{
		let (kind, body):(&str, String) = match self.content {
			MsgContent::PublicKey(publicKey) => (PUBLIC_KEY_LABEL, publicKey),
			MsgContent::SecureText(text) => (SECURE_LABEL, text),
			MsgContent::InsecureText(txt) => (INSECURE_LABEL, txt),
			MsgContent::Join(group) => (JOIN_LABEL, group),
			MsgContent::Leave(group) => (LEAVE_LABEL, group),
			MsgContent::Trust(addr) => (TRUST_LABEL, addr.asSendable()),
			MsgContent::Blank() => (BLANK_LABEL, String::from("_"))
		};
		return format!("*{}*{}*{}*", self.from.asSendable(), kind, base64::encode(body.as_bytes()))
	}
	pub fn display(self){
		let content:String = match self.content {
			MsgContent::PublicKey(_) => format!("{}* is alllowing people to trust them", PUBLIC_KEY_LABEL),
			MsgContent::SecureText(txt) => format!("{}* {}", SECURE_LABEL, txt),
			MsgContent::InsecureText(txt) => format!("{}* {}", INSECURE_LABEL, txt),
			MsgContent::Join(_) => format!("{}* went online", JOIN_LABEL),
			MsgContent::Leave(_) => format!("{}* went offline", JOIN_LABEL),
			MsgContent::Trust(addr) => format!("{}* is trusting {}", TRUST_LABEL, addr.name),
			MsgContent::Blank() => format!("{}* Error Parsing Text", BLANK_LABEL)
		};
		println!("*{}\\{}", self.from.name, content.replace("\r", "").replace("\n", ""));
	}
	pub fn toWritable(self) -> Message {
		Message::Text(self.toString())
	}
	pub fn handleSelf(mut self, state:&mut Crypto){
		if self.from.asSendable() != state.addr().asSendable(){
			if let MsgContent::PublicKey(data) = &self.content {
				state.addPublicKey(self.from.clone(), decodeToPublicKeyBytes(data.clone()));
			}else if let MsgContent::SecureText(data) = self.content {
				self.content = MsgContent::SecureText(state.decrypt(&self.from, data));
			}
			self.clone().display();
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