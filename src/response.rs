use reqwest::StatusCode;
use derive_more::From;
use lazy_static::lazy_static;
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

impl<T: ParseResponse> TryFrom<Response> for TypedResponse<T> {
    type Error = ParseResponseError;

    fn try_from(value: Response) -> Result<Self, ParseResponseError> {
        Ok(Self {
            result: T::from_json(&value.result)?,
            load: value.load,
            forwarded: value.forwarded,
        })
    }
}

pub trait ParseResponse: Sized {
    fn from_json(value: &Value) -> Result<Self, ParseResponseError>;
    fn from_string(s: &str) -> Result<Self, ParseResponseError> {
        Ok(Self::from_json(&serde_json::from_str::<Value>(s)?)?)
    }
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

// TODO: Need to extract
impl<'a> ParseResponse for Response {
    fn from_json(value: &Value) -> Result<Self, ParseResponseError> {
        let result = value.get("result").ok_or(WrongFieldsError::new())?;
        // TODO: Implement without `clone`.
        Self::parse_error(&result)?;
        Ok(Response {
            result: result.clone(),
            load: value.get("warning") == Some(&Value::String(LOAD_KEY.clone())),
            forwarded: value.get("forwarded") == Some(&Value::Bool(true)),
        })
    }
}

impl<'a> ParseResponse for StreamedResponse {
    fn from_json(value: &Value) -> Result<Self, ParseResponseError> {
        // TODO: Implement without `clone`.
        let result = value.get("result").ok_or(WrongFieldsError::new())?.clone();
        Self::parse_error(&result)?;
        let response = Response {
            result,
            load: value.get("warning") == Some(&Value::String(LOAD_KEY.clone())),
            forwarded: value.get("forwarded") == Some(&Value::Bool(true)),
        };
        Ok(StreamedResponse {
            id: value.get("id").ok_or(WrongFieldsError::new())?.as_u64().ok_or(WrongFieldsError::new())?,
            response,
        })
    }
}
