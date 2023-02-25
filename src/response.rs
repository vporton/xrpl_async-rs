extern crate serde;

use std::str::FromStr;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::Value;
use crate::connection::{XrplError, XrplStatusError};

lazy_static! {
    static ref LOAD_KEY: String = "load".to_string();
    static ref SUCCESS_KEY: String = "success".to_string();
    static ref ERROR_KEY: String = "error".to_string();
}

/// For JSON RPC.
#[derive(Debug)]
pub struct Response {
    pub result: Value,
    pub load: bool,
    // TODO: `warnings`
    pub forwarded: bool,
}

#[derive(Debug)]
pub struct TypedResponse<T> {
    pub result: T,
    pub load: bool,
    // TODO: `warnings`
    pub forwarded: bool,
}

impl<'de, T: Deserialize<'de>> TryFrom<Response> for TypedResponse<T> {
    type Error = XrplError;

    fn try_from(value: Response) -> Result<Self, XrplError> {
        Ok(Self {
            result: T::deserialize(value.result)?,
            load: value.load,
            forwarded: value.forwarded,
        })
    }
}

impl Response {
    pub fn from_json(s: &Value) -> Result<Self, XrplError> {
        #[derive(Deserialize)]
        struct Response2 {
            pub result: Value,
            // TODO: `warnings`
            pub warning: Option<String>,
            pub forwarded: Option<bool>,
        }
        let data: Response2 = serde_json::from_value(s.clone())?; // TODO: Don't `clone`.
        if data.result.get("status") != Some(&Value::String("success".to_owned())) { // TODO: Don't `.to_owned`
            return Err(XrplStatusError::new().into());
        }
        // TODO: Implement without `clone`.
        Ok(Self {
            result: data.result,
            load: data.warning == Some(LOAD_KEY.clone()),
            forwarded: data.forwarded == Some(true),
        })
    }
}

impl FromStr for Response {
    type Err = XrplError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_json(&serde_json::from_str::<Value>(s)?)
    }
}

/// For WebSocket.
#[derive(Debug)]
pub struct StreamedResponse {
    pub result: Response,
    pub id: u64,
    // TODO: `type`
}

// https://github.com/serde-rs/serde/issues/2382
// impl<'de> Deserialize<'de> for Response {

impl StreamedResponse {
    pub fn from_json(s: &Value) -> Result<Self, XrplError> {
        #[derive(Deserialize)]
        struct StreamedResponse2 {
            pub result: Value,
            pub id: u64,
            // TODO: `type`
            // TODO: `warnings`
            pub status: String,
            pub forwarded: Option<bool>,
            pub warning: Option<String>,
        }
        let data: StreamedResponse2 = serde_json::from_value(s.clone())?; // TODO: Don't `clone`.
        if data.status != "success" {
            return Err(XrplStatusError::new().into());
        }
        Ok(StreamedResponse {
            result: Response {
                result: data.result,
                load: data.warning == Some(LOAD_KEY.clone()), // TODO: Implement without `clone`.
                forwarded: data.forwarded == Some(true),
            },
            id: data.id,
        })

    }
}

impl FromStr for StreamedResponse {
    type Err = XrplError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_json(&serde_json::from_str::<Value>(s)?)
    }
}

// https://github.com/serde-rs/serde/issues/2382
// impl<'de> Deserialize<'de> for StreamedResponse;
