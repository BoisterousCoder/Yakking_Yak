use crate::utils::ServerMSG;
use crate::encrypter::Crypto;
use libsignal_protocol::Address;

pub fn onServerMSG(msg: ServerMSG, state:&mut Crypto){
    println!("*{}\\{}*{}", msg.from, msg.kind, msg.bodyText());
    if msg.from != "alice" {
        if msg.kind=="addr".to_string() {
            let addr = Address::new(&msg.from, msg.bodyText().parse::<i32>().unwrap());
        }
    }
}