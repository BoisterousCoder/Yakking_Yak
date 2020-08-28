use libsignal_protocol;
use libsignal_protocol::{Context, keys, StoreContext, Serializable, PreKeyBundle};
use libsignal_protocol::stores::*;
use std::time::SystemTime;
use base64;
use crate::utils::ConnectionData;
use crate::store::{attemptFetchIdData, storeID, RawIdData};


const EXTENDED_RANGE:i32 = 0;

pub struct Crypto{
	pub name: String,
	connData: ConnectionData,
	ctx:Context,
	store:StoreContext
}
impl Crypto{
	pub fn new(name:String, connData:ConnectionData) -> Crypto{
		let ctx = Context::default();

		//Setup Identifier Store
		let (idKeySet, reg) = match attemptFetchIdData(connData.clone()) {
			Some(x) => parseIdData(x, &ctx),
			None => initializeIdData(connData.clone(), &ctx)
		};
		let idKeyStore = InMemoryIdentityKeyStore::new(reg, &idKeySet);

		//Setup Prekey Store
		let start = 123;
		let count = 20;

		let preKeys = libsignal_protocol::generate_pre_keys(&ctx, start, count).unwrap().collect::<Vec<keys::PreKey>>();
		
		let mut preKeyStore = InMemoryPreKeyStore::default();
		let mut preKeyIter = preKeys.iter();
		loop {
			match preKeyIter.next() {
				Some(preKeySet) => {
					preKeyStore.store(preKeySet.id(), preKeySet.serialize().unwrap().as_slice());
				},
				None => {
					break;
				}
			}
		}

		//Signed Keys store
		let signedKeySet = libsignal_protocol::generate_signed_pre_key(&ctx, &idKeySet, 5, SystemTime::now(),).unwrap();
		
		let signedKeyStore = InMemorySignedPreKeyStore::default();
		signedKeyStore.store(signedKeySet.id(), signedKeySet.serialize().unwrap().as_slice());

		//Setup session store
		let sessionStore = InMemorySessionStore::default();

		//put it all together
		let store = libsignal_protocol::store_context(&ctx, preKeyStore, signedKeyStore, sessionStore, idKeyStore).unwrap();

		//Make Bundle


		return Crypto{
			name:name,
			ctx:ctx,
			store:store,
			connData: connData
		}
	}
}
// pub struct Bundle{
// 	data: PreKeyBundle
// }
// impl Bundle{
// 	pub fn new(regId:u32, deviceId:i32, prekey:keys::PreKey, signedKey:keys::SignedPreKey, idKey:keys::IdentityKeyPair){
// 		let builder = PreKeyBundle.builder();
// 		builder.pre_key()
// 	}
// }

fn parseIdData(idData:RawIdData, ctx:&Context) -> (keys::IdentityKeyPair, u32){

	let publicIdKey = keys::PublicKey::decode_point(&ctx, base64::decode(idData.id.public).unwrap().as_slice()).unwrap();
	let privateIdKey = keys::PrivateKey::decode_point(&ctx, base64::decode(idData.id.private).unwrap().as_slice()).unwrap();

	let id = keys::IdentityKeyPair::new(&publicIdKey, &privateIdKey).unwrap();

	return (id, idData.reg)
}
pub fn initializeIdData(connData:ConnectionData, ctx:&Context) -> (keys::IdentityKeyPair, u32) {
	let id = libsignal_protocol::generate_identity_key_pair(&ctx).unwrap();
	let reg = libsignal_protocol::generate_registration_id(&ctx, EXTENDED_RANGE).unwrap();

	let publicKey = id.public().serialize().unwrap().as_slice().to_vec();
	let privateKey = id.private().serialize().unwrap().as_slice().to_vec();
	
	storeID(connData, reg, publicKey, privateKey);

	return (id, reg)
}