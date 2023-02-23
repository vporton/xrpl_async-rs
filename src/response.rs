use reqwest::StatusCode;
use derive_more::From;
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use crate::connection::XrpError;

lazy_static! {
    static ref LOAD_KEY: String = "load".to_string();
    static ref SUCCESS_KEY: String = "success".to_string();
    static ref ERROR_KEY: String = "error".to_string();
}

#[derive(Debug)]
pub struct WrongFieldsError;

impl WrongFieldsError {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, From)]
pub enum ParseResponseError {
    Json(serde_json::Error),
    WrongFields(WrongFieldsError),
    Xrp(XrpError),
    /// It may be 503 for rate limited or other HTTP status code.
    HttpStatus(StatusCode),
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
    type Error = ParseResponseError;

    fn try_from(value: Response) -> Result<Self, ParseResponseError> {
        Ok(Self {
            result: T::deserialize(value.result)?,
            load: value.load,
            forwarded: value.forwarded,
        })
    }
}

pub trait ParseResponse: Sized {
    // FIXME: Use it:
    fn parse_error(result: &Value) -> Result<(), ParseResponseError> {
        let status = result.get("status");
        if status == Some(&Value::String(ERROR_KEY.clone())) {
            let error_code = result
                .get("error").ok_or::<ParseResponseError>(WrongFieldsError::new().into())?
                .as_str().ok_or::<ParseResponseError>(WrongFieldsError::new().into())?;
            Err(XrpError::new(error_code.to_owned()).into())
        } else if status == Some(&Value::String(SUCCESS_KEY.clone())) {
            Ok(())
        } else {
            Err(WrongFieldsError::new().into())
        }
    }
}

/// For WebSocket.
#[derive(Debug)]
pub struct StreamedResponse {
    pub response: Response,
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
        // ParseResponse::parse_error(&data.result).unwrap(); // FIXME: Check everything in this line!!!
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
            pub response: Response,
            pub id: u64,
            // TODO: `type`
            // TODO: `warnings`
            pub forwarded: Option<bool>,
            pub warning: Option<String>, // FIXME: Why is it missing in `Response2`?
        }
        let data: StreamedResponse2 = StreamedResponse2::deserialize(deserializer)?.into();
        // TODO: Implement without `clone`.
        let result: Value = data.response.result;
        // ParseResponse::parse_error(&result).unwrap(); // FIXME: everything in this line seems wrong!!
        Ok(StreamedResponse {
            response: Response {
                result,
                load: data.warning == Some(LOAD_KEY.clone()),
                forwarded: data.forwarded == Some(true),
            },
            id: data.id,
        })
    }
}
