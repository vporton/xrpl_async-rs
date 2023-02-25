use std::convert::From;
use serde::{de, Deserialize, Serialize, Serializer};
use crate::address::Address;
use crate::connection::{Api, XrplError};
use crate::objects::account_root::AccountRoot;
use crate::objects::signer_list::SignerList;
use crate::types::Ledger;
use crate::request::TypedRequest;
use crate::response::TypedResponse;

#[derive(Debug)]
pub struct AccountInfoRequest {
    pub account: Address,
    pub ledger: Ledger,
    pub queue: bool,
    pub signer_lists: bool,
}

impl Serialize for AccountInfoRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        #[derive(Debug, Serialize)]
        struct AccountInfoRequest2 {
            pub account: Address,
            #[serde(flatten)]
            pub ledger: Ledger,
            pub queue: Option<bool>,
            pub signer_lists: Option<bool>,
            pub strict: Option<bool>,
        }
        let request = AccountInfoRequest2 {
            account: self.account.clone(),
            ledger: self.ledger.clone(),
            queue: if self.queue { Some(true) } else { None },
            signer_lists: if self.signer_lists { Some(true) } else { None },
            strict: Some(true),
        };
        AccountInfoRequest2::serialize(&request, serializer)
    }
}

#[derive(Debug, Deserialize)]
pub struct QueueData {
    pub txn_count: u32,
    // FIXME: more fields
}

#[derive(Debug, Deserialize)]
pub struct AccountInfoResponse {
    pub account_data: AccountRoot,
    pub signer_lists: Vec<SignerList>, // FIXME: The array is always one, element; transform.
    pub ledger_current_index: Option<u32>, // FIXME: mutually exclusive with `ledger_index`
    pub ledger_index: Option<u32>, // FIXME: mutually exclusive with `ledger_index`
    pub queue_data: Option<QueueData>,
    pub validated: Option<bool>, // FIXME: None == false
}

pub async fn account_info<'a, A>(api: &'a A, data: &'a AccountInfoRequest)
    -> Result<TypedResponse<AccountInfoResponse>, A::Error>
    where A: Api,
          A::Error: From<XrplError>
{
    let request = TypedRequest {
        command: "account_info",
        api_version: Some(1),
        data,
    };
    Ok(api.call((&request).try_into().map_err(de::Error::custom)?).await?.try_into()?)
}