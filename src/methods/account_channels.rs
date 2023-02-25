use std::convert::From;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use crate::address::{AccountPublicKey, Address};
use crate::connection::{Api, XrplError};
use crate::types::{Hash, Ledger};
use crate::paginate::{Paginator, PaginatorExtractor};
use crate::request::TypedRequest;
use crate::response::TypedResponse;

#[derive(Debug, Serialize)]
pub struct ChannelsRequest {
    pub account: Address,
    pub destination_account: Option<Address>,
    #[serde(flatten)]
    pub ledger: Ledger,
    pub limit: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct ChannelResponse {
    pub ledger_hash: Option<Hash<32>>,
    pub ledger_index: Option<u32>,
    pub validated: Option<bool>,
}

#[derive(Debug)]
pub struct ChannelPaginator {
    pub account: Address,
    pub amount: u64,
    pub balance: u64,
    pub channel_id: Hash<32>,
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
            #[serde(with = "crate::types::xrp")]
            pub balance: u64,
            pub channel_id: Hash<32>,
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
        let value: ChannelPaginator2 = ChannelPaginator2::deserialize(deserializer)?;
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
    fn list_obj(result: &Value) -> Result<&Value, XrplError> {
        result.get("channels").ok_or::<XrplError>(de::Error::missing_field("channels"))
    }
}

pub async fn account_channels<'a, A>(
    api: &'a A,
    data: &'a ChannelsRequest,
) -> Result<(TypedResponse<ChannelResponse>, Paginator<'a, A, ChannelPaginator>), A::Error>
    where A: Api,
          A::Error: From<XrplError>
{
    let request = TypedRequest {
        command: "account_channels",
        api_version: Some(1),
        data,
    };
    let (response, paginator) =
        Paginator::start(api, (&request).try_into().map_err(de::Error::custom)?).await?; // TODO: wrong error
    Ok((response.try_into()?, paginator))
}