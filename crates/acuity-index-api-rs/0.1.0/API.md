# API

## Overview

`acuity-index-api-rs` is a high-level async Rust client for the `acuity-index` WebSocket API.

It wraps the JSON-over-WebSocket protocol exposed by `acuity-index` and provides typed request helpers for common operations:

- fetching indexer status
- fetching pallet/event variants
- fetching indexed events by key
- fetching database size on disk
- subscribing to status updates
- subscribing to event notifications

By default, `acuity-index` serves WebSocket traffic on `ws://127.0.0.1:8172`.

## Runtime Model

This crate is Tokio-based.

- transport: `tokio-tungstenite`
- async runtime: `tokio`
- encoding: `serde` + `serde_json`

`IndexerClient` owns a WebSocket connection and a background reader task.

- outgoing requests are assigned monotonically increasing numeric ids
- responses are matched back to the originating request
- unsolicited notifications are routed to subscription receivers

## Public Types

The crate exports these main public types from `src/lib.rs`:

- `IndexerClient`
- `StatusSubscription`
- `EventSubscription`
- `IndexerApiError`
- `ServerError`
- `Key`
- `CustomKey`
- `CustomValue`
- `Bytes32`
- `U64Text`
- `U128Text`
- `Span`
- `EventRef`
- `DecodedEvent`
- `StoredEvent`
- `EventMatch`
- `EventsResponse`
- `EventNotification`
- `StatusUpdate`
- `PalletMeta`
- `EventMeta`
- `SubscriptionTarget`

## Connecting

Create a client with `IndexerClient::connect`:

```rust
use acuity_index_api_rs::IndexerClient;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = IndexerClient::connect("ws://127.0.0.1:8172").await?;
    let spans = client.status().await?;
    println!("{spans:?}");
    Ok(())
}
```

Signature:

```rust
pub async fn connect(url: &str) -> Result<IndexerClient, IndexerApiError>
```

Behavior:

- opens a WebSocket connection to the given URL
- starts an internal background reader task
- returns a cloneable `IndexerClient`
- intended to be reused for both one-shot requests and subscriptions on the same connection

## One-Shot Requests

### `status`

Fetches the indexer's current indexed block spans.

Signature:

```rust
pub async fn status(&self) -> Result<Vec<Span>, IndexerApiError>
```

Return type:

- `Vec<Span>`

`Span`:

```rust
pub struct Span {
    pub start: u32,
    pub end: u32,
}
```

Example:

```rust
let spans = client.status().await?;
for span in spans {
    println!("{}..={}", span.start, span.end);
}
```

### `variants`

Fetches the chain metadata event variants currently known to the indexer.

Signature:

```rust
pub async fn variants(&self) -> Result<Vec<PalletMeta>, IndexerApiError>
```

Return type:

- `Vec<PalletMeta>`

`PalletMeta`:

```rust
pub struct PalletMeta {
    pub index: u8,
    pub name: String,
    pub events: Vec<EventMeta>,
}

pub struct EventMeta {
    pub index: u8,
    pub name: String,
}
```

Example:

```rust
let pallets = client.variants().await?;
for pallet in pallets {
    println!("{} ({})", pallet.name, pallet.index);
}
```

### `size_on_disk`

Fetches the total sled database size in bytes.

Signature:

```rust
pub async fn size_on_disk(&self) -> Result<u64, IndexerApiError>
```

Example:

```rust
let bytes = client.size_on_disk().await?;
println!("database size: {bytes} bytes");
```

### `get_events`

Fetches indexed events for a given key.

Signature:

```rust
pub async fn get_events(
    &self,
    key: Key,
    limit: Option<u16>,
    before: Option<EventRef>,
) -> Result<EventsResponse, IndexerApiError>
```

Arguments:

- `key`: the index key to query
- `limit`: optional maximum number of events
- `before`: optional cursor for pagination

Return type:

```rust
pub struct EventsResponse {
    pub key: Key,
    pub events: Vec<EventRef>,
    pub decoded_events: Vec<DecodedEvent>,
}
```

`EventRef`:

```rust
pub struct EventRef {
    pub block_number: u32,
    pub event_index: u16,
}
```

`DecodedEvent`:

```rust
pub struct DecodedEvent {
    pub block_number: u32,
    pub event_index: u16,
    pub event: StoredEvent,
}

pub struct StoredEvent {
    pub spec_version: u32,
    pub pallet_name: String,
    pub event_name: String,
    pub pallet_index: u8,
    pub variant_index: u8,
    pub event_index: u16,
    pub fields: serde_json::Value,
}
```

