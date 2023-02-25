use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize)]
pub struct TrustLineObject {
    pub account: Address,
    #[serde(with = "crate::types::token")]
    pub balance: f64,
    pub currency: String,
    #[serde(with = "crate::types::token")]
    pub limit: f64,
    // FIXME: more fields
}

#[derive(Debug, Deserialize)]
pub struct AccountLinesResponse {
    pub account: Address,
    pub lines: Vec<TrustLineObject>,
    pub ledger_current_index: u32, // FIXME: Check `Ledger` usage here and in other places.
    #[serde(flatten)]
    pub ledger: LedgerForResponse,
}

// TODO