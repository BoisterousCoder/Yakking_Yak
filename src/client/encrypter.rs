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
pub struct KeyBundleWrapper{
	preId: u32,
	pre: Vec<u8>,
	signedId: u32,
	signed: Vec<u8>,
	regId: u32,
	deviceId: i32,
	signature: Vec<u8>,
	id: Vec<u8>
}
impl KeyBundleWrapper{
	pub fn wrap(regId:u32, deviceId:i32, prekey:keys::PreKey, signedKey:keys::SessionSignedPreKey, idKey:keys::IdentityKeyPair) -> KeyBundleWrapper{
		return KeyBundleWrapper{
			preId: prekey.id(),
			signedId: signedKey.id(),
			regId:regId,
			deviceId:deviceId,
			pre: prekey.key_pair().public().serialize().unwrap().as_slice().to_vec(),
			signed: signedKey.key_pair().public().serialize().unwrap().as_slice().to_vec(),
			id: idKey.public().serialize().unwrap().as_slice().to_vec(),
			signature: signedKey.signature().to_vec()
		}
	}
	pub fn unwrap(&self, ctx:&Context) -> PreKeyBundle{
		let mut builder = PreKeyBundle::builder();
		let preKey = keys::PublicKey::decode_point(ctx, self.pre.as_slice()).unwrap();
		let signedKey = keys::PublicKey::decode_point(ctx, self.signed.as_slice()).unwrap();
		let idKey = keys::PublicKey::decode_point(ctx, self.id.as_slice()).unwrap();
		builder = builder.pre_key(self.preId, &preKey);
		builder = builder.signed_pre_key(self.signedId, &signedKey);
		builder = builder.signature(self.signature.as_slice());
		builder = builder.registration_id(self.regId);
		builder = builder.identity_key(&idKey);
		return builder.build().unwrap();
	}
}

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