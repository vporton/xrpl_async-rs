use std::convert::From;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use crate::address::Address;
use crate::connection::{Api, XrplError};
use crate::objects::account_root::AccountRoot;
use crate::objects::signer_list::SignerList;
use crate::types::Ledger;
use crate::request::TypedRequest;
use crate::response::TypedResponse;

#[derive(Debug)]
pub struct AccountInfoRequest {
    pub account: Address,
    pub ledger: Ledger,
    pub queue: bool,
    pub signer_lists: bool,
}

impl Serialize for AccountInfoRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        #[derive(Debug, Serialize)]
        struct AccountInfoRequest2 {
            pub account: Address,
            #[serde(flatten)]
            pub ledger: Ledger,
            pub queue: Option<bool>,
            pub signer_lists: Option<bool>,
            pub strict: Option<bool>,
        }
        let request = AccountInfoRequest2 {
            account: self.account.clone(),
            ledger: self.ledger.clone(),
            queue: if self.queue { Some(true) } else { None },
            signer_lists: if self.signer_lists { Some(true) } else { None },
            strict: Some(true),
        };
        AccountInfoRequest2::serialize(&request, serializer)
    }
}

#[derive(Debug, Deserialize)]
pub struct QueuedTransaction {
    pub auth_change: bool,
    #[serde(with = "crate::types::xrp")]
    pub fee: u64,
    pub fee_level: u64,
    #[serde(with = "crate::types::xrp")]
    pub max_spend_drops: u64,
    pub seq: u32,
}

#[derive(Debug, Deserialize)]
pub struct QueueData {
    pub txn_count: u32,
    pub auth_change_queued: Option<bool>,
    pub lowest_sequence: Option<u32>,
    pub highest_sequence: Option<u32>,
    #[serde(with = "crate::types::xrp")]
    pub max_spend_drops_total: u64,
    pub transactions: Vec<QueuedTransaction>,
}

#[derive(Debug)]
pub struct AccountInfoResponse {
    pub account_data: AccountRoot,
    pub signer_list: SignerList,
    pub ledger_index: u32,
    pub ledger_index_is_current: bool,
    pub queue_data: Option<QueueData>,
    pub validated: bool,
}

impl<'de> Deserialize<'de> for AccountInfoResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Debug, Deserialize)]
        pub struct AccountInfoResponse2 {
            pub account_data: AccountRoot,
            pub signer_lists: Vec<SignerList>,
            pub ledger_current_index: Option<u32>,
            pub ledger_index: Option<u32>,
            pub queue_data: Option<QueueData>,
            pub validated: Option<bool>,
        }
        let value: AccountInfoResponse2 = AccountInfoResponse2::deserialize(deserializer)?.into();
        Ok(AccountInfoResponse {
            account_data: value.account_data,
            signer_list: value.signer_lists.first()
                .ok_or_else(|| de::Error::custom("missing signer_lists element"))?.clone(),
            ledger_index: value.ledger_index.or(value.ledger_current_index)
                .ok_or_else(|| de::Error::custom("missing ledger_index"))?,
            ledger_index_is_current: value.ledger_current_index.is_some(),
            queue_data: value.queue_data,
            validated: value.validated == Some(true),
        })
    }
}

pub async fn account_info<'a, A>(api: &'a A, data: &'a AccountInfoRequest)
                                 -> Result<TypedResponse<AccountInfoResponse>, A::Error>
    where A: Api,
          A::Error: From<XrplError>
{
    let request = TypedRequest {
        command: "account_info",
        api_version: Some(1),
        data,
    };
    Ok(api.call((&request).try_into().map_err(de::Error::custom)?).await?.try_into()?)
}