use std::{io, str};

const FILE_EXT:&str = "keys";

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
    pub name:String,
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
        return format!("{}@{}.{}", self.name, self.get_storeName(), FILE_EXT);
    }
}