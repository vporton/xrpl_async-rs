use serde::{Deserialize, Deserializer, Serialize};
use crate::address::Address;
use crate::types::{LedgerForRequest, LedgerForResponse};

#[derive(Debug, Serialize)]
struct AccountLinesRequest {
    pub account: Address,
    #[serde(flatten)]
    pub ledger: LedgerForRequest,
    pub peer: Option<Address>,
    pub limit: Option<u16>,
}

#[derive(Debug)]
pub struct TrustLineObject {
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

impl<'de> Deserialize<'de> for TrustLineObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Deserialize)]
        struct TrustLineObject2 {
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
        let value: TrustLineObject2 = TrustLineObject2::deserialize(deserializer)?;
        Ok(TrustLineObject {
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
    pub lines: Vec<TrustLineObject>,
    pub ledger_current_index: u32,
    #[serde(flatten)]
    pub ledger: LedgerForResponse,
}

// TODO