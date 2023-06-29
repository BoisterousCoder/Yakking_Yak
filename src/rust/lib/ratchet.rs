use x25519_dalek::SharedSecret;
use sha2::Sha256;
use hkdf::Hkdf;
use serde::{Serialize, Deserialize};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};

// use crate::lib::utils::{split_and_clean, log};

#[derive(Clone, Serialize, Deserialize)]
pub struct Ratchet{
    is_encrypting:bool,
    secret_chain:Vec<PayloadHandler>
}
impl Ratchet{
    pub fn new(shared_secret:&SharedSecret, is_encrypting:bool, salt:Vec<u8>) -> Ratchet{
        let mut ratchet = Ratchet{
            is_encrypting,
            secret_chain: vec![]
        };
        let handler = UnusedLink::new(shared_secret.as_bytes().clone(), salt, None);
        ratchet.secret_chain.push(PayloadHandler::Unused(handler));
        return ratchet;
    }
    pub fn process_payload(&mut self, id:usize, payload:&str) -> String{
        self.gen_handlers_to(id);
        return match &self.secret_chain[id] {
            PayloadHandler::Unused(handler) => {
                let remaining_handler = handler.proccess_payload(self.is_encrypting, &payload);
                let res = remaining_handler.result.to_string();
                self.secret_chain[id] = PayloadHandler::HasProccessed(remaining_handler);
                res
            }
            PayloadHandler::MadeNext(handler) => {
                let res = handler.proccess_payload(self.is_encrypting, &payload).to_string();
                self.secret_chain[id] = PayloadHandler::Used(res.clone());
                res
            }
            PayloadHandler::HasProccessed(handler) => handler.result.clone(),
            PayloadHandler::Used(res) => res.to_string()
        };
    }
    pub fn set_new_shared_key(&mut self, start_id:usize, shared_secret:[u8;32]) -> Result<(), String>{
        self.gen_handlers_to(start_id-1);
        if let PayloadHandler::Unused(handler) = &self.secret_chain[start_id-1] {
            let (last_handler, next_handler) = handler.next(Some(shared_secret));
            self.secret_chain[start_id-1] = PayloadHandler::MadeNext(last_handler);
            self.secret_chain[start_id] = PayloadHandler::Unused(next_handler);
        } else if let PayloadHandler::HasProccessed(handler) = &self.secret_chain[start_id-1] {
            let next_handler = handler.next(Some(shared_secret));
            self.secret_chain[start_id-1] = PayloadHandler::Used(handler.result.clone());
            self.secret_chain[start_id] = PayloadHandler::Unused(next_handler);
        } else {
            let err = format!("Note: PayloadHandlers have already been gennerated at or beyond the id {} and a new key set cannot be started", start_id);
            return Err(err);
        }
        Ok(())
    }
    fn gen_handlers_to(&mut self, id:usize){
        loop {
            let len = self.secret_chain.len();
            if len > id {
                break;
            }

            if let PayloadHandler::Unused(handler) = &self.secret_chain[len-1] {
                let (last_handler, next_handler) = handler.next(None);
                self.secret_chain[len-1] = PayloadHandler::MadeNext(last_handler);
                self.secret_chain.push(PayloadHandler::Unused(next_handler));
            } else if let PayloadHandler::HasProccessed(handler) = &self.secret_chain[id-1] {
                let next_handler = handler.next(None);
                self.secret_chain[len-1] = PayloadHandler::Used(handler.result.clone());
                self.secret_chain.push(PayloadHandler::Unused(next_handler));
            } else {
                panic!("This should be unreachable! Can't find Shared Keys!")
            }
        }
    }
    pub fn get(&self, id:usize) -> Option<String> {
        return match &self.secret_chain[id] {
            PayloadHandler::HasProccessed(handler) => Some(handler.result.clone()),
            PayloadHandler::Used(res) => Some(res.clone()),
            _ => None,
        }
    }
    pub fn len(&self) -> usize{
        self.secret_chain.len()
    }
}

