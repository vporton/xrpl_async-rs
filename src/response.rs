extern crate serde;
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use crate::connection::MyError;

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
    type Error = MyError;

    fn try_from(value: Response) -> Result<Self, MyError> {
        Ok(Self {
            result: T::deserialize(value.result)?,
            load: value.load,
            forwarded: value.forwarded,
        })
    }
}

/// For WebSocket.
#[derive(Debug)]
pub struct StreamedResponse {
    pub result: Response,
    pub id: u64,
    // TODO: `type`
}

impl<'de> Deserialize<'de> for Response {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Deserialize)]
        struct Response2 {
            pub result: Value,
            // TODO: `warnings`
            pub warning: Option<String>,
            pub forwarded: Option<bool>,
        }
        let data: Response2 = Response2::deserialize(deserializer)?.into();
        if data.result.get("status") != Some(&Value::String("success".to_owned())) { // TODO: Don't `.to_owned`
            return Err(serde::de::Error::custom("XPRL not success")).into(); // TODO
        }
        // TODO: Implement without `clone`.
        Ok(Self {
            result: data.result,
            load: data.warning == Some(LOAD_KEY.clone()),
            forwarded: data.forwarded == Some(true),
        })
    }
}

impl<'de> Deserialize<'de> for StreamedResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
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
        let data: StreamedResponse2 = StreamedResponse2::deserialize(deserializer)?.into();
        if data.status != "success" {
            return Err(serde::de::Error::custom("XPRL not success")).into(); // TODO
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
