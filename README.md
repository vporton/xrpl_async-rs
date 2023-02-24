# xrpl_async-rs

_(This project is not ready yet.)_

This is fully asynchronous XRPL client written in idiomatic Rust.

By fully asynchronous, I mean:
- Single async call not only for JsonRpcApi to receive the result of the call, but also single async call to WebSocketApi to receive the result of the call.
- Multipage answers (with "marker") are single asynchronous stream.

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

I use `serde` for (de)serialization.