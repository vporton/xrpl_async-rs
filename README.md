# xrpl_async-rs

_(This project is not ready yet.)_

This is fully asynchronous XRPL client written in idiomatic Rust.

Example:
```rust
let request = ChannelsRequest {
    account: Address::decode("r9cZA1mLK5R5Am25ArfXFmqgNwjZgnfk59"),
    destination_account: None,
    ledger: Ledger::Validated,
    limit: None,   
};
let (response, mut paginator) = account_channels(&api, &request)?;
// Now `response` contains data from response.
// Now `paginator` is a stream that is able to traverse multiple pages.
```

Paginator is implemented using futures and streams.

Internally, `ChannelsRequest` is a type that is convertible to
`Request`. `Request` can be passed to `Api` trait method `call`, that
returns `Response` that is convertible to `ChannelsResponse`. And
similarly for other API methods.

Two implementations of `Api` method are provided:
* `JsonRpcApi`
* `WebSocketApi`