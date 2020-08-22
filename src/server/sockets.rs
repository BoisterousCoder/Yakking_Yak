use actix_web_actors::ws;

mod sockets {
	trait User {
		fn send(&self, text:String);
		fn onRecieve(&self, title:String, callback:Fn<String>)
	}
	impl User for &mut Actor{
		fn send(&self, text:String){
			
		}
	}
}