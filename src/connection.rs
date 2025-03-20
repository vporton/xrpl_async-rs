use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::Display;
use fragile::Fragile;
use serde_json::Value;
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::de;
use workflow_websocket::client::{Message, WebSocket};
use derive_more::{From, Display};
use lazy_static::lazy_static;
use crate::connection::XrplError::Connection;
use crate::request::{Request, StreamedRequest};
use crate::response::{Response, StreamedResponse};

/// Status not `"success"`
#[derive(Debug, Display)]
#[display("Server error code: {}", self.code)]
pub struct XrplStatusError {
    pub code: String,
}

impl XrplStatusError {
    #[allow(clippy::new_without_default)]
    pub fn new(code: String) -> Self {
        Self { code }
    }
}

/// `E` is error
#[async_trait]
pub trait Api {
    type Error: From<XrplError>;
    #[allow(clippy::needless_lifetimes)]
    async fn call<'a>(&self, request: Request<'a>) -> Result<Response, Self::Error>;
    // async fn call_typed<'a, T: Into<Request<'a>>, U: TryFrom<Response>>(&self, request: T) -> Result<U, Self::Error> {
    //     Ok(self.call(request.into()).await?.try_into().map_err(|_| WrongFieldsError::new())?)
    // }
}

pub struct JsonRpcApi {
    client: Client,
    url: String,
}

impl JsonRpcApi {
    pub fn new(client: Client, url: String) -> Self {
        Self {
            client,
            url,
        }
    }
}

#[derive(Debug, Display, From)]
pub enum XrplError {
    #[display("{}", _0)]
    Message(String),
    #[from(ignore)]
    #[display("Network: {}", _0)]
    Connection(String),
    #[display("Server: not JSON: {}", _0)]
    #[from(ignore)]
    JsonParse(String),
    #[display("Server: wrong JSON")]
    WrongFormat,
    #[display("HTTP status: {}", _0)]
    HttpStatus(StatusCode),
    #[display("WebSocket disconnected")]
    Disconnect,
    XrplStatus(XrplStatusError),
    #[display("Cannot construct JSON object (internal error)")]
    CannotConstructJson,
}

impl de::Error for XrplError {
    fn custom<T: Display>(msg: T) -> Self {
        XrplError::Message(msg.to_string())
    }
}

impl std::error::Error for XrplError {}

impl From<serde_json::Error> for XrplError {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonParse(value.to_string())
    }
}

impl From<reqwest::Error> for XrplError {
    fn from(value: reqwest::Error) -> Self {
        Self::Connection(value.to_string())
    }
}

#[async_trait]
impl Api for JsonRpcApi {
    type Error = XrplError;
    #[allow(clippy::needless_lifetimes)]
    async fn call<'a>(&self, request: Request<'a>) -> Result<Response, XrplError> {
        let result = self.client.get(&self.url).header("Content-Type", "application/json")
            .body(serde_json::to_string(&request)?)
            .send().await?;
        if !result.status().is_success() {
            return Err(XrplError::HttpStatus(result.status()));
        }
        Ok(Response::from_json(&result.json::<Value>().await?)?)
    }
}

// Is this efficient?
// TODO: It seems that `Mutex` or `RwLock` instead will suit here.
pub struct WebSocketApi {
    client: WebSocket,
    responses: Fragile<RefCell<HashMap<u64, Response>>>,
    id: Fragile<Cell<u64>>,
}

impl WebSocketApi {
    pub fn new(client: WebSocket) -> Self {
        Self {
            client,
            responses: Fragile::new(RefCell::new(HashMap::new())),
            id: Fragile::new(Cell::new(0)),
        }
    }
    pub async fn reconnect(&self) -> Result<(), XrplError> {
        self.client.reconnect().await.map_err(|e| XrplError::Connection(e.to_string()))
    }
}

#[async_trait]
impl Api for WebSocketApi {
    type Error = XrplError;

