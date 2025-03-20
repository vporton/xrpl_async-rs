use std::array::TryFromSliceError;
use std::fmt::Formatter;
use std::iter::once;
use ::hex::FromHexError;
use derive_more::{Display, From};
use xrpl::core::addresscodec::exceptions::XRPLAddressCodecException;
use xrpl::core::addresscodec::utils::{decode_base58, encode_base58};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;
use crate::types::Hash;

#[derive(Debug)]
pub struct WrongPrefixError;

impl WrongPrefixError {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Display, From)]
pub enum FromXRPDecodingError {
    #[display("Wrong base58 format.")]
    FromBase58Check(XRPLAddressCodecException),
    #[display("Wrong hex number.")]
    Hex(FromHexError),
    #[display("Wrong base58 prefix.")]
    WrongPrefix(WrongPrefixError),
    #[display("Wrong array length.")]
    WrongLength(TryFromSliceError),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Encoding<
    const LENGTH: usize,
    const TYPE_PREFIX: u8,
    const HUMAN_REPRESENTATION_STARTS_WITH: char
>(pub [u8; LENGTH]);

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
    // TODO: Isn't it always hex? and therefore to remove `_hex` suffix?
    pub fn encode_hex(&self) -> String {
        ::hex::encode_upper(self.0)
    }
    pub fn decode_hex(s: &str) -> Result<Self, FromXRPDecodingError> {
        Ok(Self(::hex::decode(s)?.as_slice().try_into()?))
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
        Ok(Encoding::from_bytes_without_prefix(bytes.as_slice().try_into()?))
    }
}

/// Account address
///
/// `Address` is a separate type, because it has other type code and is prefixed by length
/// unlike hash of the same length. It also serializes not to hex.
#[derive(Clone, Debug, PartialEq)]
pub struct Address(pub Encoding<20, 0x00, 'r'>);

impl Address {
    pub const TYPE_PREFIX: u8 = 0x00;
    pub const LENGTH: usize = 20;
    pub const HUMAN_REPRESENTATION_STARTS_WITH: char = 'r';
    pub fn encode(&self) -> String {
        self.0.encode()
    }
    pub fn decode(s: &str) -> Result<Self, FromXRPDecodingError> {
        Ok(Self(Encoding::decode(s)?))
    }
}

/// Account public key
pub type AccountPublicKey = Encoding<33, 0x23, 'a'>;

/// Seed value (for secret keys)
pub type SeedValue = Encoding<16, 0x21, 's'>;

/// Validation public key or node public key
pub type ValidationOrNodePublicKey = Encoding<33, 0x1C, 'n'>;

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        Ok(Self(Encoding::<20, 0x00, 'r'>::deserialize(deserializer)?))
    }
}

impl<
    const LENGTH: usize,
    const TYPE_PREFIX: u8,
    const HUMAN_REPRESENTATION_STARTS_WITH: char,
> Serialize for Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        serializer.serialize_str(&self.encode())
    }
}

impl<
    'de,
    const LENGTH: usize,
    const TYPE_PREFIX: u8,
    const HUMAN_REPRESENTATION_STARTS_WITH: char,
> Deserialize<'de> for Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        // FIXME: Or hex?
        String::deserialize(deserializer)
            .and_then(|string| Encoding::decode(&string).map_err(de::Error::custom))
    }
}

struct HashVisitor<
    const LENGTH: usize,
    const TYPE_PREFIX: u8,
    const HUMAN_REPRESENTATION_STARTS_WITH: char,
>;

impl<
    'de,
    const LENGTH: usize,
    const TYPE_PREFIX: u8,
    const HUMAN_REPRESENTATION_STARTS_WITH: char,
> Visitor<'de> for HashVisitor<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH> {
    type Value = Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("an address")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        Self::Value::decode(value).map_err(de::Error::custom)
    }
}

pub mod hex {
    use serde::{Deserialize, Deserializer, Serializer};
    use super::*;

    pub fn serialize<
        const LENGTH: usize,
        const TYPE_PREFIX: u8,
        const HUMAN_REPRESENTATION_STARTS_WITH: char,
        S: Serializer,
    >(x: &Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>, s: S) -> Result<S::Ok, S::Error>
    {
        s.serialize_str(&x.encode_hex())
    }

