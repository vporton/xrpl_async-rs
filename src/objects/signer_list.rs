use serde::{de, Deserialize, Deserializer};
use crate::address::Address;
use crate::types::Hash;

#[derive(Debug)]
pub struct SignerListFlags(u32);
// TODO: Values of flags: https://xrpl.org/signerlist.html#signerlist-flags

impl<'de> Deserialize<'de> for SignerListFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Ok(SignerListFlags(u32::deserialize(deserializer)?.try_into().map_err(de::Error::custom)?))
    }
}

#[derive(Debug, Deserialize)]
pub struct SignerEntry {
    #[serde(rename = "Account")]
    pub account: Address,
    #[serde(rename = "SignerWeight")]
    pub signer_weight: u16,
    #[serde(rename = "WalletLocator")]
    pub wallet_locator: Hash<32>,
}

#[derive(Debug, Deserialize)]
pub struct SignerList {
    #[serde(rename = "Flags")]
    pub flags: SignerListFlags,
    #[serde(rename = "OwnerNode")]
    pub owner_node: u64,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash<32>,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    #[serde(rename = "SignerEntries")]
    pub signer_entries: Vec<SignerEntry>,
    #[serde(rename = "SignerListID")]
    pub signer_list_id: u32,
    #[serde(rename = "SignerQuorum")]
    pub signer_quorum: u32,
}