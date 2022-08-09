
use actix::*;
use crate::serverhandlers::MsgContent;
use crate::encrypter::Crypto;
use base64;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientCommand(pub String);
impl ClientCommand {
	pub fn handleSelf(self, state:&mut Crypto) -> Option<MsgContent>{
		let shareCMD:String = "/share".to_string();
		let broadCMD:String = "/broadcast".to_string();

		if self.0.chars().collect::<Vec<char>>()[0] == '/' {
			let words:Vec<&str> = self.0.split(' ').filter(|word| !word.is_empty()).collect();
			let cmd = words[0].to_string();
			return match cmd{
				shareCMD => {
					Some(MsgContent::PublicKey(base64::encode(state.publicKey.as_bytes())))
				}
				broadCMD => {
					Some(MsgContent::InsecureText(words[1..].join(" ")))
				}
				
				// "/trust" => {
				// 	let addr = crypto.trust(words[1..].join(" "));
				// 	Some(MsgContent::Trust(addr))
				// }
				_ => {
					eprintln!("Command {} was not recognized.", cmd);
					None
				}
			}
		}else{
			return Some(MsgContent::InsecureText(self.0));
		}
	}
}