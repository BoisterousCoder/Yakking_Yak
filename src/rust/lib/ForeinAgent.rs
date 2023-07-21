use serde::{Serialize, Deserialize};
use crate::lib::KeyBundle::KeyBundle;
use crate::lib::ratchet::Ratchet;

#[derive(Serialize, Deserialize, Clone)]
pub struct ForeinAgent {
    pub to_ratchet:Option<Ratchet>,
    pub from_ratchet:Option<Ratchet>,
    pub keys:KeyBundle,
    pub is_online:bool
}
