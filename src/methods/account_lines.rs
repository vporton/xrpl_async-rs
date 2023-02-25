use serde::{Deserialize, Serialize};
use crate::address::Address;
use crate::types::Ledger;

#[derive(Debug, Serialize)]
struct AccountLinesRequest {
    pub account: Address,
    #[serde(flatten)]
    pub ledger: Ledger,
    pub peer: Option<Address>,
    pub limit: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct AccountLinesResponse {
    // FIXME: more fields
}

// TODO