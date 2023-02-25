use serde::{Deserialize, Serialize};
use crate::address::Address;
use crate::types::LedgerForRequest;

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
    // FIXME: more fields
}

#[derive(Debug, Deserialize)]
pub struct AccountLinesResponse {
    pub account: Address,
    pub lines: Vec<TrustLineObject>,
    pub ledger_current_index: u32, // FIXME: Check `Ledger` usage here and in other places.
    // FIXME
    // #[serde(flatten)]
    // ledger: Ledger,
}

// TODO