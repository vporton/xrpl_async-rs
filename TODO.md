- X-addresses: https://xrpl.org/basic-data-types.html
- Generating private keys and transforming to public and addresses.
- Replace `workflow-websocket` by plain `tokio-tungstenite`?
  (however, that is more low-level).
- Remove dependency to `xrpl-rust`.
- Doc comments.
- Better error values.
- Workaround of this [Serde issue](https://github.com/serde-rs/serde/issues/2382)
  (can be done creating our own high-level deserializers independent of
  Serde stages of deserializing).
- `FIXME` and `TODO` in the sources.