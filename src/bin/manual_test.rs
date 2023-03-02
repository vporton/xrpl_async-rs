use reqwest::Client;
use xrpl::core::keypairs::derive_keypair;
use xrpl_async::address::{Address, Encoding};
use xrpl_async::connection::JsonRpcApi;
use xrpl_async::methods::submit::sign_and_submit;
use xrpl_async::objects::amount::Amount;
use xrpl_async::txs::payment::PaymentTransaction;
// use xrpl::core::addresscodec::utils::decode_base58;
// use xrpl_async::methods::submit::sign_and_submit;
// use xrpl_async::objects::amount::Amount;
// use xrpl_async::txs::payment::PaymentTransaction;

// async fn basic_test<A: Api>(api: &A)
//     where A::Error: From<XrplError> + Debug
// {
//     let request = ChannelsRequest {
//         account: Address::decode("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn").unwrap(),
//         destination_account: None,
//         ledger: LedgerForRequest::Validated,
//         limit: None,
//     };
//     let (response, mut paginator) = account_channels(api, &request).await.unwrap();
//     println!("{:?}", response);
//     while let Some(item) = paginator.next().await {
//         let item = item.unwrap();
//         println!("- {:?}", item);
//     }
// }

#[tokio::main]
async fn main() {
    println!("** JsonRpcApi **");
    let http_client = Client::new();
    let api = JsonRpcApi::new(http_client, "http://s1.ripple.com:51234/".to_owned());
    // basic_test(&api).await;
    //
    // println!("** WebSocketApi **");
    // let ws = WebSocket::new("wss://s1.ripple.com/", Options::default()).unwrap();
    // ws.connect(true).await.unwrap();
    // let api2 = WebSocketApi::new(ws);
    // basic_test(&api2).await;

    println!("** JsonRpcApi transaction **");
    let our_address = Address::decode("rU4Ai74ohgtUP8evP3qd2HuxWSFvLVt7uh").unwrap();
    let our_seed = "sEdTWjtgXkxfh2p4KrTyDzmKu8aYNnK";
    let (public_key, private_key) = derive_keypair(our_seed, false).unwrap(); // TODO: ineffective!
    let (public_key, private_key) =
        (hex::decode(public_key).unwrap(), hex::decode(private_key).unwrap());
    let private_key = &private_key[1..33];
    let tx = PaymentTransaction {
        transaction_type: 0, // FIXME: not here
        account: our_address.clone(),
        amount: Amount {
            value: 10.0,
            currency: "USD".to_string(),
            issuer: Address::decode("rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn").unwrap(),
        },
        destination: our_address,
        destination_tag: None,
        invoice_id: None,
        send_max: None,
        deliver_min: None,
        signature: None,
        public_key: None,
    };
    let response = sign_and_submit(&api,
                                   tx,
                                   Encoding(public_key.as_slice().try_into().unwrap()),
                                   private_key, //.as_slice(),
                                   true).await;//.unwrap();
    println!("{:?}", response);
}