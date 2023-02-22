use lazy_static::lazy_static;
use serde_json::{json, Number, Value};

lazy_static! {
    static ref API_VERSION_KEY: String = "api_version".to_string();
    static ref ID_KEY: String = "id".to_string();
    static ref COMMAND_KEY: String = "command".to_string();
}

#[derive(Clone)]
pub struct Request<'a> {
    pub command: &'a str,
    pub api_version: Option<u32>,
    pub params: &'a Value,
}

/// For JSON RPC.
#[derive(Clone)]
pub struct TypedRequest<'a, T> {
    pub command: &'a str,
    pub api_version: Option<u32>,
    pub data: T,
}

impl<'a, T: FormatRequest> From<&TypedRequest<'a, T>> for Request<'a>
{
    fn from(value: &TypedRequest<'a, T>) -> Self {
        Self {
            command: value.command,
            api_version: value.api_version,
            params: &value.data.to_json(),
        }
    }
}

impl<'a, T: FormatRequest> FormatRequest for TypedRequest<'a, T> {
    fn to_json(&self) -> Value {
        Request::from(self).to_json()
    }
}

impl<'a> From<Request<'a>> for Value  {
    fn from(value: Request<'a>) -> Self {
        value.to_json()
    }
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

/// For WebSocket.
/// TODO: `pub`?
pub struct StreamedRequest<'a> {
    pub request: Request<'a>,
    pub id: u64, // TODO: `pub`?
}

impl<'a> FormatRequest for Request<'a> {
    fn to_json(&self) -> Value {
        let mut params = serde_json::Map::<String, Value>::new();
        if let Some(api_version) = self.api_version {
            params[&*API_VERSION_KEY] = Value::String(api_version.to_string());
        }
        json!({
            "method": self.command,
            "params": self.params,
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
        if let Some(params) = self.request.params.as_object() { // dirty hack!
            for (key, value) in params {
                params[key] = value.to_owned();
            }
        }
        json!(params)
    }
}
