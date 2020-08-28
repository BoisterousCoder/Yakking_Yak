use crate::utils::ServerMSG;

pub fn onServerMSG(msg: ServerMSG){
    println!("*{}\\{}*{}", msg.from, msg.kind, msg.bodyText())
}