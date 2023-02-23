use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use fragile::Fragile;
use derive_more::From;
use serde_json::Value;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use workflow_websocket::client::{Message, WebSocket};
use crate::response::ParseResponseError::HttpStatus;
use crate::request::{FormatRequest, Request, StreamedRequest};
use crate::response::{ParseResponseError, Response, StreamedResponse, WrongFieldsError};

#[derive(Debug)]
pub struct XrpError {
    error_code: String,
}

impl XrpError {
    pub fn new(error_code: String) -> Self {
        Self {
            error_code,
        }
    }
    pub fn error_code(self) -> String {
        self.error_code
    }
}

/// `E` is error
#[async_trait]
pub trait Api {
    type Error;
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
pub enum JsonRpcApiError {
    Reqwest(reqwest::Error),
    Parse(ParseResponseError),
}

impl From<WrongFieldsError> for JsonRpcApiError {
    fn from(value: WrongFieldsError) -> Self {
        Self::Parse(value.into())
    }
}

impl From<serde_json::Error> for JsonRpcApiError {
    fn from(value: serde_json::Error) -> Self {
        Self::Parse(value.into())
    }
}

#[async_trait]
impl Api for JsonRpcApi {
    type Error = JsonRpcApiError;
    #[allow(clippy::needless_lifetimes)]
    async fn call<'a>(&self, request: Request<'a>) -> Result<Response, JsonRpcApiError> {
        let result = self.client.get(&self.url).header("Content-Type", "application/json")
            .body(request.to_string()?)
            .send().await?;
        if !result.status().is_success() {
            return Err(HttpStatus(result.status()).into());
        }
        Ok(Response::deserialize(&result.json::<Value>().await?)?)
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
        Ok(self.client.reconnect().await?)
    }
}

#[derive(Debug, From)]
pub enum WebSocketApiError {
    WebSocketFail(workflow_websocket::client::Error),
    Parse(ParseResponseError),
    Disconnect,
}

impl From<WrongFieldsError> for WebSocketApiError {
    fn from(value: WrongFieldsError) -> Self {
        Self::Parse(value.into())
    }
}

impl From<serde_json::Error> for WebSocketApiError {
    fn from(value: serde_json::Error) -> Self {
        Self::Parse(value.into())
    }
}

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
        api.client.post(full_request.to_string()?.into()).await?;
        Ok(Self {
            id,
            api,
        })
    }
    pub async fn wait(&self) -> Result<Response, WebSocketApiError> {
        loop {
            match self.api.client.recv().await? {
                Message::Open => {},
                Message::Close => {
                    self.api.client.disconnect().await?; // Prevent attempts to re-connect...
                    // ... because we lost state.
                    unsafe { &mut *self.api.responses.get().as_ptr() }.clear(); // TODO: Check `unsafe`s again.
                    return Err(WebSocketApiError::Disconnect);
                },
                Message::Text(msg) => {
                    let response: StreamedResponse = serde_json::from_str(&msg)?;
                    // TODO: Check `unsafe`s again.
                    unsafe { &mut *self.api.responses.get().as_ptr() }.insert(response.id, response.response);
                    if let Some(response) = unsafe { &mut *self.api.responses.get().as_ptr() }.remove(&response.id) {
                        return Ok(response);
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
                params: serde_json::Map::new(),
            });
        let _waiter2 =
            WebSocketMessageWaiter::create(&api, Request {
                command: "test",
                api_version: None,
                params: serde_json::Map::new(),
            });
    }
}