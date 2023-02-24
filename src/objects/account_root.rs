use serde::{de, Deserialize, Deserializer};
use ubits::bitfield;
use crate::address::Address;
use crate::types::Hash;
use crate::types::xrp::deserialize;

pub struct Flags(u32);

impl<'de> Deserialize<'de> for Flags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Ok(Flags(deserialize(deserializer)?.try_into().map_err(de::Error::custom("flags field too bug"))?))
    }
}

#[derive(Deserialize)]
pub struct AccountRoot {
    #[serde(rename = "Account")]
    pub account: Address,
    #[serde(rename = "AccountTxnID")]
    pub account_txn_id: Option<Hash<32>>,
    #[serde(rename = "Balance")]
    #[serde(with = "crate::types::option_xrp")]
    pub balance: Option<u64>,
    #[serde(rename = "BurnedNFTokens")]
    pub burned_nft_tokens: Option<u32>,
    #[serde(rename = "Domain")]
    pub domain: Option<String>,
    #[serde(rename = "EmailHash")]
    pub email_hash: Option<Hash<16>>,
    #[serde(rename = "Flags")]
    pub flags: Flags,
    // FIXME: more fields
}