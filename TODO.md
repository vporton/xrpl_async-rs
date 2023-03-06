- X-addresses: https://xrpl.org/basic-data-types.html
- Generating private keys and transforming to public and addresses.
- Replace `workflow-websocket` by plain `tokio-tungstenite`?
  (however, that is more low-level).
- Remove dependency to `xrpl-rust`.
- Doc comments.
- Debug print `Hash` and `Account` in hex.
- Asynchronous interface for watching when a ledger/transaction becomes
  included into the ledger (subproject on hold till answer to
  [this issue](https://github.com/XRPLF/xrpl-dev-portal/issues/1794)).
- `FIXME` and `TODO` in the sources.
- File `TODO-checkboxes.md`.