
use actix::*;
use crate::serverhandlers::MsgContent;
use crate::encrypter::Crypto;
use crate::utils::splitAndClean;
use base64;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientCommand(pub String);
impl ClientCommand {
	pub fn handleSelf(self, state:&mut Crypto) -> Option<MsgContent>{
		if self.0.chars().collect::<Vec<char>>()[0] == '/' {
			let words:Vec<&str> = splitAndClean(&self.0, ' ');
			let cmd = words[0];
			return match cmd{
				"/allowTrust" => {
					Some(MsgContent::PublicKey(base64::encode(state.publicKey.as_bytes())))
				}
				"/broadcast" => {
					Some(MsgContent::InsecureText(words[1..].join(" ")))
				}
				
				"/trust" => {
				 	match state.trust(words[1].to_string()) {
						Some(addr) => Some(MsgContent::Trust(addr.clone())),
						None => {
							println!("{} has not allowed you to trust them or does not exist", words[1]);
							None
						}
					}
				}
				"/list" => {
					state.listPeople();
					None
				}
				_ => {
					eprintln!("Command {} was not recognized.", cmd);
					None
				}
			}
		}else{
			if state.isTrusting() {
				return Some(MsgContent::SecureText(state.encrypt(self.0)));
			}else{
				return Some(MsgContent::InsecureText(self.0));
			}
		}
	}
}