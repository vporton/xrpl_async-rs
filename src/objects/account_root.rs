use serde::{Deserialize, Deserializer};
use crate::hashes::{AccountPublicKey, Address};
use crate::types::Hash;

#[derive(Clone, Copy, Debug)]
pub struct AccountRootFlags(u32);

pub mod account_root_flags {
    pub const LSF_AMM: u64 = 0x02000000;
    pub const LSF_DEFAULT_RIPPLE: u64 = 0x00800000;
    pub const LSF_DEPOSIT_AUTH: u64 = 0x01000000;
    pub const LSF_DISABLE_MASTER: u64 = 0x00100000;
    pub const LSF_DISALLOW_XRP: u64 = 0x00080000;
    pub const LSF_GLOBAL_FREEZE: u64 = 0x00400000;
    pub const LSF_NO_FREEZE: u64 = 0x00200000;
    pub const LSF_PASSWORD_SPENT: u64 = 0x00010000;
    pub const LSF_REQUIRE_AUTH: u64 = 0x00040000;
    pub const LSF_REQUIRE_DEST_TAG: u64 = 0x00020000;
}

impl<'de> Deserialize<'de> for AccountRootFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Ok(AccountRootFlags(u32::deserialize(deserializer)?))
    }
}

#[derive(Debug, Deserialize)]
pub struct AccountRoot {
    #[serde(rename = "Account")]
    pub account: Address,
    #[serde(rename = "AccountTxnID")]
    pub account_txn_id: Option<Hash<32>>,
    #[serde(rename = "Balance")]
    #[serde(with = "crate::types::option_xrp")]
    pub balance: Option<u64>,
    #[serde(rename = "BurnedNFTokens")]
    pub burned_nf_tokens: Option<u32>,
    #[serde(rename = "Domain")]
    pub domain: Option<String>,
    #[serde(rename = "EmailHash")]
    pub email_hash: Option<Hash<16>>,
    #[serde(rename = "Flags")]
    pub flags: AccountRootFlags,
    #[serde(rename = "MessageKey")]
    pub message_key: Option<AccountPublicKey>,
    #[serde(rename = "MintedNFTokens")]
    pub minted_nf_tokens: Option<u32>,
    #[serde(rename = "NFTokenMinter")]
    pub nf_token_minter: Option<Address>,
    #[serde(rename = "OwnerCount")]
    pub owner_count: u32,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash<32>,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    #[serde(rename = "RegularKey")]
    pub regular_key: Option<Address>,
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    #[serde(rename = "TicketCount")]
    pub ticket_count: Option<u8>,
    #[serde(rename = "TickSize")]
    pub tick_size: Option<u8>,
    #[serde(rename = "TransferRate")]
    pub transfer_rate: Option<u32>,
    #[serde(rename = "WalletLocator")]
    pub wallet_locator: Option<Hash<32>>,
}
