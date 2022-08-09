//server
#![allow(non_snake_case)]

#[macro_use] extern crate lazy_static;

use std::time::{Duration, Instant};
use std::sync::Mutex;
use std::collections::{HashMap, HashSet};

use actix::prelude::*;
use actix_files as fs;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

lazy_static!{
	static ref chatState: Mutex<HashMap<String, HashSet<Addr<MyWebSocket>>>> = Mutex::new(HashMap::new());
}

/// do websocket handshake and start `MyWebSocket` actor
async fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
	println!("{:?}", r);
	let res = ws::start(MyWebSocket::new(), &r, stream);
	println!("{:?}", res);
	res
}

/// websocket connection is long running connection, it easier
/// to handle with an actor
struct MyWebSocket {
	/// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
	/// otherwise we drop connection.
	hb: Instant,
}

impl Actor for MyWebSocket {
	type Context = ws::WebsocketContext<Self>;

	/// Method is called on actor start. We start the heartbeat process here.
	fn started(&mut self, ctx: &mut Self::Context) {
		self.hb(ctx);
	}
}

#[derive(Message)]
#[rtype(result = "()")]
struct EchoedMsg(String);

impl Handler<EchoedMsg> for MyWebSocket{
	type Result = ();

	fn handle(&mut self, msg: EchoedMsg, ctx: &mut ws::WebsocketContext<Self>) {
		ctx.text(msg.0);
	}
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
	fn handle(
		&mut self,
		msg: Result<ws::Message, ws::ProtocolError>,
		ctx: &mut Self::Context,
	) {
		// process websocket messages
		match msg {
			Ok(ws::Message::Ping(msg)) => {
				self.hb = Instant::now();
				ctx.pong(&msg);
			}
			Ok(ws::Message::Pong(_)) => {
				self.hb = Instant::now();
			}
			Ok(ws::Message::Text(text)) => {
				if !text.is_empty(){
					println!("WS: {}", text);
					let segments: Vec<&str> = text.split('*').filter(|seg| !seg.is_empty()).collect();
					if segments[1]=="j" {
						let subSegments = segments[2].split('&').filter(|seg| !seg.is_empty()).collect();
						chatState.lock().unwrap().entry(String::from(subSegments[0])).or_insert(HashSet::new()).insert(ctx.address());
					}
					for (_, members) in chatState.lock().unwrap().clone().iter(){
						if members.get(&ctx.address()) != None {
							for member in members.iter(){
								member.do_send(EchoedMsg(text.clone()));
							}
							break;
						}
					}
					ctx.text(text)
				}
			},
			Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
			Ok(ws::Message::Close(reason)) => {
				for (group, members) in chatState.lock().unwrap().clone().iter(){
					if members.get(&ctx.address()) != None {
						if members.len() > 1 {
							chatState.lock().unwrap().get_mut(group).unwrap().remove(&ctx.address());
						}else{
							chatState.lock().unwrap().remove(group);
						}
						break;
					}
				}
				ctx.close(reason);
				ctx.stop();
			}
			_ => ctx.stop(),
		}
	}
}

impl MyWebSocket {
	fn new() -> Self {
		Self { hb: Instant::now() }
	}

	/// helper method that sends ping to client every second.
	///
	/// also this method checks heartbeats from client
	fn hb(&self, ctx: &mut <Self as Actor>::Context) {
		ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
			// check client heartbeats
			if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
				// heartbeat timed out
				println!("Websocket Client heartbeat failed, disconnecting!");

				// stop actor
				ctx.stop();

				// don't try to send a ping
				return;
			}

			ctx.ping(b"");
		});
	}
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
	std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
	env_logger::init();

	HttpServer::new(|| {
		App::new()
			// enable logger
			.wrap(middleware::Logger::default())
			// websocket route
			.service(web::resource("/ws/").route(web::get().to(ws_index)))
			// static files
			.service(fs::Files::new("/", "static/").index_file("index.html"))
	})
	// start http server on 127.0.0.1:4000
	.bind("127.0.0.1:4000")?
	.run()
	.await
}
