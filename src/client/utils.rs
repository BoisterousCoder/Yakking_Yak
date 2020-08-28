use std::{io, option};
use base64;

const FILE_EXT:&str = ".keys";

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

pub struct ServerMSG{
	pub from:String,
	pub kind:String,
	pub body:String
}
impl ServerMSG{
    pub fn fromData(&mut self, slice:&[u8]){
        self.body = base64::encode(slice.to_vec())
    }
    pub fn toData(&self) -> Vec<u8>{
        base64::decode(&self.body).unwrap()
    }
}