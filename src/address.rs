use std::array::TryFromSliceError;
use std::iter::once;
// use derive::Debug;
use derive_more::From;
use base58check::{FromBase58Check, FromBase58CheckError, ToBase58Check};

#[derive(Debug)]
pub struct WrongPrefixError;

impl WrongPrefixError {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, From)]
pub enum FromXRPDecodingError {
    FromBase58Check(FromBase58CheckError),
    WrongPrefix(WrongPrefixError),
    WrongLength(TryFromSliceError),
}

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
        (&self.bytes_without_prefix() as &[u8]).to_base58check(Self::TYPE_PREFIX)
    }
    pub fn decode(s: String) -> Result<Self, FromXRPDecodingError> {
        let (prefix, bytes) = s.from_base58check()?;
        if prefix != Self::TYPE_PREFIX {
            return Err(WrongPrefixError::new().into());
        }
        Ok(Self::from_bytes_without_prefix(bytes.as_slice().try_into()?))
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