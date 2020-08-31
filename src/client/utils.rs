use std::{io, str};
use base64;
use libsignal_protocol::Address;
use crate::encrypter::KeyBundleWrapper;
use awc::ws::Message;

const FILE_EXT:&str = ".keys";
const BUNDLE_LABEL:&str = "b";
const INSECURE_LABEL:&str = "i";
const BLANK_LABEL:&str = "_";

pub fn getUserIn(prompt:String) -> String{
    let mut line = String::new();
    println!("{}", prompt);
    io::stdin().read_line(&mut line).unwrap();
    return line.replace("\r", "").replace("\n", "");
}
#[derive(Clone)]
pub struct ConnectionData{
    pub server:String,
    pub group:String,
    pub password:String
}
impl ConnectionData{
    pub fn get_ip(&self) -> String {
        return format!("http://{}/ws/", self.server)
    }
    pub fn get_storeName(&self) -> String {
        return format!("{}:{}", self.server.split(":").next().unwrap(), self.group);
    }
    pub fn get_fileName(&self) -> String {
        return self.get_storeName() + FILE_EXT;
    }
}

#[derive(Clone, Debug)]
pub enum MsgContent{
    Bundle(KeyBundleWrapper),
    InsecureText(String),
    Blank()
}

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
        let segments: Vec<&str> = txt.split('*').filter(|seg| !seg.is_empty()).collect();
        let addrSegments: Vec<&str> = segments[0].split('&').filter(|seg| !seg.is_empty()).collect();
        let nameData = base64::decode(addrSegments[0]).unwrap();
        let contentData = str::from_utf8(base64::decode(addrSegments[2]).unwrap().as_slice()).unwrap().to_string();
        let content = match segments[1] {
            BUNDLE_LABEL => MsgContent::Bundle(KeyBundleWrapper::fromString(contentData)),
            INSECURE_LABEL => MsgContent::InsecureText(contentData),
            BLANK_LABEL => MsgContent::Blank(),
            &_ => MsgContent::Blank()
        };
        return ServerMsg{
            from: Address::new(nameData, addrSegments[1].parse().unwrap()), 
            content: content
        }
    }
    pub fn toString(self) -> String{
        let addrData = format!("{}&{}", base64::encode(&self.from.bytes()), &self.from.device_id());
        let (kind, body) = match self.content {
            MsgContent::Bundle(bundle) => (BUNDLE_LABEL, bundle.toString()),
            MsgContent::InsecureText(txt) => (INSECURE_LABEL, txt),
            MsgContent::Blank() => (BLANK_LABEL, String::from("_"))
        };
        return format!("*{}*{}*{}*", addrData, kind, base64::encode(body.as_bytes()))
    }
    pub fn display(self){
        let content = match self.content {
            MsgContent::Bundle(_) => format!("{}* is requesting to be trusted", BUNDLE_LABEL),
            MsgContent::InsecureText(txt) => format!("{}* {}", INSECURE_LABEL, txt),
            MsgContent::Blank() => format!("{}", BLANK_LABEL)
        };
        println!("*{}\\{}", self.from.as_str().unwrap(), content);
    }
    pub fn toWritable(self) -> Message {
        Message::Text(self.toString())
    }
}


