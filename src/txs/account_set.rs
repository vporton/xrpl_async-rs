use serde::Serialize;
use crate::address::Address;

#[derive(Serialize)]
pub struct AccountSet {
    #[serde(rename = "Account")]
    pub account: Address,
    // FIXME: more fields
}