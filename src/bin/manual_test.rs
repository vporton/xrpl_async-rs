use std::fmt::Debug;
use reqwest::Client;
use tokio_stream::StreamExt;
use workflow_websocket::client::{Options, WebSocket};
use xrpl::core::keypairs::derive_keypair;
use xrpl_async::hashes::{Address, Encoding, SecretKey};
use xrpl_async::connection::{Api, JsonRpcApi, WebSocketApi, XrplError};
use xrpl_async::methods::account_channels::{account_channels, ChannelsRequest};
use xrpl_async::methods::submit::sign_and_submit;
use xrpl_async::objects::amount::Amount;
use xrpl_async::txs::payment::{PaymentTransaction, TRANSACTION_TYPE_PAYMENT};
use xrpl_async::types::{Hash, LedgerForRequest};
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

    let our_address = Address::decode("rU4Ai74ohgtUP8evP3qd2HuxWSFvLVt7uh").unwrap();
    let our_seed = "sEdTWjtgXkxfh2p4KrTyDzmKu8aYNnK";
    let (public_key, private_key) = derive_keypair(our_seed, false).unwrap(); // ineffective!
    let (public_key, private_key) =
        (hex::decode(public_key).unwrap(), hex::decode(private_key).unwrap());
    let private_key = SecretKey(Hash(*(<&[u8; 32]>::try_from(&private_key[1..33]).unwrap())));
    let tx = PaymentTransaction {
        account: our_address.clone(),
        transaction_type: TRANSACTION_TYPE_PAYMENT,
        account_txn_id: None,
        fee: None,
        flags: None,
        last_ledger_sequence: None,
        sequence: None,
        source_tag: None,
        ticket_sequence: None,
        amount: Amount {
            value: 10.0,
            currency: "USD".to_string(),
            issuer: Address::decode("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn").unwrap(),
        },
        destination: our_address,
        destination_tag: None,
        invoice_id: None,
        send_max: Some(Amount {
            value: 10.0,
            currency: "USD".to_string(),
            issuer: Address::decode("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn").unwrap(),
        }),
        deliver_min: None,
        signature: None,
        public_key: None,
    };

    println!("** JsonRpcApi transaction **");
    let response = sign_and_submit(&api,
                                   tx.clone(),
                                   &Encoding(public_key.as_slice().try_into().unwrap()),
                                   &private_key,
                                   true).await.unwrap_err();
    println!("TX RESPONSE: {}", response);

    println!("** WebSocketApi transaction **");
    let response = sign_and_submit(&api2,
                                   tx,
                                   &Encoding(public_key.as_slice().try_into().unwrap()),
                                   &private_key, //.as_slice(),
                                   true).await.unwrap_err();
    println!("TX RESPONSE: {}", response);
}