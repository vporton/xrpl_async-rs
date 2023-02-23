use std::convert::{From, Into};
use serde::Deserialize;
use serde_json::{Map, Value};
use crate::address::Address;
use crate::connection::Api;
use crate::types::{Hash, Ledger};
use crate::request::{FormatParams, TypedRequest};
use crate::response::{ParseResponseError, TypedResponse, WrongFieldsError};

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

#[derive(Debug, Deserialize)]
pub struct CurrenciesResponse {
    pub ledger_hash: Option<Hash>,
    pub ledger_index: Option<u32>,
    pub receive_currencies: Vec<String>,
    pub send_currencies: Vec<String>,
    pub validated: Option<bool>,
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