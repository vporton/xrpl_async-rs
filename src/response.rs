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

#[derive(Clone, Debug, Deserialize)]
pub struct Warning {
    pub id: u32,
    pub message: String,
    pub details: Option<Value>,
}

/// For JSON RPC.
#[derive(Debug)]
pub struct Response {
    pub result: Value,
    pub load: bool,
    pub warnings: Option<Vec<Warning>>,
    pub forwarded: bool,
}

#[derive(Debug)]
pub struct TypedResponse<T> {
    pub result: T,
    pub load: bool,
    pub warnings: Option<Vec<Warning>>,
    pub forwarded: bool,
}

impl<'de, T: Deserialize<'de>> TryFrom<Response> for TypedResponse<T> {
    type Error = XrplError;

    fn try_from(value: Response) -> Result<Self, XrplError> {
        Ok(Self {
            result: T::deserialize(value.result)?,
            load: value.load,
            warnings: value.warnings,
            forwarded: value.forwarded,
        })
    }
}

impl Response {
    pub fn from_json(s: &Value) -> Result<Self, XrplError> {
        #[derive(Deserialize)]
        struct Response2 {
            pub result: Value,
            pub warning: Option<String>,
            pub warnings: Option<Vec<Warning>>,
            pub forwarded: Option<bool>,
        }
        let data: Response2 = serde_json::from_value(s.clone())?; // TODO: Don't `clone`.
        if data.result.get("status") != Some(&Value::String("success".to_owned())) { // TODO: Don't `.to_owned`
            // TODO: duplicate code
            return if let Some(Value::String(err)) = data.result.get("error") {
                Err(XrplStatusError::new(err.clone()).into())
            } else {
                Err(XrplError::WrongFormat)
            };
        }
        // TODO: Implement without `clone`.
        Ok(Self {
            result: data.result,
            load: data.warning == Some(LOAD_KEY.clone()),
            warnings: data.warnings,
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
    // TODO: `type` // FIXME: Accept replies only with `type == "response"`.
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
            pub warnings: Option<Vec<Warning>>,
            // pub status: String, // no need, know by missing `result`
            pub forwarded: Option<bool>,
            pub warning: Option<String>,
        }
        // TODO: Don't `clone`.
        let data: StreamedResponse2 = match serde_json::from_value(s.clone()) {
            Ok(data) => data,
            // TODO: Rewrite this:
            Err(_) => { // no `result`
                return if let Some(Value::String(s)) = s.get("error".to_owned()) {
                    Err(XrplStatusError::new(s.clone()).into())
                } else {
                    Err(XrplError::WrongFormat)
                }
            },
        };
        Ok(StreamedResponse {
            result: Response {
                result: data.result,
                load: data.warning.as_ref() == Some(&*LOAD_KEY),
                warnings: data.warnings,
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