    pub fn deserialize<'de,
        const LENGTH: usize,
        const TYPE_PREFIX: u8,
        const HUMAN_REPRESENTATION_STARTS_WITH: char,
        D: Deserializer<'de>,
    >(deserializer: D) -> Result<Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>, D::Error>
    {
        String::deserialize(deserializer)
            .and_then(|string| Encoding::decode_hex(&string).map_err(de::Error::custom))
    }
}

pub mod base58 {
    use serde::{Deserialize, Deserializer, Serializer};
    use super::*;

    pub fn serialize<
        const LENGTH: usize,
        const TYPE_PREFIX: u8,
        const HUMAN_REPRESENTATION_STARTS_WITH: char,
        S: Serializer,
    >(x: &Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>, s: S) -> Result<S::Ok, S::Error>
    {
        s.serialize_str(&x.encode())
    }

    pub fn deserialize<'de,
        const LENGTH: usize,
        const TYPE_PREFIX: u8,
        const HUMAN_REPRESENTATION_STARTS_WITH: char,
        D: Deserializer<'de>,
    >(deserializer: D) -> Result<Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>, D::Error>
    {
        String::deserialize(deserializer)
            .and_then(|string| Encoding::decode(&string).map_err(de::Error::custom))
    }
}

pub mod option_hex {
    use super::*;

    struct Wrap<
        const LENGTH: usize,
        const TYPE_PREFIX: u8,
        const HUMAN_REPRESENTATION_STARTS_WITH: char>(Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>
    );

    impl<
        'de,
        const LENGTH: usize,
        const TYPE_PREFIX: u8,
        const HUMAN_REPRESENTATION_STARTS_WITH: char
    >
    Deserialize<'de> for Wrap<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Ok(Wrap(hex::deserialize(deserializer)?))
        }
    }

    pub fn serialize<
        const LENGTH: usize,
        const TYPE_PREFIX: u8,
        const HUMAN_REPRESENTATION_STARTS_WITH: char,
        S: Serializer,
    >(x: &Option<Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>>, s: S) -> Result<S::Ok, S::Error>
    {
        if let Some(x) = x {
            hex::serialize(x, s)
        } else {
            None::<()>.serialize(s)
        }
    }

    pub fn deserialize<'de,
        const LENGTH: usize,
        const TYPE_PREFIX: u8,
        const HUMAN_REPRESENTATION_STARTS_WITH: char,
        D: Deserializer<'de>,
    >(deserializer: D) -> Result<Option<Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>>, D::Error>
    {
        let result = Option::<Wrap::<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>>::deserialize(deserializer)?;
        Ok(result.map(|v| v.0))
    }
}

pub mod option_base58 {
    use super::*;

    struct Wrap<
        const LENGTH: usize,
        const TYPE_PREFIX: u8,
        const HUMAN_REPRESENTATION_STARTS_WITH: char>(Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>
    );

    impl<
        'de,
        const LENGTH: usize,
        const TYPE_PREFIX: u8,
        const HUMAN_REPRESENTATION_STARTS_WITH: char
    >
    Deserialize<'de> for Wrap<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Ok(Wrap(base58::deserialize(deserializer)?))
        }
    }

    pub fn serialize<
        const LENGTH: usize,
        const TYPE_PREFIX: u8,
        const HUMAN_REPRESENTATION_STARTS_WITH: char,
        S: Serializer,
    >(x: &Option<Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>>, s: S) -> Result<S::Ok, S::Error>
    {
        if let Some(x) = x {
            base58::serialize(x, s)
        } else {
            None::<()>.serialize(s)
        }
    }

    pub fn deserialize<'de,
        const LENGTH: usize,
        const TYPE_PREFIX: u8,
        const HUMAN_REPRESENTATION_STARTS_WITH: char,
        D: Deserializer<'de>,
    >(deserializer: D) -> Result<Option<Encoding<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>>, D::Error>
    {
        let result = Option::<Wrap::<LENGTH, TYPE_PREFIX, HUMAN_REPRESENTATION_STARTS_WITH>>::deserialize(deserializer)?;
        Ok(result.map(|v| v.0))
    }
}

pub struct SecretKey(pub Hash<32>);