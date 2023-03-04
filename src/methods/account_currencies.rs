use std::convert::From;
use serde::{de, Deserialize, Serialize};
use crate::hashes::Address;
use crate::connection::{Api, XrplError};
use crate::types::{Hash, LedgerForRequest};
use crate::request::TypedRequest;
use crate::response::TypedResponse;

#[derive(Debug, Serialize)]
pub struct CurrenciesRequest {
    pub account: Address,
    #[serde(flatten)]
    pub ledger: LedgerForRequest,
}

#[derive(Debug, Deserialize)]
pub struct CurrenciesResponse {
    pub ledger_hash: Option<Hash<32>>,
    pub ledger_index: Option<u32>,
    pub receive_currencies: Vec<String>,
    pub send_currencies: Vec<String>,
    pub validated: Option<bool>,
}

pub async fn account_currencies<'a, A>(api: &'a A, data: &'a CurrenciesRequest)
    -> Result<TypedResponse<CurrenciesResponse>, A::Error>
    where A: Api,
          A::Error: From<XrplError>
{
    let request = TypedRequest {
        command: "account_currencies",
        api_version: Some(1),
        data,
    };
    Ok(api.call((&request).try_into().map_err(de::Error::custom)?).await?.try_into()?)
}