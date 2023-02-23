// extern crate serde;
use std::convert::{From, Into};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};
use crate::address::{AccountPublicKey, Address};
use crate::connection::Api;
use crate::types::{Hash, Ledger};
use crate::json::ValueExt;
use crate::paginate::{Paginator, PaginatorExtractor};
use crate::request::{FormatParams, TypedRequest};
use crate::response::{ParseResponseError, TypedResponse, WrongFieldsError};

#[derive(Debug)]
pub struct ChannelsRequest {
    pub account: Address,
    pub destination_account: Option<Address>,
    pub ledger: Ledger,
    pub limit: Option<u16>,
}

impl FormatParams for &ChannelsRequest {
    fn to_json(&self) -> Map<String, Value> {
        let mut j = Map::new();
        // TODO: Move to `lazy_static`.
        j.insert("account".to_owned(), Value::String(self.account.encode()));
        if let Some(address) = &self.destination_account {
            j.insert("destination_account".to_owned(), address.encode().into());
        }
        if let Some(limit) = self.limit {
            j.insert("limit".to_owned(), limit.into());
        }
        let (ledger_key, ledger_value) = self.ledger.to_json();
        j.insert(ledger_key.to_owned(), ledger_value);
        j
    }
}

// TODO: Remove `Clone`?
#[derive(Clone, Debug, Deserialize)]
pub struct ChannelResponse {
    pub ledger_hash: Option<Hash>,
    pub ledger_index: Option<u32>,
    pub validated: Option<bool>,
}

#[derive(Debug)]
pub struct ChannelPaginator {
    pub account: Address,
    pub amount: u64,
    pub balance: u64,
    pub channel_id: Hash,
    pub destination_account: Address,
    pub settle_delay: u64,
    pub public_key: Option<AccountPublicKey>,
    // pub public_key_hex: Option<AccountPublicKey>,
    pub expiration: Option<u64>,
    pub cancel_after: Option<u64>,
    pub source_tag: Option<u32>,
    pub destination_tag: Option<u32>,
}

impl<'de> Deserialize<'de> for ChannelPaginator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Debug, Deserialize)]
        pub struct ChannelPaginator2 {
            pub account: Address,
            #[serde(with = "crate::types::xrp")]
            pub amount: u64,
            pub balance: u64,
            pub channel_id: Hash,
            pub destination_account: Address,
            pub settle_delay: u64,
            #[serde(with = "crate::address::option_base58")]
            pub public_key: Option<AccountPublicKey>,
            #[serde(with = "crate::address::option_hex")]
            pub public_key_hex: Option<AccountPublicKey>,
            pub expiration: Option<u64>,
            pub cancel_after: Option<u64>,
            pub source_tag: Option<u32>,
            pub destination_tag: Option<u32>,
        }
        let value: ChannelPaginator2 = ChannelPaginator2::deserialize(deserializer)?.into();
        Ok(ChannelPaginator {
            account: value.account,
            amount: value.amount,
            balance: value.balance,
            channel_id: value.channel_id,
            destination_account: value.destination_account,
            settle_delay: value.settle_delay,
            public_key: value.public_key.or(value.public_key_hex),
            expiration: value.expiration,
            cancel_after: value.cancel_after,
            source_tag: value.source_tag,
            destination_tag: value.destination_tag,
        })
    }
}

impl<'a> PaginatorExtractor<'a> for ChannelPaginator {
    fn list_obj(result: &Value) -> Result<&Value, WrongFieldsError> {
        Ok(result.get_valid("channels")?)
    }
}

pub async fn account_channels<'a, A>(
    api: &'a A,
    data: &'a ChannelsRequest,
) -> Result<(TypedResponse<ChannelResponse>, Paginator<'a, A, ChannelPaginator>), A::Error>
    where A: Api,
          A::Error: From<ParseResponseError> + From<WrongFieldsError>
{
    let request = TypedRequest {
        command: "account_channels",
        api_version: Some(1),
        data,
    };
    let (response, paginator) = Paginator::start(api, (&request).into()).await?;
    Ok((response.try_into()?, paginator))
}