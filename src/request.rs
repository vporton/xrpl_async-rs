use lazy_static::lazy_static;
use serde::{Serialize, Serializer};
use serde_json::{json, Number, Value};

lazy_static! {
    static ref API_VERSION_KEY: String = "api_version".to_string();
    static ref ID_KEY: String = "id".to_string();
    static ref COMMAND_KEY: String = "command".to_string();
}

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub command: &'a str,
    pub api_version: Option<u32>,
    pub params: Value,
}

/// For JSON RPC.
#[derive(Clone, Debug)]
pub struct TypedRequest<'a, T> {
    pub command: &'a str,
    pub api_version: Option<u32>,
    pub data: T,
}

impl<'a, T: Serialize> TryFrom<&TypedRequest<'a, T>> for Request<'a>
{
    type Error = serde_json::Error;

    fn try_from(value: &TypedRequest<'a, T>) -> Result<Self, Self::Error> {
        Ok(Self {
            command: value.command,
            api_version: value.api_version,
            params: value.data.serialize(serde_json::value::Serializer)?,
        })
    }
}

impl<'a, T: Serialize> Serialize for TypedRequest<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        Request::try_from(self).map_err(serde::ser::Error::custom)?.serialize(serializer)
    }
}

/// For WebSocket.
#[derive(Debug)]
pub struct StreamedRequest<'a> {
    pub request: Request<'a>,
    pub id: u64,
}

impl<'a> Serialize for Request<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut params = serde_json::Map::<String, Value>::new();
        if let Some(api_version) = self.api_version {
            params.insert(API_VERSION_KEY.clone(), api_version.into()); // TODO: Don't clone.
        }
        json!({
            "method": self.command,
            "params": [self.params], // yes, the docs say use one-item array
        }).serialize(serializer)
    }
}

impl<'a> Serialize for StreamedRequest<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut params = serde_json::Map::<String, Value>::new();
        params.insert(ID_KEY.clone(), Value::Number(Number::from(self.id)));
        params.insert(COMMAND_KEY.clone(), Value::String(self.request.command.to_owned()));
        if let Some(api_version) = self.request.api_version {
            params.insert(API_VERSION_KEY.clone(), api_version.into());
        }
        if let Some(params2) = self.request.params.as_object() { // hack
            for (key, value) in params2 {
                params.insert(key.to_owned(), value.to_owned());
            }
        }
        json!(params).serialize(serializer)
    }
}