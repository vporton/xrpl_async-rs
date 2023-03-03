use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;
use fragile::Fragile;
use serde_json::Value;
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::de;
use workflow_websocket::client::{Message, WebSocket};
use derive_more::From;
use crate::connection::XrplError::Connection;
use crate::request::{Request, StreamedRequest};
use crate::response::{Response, StreamedResponse};

/// Status not `"success"`
#[derive(Debug)]
pub struct XrplStatusError(pub Option<String>); // FIXME: Need `Option`?

impl XrplStatusError {
    #[allow(clippy::new_without_default)]
    pub fn new(err: Option<String>) -> Self {
        Self(err)
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

#[derive(Debug, From)]
pub enum XrplError {
    Message(String),
    #[from(ignore)]
    Connection(String),
    #[from(ignore)]
    Json(String),
    HttpStatus(StatusCode),
    Disconnect, // WebSocket disconnect
    XrpStatus(XrplStatusError),
}

impl de::Error for XrplError {
    fn custom<T: Display>(msg: T) -> Self {
        XrplError::Message(msg.to_string())
    }
}

impl std::error::Error for XrplError {}

impl Display for XrplError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XrplError::Message(msg) => formatter.write_str(msg),
            other => formatter.write_str(&format!("{:?}", other)), // TODO
            /* and so forth */
        }
    }
}

impl From<serde_json::Error> for XrplError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
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

// TODO: Is this efficient?
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
    pub async fn reconnect(&self) -> Result<(), WebSocketApiError> {
        self.client.reconnect().await.map_err(|e| XrplError::Connection(e.to_string()))
    }
}

pub type WebSocketApiError = XrplError; // TODO

#[async_trait]
impl Api for WebSocketApi {
    type Error = WebSocketApiError;

    #[allow(clippy::needless_lifetimes)]
    async fn call<'a>(&self, request: Request<'a>) -> Result<Response, WebSocketApiError> {
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

impl<'a> WebSocketMessageWaiterWithoutDrop<'a> {
    pub async fn create(api: &'a WebSocketApi, request: Request<'a>)
                        -> Result<WebSocketMessageWaiterWithoutDrop<'a>, WebSocketApiError>
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
    pub async fn wait(&self) -> Result<Response, WebSocketApiError> {
        loop {
            match self.api.client.recv().await.map_err(|e| Connection(e.to_string()))? {
                Message::Open => {},
                Message::Close => {
                    self.api.client.disconnect().await.map_err(|e| Connection(e.to_string()))?; // Prevent attempts to re-connect...
                    // ... because we lost state.
                    unsafe { &mut *self.api.responses.get().as_ptr() }.clear(); // TODO: Check `unsafe`s again.
                    return Err(WebSocketApiError::Disconnect);
                },
                Message::Text(msg) => {
                    let response: Result<StreamedResponse, XrplError> = StreamedResponse::from_str(&msg);
                    match response {
                        Ok(response) => {
                            // TODO: Check `unsafe`s again.
                            unsafe { &mut *self.api.responses.get().as_ptr() }.insert(response.id, response.result);
                            if let Some(response) = unsafe { &mut *self.api.responses.get().as_ptr() }.remove(&response.id) {
                                return Ok(response);
                            }
                        },
                        Err(err) => {
                            return Err(err); // TODO
                        },
                    }
                },
                Message::Binary(_) => {},
            }
        }
    }
    pub fn do_drop(&mut self) {
        // TODO: Check `unsafe` again.
        unsafe { &mut *self.api.responses.get().as_ptr() }.remove(&self.id);
    }
}

/// Wait (by calling `wait` method) for WebSocket response for request passed to `create`
/// while this object exists. Free memory, when the object drops.
pub struct WebSocketMessageWaiter<'a>(WebSocketMessageWaiterWithoutDrop<'a>);

impl<'a> WebSocketMessageWaiter<'a> {
    pub async fn create(api: &'a WebSocketApi, request: Request<'a>)
                        -> Result<WebSocketMessageWaiter<'a>, WebSocketApiError>
    {
        Ok(Self(WebSocketMessageWaiterWithoutDrop::create(api, request).await?))
    }
    pub async fn wait(&self) -> Result<Response, WebSocketApiError> {
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
        let websocket = WebSocket::new("ws://example.com", workflow_websocket::client::Options::default()).unwrap();
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