    #[allow(clippy::needless_lifetimes)]
    async fn call<'a>(&self, request: Request<'a>) -> Result<Response, XrplError> {
        let waiter =
            WebSocketMessageWaiterWithoutDrop::create(self, request).await?;
        waiter.wait().await
    }
}

/// Usually you should use `WebSocketMessageWaiter` instead,
/// because this struct does not free memory automatically.
/// The memory allocated by `create` can be freed by `do_drop`.
struct WebSocketMessageWaiterWithoutDrop<'a> {
    api: &'a WebSocketApi,
    id: u64,
}

lazy_static!{
    static ref RESPONSE_KEY: String = "response".to_string();
}

impl<'a> WebSocketMessageWaiterWithoutDrop<'a> {
    pub async fn create(api: &'a WebSocketApi, request: Request<'a>)
                        -> Result<WebSocketMessageWaiterWithoutDrop<'a>, XrplError>
    {
        let id = api.id.get().get();
        api.id.get().set(id + 1);
        let full_request = StreamedRequest {
            id,
            request,
        };
        api.client.post(serde_json::to_string(&full_request)?.into()).await
            .map_err(|e| XrplError::Connection(e.to_string()))?;
        Ok(Self {
            id,
            api,
        })
    }
    pub async fn wait(&self) -> Result<Response, XrplError> {
        loop {
            match self.api.client.recv().await.map_err(|e| Connection(e.to_string()))? {
                Message::Open => {},
                Message::Close => {
                    self.api.client.disconnect().await.map_err(|e| Connection(e.to_string()))?; // Prevent attempts to re-connect...
                    // ... because we lost state.
                    self.api.responses.get().borrow_mut().clear();
                    return Err(XrplError::Disconnect);
                },
                Message::Text(msg) => {
                    let r: Value = serde_json::from_str(&msg)?;
                    if r.get("type") != Some(&Value::String(RESPONSE_KEY.clone())) {
                        // TODO: Deal with non-`response` ("asynchronous") WebSocket messages.
                        continue;
                    }
                    let response: Result<StreamedResponse, XrplError> = StreamedResponse::from_json(&r);
                    match response {
                        Ok(response) => {
                            self.api.responses.get().borrow_mut().insert(response.id, response.result);
                            if let Some(response) = self.api.responses.get().borrow_mut().remove(&response.id) {
                                return Ok(response);
                            }
                        },
                        Err(err) => {
                            return Err(err);
                        },
                    }
                },
                Message::Binary(_) => {},
            }
        }
    }
    pub fn do_drop(&mut self) {
        self.api.responses.get().borrow_mut().remove(&self.id);
    }
}

/// Wait (by calling `wait` method) for WebSocket response for request passed to `create`
/// while this object exists. Free memory, when the object drops.
pub struct WebSocketMessageWaiter<'a>(WebSocketMessageWaiterWithoutDrop<'a>);

impl<'a> WebSocketMessageWaiter<'a> {
    pub async fn create(api: &'a WebSocketApi, request: Request<'a>)
                        -> Result<WebSocketMessageWaiter<'a>, XrplError>
    {
        Ok(Self(WebSocketMessageWaiterWithoutDrop::create(api, request).await?))
    }
    pub async fn wait(&self) -> Result<Response, XrplError> {
        self.0.wait().await
    }
}

impl<'a> Drop for WebSocketMessageWaiter<'a> {
    fn drop(&mut self) {
        self.0.do_drop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Check that we can acquire two waiters at once.
    // #[test]
    #[allow(unused)]
    fn two_waiters() {
        let websocket = WebSocket::new(Some("ws://example.com"), None).unwrap();
        let api = WebSocketApi::new(websocket);
        let _waiter1 =
            WebSocketMessageWaiter::create(&api, Request {
                command: "test",
                api_version: None,
                params: serde_json::Map::new().into(),
            });
        let _waiter2 =
            WebSocketMessageWaiter::create(&api, Request {
                command: "test",
                api_version: None,
                params: serde_json::Map::new().into(),
            });
    }
}