use std::array::TryFromSliceError;
use std::iter::{once, repeat};
use std::num::ParseIntError;
use hex::{decode, FromHexError};
use derive_more::From;
use serde_json::{json, Value};

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

const XPR_DIGITS_AFTER_DOT: usize = 6;

#[derive(Debug)]
pub struct TokenAmountError;

impl TokenAmountError {
    pub fn new() -> Self {
        Self {}
    }
}

pub fn encode_token_amount(amount: f64) -> Result<String, TokenAmountError> {
    if amount < -9999999999999999e80f64 || amount > 9999999999999999e80f64 {
        return Err(TokenAmountError);
    }
    Ok(amount.to_string())
}

pub fn decode_token_amount(s: &str) -> Result<f64, TokenAmountError> {
    s.parse::<f64>().map_err(|_| TokenAmountError::new())
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

pub enum Ledger {
    Index(u32),
    Hash(Hash),
    Validated,
    Closed,
    Current,
}

impl Ledger {
    pub fn to_json(&self) -> (&str, Value) {
        match self {
            Ledger::Index(ind) => ("ledger_index", json!(ind)),
            Ledger::Hash(hash) => ("ledger_hash", Value::String(hash.to_string())),
            Ledger::Validated => ("ledger_index", Value::String("validated".to_owned())),
            Ledger::Closed => ("ledger_index", Value::String("closed".to_owned())),
            Ledger::Current => ("ledger_index", Value::String("current".to_owned())),
        }
    }
}