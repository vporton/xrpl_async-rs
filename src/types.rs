use std::array::TryFromSliceError;
use std::iter::{once, repeat};
use std::num::ParseIntError;
use hex::{decode, FromHexError};
use derive_more::{Display, From};
use serde::{de, Deserialize, Deserializer, ser, Serialize, Serializer};
use serde::de::Visitor;
use serde::ser::SerializeMap;
use serde_json::json;
use sha2::{Digest, Sha512_256};

#[derive(Clone, Debug)]
pub struct Hash<const LENGTH: usize>(pub [u8; LENGTH]);

impl<const LENGTH: usize> ToString for Hash<LENGTH> {
    fn to_string(&self) -> String {
        hex::encode_upper(self.0)
    }
}

#[derive(Debug, From, Display)]
pub enum HexDecodeError {
    Hex(FromHexError),
    Slice(TryFromSliceError),
}

impl<const LENGTH: usize> Hash<LENGTH> {
    pub fn from_hex(s: &str) -> Result<Self, HexDecodeError> {
        Ok(Self(decode(s)?.as_slice().try_into()?))
    }
}

impl<const LENGTH: usize> Serialize for Hash<LENGTH> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct HashVisitor<const LENGTH: usize>;

impl<'de, const LENGTH: usize> Visitor<'de> for HashVisitor<LENGTH> {
    type Value = Hash<LENGTH>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a hex hash")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where E: de::Error,
    {
        Hash::from_hex(value).map_err(de::Error::custom)
    }
}

impl<'de, const LENGTH: usize> Deserialize<'de> for Hash<LENGTH> {
    fn deserialize<D>(deserializer: D) -> Result<Hash<LENGTH>, D::Error>
        where D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HashVisitor)
    }
}

pub struct Seed(pub [u8; 16]);

pub struct PrivateKey(pub [u8; 32]);

pub fn seed_to_private_key(seed: &Seed) -> PrivateKey {
    let mut hasher = Sha512_256::new();
    hasher.update(seed.0);
    PrivateKey(hasher.finalize()[.. 32].try_into().unwrap())
}

pub fn encode_xrp_amount(amount: u64) -> String {
    amount.to_string()
}

pub fn decode_xrp_amount(s: &str) -> Result<u64, ParseIntError> {
    s.parse::<u64>()
}

pub mod xrp {
    use serde::{Deserialize, Deserializer, Serializer};
    use super::*;

    pub fn serialize<S>(x: &u64, s: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        s.serialize_str(&encode_xrp_amount(*x))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
        where D: Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .and_then(|string| decode_xrp_amount(&string).map_err(de::Error::custom))
    }
}

pub mod option_xrp {
    use super::*;

    struct Wrap(u64);

    impl<'de> Deserialize<'de> for Wrap {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Ok(Wrap(xrp::deserialize(deserializer)?))
        }
    }

    pub fn serialize<S: Serializer>(x: &Option<u64>, s: S) -> Result<S::Ok, S::Error>
    {
        if let Some(x) = x {
            xrp::serialize(x, s)
        } else {
            None::<()>.serialize(s)
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<u64>, D::Error>
    {
        let result = Option::<Wrap>::deserialize(deserializer)?;
        Ok(result.map(|v| v.0))
    }
}

const XPR_DIGITS_AFTER_DOT: usize = 6;

#[derive(Debug, Display)]
#[display(fmt = "Token amount out of bounds.")]
pub struct TokenAmountError;

impl TokenAmountError {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}

pub fn encode_token_amount(amount: f64) -> Result<String, TokenAmountError> {
    if !(-9999999999999999e80f64..=9999999999999999e80f64).contains(&amount) {
        return Err(TokenAmountError::new());
    }
    Ok(amount.to_string())
}

pub fn decode_token_amount(s: &str) -> Result<f64, TokenAmountError> {
    s.parse::<f64>().map_err(|_| TokenAmountError::new())
}

pub mod token {
    use serde::{Deserialize, Deserializer, Serializer};
    use super::*;

    pub fn serialize<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        s.serialize_str(&encode_token_amount(*x).map_err(ser::Error::custom)?)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
        where D: Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .and_then(|string| decode_token_amount(&string).map_err(de::Error::custom))
    }
}

pub mod option_token {
    use super::*;

    struct Wrap(f64);

    impl<'de> Deserialize<'de> for Wrap {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Ok(Wrap(token::deserialize(deserializer)?))
        }
    }

    pub fn serialize<S: Serializer>(x: &Option<f64>, s: S) -> Result<S::Ok, S::Error>
    {
        if let Some(x) = x {
            token::serialize(x, s)
        } else {
            None::<()>.serialize(s)
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<f64>, D::Error>
    {
        let result = Option::<Wrap>::deserialize(deserializer)?;
        Ok(result.map(|v| v.0))
    }
}

pub fn xrp_to_human_representation(amount: u64) -> String {
    let mut s = amount.to_string();
    // Add zeros prefix:
    if s.len() < XPR_DIGITS_AFTER_DOT + 1 { // at least one digit before the dot
        s = repeat("0").take(XPR_DIGITS_AFTER_DOT + 1 - s.len()).chain(once(s.as_str()))
            .flat_map(|s| s.chars()).collect();
    }
    assert!(s.len() > XPR_DIGITS_AFTER_DOT);
    s.insert(s.len() - XPR_DIGITS_AFTER_DOT, '.');
    s
        .trim_matches(&['0'] as &[_])
        .trim_end_matches(&['.'] as &[_]).to_owned()
}

// TODO: Unit tests.

#[derive(Clone, Debug)]
pub enum LedgerForRequest {
    Index(u32),
    Hash(Hash<32>),
    Validated,
    Closed,
    Current,
}

impl Serialize for LedgerForRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            LedgerForRequest::Index(ind) => map.serialize_entry("ledger_index", &json!(ind))?,
            LedgerForRequest::Hash(hash) => map.serialize_entry("ledger_hash", &json!(hash))?,
            LedgerForRequest::Validated => map.serialize_entry("ledger_index", &json!("validated"))?,
            LedgerForRequest::Closed => map.serialize_entry("ledger_index", &json!("closed"))?,
            LedgerForRequest::Current => map.serialize_entry("ledger_index", &json!("current"))?,
        }
        map.end()
    }
}

#[derive(Clone, Debug)]
pub struct LedgerForResponse {
    pub index: Option<u32>,
    pub hash: Option<Hash<32>>,
    pub current: bool,
}

impl<'de> Deserialize<'de> for LedgerForResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Debug, Deserialize)]
        pub struct LedgerForResponse2 {
            pub ledger_current_index: Option<u32>,
            pub ledger_index: Option<u32>,
            pub ledger_hash: Option<Hash<32>>,
        }
        let value: LedgerForResponse2 = LedgerForResponse2::deserialize(deserializer)?;
        Ok(LedgerForResponse {
            index: value.ledger_current_index.or(value.ledger_index),
            hash: value.ledger_hash,
            current: value.ledger_current_index.is_some(),
        })
    }
}