use std::fmt;
use serde::{
	ser::{Serialize, Serializer, SerializeStruct}, 
	de::{self, Deserialize, Deserializer, Visitor, MapAccess}
};
// use web_sys::console;
use crate::client::utils::{log, Address};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};
use rand_chacha::{ChaCha20Rng, rand_core::SeedableRng};

pub enum SecretKey {
	Empty(),
	Shared(SharedSecret),
	Ephemeral(EphemeralSecret)
}
pub struct KeyBundle{
	pub public_key: PublicKey,
	pub secret: SecretKey,
	pub address: Address
}
impl KeyBundle{
	pub fn new_self_key_set(addr:Address, seed:u64) -> KeyBundle{
		log("Creating Ephemeral Key..");
		let rng: ChaCha20Rng = ChaCha20Rng::seed_from_u64(seed);
		let secret = EphemeralSecret::new(rng);
		log("Creating Public Key..");
		return KeyBundle{
			public_key: PublicKey::from(&secret),
			secret: SecretKey::Ephemeral(secret),
			address: addr
		};
	}
}
impl Clone for KeyBundle{
	fn clone(&self) -> Self {
		let secret_key = match &self.secret {
			SecretKey::Shared(shared) => {
				let bytes = shared.as_bytes();
				SecretKey::Shared(SharedSecret::from(bytes.clone()))
			}
			SecretKey::Ephemeral(ephemeral) => {
				let bytes = ephemeral.as_bytes();
				SecretKey::Ephemeral(EphemeralSecret::from(bytes.clone()))
			},
			_ => SecretKey::Empty()
		};
		return Self{
			public_key: self.public_key.clone(),
			address: self.address.clone(),
			secret:secret_key
		};
	}
}
impl Serialize for KeyBundle {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where S: Serializer,
	{
		let mut state = serializer.serialize_struct("KeyBundle", 3)?;
		
		state.serialize_field("public", &self.public_key.as_bytes())?;
		match &self.secret {
			SecretKey::Ephemeral(secret) => state.serialize_field("ephemeral", secret.as_bytes()),
			SecretKey::Shared(secret) => state.serialize_field("shared", secret.as_bytes()),
			SecretKey::Empty() => Ok(())
		}.unwrap();
		state.serialize_field("address", &self.address)?;
		state.end()
	}
}
impl<'de> Deserialize<'de> for KeyBundle {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de>,
	{
		enum Field { Public, Shared, Ephemeral, Addr}
		impl<'de> Deserialize<'de> for Field {
			fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
			where
				D: Deserializer<'de>,
			{
				struct FieldVisitor;

				impl<'de> Visitor<'de> for FieldVisitor {
					type Value = Field;

					fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
						formatter.write_str("`public_key`, `shared`, `ephemeral` or `address`")
					}

					fn visit_str<E>(self, value: &str) -> Result<Field, E>
					where
						E: de::Error,
					{
						match value {
							"public" => Ok(Field::Public),
							"shared" => Ok(Field::Shared),
							"ephemeral" => Ok(Field::Ephemeral),
							"address" => Ok(Field::Addr),
							_ => Err(de::Error::unknown_field(value, FIELDS)),
						}
					}
				}

				deserializer.deserialize_identifier(FieldVisitor)
			}
		}

		struct KeyBundleVisitor;
		
		impl<'de> Visitor<'de> for KeyBundleVisitor {
			type Value = KeyBundle;
			
			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("struct KeyBundle")
			}

			fn visit_map<V>(self, mut map: V) -> Result<KeyBundle, V::Error>
			where
				V: MapAccess<'de>,
			{
				//console::log_1(&"Rebuilding Keyset..".into());

				let mut public = None;
				let mut secret = None;
				let mut addr = None;
				
				loop {
					let key = map.next_key()?;
					if !key.is_some() { break;}
					else {
						match key.unwrap() {
							Field::Public => {
								if public.is_some() {
									return Err(de::Error::duplicate_field("public"));
								}
								let data: [u8; 32] = map.next_value()?;
								public = Some(PublicKey::from(data));
							}
							Field::Shared => {
								if secret.is_some() {
									return Err(de::Error::duplicate_field("secret"));
								}
								let data: [u8; 32] = map.next_value()?;
								secret = Some(SecretKey::Shared(SharedSecret::from(data)));
							}
							Field::Ephemeral => {
								if secret.is_some() {
									return Err(de::Error::duplicate_field("secret"));
								}
								let data: [u8; 32] = map.next_value()?;
								secret = Some(SecretKey::Ephemeral(EphemeralSecret::from(data)));
							}
							Field::Addr => {
								if addr.is_some() {
									return Err(de::Error::duplicate_field("address"));
								}
								addr = Some(map.next_value()?);
							}
						}
					}
				}

				let public = public.ok_or_else(|| de::Error::missing_field("public"))?;
				let addr = addr.ok_or_else(|| de::Error::missing_field("addresss"))?;
				let secret= secret.unwrap_or_else(||SecretKey::Empty());

				//console::log_1(&"Finished Rebuilding Keyset".into());

				Ok(KeyBundle{
					public_key:public, 
					secret:secret, 
					address:addr
				})
			}
		}

		const FIELDS: &'static [&'static str] = &["public", "shared", "ephemeral", "address"];
		deserializer.deserialize_struct("KeyBundle", FIELDS, KeyBundleVisitor)
	}
}