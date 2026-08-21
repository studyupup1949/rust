# acuity-index-api-rs

High-level Rust client for the `acuity-index` WebSocket API.

The indexer serves JSON-over-WebSocket on `ws://127.0.0.1:8172` by default.

Recent `acuity-index` server versions enforce connection and request limits.
Client integrations should expect connection attempts to fail under overload,
oversized custom keys to be rejected, idle connections to be closed, and
subscription requests beyond the per-connection cap to return a structured
`subscription_limit` server error.

## Quick Start

Add the crate to your `Cargo.toml`, connect to the indexer, and issue typed requests with `IndexerClient`.

Architecture overview: [`ARCHITECTURE.md`](./ARCHITECTURE.md)

Full API reference: [`API.md`](./API.md)

Current server-side limits and failure modes are documented in [`API.md`](./API.md).

```rust
use acuity_index_api_rs::{CustomKey, CustomValue, IndexerClient, Key};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = IndexerClient::connect("ws://127.0.0.1:8172").await?;

    let spans = client.status().await?;
    println!("indexed spans: {spans:?}");

    let events = client
        .get_events(
            Key::Custom(CustomKey {
                name: "ref_index".into(),
                value: CustomValue::U32(42),
            }),
            Some(100),
            None,
        )
        .await?;

    println!("{} matching events", events.events.len());
    for event_match in events.event_matches() {
        if let Some(decoded) = event_match.decoded_event {
            println!("{}::{}", decoded.pallet_name(), decoded.event_name());
        }
    }
    Ok(())
}
```

## Subscription Example

```rust
use acuity_index_api_rs::{CustomKey, CustomValue, IndexerClient, Key};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = IndexerClient::connect("ws://127.0.0.1:8172").await?;

    let spans = client.status().await?;
    println!("indexed spans: {spans:?}");

    let mut subscription = client
        .subscribe_events(Key::Custom(CustomKey {
            name: "ref_index".into(),
            value: CustomValue::U32(42),
        }))
        .await?;

    while let Some(notification) = subscription.next().await {
        let notification = notification?;
        if let Some(decoded) = notification.decoded_event {
            println!("{}::{}", decoded.pallet_name(), decoded.event_name());
        }
        break;
    }

    subscription.unsubscribe().await?;
    Ok(())
}
```

`subscribe_events` and `subscribe_status` return owned subscription handles. Dropping a handle removes only that local receiver. Calling `unsubscribe()` also sends the corresponding unsubscribe request to the server.
