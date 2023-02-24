use std::convert::From;
use serde::{Deserialize, Serialize};
use crate::address::Address;
use crate::connection::Api;
use crate::types::{Hash, Ledger};
use crate::request::TypedRequest;
use crate::response::{ParseResponseError, TypedResponse, WrongFieldsError};

#[derive(Debug, Serialize)]
pub struct CurrenciesRequest {
    pub account: Address,
    #[serde(flatten)]
    pub ledger: Ledger,
}

#[derive(Debug, Deserialize)]
pub struct CurrenciesResponse {
    pub ledger_hash: Option<Hash>,
    pub ledger_index: Option<u32>,
    pub receive_currencies: Vec<String>,
    pub send_currencies: Vec<String>,
    pub validated: Option<bool>,
}

pub async fn account_currencies<'a, A>(api: &'a A, data: &'a CurrenciesRequest)
    -> Result<TypedResponse<CurrenciesResponse>, A::Error>
    where A: Api,
          A::Error: From<ParseResponseError> + From<WrongFieldsError>
{
    let request = TypedRequest {
        command: "account_currencies",
        api_version: Some(1),
        data,
    };
    Ok(api.call((&request).try_into().map_err(|_| WrongFieldsError::new())?).await?.try_into()?)
}