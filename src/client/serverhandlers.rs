use crate::utils::{ServerMsg, MsgContent};
use crate::encrypter::Crypto;
use actix::io::SinkWrite;
use awc::{
	ws::{Codec, Message},
	BoxedSocket
};
use actix_codec::Framed;
use futures::stream::SplitSink;

pub fn onStart(websocket:&mut SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>, state:&mut Crypto){
    websocket.write(ServerMsg::new(&state.addr(), MsgContent::Bundle(state.getBundle())).toWritable());
}

pub fn onServerMSG(msg: ServerMsg, state:&mut Crypto){
    msg.display();
    let optionBundle = "bundle".to_string();
    let optionEncrypetedMsg = "encryptedMsg".to_string();
    // if msg.from != state.addr() {
    //     match msg.kind {
    //         optionBundle => {

    //         } 
    //     }
    // }
}