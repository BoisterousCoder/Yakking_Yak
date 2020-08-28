use std::{io, str};
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

#[derive(Clone)]
pub struct ServerMSG{
	pub from:String,
	pub kind:String,
	pub body:Vec<u8>
}
impl ServerMSG{
    pub fn new(from:String, kind:String, body:String) -> ServerMSG{
        return ServerMSG{
            from: from,
            kind: kind,
            body: body.as_bytes().to_vec()
        }
    }
    pub fn fromServer(data:&Vec<u8>) -> ServerMSG{
        let txt = str::from_utf8(data).unwrap();
        let segments: Vec<&str> = txt.split('*').filter(|seg| !seg.is_empty()).collect();
        return ServerMSG{
            from: segments[0].to_string(), 
            kind: segments[1].to_string(), 
            body: base64::decode(segments[2]).unwrap().to_vec()
        }
    }
    pub fn toString(&self) -> String{
        return format!("*{}*{}*{}*", self.from, self.kind, base64::encode(self.body.as_slice()))
    }
    pub fn bodyText(&self) -> &str{
        return str::from_utf8(self.body.as_slice()).unwrap();
    }
    // pub fn fromData(&mut self, slice:&[u8]){
    //     self.body = base64::encode(slice.to_vec())
    // }
    // pub fn toData(&self) -> Vec<u8>{
    //     base64::decode(&self.body).unwrap()
    // }
}


