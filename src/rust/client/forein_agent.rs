use serde::{Serialize, Deserialize};
use crate::client::key_bundle::KeyBundle;
use crate::client::ratchet::Ratchet;

#[derive(Serialize, Deserialize, Clone)]
pub struct ForeinAgent {
    pub to_ratchet:Option<Ratchet>,
    pub from_ratchet:Option<Ratchet>,
    pub keys:KeyBundle,
    pub is_online:bool
}
