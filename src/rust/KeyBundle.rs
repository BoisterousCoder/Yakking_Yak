use std::fmt;
use serde::{
	ser::{Serialize, Serializer, SerializeStruct}, 
	de::{self, Deserialize, Deserializer, Visitor, MapAccess}
};
use web_sys::console;
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};
use rand_chacha::{ChaCha20Rng, rand_core::SeedableRng};

use crate::utils::Address;

pub enum SecretKey {
	Empty(),
	Shared(SharedSecret),
	Ephemeral(EphemeralSecret)
}

pub struct KeyBundle{
	pub publicKey: PublicKey,
	pub secret: SecretKey,
	pub address: Address
}
impl KeyBundle{
	pub fn isTrusting(&self) -> bool{
		return match self.secret {
			SecretKey::Shared(_) => true,
			SecretKey::Ephemeral(_) => false,
			SecretKey::Empty() => false
		}
	}
	pub fn newSelfKeySet(addr:Address, randNum:u64) -> KeyBundle{
		console::log_1(&"Creating Ephemeral Key..".into());
		let rng = ChaCha20Rng::seed_from_u64(randNum);
		let secret = EphemeralSecret::new(rng);
		console::log_1(&"Creating Public Key..".into());
		return KeyBundle{
			publicKey: PublicKey::from(&secret),
			secret: SecretKey::Ephemeral(secret),
			address: addr
		};
	}
}
impl Serialize for KeyBundle {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where S: Serializer,
	{
		let mut state = serializer.serialize_struct("KeyBundle", 3)?;
		
		state.serialize_field("publicKey", &self.publicKey.as_bytes())?;
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
						formatter.write_str("`publicKey`, `shared`, `ephemeral` or `address`")
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

			// fn visit_seq<V>(self, mut seq: V) -> Result<, V::Error>
			// where
			//     V: SeqAccess<'de>,
			// {
			//     let publicKey = seq.next_element()?
			//         .ok_or_else(|| de::Error::invalid_length(0, &self))?;
			//     let secretKey = seq.next_element()?
			//         .ok_or_else(|| de::Error::invalid_length(1, &self))?;
			//     let addr = seq.next_element()?
			//         .ok_or_else(|| de::Error::invalid_length(1, &self))?;
			//     Ok(Duration::new(secs, nanos))
			// }

			fn visit_map<V>(self, mut map: V) -> Result<KeyBundle, V::Error>
			where
				V: MapAccess<'de>,
			{
				console::log_1(&"Rebuilding Keyset..".into());

				let mut public = None;
				let mut secret = None;
				let mut addr = None;
				
				loop {
					console::log_1(&"start 1".into());
					let key = map.next_key()?;
					console::log_1(&"start 2".into());
					if !key.is_some() { break;}
					else {
						match key.unwrap() {
							Field::Public => {
								console::log_1(&"start a".into());
								if public.is_some() {
									return Err(de::Error::duplicate_field("public"));
								}
								let data: [u8; 32] = map.next_value()?;
								public = Some(PublicKey::from(data));
								console::log_1(&"end a".into());
							}
							Field::Shared => {
								console::log_1(&"start b".into());
								if secret.is_some() {
									return Err(de::Error::duplicate_field("secret"));
								}
								let data: [u8; 32] = map.next_value()?;
								secret = Some(SecretKey::Shared(SharedSecret::from(data)));
								console::log_1(&"end b".into());
							}
							Field::Ephemeral => {
								console::log_1(&"start c".into());
								if secret.is_some() {
									return Err(de::Error::duplicate_field("secret"));
								}
								let data: [u8; 32] = map.next_value()?;
								secret = Some(SecretKey::Ephemeral(EphemeralSecret::from(data)));
								console::log_1(&"end c".into());
							}
							Field::Addr => {
								console::log_1(&"start d".into());
								if addr.is_some() {
									return Err(de::Error::duplicate_field("address"));
								}
								addr = Some(map.next_value()?);
								console::log_1(&"end d".into());
							}
						}
					}
				}

				let public = public.ok_or_else(|| de::Error::missing_field("public"))?;
				let addr = addr.ok_or_else(|| de::Error::missing_field("addresss"))?;
				let secret= secret.unwrap_or_else(||SecretKey::Empty());

				console::log_1(&"Finished Rebuilding Keyset".into());

				Ok(KeyBundle{
					publicKey:public, 
					secret:secret, 
					address:addr
				})
			}
		}

		const FIELDS: &'static [&'static str] = &["publicKey", "shared", "ephemeral", "address"];
		deserializer.deserialize_struct("KeyBundle", FIELDS, KeyBundleVisitor)
	}
}