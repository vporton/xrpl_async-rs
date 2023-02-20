use std::array::TryFromSliceError;
use std::cmp::max;
use std::iter::repeat;
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

const TOKEN_DIGITS: usize = 6;

pub fn encode_token_amount(amount: u64) -> String {
    let mut s = amount.to_string();
    // The following is presumably a little slow...
    // while s.len() < TOKEN_DIGITS + 1 { // at least one digit before the dot
    //     s = ["0", &s].concat();
    // }
    // ... so iterator magic
    if s.len() < TOKEN_DIGITS + 1 { // at least one digit before the dot
        s = repeat('0').take(TOKEN_DIGITS + 1 - s.len()).chain(s.chars()).collect();
    }
    s.insert(s.len() - TOKEN_DIGITS, '.');
    s
        .trim_matches(&['0'] as &[_])
        .trim_end_matches(&['.'] as &[_]).to_owned()
}

/// FIXME: Support scientific notation
pub fn decode_token_amount(s: &str) -> Result<u64, ParseIntError> {
    if let Some(dot_pos) = s.chars().position(|c| c== '.') {
        let mut s = s.to_owned();
        let digits_after_dot = s.len() - dot_pos;
        if digits_after_dot < TOKEN_DIGITS {
            s = s.chars().chain(repeat('0').take(TOKEN_DIGITS - digits_after_dot)).collect();
        }
        s.remove(dot_pos);
        s = s[.. max(digits_after_dot, TOKEN_DIGITS) - TOKEN_DIGITS].to_owned();
        s.parse::<u64>()
    } else {
        s.parse::<u64>()
    }
}

// TODO: Unit tests.