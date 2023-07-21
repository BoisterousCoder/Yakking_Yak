#![cfg(not(target_arch = "wasm32"))]
use serde::{Serialize, Deserialize};

use crate::all::forein_agent::ForeinAgent;
use crate::all::serverhandlers::ServerMsg;
use crate::all::ratchet::Ratchet;
use crate::all::utils::Address;

use magic_crypt::MagicCryptTrait;
use magic_crypt::new_magic_crypt;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::io::prelude::*;
use std::fs;
use std::error::Error;
use std::fs::File;
use std::path::Path;

const SAVE_DIR:&str = "./saves";
const FILE_EXT:&str = "msglog";

#[derive(Serialize, Deserialize, Clone)]
pub struct GroupSave {
    pub group:String,
    pub addr:Address,
    pub proxy_ratchet:Ratchet,
	pub agents: Vec<ForeinAgent>,
	pub msgs: Vec<ServerMsg>
}

fn calc_hash(text:&str) -> String{
    let mut hasher = DefaultHasher::new();
    text.to_string().hash(&mut hasher);
    format!("{}", hasher.finish())
}

impl GroupSave {
    pub fn save(self, password:&str) -> Result<(), Box<dyn Error>>{
        let plain_data = serde_json::to_string(&self).unwrap();
        let hashed_password = calc_hash(&password);

        if !Path::new(SAVE_DIR).is_dir() {
            fs::create_dir(SAVE_DIR)?;
        }

        let mut file = File::create(Self::filename(&self.addr, &self.group))?;
     	let key = new_magic_crypt!(hashed_password, 256);
     	let data = key.encrypt_str_to_base64(plain_data);

     	file.write_all(data.as_bytes())?;
     	file.sync_all()?;
        Ok(())
    }
    pub fn load(address:Address, group:&str, password:&str) -> Option<Self>{
        let filename = Self::filename(&address, group);
        let hashed_password = calc_hash(&password);

        if fs::metadata(&filename).is_ok(){
            if let Some(mut file) = File::open(filename).ok(){
                let mut data = String::new();
                if file.read_to_string(&mut data).is_ok(){
                    let key = new_magic_crypt!(hashed_password, 256);
                    if let Some(plain_data) = key.decrypt_base64_to_string(&data).ok(){
                        return serde_json::from_str(&plain_data).ok();
                    };
                }
            };
        }
        return None;
    }
    fn filename(addr:&Address, group:&str) -> String {
        #[allow(deprecated)]
        let safe_group = base64::encode(group);
        #[allow(deprecated)]
        let filename = &format!("{}@{}", addr.as_sendable(), base64::encode(safe_group));
        format!("{}/{}.{}", SAVE_DIR, filename, FILE_EXT)
    }
}