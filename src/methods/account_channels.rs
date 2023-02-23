// extern crate serde;
use std::convert::{From, Into};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use crate::address::{AccountPublicKey, Address};
use crate::connection::Api;
use crate::types::{Hash, Ledger};
use crate::json::ValueExt;
use crate::paginate::{Paginator, PaginatorExtractor};
use crate::request::{FormatParams, TypedRequest};
use crate::response::{ParseResponse, ParseResponseError, TypedResponse, WrongFieldsError};

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

#[derive(Debug)]
pub struct ChannelResponse {
    pub ledger_hash: Option<Hash>,
    pub ledger_index: Option<u32>,
    pub validated: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ChannelPaginator {
    pub account: Address,
    #[serde(with = "crate::types::xrp")]
    pub amount: u64,
    pub balance: u64,
    pub channel_id: Hash,
    pub destination_account: Address,
    pub settle_delay: u64,
    #[serde(with = "crate::address::option_base58")]
    pub public_key: Option<AccountPublicKey>,
    // pub public_key_hex: Option<AccountPublicKey>,
    pub expiration: Option<u64>,
    pub cancel_after: Option<u64>,
    pub source_tag: Option<u32>,
    pub destination_tag: Option<u32>,
}

impl Deserialize for ChannelPaginator {
    fn deserialize<D>(deserializer: D) -> Result<Self, serde::de::Error> where D: Deserializer<'de> {
        let mut value =
    }
}

impl ParseResponse for ChannelPaginator {
    fn from_json(value: &Value) -> Result<Self, ParseResponseError> {
        Ok(Self {
            account: value.get_valid("account")?.as_address_valid()?,
            amount: value.get_valid("amount")?.as_xrp_valid()?,
            balance: value.get_valid("balance")?.as_xrp_valid()?,
            channel_id: value.get_valid("channel_id")?.as_hash_valid()?,
            destination_account: value.get_valid("destination_account")?.as_address_valid()?,
            settle_delay: value.get_valid("settle_delay")?.as_u64_valid()?,
            public_key: value.get("public_key").map(|s| -> Result<_, WrongFieldsError> { AccountPublicKey::decode(s.as_str_valid()?).map_err(|_| WrongFieldsError::new()) })
                .or(value.get("public_key_hex").map(|s| AccountPublicKey::decode_hex(s.as_str_valid()?).map_err(|_| WrongFieldsError::new())))
                .transpose()?,
            expiration: value.get("expiration").map(|s| s.as_u64_valid()).transpose()?,
            cancel_after: value.get("cancel_after").map(|s| s.as_u64_valid()).transpose()?,
            source_tag: value.get("source_tag").map(|s| s.as_u32_valid()).transpose()?,
            destination_tag: value.get("destination_tag").map(|s| s.as_u32_valid()).transpose()?,
        })
    }
}

impl PaginatorExtractor for ChannelPaginator {
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