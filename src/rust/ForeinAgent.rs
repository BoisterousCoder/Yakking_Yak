use serde::{Serialize, Deserialize};
use crate::KeyBundle::KeyBundle;
use crate::ratchet::Ratchet;

#[derive(Serialize, Deserialize)]
pub struct ForeinAgent {
    pub to_ratchet:Option<Ratchet>,
    pub from_ratchet:Option<Ratchet>,
    pub keys:KeyBundle
}
