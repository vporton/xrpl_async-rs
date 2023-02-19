use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio_stream::Stream;
use fragile::Fragile;
use derive_more::From;
use lazy_static::lazy_static;
use serde_json::{Number, Value, json};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use workflow_websocket::client::{Message, WebSocket};
use crate::connection::ParseResponseError::HttpStatus;

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
    fn from_json(value: &Value) -> Result<Self, ParseResponseError>;
    fn from_string(s: &str) -> Result<Self, ParseResponseError> {
        Ok(Self::from_json(&serde_json::from_str::<Value>(s)?)?)
    }
    fn parse_error(result: &Value) -> Result<(), ParseResponseError> {
        if result.get("status") == Some(&Value::String(ERROR_KEY.clone())) {
            let error_code = result
                .get("error").ok_or::<ParseResponseError>(WrongFieldsError::new().into())?
                .as_str().ok_or::<ParseResponseError>(WrongFieldsError::new().into())?;
            Err(XrpError::new(error_code.to_owned()).into())
        } else {
            Ok(())
        }
    }
}

/// For JSON RPC.
#[derive(Clone)]
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
    static ref ERROR_KEY: String = "error".to_string();
    static ref MARKER_KEY: String = "marker".to_string();
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

// TODO: Need to extract
impl<'a> ParseResponse for Response {
    fn from_json(value: &Value) -> Result<Self, ParseResponseError> {
        let result = value.get("result").ok_or(WrongFieldsError::new())?;
        // TODO: Implement without `clone`.
        Self::parse_error(result)?;
        Ok(Response {
            result: result.clone(),
            success: result.get("status") == Some(&Value::String(SUCCESS_KEY.clone())),
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

/// `E` is error
#[async_trait]
pub trait Api {
    type Error;
    async fn call<'a>(&self, request: Request<'a>) -> Result<Response, Self::Error>;
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
        Ok(Response::from_json(&result.json::<Value>().await?)?)
    }
}

// TODO: Handling server-side disconnection (e.g. on rate-limiting).
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
}

#[derive(Debug, From)]
pub enum WebSocketApiError {
    WebSocketFail(workflow_websocket::client::Error),
    WebSocketFail2(Arc<workflow_websocket::client::Error>),
    Parse(ParseResponseError),
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
        // FIXME: "This function will block until until the message was relayed to the underlying websocket implementation." - ?
        api.client.send(full_request.to_string()?.into()).await?; // Why do they use `Arc`?
        Ok(Self {
            id,
            api,
        })
    }
    pub async fn wait(&self) -> Result<Response, WebSocketApiError> {
        loop {
            // FIXME: "This function will block until until the message was relayed to the underlying websocket implementation." - ?
            if let Message::Text(msg) = self.api.client.recv().await? {
                let response = StreamedResponse::from_string(&msg)?;
                // TODO: Check `unsafe`s again.
                unsafe { &mut *self.api.responses.get().as_ptr() }.insert(response.id, response.response);
                if let Some(response) = unsafe { &mut *self.api.responses.get().as_ptr() }.remove(&response.id) {
                    return Ok(response);
                }
            } else {
                return Err(WrongFieldsError::new().into()); // TODO: not the best error
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

struct Paginator<'a, A: Api, T: Unpin, F: Fn(&Response) -> Pin<Box<dyn Iterator<Item = T> + Unpin>>> {
    api: &'a A,
    request: Request<'a>,
    list: VecDeque<T>, // more efficient than `Vec`
    // position: usize,
    marker: Option<Value>,
    f: Pin<Box<F>>,
}

impl<'a, A: Api, T: Unpin, F: Fn(&Response) -> Pin<Box<dyn Iterator<Item = T> + Unpin>>> Stream for Paginator<'a, A, T, F> {
    /// TODO: More special error?
    type Item = Result<T, A::Error>;
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>> {
        // FIXME: Check edge cases.
        let this = self.get_mut();
        if let Some(front) = this.list.pop_front() {
            Poll::Ready(Some(Ok(front)))
        } else {
            if let Some(marker) = this.marker.clone() { // Can be done without `clone`?
                let mut request = this.request.clone();
                request.params.insert(MARKER_KEY.clone(), marker);
                match this.api.call(request).as_mut().poll(cx) {
                    Poll::Ready(val) => {
                        let val = val?;
                        this.list = (this.f)(&val).into_iter().collect();
                        this.marker = val.result.get(&*MARKER_KEY).map(|v| v.clone());
                        if let Some(front) = this.list.pop_front() {
                            Poll::Ready(Some(Ok(front)))
                        } else {
                            Poll::Ready(None)
                        }
                    },
                    Poll::Pending => Poll::Pending,
                }
            } else {
                Poll::Ready(None)
            }
        }
    }
}

#[async_trait]
trait Paginated<Api, Element> {
    fn get_raw_list_one_page(result: &Value) -> &Vec<Value>;
    fn parse_element(e: &Value) -> Element;
    fn get_list_one_page(result: &Value) -> Vec<Element> {
        Self::get_raw_list_one_page(result).into_iter().map(|e| Self::parse_element(e)).collect()
    }
    // FIXME
    // async fn get_full_list(api: &Api, request: Request<'async_trait>) {
    //     let result = api.call(request).await?;
    //     yield from get_list_one_page(&result);
    // }
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