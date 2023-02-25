use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use crate::address::Address;
use crate::connection::{Api, XrplError};
use crate::methods::account_channels::ChannelPaginator;
use crate::paginate::{Paginator, PaginatorExtractor};
use crate::request::TypedRequest;
use crate::response::TypedResponse;
use crate::types::{LedgerForRequest, LedgerForResponse};

#[derive(Debug, Serialize)]
pub struct AccountLinesRequest {
    pub account: Address,
    #[serde(flatten)]
    pub ledger: LedgerForRequest,
    pub peer: Option<Address>,
    pub limit: Option<u16>,
}

#[derive(Debug)]
pub struct AccountLinesPaginator {
    pub account: Address,
    pub balance: f64,
    pub currency: String,
    pub limit: f64,
    pub limit_peer: f64,
    pub quality_in: u32,
    pub quality_out: u32,
    pub no_ripple: Option<bool>,
    pub no_ripple_peer: Option<bool>,
    pub authorized: bool,
    pub peer_authorized: bool,
    pub freeze: bool,
    pub freeze_peer: bool,
}

impl<'de> Deserialize<'de> for AccountLinesPaginator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Deserialize)]
        struct AccountLinesPaginator2 {
            pub account: Address,
            #[serde(with = "crate::types::token")]
            pub balance: f64,
            pub currency: String,
            #[serde(with = "crate::types::token")]
            pub limit: f64,
            pub limit_peer: f64,
            pub quality_in: u32,
            pub quality_out: u32,
            pub no_ripple: Option<bool>,
            pub no_ripple_peer: Option<bool>,
            pub authorized: Option<bool>, // FIXME: None == false
            pub peer_authorized: Option<bool>, // FIXME: None == false
            pub freeze: Option<bool>, // FIXME: None == false
            pub freeze_peer: Option<bool>, // FIXME: None == false
        }
        let value: AccountLinesPaginator2 = AccountLinesPaginator2::deserialize(deserializer)?;
        Ok(AccountLinesPaginator {
            account: value.account,
            balance: value.balance,
            currency: value.currency,
            limit: value.limit,
            limit_peer: value.limit_peer,
            quality_in: value.quality_in,
            quality_out: value.quality_out,
            no_ripple: value.no_ripple,
            no_ripple_peer: value.no_ripple_peer,
            authorized: value.authorized == Some(true),
            peer_authorized: value.peer_authorized == Some(true),
            freeze: value.freeze == Some(true),
            freeze_peer: value.freeze_peer == Some(true),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct AccountLinesResponse {
    pub account: Address,
    pub ledger_current_index: u32,
    #[serde(flatten)]
    pub ledger: LedgerForResponse,
}

impl<'a> PaginatorExtractor<'a> for AccountLinesPaginator {
    fn list_obj(result: &Value) -> Result<&Value, XrplError> {
        result.get("lines").ok_or::<XrplError>(de::Error::missing_field("lines"))
    }
}

pub async fn account_lines<'a, A>(
    api: &'a A,
    data: &'a AccountLinesRequest,
) -> Result<(TypedResponse<AccountLinesResponse>, Paginator<'a, A, ChannelPaginator>), A::Error>
    where A: Api,
          A::Error: From<XrplError>
{
    let request = TypedRequest {
        command: "account_lines",
        api_version: Some(1),
        data,
    };
    let (response, paginator) =
        Paginator::start(api, (&request).try_into().map_err(de::Error::custom)?).await?; // TODO: wrong error
    Ok((response.try_into()?, paginator))
}