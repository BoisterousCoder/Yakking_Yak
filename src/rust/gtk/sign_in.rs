
use std::{path::Path, fs::{self, File}, io::{Write, Read}};

use gtk::{prelude::*, Button, Entry, Box};
use magic_crypt::MagicCryptTrait;
use magic_crypt::new_magic_crypt;
use rand_core::{RngCore, OsRng};

use crate::{SignInDetails, STATE, all::{store::Crypto, utils::calc_hash}};

use super::build_ui::build_content;

const SAVE_DIR:&str = "./saves";
const FILE_EXTENTION:&str = "user";

pub fn on_sign_in(sign_in_button:&Button){
    if let Some(options) = on_sign_in_attempt(&sign_in_button){
        sign_in_button.parent().unwrap().hide();
        let state = &mut STATE.lock().unwrap();
        **state = Crypto::new(
            &options.username, 
            &options.password, 
            options.private_device_id, 
            OsRng.next_u64(), 
            OsRng.next_u64());

        let content:Box = sign_in_button
            .parent().unwrap()
            .parent().unwrap()
            .downcast().expect("Found UI emlement but is not entry! UI is broke pls fix");
        build_content(&content);
    };
}

fn on_sign_in_attempt(sign_in_button:&Button) -> Option<SignInDetails>{
    let username_input:Entry = sign_in_button
        .parent().unwrap()
        .first_child().unwrap()
        .next_sibling().unwrap()
        .downcast().expect("Found UI emlement but is not entry! UI is broke pls fix");
    let password_input:Entry = username_input
        .next_sibling().unwrap()
        .next_sibling().unwrap()
        .downcast().expect("Found UI emlement but is not entry! UI is broke pls fix");
    let username = username_input.buffer().text().to_string();
    let password = password_input.buffer().text().to_string();
    
    if !Path::new(SAVE_DIR).is_dir() {
        fs::create_dir(SAVE_DIR).expect("can't make save directory");
    }

    #[allow(deprecated)]
    let filename = format!("{}/{}.{}", SAVE_DIR, base64::encode(&username), FILE_EXTENTION);
    if !fs::metadata(&filename).is_ok() {
        let mut device_id = [0u8; 32];
        OsRng.fill_bytes(&mut device_id);
        #[allow(deprecated)]
        let device_id_str = base64::encode(device_id);

        let mut file = File::create(&filename).expect("unable to create file");
        let key = new_magic_crypt!(calc_hash(&password), 256);
        let data = key.encrypt_str_to_base64(device_id_str);
        
        file.write_all(data.as_bytes()).unwrap();
        file.sync_all().unwrap();

        return Some(SignInDetails {
            private_device_id: device_id, 
            username, 
            password 
        });
    }else{
        let mut file = File::open(filename).expect("Unable to open user file");
        let mut data = String::new();
        if file.read_to_string(&mut data).is_ok(){
            let key = new_magic_crypt!(&password, 256);
            if let Some(device_id_str) = key.decrypt_base64_to_string(&data).ok(){
                #[allow(deprecated)]
                let device_id_vec = base64::decode(device_id_str).unwrap();
                let mut device_id = [0u8; 32];
                let mut i = 0;
                for byte in device_id_vec{
                    device_id[i] = byte;
                    i+=1;
                    if i == device_id.len(){
                        break;
                    }
                };

                return Some(SignInDetails { 
                    private_device_id: device_id, 
                    username, 
                    password 
                });
            };
        }
        return None;
    }
}