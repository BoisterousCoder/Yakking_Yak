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
    websocket.write(ServerMsg::new(&state.addr(), MsgContent::JoinGroup(state.connData.group.clone())).toWritable());
    //websocket.write(ServerMsg::new(&state.addr(), MsgContent::Bundle(state.getBundle())).toWritable());
}

pub fn onServerMSG(msg: ServerMsg, state:&mut Crypto){
    if msg.from != state.addr() {
        msg.clone().display();
        match msg.content {
            MsgContent::Bundle(bundle) =>{
                
            },
            MsgContent::JoinGroup(_) => {},
            MsgContent::InsecureText(_) =>{},
            MsgContent::Blank() => {}
        }
    }
}