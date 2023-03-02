use std::fmt::Debug;
use reqwest::Client;
use tokio_stream::StreamExt;
use workflow_websocket::client::{Options, WebSocket};
use xrpl_async::methods::account_channels::{account_channels, ChannelsRequest};
use xrpl_async::address::Address;
use xrpl_async::connection::{Api, JsonRpcApi, XrplError, WebSocketApi};
use xrpl_async::types::LedgerForRequest;
// use xrpl::core::addresscodec::utils::decode_base58;
// use xrpl_async::methods::submit::sign_and_submit;
// use xrpl_async::objects::amount::Amount;
// use xrpl_async::txs::payment::PaymentTransaction;

async fn basic_test<A: Api>(api: &A)
    where A::Error: From<XrplError> + Debug
{
    let request = ChannelsRequest {
        account: Address::decode("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn").unwrap(),
        destination_account: None,
        ledger: LedgerForRequest::Validated,
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
    ws.connect(true).await.unwrap();
    let api2 = WebSocketApi::new(ws);
    basic_test(&api2).await;

    // let our_address = Address::decode("rU4Ai74ohgtUP8evP3qd2HuxWSFvLVt7uh").unwrap();
    // let our_secret = decode_base58("sEdTWjtgXkxfh2p4KrTyDzmKu8aYNnK", &[0x21]).unwrap(); // TODO
    // let tx = PaymentTransaction {
    //     amount: Amount {
    //         value: 10.0,
    //         currency: "XRP".to_string(),
    //         issuer: our_address.clone(),
    //     },
    //     destination: our_address,
    //     destination_tag: None,
    //     invoice_id: None,
    //     send_max: None,
    //     deliver_min: None,
    //     signature: None,
    //     public_key: None,
    // };
    // sign_and_submit(&api,
    //                 tx,
    //                 AccountPublicKey(our_address),
    //                 our_secret.as_slice(),
    //                 true).await.unwrap();
}