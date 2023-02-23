use std::convert::{From, Into};
use serde_json::{Map, Value};
use crate::address::Address;
use crate::connection::Api;
use crate::types::{Hash, Ledger};
use crate::json::ValueExt;
use crate::request::{FormatParams, TypedRequest};
use crate::response::{ParseResponse, ParseResponseError, TypedResponse, WrongFieldsError};

#[derive(Debug)]
pub struct CurrenciesRequest {
    pub account: Address,
    pub ledger: Ledger,
}

impl FormatParams for &CurrenciesRequest {
    fn to_json(&self) -> Map<String, Value> {
        let mut j = Map::new();
        // TODO: Move to `lazy_static`.
        j.insert("account".to_owned(), Value::String(self.account.encode()));
        let (ledger_key, ledger_value) = self.ledger.to_json();
        j.insert(ledger_key.to_owned(), ledger_value);
        j
    }
}

#[derive(Debug)]
pub struct CurrenciesResponse {
    pub ledger_hash: Option<Hash>,
    pub ledger_index: Option<u32>,
    pub receive_currencies: Vec<String>,
    pub send_currencies: Vec<String>,
    pub validated: Option<bool>,
}

impl ParseResponse for CurrenciesResponse {
    fn from_json(value: &Value) -> Result<Self, ParseResponseError> {
        Ok(CurrenciesResponse {
            ledger_hash: value.get("ledger_hash").map(|v| Ok::<_, WrongFieldsError>(v.as_hash_valid()?)).transpose()?,
            ledger_index: value.get("ledger_index").map(|v| Ok::<_, WrongFieldsError>(v.as_u32_valid()?)).transpose()?,
            receive_currencies: value.get_valid("receive_currencies")?.as_array_valid()?
                .into_iter().map(|v| -> Result<_, WrongFieldsError> { Ok(v.as_str_valid()?.to_owned()) })
                .collect::<Result<Vec<String>, WrongFieldsError>>()?,
            send_currencies: value.get_valid("send_currencies")?.as_array_valid()?
                .into_iter().map(|v| -> Result<_, WrongFieldsError> { Ok(v.as_str_valid()?.to_owned()) })
                .collect::<Result<Vec<String>, WrongFieldsError>>()?,
            validated: value.get("validated").map(|v| Ok::<_, WrongFieldsError>(v.as_bool_valid()?)).transpose()?,
        })
    }
}

pub async fn account_channels<'a, A>(api: &'a A, data: &'a CurrenciesRequest)
    -> Result<TypedResponse<CurrenciesResponse>, A::Error>
    where A: Api,
          A::Error: From<ParseResponseError> + From<WrongFieldsError>
{
    let request = TypedRequest {
        command: "account_currencies",
        api_version: Some(1),
        data,
    };
    Ok(api.call((&request).into()).await?.try_into()?)
}