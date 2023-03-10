use std::convert::From;
use serde::{de, Deserialize, Serialize, Serializer};
use xrpl_binary_codec::serializer::HASH_PREFIX_TRANSACTION;
use crate::hashes::{AccountPublicKey, SecretKey};
use crate::connection::{Api, XrplError};
use crate::request::TypedRequest;
use crate::response::TypedResponse;
use crate::txs::{sign_transaction, Transaction};

#[derive(Debug)]
pub struct TransactionRequest {
    pub tx_blob: Vec<u8>,
    pub fail_hard: bool,
}

impl Serialize for TransactionRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        #[derive(Debug, Serialize)]
        pub struct TransactionRequest2<'a> {
            pub tx_blob: &'a String,
            pub fail_hard: Option<bool>,
        }
        TransactionRequest2 {
            tx_blob: &hex::encode(&self.tx_blob),
            fail_hard: if self.fail_hard { Some(true) } else { None },
        }.serialize(serializer)
    }
}

#[derive(Debug, Deserialize)]
pub struct TransactionResponse {
    pub engine_result: String,
    pub engine_result_message: String,
    // pub tx_blob: Vec<u8>,
    // pub tx_json: Value,
    pub accepted: bool,
    pub account_sequence_available: u32,
    pub account_sequence_next: u32,
    pub applied: bool,
    pub broadcast: bool,
    pub kept: bool,
    pub queued: bool,
    #[serde(with = "crate::types::xrp")]
    pub open_ledger_cost: u64,
    pub validated_ledger_index: u32,
}

pub async fn submit<'a, A>(api: &'a A, data: &'a TransactionRequest)
                           -> Result<TypedResponse<TransactionResponse>, A::Error>
    where A: Api,
          A::Error: From<XrplError>
{
    let request = TypedRequest {
        command: "submit",
        api_version: Some(1),
        data,
    };
    Ok(api.call((&request).try_into().map_err(de::Error::custom)?).await?.try_into()?)
}

pub async fn sign_and_submit<'a, A, T>(api: &A,
                                       tx: T,
                                       public_key: &AccountPublicKey,
                                       secret_key: &SecretKey,
                                       fail_hard: bool)
                                       -> Result<TypedResponse<TransactionResponse>, A::Error>
    where A: Api,
          A::Error: From<XrplError>,
          T: Transaction,
{
    let tx = sign_transaction(tx, public_key, secret_key);
    let mut ser = Vec::new();
    T::serialize(&tx, &HASH_PREFIX_TRANSACTION, &mut ser)
        .map_err(|_| de::Error::custom("Cannot serialize a transaction (internal error)"))?;
    let request = TransactionRequest {
        tx_blob: ser,
        fail_hard,
    };
    submit(api, &request).await
}