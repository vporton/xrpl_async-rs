use std::fmt::Debug;
use reqwest::Client;
use tokio_stream::StreamExt;
use workflow_websocket::client::{Options, WebSocket};
use xrpl_async::account::{account_channels, ChannelsRequest};
use xrpl_async::address::Address;
use xrpl_async::connection::{Api, JsonRpcApi, WebSocketApi};
use xrpl_async::response::{ParseResponseError, WrongFieldsError};
use xrpl_async::types::Ledger;

async fn basic_test<A: Api>(api: &A)
    where A::Error: From<ParseResponseError> + From<WrongFieldsError> + Debug
{
    let request = ChannelsRequest {
        account: Address::decode("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn").unwrap(),
        destination_account: None,
        ledger: Ledger::Validated,
        limit: None,
    };
    let (response, mut paginator) = account_channels(api, &request).await.unwrap();
    println!("{:?}", response);
    while let Some(item) = paginator.next().await {
        let item = item.unwrap();
        println!("- {:?}", item);
    }
}

#[tokio::main]
async fn main() {
    println!("** JsonRpcApi **");
    let http_client = Client::new();
    let api = JsonRpcApi::new(http_client, "http://s1.ripple.com:51234/".to_owned());
    basic_test(&api).await;

    println!("** WebSocketApi **");
    let ws = WebSocket::new("wss://s1.ripple.com/", Options::default()).unwrap();
    ws.connect(true).await.unwrap(); // TODO: Move it into `WebSocketApi::new`?
    let api = WebSocketApi::new(ws);
    basic_test(&api).await;
}