Helpers:

```rust
impl DecodedEvent {
    pub fn pallet_name(&self) -> &str
    pub fn event_name(&self) -> &str
    pub fn variant(&self) -> (u8, u8)
    pub fn field(&self, name: &str) -> Option<&serde_json::Value>
}

impl StoredEvent {
    pub fn variant(&self) -> (u8, u8)
    pub fn field(&self, name: &str) -> Option<&serde_json::Value>
}
```

Correlated event view:

```rust
pub struct EventMatch {
    pub event_ref: EventRef,
    pub decoded_event: Option<DecodedEvent>,
}

impl EventsResponse {
    pub fn event_matches(&self) -> Vec<EventMatch>
}
```

Example:

```rust
use acuity_index_api_rs::{CustomKey, CustomValue, EventRef, Key};

let response = client
    .get_events(
        Key::Custom(CustomKey {
            name: "ref_index".into(),
            value: CustomValue::U32(42),
        }),
        Some(100),
        Some(EventRef {
            block_number: 500,
            event_index: 3,
        }),
    )
    .await?;

println!("{} raw event refs", response.events.len());
println!("{} decoded events", response.decoded_events.len());

for event_match in response.event_matches() {
    if let Some(decoded) = event_match.decoded_event {
        println!("{}::{}", decoded.pallet_name(), decoded.event_name());
    }
}
```

## Subscriptions

### `subscribe_status`

Subscribes to ongoing indexer status updates.

Signature:

```rust
pub async fn subscribe_status(&self) -> Result<StatusSubscription, IndexerApiError>
```

`StatusSubscription` exposes:

```rust
pub async fn next(&mut self) -> Option<Result<StatusUpdate, IndexerApiError>>
pub async fn unsubscribe(self) -> Result<(), IndexerApiError>
```

Notes:

- dropping a `StatusSubscription` removes only that local receiver
- `unsubscribe()` also sends `UnsubscribeStatus` to the server
- `IndexerClient::unsubscribe_status()` removes all local status subscribers for that client connection and unsubscribes the shared server-side status subscription

`StatusUpdate`:

```rust
pub struct StatusUpdate {
    pub spans: Vec<Span>,
}
```

Example:

```rust
let mut subscription = client.subscribe_status().await?;

while let Some(update) = subscription.next().await {
    match update {
        Ok(update) => println!("status update: {:?}", update.spans),
        Err(error) => {
            eprintln!("subscription failed: {error}");
            break;
        }
    }
}
```

### `unsubscribe_status`

Cancels the status subscription on the server and clears local status subscribers.

Signature:

```rust
pub async fn unsubscribe_status(&self) -> Result<(), IndexerApiError>
```

### `subscribe_events`

Subscribes to matching indexed events for a key.

Signature:

```rust
pub async fn subscribe_events(&self, key: Key) -> Result<EventSubscription, IndexerApiError>
```

`EventSubscription` exposes:

```rust
pub async fn next(&mut self) -> Option<Result<EventNotification, IndexerApiError>>
pub async fn unsubscribe(self) -> Result<(), IndexerApiError>
```

Notes:

- dropping an `EventSubscription` removes only that local receiver
- notifications are delivered only to local subscribers whose `Key` matches the incoming notification key
- `unsubscribe()` also sends `UnsubscribeEvents` for that key to the server
- `IndexerClient::unsubscribe_events(key)` removes all local subscribers for that key and unsubscribes that key on the server for the shared connection

`EventNotification`:

```rust
pub struct EventNotification {
    pub key: Key,
    pub event: EventRef,
    pub decoded_event: Option<DecodedEvent>,
}
```

Example:

```rust
use acuity_index_api_rs::{CustomKey, CustomValue, Key};

let mut subscription = client
    .subscribe_events(Key::Custom(CustomKey {
        name: "item_id".into(),
        value: CustomValue::Bytes32(acuity_index_api_rs::Bytes32([0x11; 32])),
    }))
    .await?;

while let Some(notification) = subscription.next().await {
    match notification {
        Ok(notification) => println!("event at #{}:{}", notification.event.block_number, notification.event.event_index),
        Err(error) => {
            eprintln!("event subscription failed: {error}");
            break;
        }
    }
}
```

### `unsubscribe_events`

Cancels the event subscription on the server and clears local event subscribers.

