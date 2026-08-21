# Activeledger - Rust SSE Helper

<img src="https://www.activeledger.io/wp-content/uploads/2018/09/Asset-23.png" alt="Activeledger" width="300"/>

Activeledger is a powerful distributed ledger technology.
Think about it as a single ledger updated simultaneously in multiple locations.
As the data is written to a ledger, it is approved and confirmed by all other locations.

[GitHub](https://github.com/activeledger/activeledger)

[NPM](https://www.npmjs.com/package/@activeledger/activeledger)

---
## This Crate

This crate provides SSE helper functions for Activeledger. It can be used in addition to the main [Activeledger Rust SDK](https://crates.io/crates/activeledger) or on its own.
This crate handles the setup and configuration of Server Sent Events (Eventsource) calls to Activeledger.

## Additional Activeledger crates
Adhearing to the Rust mentality of keeping things small we have created other crates that can be used in conjunction
with this one to add additional functionality.

These crates are:
* [activeledger](https://github.com/activeledger/SDK-Rust) - For handling connections and keys ([Crate](https://crates.io/crates/activeledger))
* [active_tx](https://github.com/activeledger/SDK-Rust-TxBuilder) - To build transactions without worrying about the JSON. ([Crate](https://crates.io/crates/active_tx))

[GitHub](https://github.com/activeledger/SDK-Rust-Events)
[Issues](https://github.com/activeledger/SDK-Rust-Events/issues)

### Activeledger

[Visit Activeledger.io](https://activeledger.io/)

[Read Activeledgers documentation](https://github.com/activeledger/activeledger)

## License

---

This project is licensed under the [MIT](https://github.com/activeledger/activeledger/blob/master/LICENSE) License
