use lazy_static::lazy_static;
use serde_json::{Value, json};

pub trait FormatRequest {
    fn to_json(&self) -> Value;
    fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.to_json())
    }
    fn to_string_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.to_json())
    }
}

/// For JSON RPC.
pub struct Request<'a> {
    command: &'a str,
    api_version: Option<u32>,
    params: serde_json::Map<String, Value>,
}

/// For WebSocket.
pub struct StreamedRequest<'a> {
    request: Request<'a>,
    id: &'a str,
}

lazy_static! {
    static ref API_VERSION_KEY: String = "api_version".to_string();
    static ref ID_KEY: String = "id".to_string();
    static ref COMMAND_KEY: String = "command".to_string();
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
        params[&*ID_KEY] = Value::String(self.id.to_owned());
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