Signature:

```rust
pub async fn unsubscribe_events(&self, key: Key) -> Result<(), IndexerApiError>
```

## Key Types

Queries and event subscriptions use `Key`.

```rust
pub enum Key {
    Variant(u8, u8),
    Custom(CustomKey),
}
```

### `Variant`

Matches a pallet event variant directly by `(pallet_index, variant_index)`.

Example:

```rust
let key = acuity_index_api_rs::Key::Variant(42, 0);
```

### `Custom`

Matches a custom named key value.

```rust
pub struct CustomKey {
    pub name: String,
    pub value: CustomValue,
}
```

`CustomValue` supports:

```rust
pub enum CustomValue {
    Bytes32(Bytes32),
    U32(u32),
    U64(U64Text),
    U128(U128Text),
    String(String),
    Bool(bool),
}
```

Examples:

```rust
use acuity_index_api_rs::{Bytes32, CustomKey, CustomValue, Key, U128Text, U64Text};

let bytes32_key = Key::Custom(CustomKey {
    name: "item_id".into(),
    value: CustomValue::Bytes32(Bytes32([0x11; 32])),
});

let u32_key = Key::Custom(CustomKey {
    name: "ref_index".into(),
    value: CustomValue::U32(42),
});

let u64_key = Key::Custom(CustomKey {
    name: "big_id".into(),
    value: CustomValue::U64(U64Text(42)),
});

let u128_key = Key::Custom(CustomKey {
    name: "huge_id".into(),
    value: CustomValue::U128(U128Text(42)),
});

let string_key = Key::Custom(CustomKey {
    name: "slug".into(),
    value: CustomValue::String("hello-world".into()),
});

let bool_key = Key::Custom(CustomKey {
    name: "published".into(),
    value: CustomValue::Bool(true),
});
```

## Error Handling

All public async methods return `Result<_, IndexerApiError>`.

```rust
pub enum IndexerApiError {
    Url(url::ParseError),
    WebSocket(tokio_tungstenite::tungstenite::Error),
    Json(serde_json::Error),
    RequestCancelled { request_id: u64 },
    ResponseChannelClosed { request_id: u64 },
    Server { code: String, message: String },
    StatusSubscriptionTerminated { reason: String, message: String },
    EventSubscriptionTerminated { reason: String, message: String },
    UnexpectedResponseType { request_id: u64, message_type: String },
    NonUtf8Binary,
    ConnectionClosed,
    BackgroundTaskEnded,
}
```

### Common cases

- `Server { .. }`
  - the indexer returned a structured protocol error
- `UnexpectedResponseType { .. }`
  - the server replied to a request with a different response type than expected
- `StatusSubscriptionTerminated { .. }`
  - the server terminated the status subscription
- `EventSubscriptionTerminated { .. }`
  - the server terminated the event subscription
- `ConnectionClosed`
  - the socket closed while waiting for traffic
- `ResponseChannelClosed { .. }`
  - the client-side waiter was dropped before a response arrived
- `RequestCancelled { .. }`
  - pending request was cancelled because the background connection ended

Example:

```rust
match client.status().await {
    Ok(spans) => println!("{spans:?}"),
    Err(error) => eprintln!("status request failed: {error}"),
}
```

## Protocol Mapping

This crate is a high-level wrapper over the underlying `acuity-index` request types:

- `status()` -> `Status`
- `variants()` -> `Variants`
- `size_on_disk()` -> `SizeOnDisk`
- `get_events(...)` -> `GetEvents`
- `subscribe_status()` -> `SubscribeStatus`
- `unsubscribe_status()` -> `UnsubscribeStatus`
- `subscribe_events(key)` -> `SubscribeEvents`
- `unsubscribe_events(key)` -> `UnsubscribeEvents`

Incoming protocol messages are interpreted as follows:

- request responses with matching `id` complete the waiting request future
- `status` notifications are routed to `StatusSubscription`
- `eventNotification` notifications are routed to `EventSubscription`
- `subscriptionTerminated` is surfaced as subscription errors
- `error` with a matching `id` becomes `IndexerApiError::Server`

## Current Notes

The crate intentionally exposes a high-level API only.

- it does not expose raw request/response wire enums publicly
- it assumes Tokio and `tokio-tungstenite`

## Testing

The crate currently includes unit tests for:

- request serialization
- payload deserialization
- scalar conversions
- notification routing
- error propagation

Run them with:

```bash
cargo test
```
