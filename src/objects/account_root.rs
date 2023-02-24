use serde::Deserialize;
use crate::address::Address;
use crate::types::Hash;

#[derive(Deserialize)]
pub struct AccountRoot {
    #[serde(rename = "Account")]
    pub account: Address,
    #[serde(rename = "AccountTxnID")]
    pub account_txn_id: Option<Hash>,
    #[serde(rename = "Balance")]
    #[serde(with = "crate::types::option_xrp")]
    pub balance: Option<u64>,
    #[serde(rename = "BurnedNFTokens")]
    pub burned_nft_tokens: Option<u32>,
    #[serde(rename = "Domain")]
    pub domain: Option<String>,
    // FIXME: more fields
}