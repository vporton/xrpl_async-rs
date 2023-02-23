use std::array::TryFromSliceError;
use std::fmt::Formatter;
use std::iter::once;
// use derive::Debug;
use derive_more::From;
use hex::FromHexError;
use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
use xrpl::core::addresscodec::utils::{decode_base58, encode_base58};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;

#[derive(Debug)]
pub struct WrongPrefixError;

impl WrongPrefixError {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, From)]
pub enum FromXRPDecodingError {
    FromBase58Check(XRPLAddressCodecException),
    Hex(FromHexError),
    WrongPrefix(WrongPrefixError),
    WrongLength(TryFromSliceError),
}

#[derive(Debug)]
pub struct Encoding<
    const LENGTH: usize,
    const TYPE_PREFIX: u8,
    const HUMAN_REPRESENTATION_STARTS_WITH: char
>([u8; LENGTH]);

// TODO: Unit test that `human_representation_starts_with` and `type_prefix` agree.
impl<
    const LENGTH: usize,
    const TYPE_PREFIX: u8,
    const HUMAN_REPRESENTATION_STARTS_WITH: char,
> Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH> {
    /// Byte added as prefix to sequence before encoding
    pub const TYPE_PREFIX: u8 = TYPE_PREFIX;
    /// The letter from which base58 representation starts
    pub const HUMAN_REPRESENTATION_STARTS_WITH: char = HUMAN_REPRESENTATION_STARTS_WITH;
    pub fn bytes_without_prefix(&self) -> [u8; LENGTH] {
        self.0
    }
    pub fn from_bytes_without_prefix(bytes: [u8; LENGTH]) -> Self {
        Self(bytes)
    }
    pub fn bytes_with_prefix(&self) -> Vec<u8> {
        once(TYPE_PREFIX).chain(self.bytes_without_prefix()).collect()
    }
    pub fn encode(&self) -> String {
        // (&self.bytes_without_prefix() as &[u8]).to_base58check(Self::TYPE_PREFIX)
        encode_base58(&self.bytes_without_prefix() as &[u8], &[Self::TYPE_PREFIX], Some(LENGTH)).unwrap()
    }
    pub fn decode(s: &str) -> Result<Self, FromXRPDecodingError> {
        let bytes = decode_base58(s, &[Self::TYPE_PREFIX]).map_err(|_| WrongPrefixError::new())?;
        // if prefix != Self::TYPE_PREFIX {
        //     return Err(WrongPrefixError::new().into());
        // }
        Ok(Self::from_bytes_without_prefix(bytes.as_slice().try_into()?))
    }
    pub fn encode_hex(&self) -> String {
        hex::encode_upper(self.0)
    }
    pub fn decode_hex(s: &str) -> Result<Self, FromXRPDecodingError> {
        Ok(Self(hex::decode(s)?.as_slice().try_into()?))
    }
}

/// Account address
pub type Address = Encoding<20, 0x00, 'r'>;

/// Account public key
pub type AccountPublicKey = Encoding<33, 0x23, 'a'>;

/// Seed value (for secret keys)
pub type SeedValue = Encoding<16, 0x21, 's'>;

/// Validation public key or node public key
pub type ValidationOrNodePublicKey = Encoding<33, 0x1C, 'n'>;

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&self.encode())
    }
}

struct AddressVisitor;

impl<'de> Visitor<'de> for AddressVisitor {
    type Value = Address;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("an address")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        Address::decode(&value).map_err(|_| de::Error::custom("invalid address"))
    }

}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Address, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_str(AddressVisitor)
    }
}