#[derive(Clone, Serialize, Deserialize)]
enum PayloadHandler{
    Unused(UnusedLink),
    MadeNext(MadeNextLink),
    HasProccessed(HasProccessedLink),
    Used(String)
}

#[derive(Clone, Serialize, Deserialize)]
struct UnusedLink{
    aes_key:[u8; 32],
    unique_iv:[u8; 12],
    secret:[u8; 32],
    salt:Vec<u8>,
    shared_secret:[u8; 32]
}
impl UnusedLink{
    pub fn new(shared_secret:[u8; 32], salt:Vec<u8>, last_secret_:Option<&[u8; 32]>) -> UnusedLink{
        let last_secret = match last_secret_ {
            Some(secret) => secret,
            None => &[0u8; 32]
        };
        let hk = Hkdf::<Sha256>::new(Some(&salt[..]), &shared_secret);
        let mut key_data = [0u8; 76];
        hk.expand(last_secret, &mut key_data).expect("76 is probably a valid length for Sha256 to output");
        let mut secret = [0u8; 32];
        let mut aes_key = [0u8; 32];
        let mut unique_iv = [0u8; 12];
        let mut i = 0;
        for byte in key_data {
            if i < secret.len(){
                secret[i] = byte;
            }else if i < secret.len() + aes_key.len(){
                aes_key[i - secret.len()] = byte;
            }else{
                unique_iv[i - secret.len() - aes_key.len()] = byte;
            }
            i += 1;
        }
    
        return UnusedLink{ 
            secret,
            aes_key, 
            unique_iv, 
            salt,
            shared_secret
        };
    }
    pub fn proccess_payload(&self, is_encrypting:bool, payload:&str) -> HasProccessedLink{
        let res = proccess_payload(&self.aes_key, &self.unique_iv, is_encrypting, payload);
        return HasProccessedLink {
            result: res,
            salt: self.salt.clone(),
            shared_secret: self.shared_secret.clone(),
            secret: self.secret.clone()
        };
    }
    pub fn next(&self, new_shared_secret:Option<[u8;32]>) -> (MadeNextLink, UnusedLink) {
        let shared_secret = match new_shared_secret {
            Some(shared_secret) => shared_secret,
            None => self.shared_secret.clone()
        };
        return (MadeNextLink {
            aes_key: self.aes_key.clone(),
            unique_iv: self.unique_iv.clone()
        }, UnusedLink::new(shared_secret, self.salt.clone(), Some(&self.secret)))
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct MadeNextLink{
    aes_key:[u8; 32],
    unique_iv:[u8; 12]
}
impl MadeNextLink{
    pub fn proccess_payload(&self, is_encrypting:bool, payload:&str) -> String{
        return proccess_payload(&self.aes_key, &self.unique_iv, is_encrypting, payload);
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct HasProccessedLink{
    pub result:String,
    salt:Vec<u8>,
    shared_secret:[u8; 32],
    secret:[u8; 32]
}
impl HasProccessedLink{
    pub fn next(&self, new_shared_secret:Option<[u8;32]>) -> UnusedLink {
        let shared_secret = match new_shared_secret {
            Some(shared_secret) => shared_secret,
            None => self.shared_secret.clone()
        };
        return UnusedLink::new(shared_secret, self.salt.clone(), Some(&self.secret))
    }
}

fn proccess_payload(aes_key:&[u8; 32], unique_iv:&[u8;12], is_encrypting:bool, payload:&str) -> String {
    let cipher = Aes256Gcm::new_from_slice(aes_key).expect("aes_key is 32bytes long");
    let nonce = Nonce::from_slice(unique_iv);
    let payload_bytes = base64::decode(payload).expect("incorrect base64 data for decryption");
    return if is_encrypting {
        base64::encode(cipher.encrypt(nonce, payload_bytes.as_ref()).expect("failed to encrypt"))
    } else {
        let result_bytes = cipher.decrypt(nonce, payload_bytes.as_ref()).unwrap();
        std::str::from_utf8(&result_bytes).expect("malformed string on decrypt").to_string()
    };
}