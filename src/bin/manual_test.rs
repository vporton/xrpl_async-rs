use reqwest::Client;
use xrpl_async::account::{account_channels, ChannelsRequest};
use xrpl_async::address::Address;
use xrpl_async::connection::JsonRpcApi;
use xrpl_async::types::Ledger;

#[tokio::main]
async fn main() {
    let http_client = Client::new();
    let api = JsonRpcApi::new(http_client, "http://s1.ripple.com:51234/".to_owned());
    let request = ChannelsRequest {
        account: Address::decode("r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59").unwrap(),
        destination_account: None,
        ledger: Ledger::Validated,
        limit: None,
    };
    let (response, _paginator) = account_channels(&api, &request).await.unwrap();
    println!("{:?}", response);
}