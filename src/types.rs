use std::array::TryFromSliceError;
use std::convert::Infallible;
use std::iter::{once, repeat};
use std::num::ParseIntError;
use hex::{decode, FromHexError};
use derive_more::From;

pub struct Hash([u8; 32]);

impl ToString for Hash {
    fn to_string(&self) -> String {
        hex::encode_upper(self.0)
    }
}

#[derive(Debug, From)]
pub enum HexDecodeError {
    Hex(FromHexError),
    Slice(TryFromSliceError),
}

impl Hash {
    pub fn from_hex(s: &str) -> Result<Self, HexDecodeError> {
        Ok(Self(decode(s)?.as_slice().try_into()?))
    }
}

pub fn encode_xrp_amount(amount: u64) -> String {
    amount.to_string()
}

pub fn decode_xrp_amount(s: &str) -> Result<u64, ParseIntError> {
    s.parse::<u64>()
}

pub fn encode_token_amount(amount: u64) -> String {
    let mut s = amount.to_string();
    const DIGITS: usize = 6;
    if s.len() < DIGITS + 1 { // at least one digit before the dot
        s = repeat("0").take(DIGITS + 1 - s.len()).chain(once(s.as_str()))
            .flat_map(|s| s.chars()).collect();
    }
    // while s.len() < DIGITS + 1 { // at least one digit before the dot
    //     s = ["0", &s].concat();
    // }
    s.insert(s.len() - DIGITS, '.');
    s
        .trim_matches(&['0'] as &[_])
        .trim_matches(&['.'] as &[_]).to_owned()
    // FIXME: Remove leading zeros.
}

/// FIXME: Scientific notation
pub fn decode_token_amount(s: &str) -> Result<u64, ParseIntError> {
    // FIXME
    s.parse::<u64>()
}