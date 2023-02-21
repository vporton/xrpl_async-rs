use serde_json::{json, Value};
use crate::address::Address;
use crate::connection::{FormatRequest, ParseResponse, ParseResponseError, WrongFieldsError};
use crate::types::{Hash, Ledger};
use crate::json::ValueExt;

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

struct ChannelResponse {
    ledger_hash: Option<Hash>,
    ledger_index: Option<u32>,
    validated: Option<bool>,
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

struct ChannelPaginator {

}

// impl Paginator for ChannelPaginator {}