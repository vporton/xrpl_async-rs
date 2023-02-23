use lazy_static::lazy_static;
use serde_json::{json, Map, Number, Value};

lazy_static! {
    static ref API_VERSION_KEY: String = "api_version".to_string();
    static ref ID_KEY: String = "id".to_string();
    static ref COMMAND_KEY: String = "command".to_string();
}

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub command: &'a str,
    pub api_version: Option<u32>,
    pub params: Map<String, Value>,
}

/// For JSON RPC.
#[derive(Clone, Debug)]
pub struct TypedRequest<'a, T> {
    pub command: &'a str,
    pub api_version: Option<u32>,
    pub data: T,
}

impl<'a, T: FormatParams> From<&TypedRequest<'a, T>> for Request<'a>
{
    fn from(value: &TypedRequest<'a, T>) -> Self {
        Self {
            command: value.command,
            api_version: value.api_version,
            params: value.data.to_json(),
        }
    }
}

impl<'a, T: FormatParams> FormatRequest for TypedRequest<'a, T> {
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

pub trait FormatParams {
    fn to_json(&self) -> Map<String, Value>;
}

/// For WebSocket.
/// TODO: `pub`?
#[derive(Debug)]
pub struct StreamedRequest<'a> {
    pub request: Request<'a>,
    pub id: u64, // TODO: `pub`?
}

impl<'a> FormatRequest for Request<'a> {
    fn to_json(&self) -> Value {
        let mut params = serde_json::Map::<String, Value>::new();
        if let Some(api_version) = self.api_version {
            params.insert(API_VERSION_KEY.clone(), api_version.into()); // TODO: Don't clone.
        }
        json!({
            "method": self.command,
            "params": [self.params], // yes, the docs say use one-item array
        })
    }
}

impl<'a> FormatRequest for StreamedRequest<'a> {
    fn to_json(&self) -> Value {
        let mut params = serde_json::Map::<String, Value>::new();
        // TODO: Don't clone
        params.insert(ID_KEY.clone(), Value::Number(Number::from(self.id)));
        params.insert(COMMAND_KEY.clone(), Value::String(self.request.command.to_owned()));
        if let Some(api_version) = self.request.api_version {
            params.insert(API_VERSION_KEY.clone(), api_version.into());
        }
        for (key, value) in &self.request.params {
            params.insert(key.clone(), value.to_owned());
        }
        json!(params)
    }
}