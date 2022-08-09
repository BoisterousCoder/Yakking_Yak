use crate::encrypter::{Crypto, Address};
use std::str;
use actix::io::SinkWrite;
use awc::{
	ws::{Codec, Message},
	BoxedSocket
};
use actix_codec::Framed;
use futures::stream::SplitSink;
use x25519_dalek::PublicKey;
use base64;

// const BUNDLE_LABEL:&str = "b";
const INSECURE_LABEL:&str = "i";
const JOIN_LABEL:&str = "j";
const LEAVE_LABEL:&str = "l";
const PUBLIC_KEY_LABEL:&str = "p";
//const TRUST_LABEL:&str = "t";
const BLANK_LABEL:&str = "_";

pub fn onStart(websocket:&mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>, state:&mut Crypto){
	websocket.write(ServerMsg::new(&state.addr(), MsgContent::Join(state.connData.group.clone())).toWritable());
}

#[derive(Clone, Debug)]
pub enum MsgContent{
	InsecureText(String),
	Join(String),
	PublicKey(String),
	Leave(String),
	//Trust(Address),
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
		let segments: Vec<&str> = txt.split('*').filter(|seg| !seg.is_empty()).collect();
		let addrSegments: Vec<&str> = segments[0].split('@').filter(|seg| !seg.is_empty()).collect();
		//let nameData = str::from_utf8(base64::decode(addrSegments[0]).unwrap().as_slice()).unwrap();
		let contentData = decodeBase64(segments[2]);
		let content = match segments[1] {
			INSECURE_LABEL => MsgContent::InsecureText(contentData),
			JOIN_LABEL => MsgContent::Join(contentData),
			PUBLIC_KEY_LABEL => MsgContent::PublicKey(contentData),
			// TRUST_LABEL => {
			// 	let addrData:Vec<&str> = contentData.split("&").collect();
			// 	let name = base64::decode(addrData[0].to_string()).unwrap();
			// 	MsgContent::Trust(Address::new(name, str::parse(addrData[1]).unwrap()))
			// }
			LEAVE_LABEL => MsgContent::Leave(contentData),
			BLANK_LABEL => MsgContent::Blank(),
			&_ => MsgContent::Blank()
		};
		return ServerMsg{
			from: Address::new(&decodeBase64(addrSegments[0]), addrSegments[1].parse().unwrap()), 
			content: content
		}
	}
	pub fn toString(self) -> String{
		let (kind, body):(&str, String) = match self.content {
			MsgContent::PublicKey(publicKey) => (PUBLIC_KEY_LABEL, publicKey),
			MsgContent::InsecureText(txt) => (INSECURE_LABEL, txt),
			MsgContent::Join(group) => (JOIN_LABEL, group),
			MsgContent::Leave(group) => (LEAVE_LABEL, group),
			//MsgContent::Trust(foreinAddr) => (TRUST_LABEL, format!("{}&{}", base64::encode(foreinAddr.bytes()), foreinAddr.device_id())),
			MsgContent::Blank() => (BLANK_LABEL, String::from("_"))
		};
		return format!("*{}*{}*{}*", self.from.asSendable(), kind, base64::encode(body.as_bytes()))
	}
	pub fn display(self){
		let content:String = match self.content {
			MsgContent::PublicKey(_) => format!("{}* sent their public", PUBLIC_KEY_LABEL),
			MsgContent::InsecureText(txt) => format!("{}* {}", INSECURE_LABEL, txt),
			MsgContent::Join(_) => format!("{}* went online", JOIN_LABEL),
			MsgContent::Leave(_) => format!("{}* went offline", JOIN_LABEL),
			//MsgContent::Trust(addr) => format!("{}* is trusting {}", TRUST_LABEL, addr.as_str().unwrap()),
			MsgContent::Blank() => format!("{}* Error Parsing Text", BLANK_LABEL)
		};
		println!("*{}\\{}", self.from.name, content.replace("\r", "").replace("\n", ""));
	}
	pub fn toWritable(self) -> Message {
		Message::Text(self.toString())
	}
	pub fn handleSelf(self, crypto:&mut Crypto){
		if self.from.asSendable() != crypto.addr().asSendable(){
			self.display();
		}
	}
}
fn decodeBase64(inTxt:&str) -> String{
	return str::from_utf8(base64::decode(inTxt).unwrap().as_slice()).unwrap().to_string()
}