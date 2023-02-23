use reqwest::Client;
use tokio_stream::StreamExt;
use xrpl_async::account::{account_channels, ChannelsRequest};
use xrpl_async::address::Address;
use xrpl_async::connection::JsonRpcApi;
use xrpl_async::types::Ledger;

#[tokio::main]
async fn main() {
    let http_client = Client::new();
    let api = JsonRpcApi::new(http_client, "http://s1.ripple.com:51234/".to_owned());
    // TODO: Does not work:
    // let ws = WebSocket::new("wss://s1.ripple.com/", Options::default()).unwrap();
    // ws.connect(true).await.unwrap();
    // let api = WebSocketApi::new(ws);
    let request = ChannelsRequest {
        account: Address::decode("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn").unwrap(),
        destination_account: None,
        ledger: Ledger::Validated,
        limit: None,
    };
    let (response, mut paginator) = account_channels(&api, &request).await.unwrap();
    println!("{:?}", response);
    while let Some(item) = paginator.next().await {
        let item = item.unwrap();
        println!("- {:?}", item);
    }}