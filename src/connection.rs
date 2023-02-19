use derive_more::From;
use lazy_static::lazy_static;
use serde_json::{Number, Value, json};

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
}

pub trait FormatRequest {
    fn to_json(&self) -> Value;
    fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.to_json())
    }
    fn to_string_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.to_json())
    }
}

pub trait ParseResponse: Sized {
    fn from_json(value: &Value) -> Result<Self, WrongFieldsError>;
    fn from_string(s: &str) -> Result<Self, ParseResponseError> {
        Ok(Self::from_json(&serde_json::from_str::<Value>(s)?)?)
    }
}

/// For JSON RPC.
pub struct Request<'a> {
    pub command: &'a str,
    pub api_version: Option<u32>,
    pub params: serde_json::Map<String, Value>,
}

/// For WebSocket.
pub struct StreamedRequest<'a> {
    pub request: Request<'a>,
    pub id: u64,
}

lazy_static! {
    static ref API_VERSION_KEY: String = "api_version".to_string();
    static ref ID_KEY: String = "id".to_string();
    static ref COMMAND_KEY: String = "command".to_string();
    static ref LOAD_KEY: String = "load".to_string();
    static ref SUCCESS_KEY: String = "success".to_string();
}

impl<'a> FormatRequest for Request<'a> {
    fn to_json(&self) -> Value {
        let mut params = serde_json::Map::<String, Value>::new();
        if let Some(api_version) = self.api_version {
            params[&*API_VERSION_KEY] = Value::String(api_version.to_string());
        }
        for (key, value) in &self.params {
            params[key] = value.to_owned();
        }
        json!({
            "method": self.command,
            "params": params,
        })
    }
}

impl<'a> FormatRequest for StreamedRequest<'a> {
    fn to_json(&self) -> Value {
        let mut params = serde_json::Map::<String, Value>::new();
        params[&*ID_KEY] = Value::Number(Number::from(self.id));
        params[&*COMMAND_KEY] = Value::String(self.request.command.to_owned());
        if let Some(api_version) = self.request.api_version {
            params[&*API_VERSION_KEY] = Value::String(api_version.to_string());
        }
        for (key, value) in &self.request.params {
            params[key] = value.to_owned();
        }
        json!(params)
    }
}

/// For JSON RPC.
pub struct Response {
    pub result: Value,
    pub success: bool,
    pub load: bool,
    // TODO: `warnings`
    pub forwarded: bool,
}

/// For WebSocket.
pub struct StreamedResponse {
    pub response: Response,
    pub id: u64,
    // TODO: `type`
}

impl<'a> ParseResponse for Response {
    fn from_json(value: &Value) -> Result<Self, WrongFieldsError> {
        let result = value.get("result").ok_or(WrongFieldsError::new())?;
        // TODO: Implement without `clone`.
        Ok(Response {
            result: result.clone(),
            success: result.get("status") == Some(&Value::String(SUCCESS_KEY.clone())),
            load: value.get("warning") == Some(&Value::String(LOAD_KEY.clone())),
            forwarded: value.get("forwarded") == Some(&Value::Bool(true)),
        })
    }
}

impl<'a> ParseResponse for StreamedResponse {
    fn from_json(value: &Value) -> Result<Self, WrongFieldsError> {
        // TODO: Implement without `clone`.
        let response = Response {
            result: value.get("result").ok_or(WrongFieldsError::new())?.clone(),
            success: value.get("status") == Some(&Value::String(SUCCESS_KEY.clone())),
            load: value.get("warning") == Some(&Value::String(LOAD_KEY.clone())),
            forwarded: value.get("forwarded") == Some(&Value::Bool(true)),
        };
        Ok(StreamedResponse {
            id: value.get("id").ok_or(WrongFieldsError::new())?.as_u64().ok_or(WrongFieldsError::new())?,
            response,
        })
    }
}