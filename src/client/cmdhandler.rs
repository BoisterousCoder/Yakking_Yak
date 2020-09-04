
use actix::*;
use crate::serverhandlers::MsgContent;
use crate::encrypter::Crypto;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientCommand(pub String);
impl ClientCommand {
    pub fn handleSelf(self, crypto:&mut Crypto) -> Option<MsgContent>{
        if self.0.chars().collect::<Vec<char>>()[0] == '/' {
            let words:Vec<&str> = self.0.split(' ').filter(|word| !word.is_empty()).collect();
            let cmd:&str = words[0];
            return match cmd{
                "/broadcast" => {
                    Some(MsgContent::InsecureText(words[1..].join(" ")))
                }
                "/trust" => {
                    let addr = crypto.trust(words[1..].join(" "));
                    Some(MsgContent::Trust(addr))
                }
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