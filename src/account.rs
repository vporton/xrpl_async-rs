use serde_json::{json, Value};
use crate::address::{AccountPublicKey, Address};
use crate::connection::Api;
use crate::types::{Hash, Ledger};
use crate::json::ValueExt;
use crate::paginate::{Paginator, PaginatorExtractor};
use crate::request::{FormatRequest, TypedRequest};
use crate::response::{ParseResponse, ParseResponseError, TypedResponse, WrongFieldsError};

struct ChannelsRequest {
    account: Address,
    destination_account: Option<Address>,
    ledger: Ledger,
    limit: Option<u16>,
}

impl FormatRequest for ChannelsRequest {
    fn to_json(&self) -> Value {
        let mut j = json!({
           "account": Value::String(self.account.encode()),
        });
        if let Some(address) = &self.destination_account {
            j["destination_account"] = address.encode().into();
        }
        if let Some(limit) = self.limit {
            j["limit"] = limit.into();
        }
        let (ledger_key, ledger_value) = self.ledger.to_json();
        j[ledger_key] = ledger_value;
        j
    }
}

pub struct ChannelResponse {
    pub ledger_hash: Option<Hash>,
    pub ledger_index: Option<u32>,
    pub validated: Option<bool>,
}

impl ParseResponse for ChannelResponse {
    fn from_json(value: &Value) -> Result<Self, ParseResponseError> {
        Ok(ChannelResponse {
            ledger_hash: value.get("ledger_hash").map(|v| Ok::<_, WrongFieldsError>(v.as_hash_valid()?)).transpose()?,
            ledger_index: value.get("ledger_index").map(|v| Ok::<_, WrongFieldsError>(v.as_u32_valid()?)).transpose()?,
            validated: value.get("validated").map(|v| Ok::<_, WrongFieldsError>(v.as_bool_valid()?)).transpose()?,
        })
    }
}

pub struct ChannelPaginator {
    pub account: Address,
    pub amount: f64,
    pub balance: f64,
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

impl ParseResponse for ChannelPaginator {
    fn from_json(value: &Value) -> Result<Self, ParseResponseError> {
        Ok(Self {
            account: value.get_valid("account")?.as_address_valid()?,
            amount: value.get_valid("amount")?.as_f64_valid()?,
            balance: value.get_valid("balance")?.as_f64_valid()?,
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
        Ok(result.get("channels").ok_or(WrongFieldsError::new())?)
    }
}

pub async fn account_channels<'a, A>(api: &A, request: &'a TypedRequest<'a, ChannelsRequest>)
                           -> Result<(TypedResponse<ChannelResponse>, ChannelPaginator), A::Error>
    where A: Api,
          A::Error: From<ParseResponseError> + From<WrongFieldsError>
{
    let (response, paginator) = Paginator::start(api, request.into()).await?;
    let response: TypedResponse<ChannelResponse> = response.try_into()?;
    Ok((response, paginator))
}