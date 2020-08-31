//client
#![allow(non_snake_case)]

//! Simple websocket client.
use std::time::Duration;
use std::{io, thread};

use actix::io::SinkWrite;
use actix::*;
use actix_codec::Framed;
use awc::{
	error::WsProtocolError,
	ws::{Codec, Frame, Message},
	BoxedSocket, Client,
};
use bytes::Bytes;
use futures::stream::{SplitSink, StreamExt};

mod utils;
mod encrypter;
mod store;
mod serverhandlers;


fn main() {
	::std::env::set_var("RUST_LOG", "actix_web=info");
	env_logger::init();

	let sys = System::new("websocket-client");

	Arbiter::spawn(async {
		// let senderConnData = utils::ConnectionData{
		// 	server: utils::getUserIn("Server:".to_string()),
		// 	group: utils::getUserIn("Group:".to_string()),
		// 	password: utils::getUserIn("Please enter your password".to_string())
		// };

		//this is for speed of debugging only. Replace with comment above when done
		let connData = utils::ConnectionData{
			server: "localhost:4000".to_string(),
			name: utils::getUserIn(String::from("Name:")),
			group: utils::getUserIn("Group:".to_string()),
			password: "test".to_string()
		};

		println!("Connecting to {}...", connData.get_ip());
		let (response, framed) = Client::new()
			.ws(connData.get_ip())
			.connect()
			.await
			.map_err(|e| {
				println!("Error: {}", e);
			})
			.unwrap();

		println!("{:?}", response);

		let crypto = encrypter::Crypto::new(connData.clone());

		let (sink, stream) = framed.split();
		let addr = ChatClient::create(|ctx| {
			ChatClient::add_stream(stream, ctx);
			ChatClient(SinkWrite::new(sink, ctx), crypto)
		});

		// start console loop
		thread::spawn(move || loop {
			let mut cmd = String::new();
			if io::stdin().read_line(&mut cmd).is_err() {
				println!("error");
				return;
			}
			//let msg = utils::ServerMSG::new("Alice".to_string(), "test".to_string(), cmd);
			//let msg = utils::ServerMsg::new(&aliceCrypto.addr(), utils::MsgContent::InsecureText(&cmd));
			addr.do_send(ClientCommand(cmd));
		});
	});
	sys.run().unwrap();
}

struct ChatClient(SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>, encrypter::Crypto);

#[derive(Message)]
#[rtype(result = "()")]
struct ClientCommand(String);


impl Actor for ChatClient {
	type Context = Context<Self>;

	fn started(&mut self, ctx: &mut Context<Self>) {
		//let reg = format!("{}", self.1.registration_id().unwrap());
		//let msg = utils::ServerMSG::new("Alice".to_string(), "addr".to_string(), reg);
		//self.0.write(Message::Text(msg.toString())).unwrap();

		// start heartbeats otherwise server will disconnect after 10 seconds
		self.hb(ctx);
		serverhandlers::onStart(&mut self.0, &mut self.1);
	}

	fn stopped(&mut self, _: &mut Context<Self>) {
		println!("Disconnected");

		// Stop application on disconnect
		System::current().stop();
	}
}

impl ChatClient {
	fn hb(&self, ctx: &mut Context<Self>) {
		ctx.run_later(Duration::new(1, 0), |act, ctx| {
			act.0.write(Message::Ping(Bytes::from_static(b""))).unwrap();
			act.hb(ctx);

			// client should also check for a timeout here, similar to the
			// server code
		});
	}
}

/// Handle stdin commands
impl Handler<ClientCommand> for ChatClient {
	type Result = ();

	fn handle(&mut self, cmd: ClientCommand, _ctx: &mut Context<Self>) {
		let msg = utils::ServerMsg::new(&self.1.addr(), utils::MsgContent::InsecureText(cmd.0));
		self.0.write(Message::Text(msg.toString())).unwrap();
	}
}

/// Handle server websocket messages
impl StreamHandler<Result<Frame, WsProtocolError>> for ChatClient {
	fn handle(&mut self, msg: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {
		if let Ok(Frame::Text(txt)) = msg {
			let msg = utils::ServerMsg::fromServer(&txt.to_vec());
			serverhandlers::onServerMSG(msg.clone(), &mut self.1);
		}
	}

	fn started(&mut self, _ctx: &mut Context<Self>) {
		println!("Connected");
	}

	fn finished(&mut self, ctx: &mut Context<Self>) {
		println!("Server disconnected");
		ctx.stop()
	}
}

impl actix::io::WriteHandler<WsProtocolError> for ChatClient {}