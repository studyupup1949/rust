# Architecture

## Overview

`acuity-index-api-rs` is a small Tokio-based library crate that provides a typed client for the `acuity-index` JSON-over-WebSocket API.

The codebase is organized around three concerns:

- `src/client.rs`: connection management, request/response correlation, subscription routing, and background I/O
- `src/types.rs`: protocol-facing data model and serde mappings
- `src/error.rs`: public error surface

`src/lib.rs` keeps the public API intentionally narrow by re-exporting the main client, subscription handles, errors, and protocol data types.

## High-Level Runtime Model

The library centers on a single `IndexerClient` value per WebSocket connection.

When `IndexerClient::connect(...)` is called:

1. a WebSocket connection is established with `tokio-tungstenite`
2. the stream is split into a writer half and reader half
3. shared state is allocated behind `Arc<Mutex<...>>`
4. a background Tokio task is spawned to continuously read and route incoming messages

`IndexerClient::close()` uses the writer half to send a WebSocket close frame. The background reader then exits through the existing connection-shutdown path and fans the closure out to pending requests and subscriptions.

This means the client has a split architecture:

- foreground tasks call public async methods like `status()` or `get_events()` and write requests
- one background reader task owns all inbound message handling

That separation is the core architectural choice in this crate.

## Module Responsibilities

### `lib.rs`

`lib.rs` is only the crate boundary. It:

- defines the crate-level example
- declares `client`, `error`, and `types` modules
- re-exports the public API

There is no business logic here.

### `client.rs`

`client.rs` contains almost all runtime behavior.

Main responsibilities:

- create the WebSocket connection
- serialize outgoing requests
- assign request ids
- correlate responses back to the waiting caller
- maintain local subscription registries
- fan out unsolicited notifications to subscribers
- convert protocol failures into `IndexerApiError`
- clean up pending requests and subscriptions when the background task ends

The main public types in this file are:

- `IndexerClient`
- `StatusSubscription`
- `EventSubscription`

### `types.rs`

`types.rs` defines the protocol model exposed by the crate.

It includes:

- query keys: `Key`, `CustomKey`, `CustomValue`
- numeric/text wrappers: `Bytes32`, `U64Text`, `U128Text`
- event and metadata types: `EventRef`, `DecodedEvent`, `StoredEvent`, `PalletMeta`, `EventMeta`, `Span`
- subscription payloads: `EventNotification`, `StatusUpdate`, `SubscriptionTarget`
- internal wire helpers used by the client implementation: `RequestMessage`, `Envelope`, and request/response payload structs

This file also contains helper methods such as:

- `StoredEvent::field()`
- `StoredEvent::variant()`
- `DecodedEvent::pallet_name()`
- `DecodedEvent::event_name()`
- `EventsResponse::event_matches()`

### `error.rs`

`error.rs` defines the crate's error boundary.

The central type is `IndexerApiError`, which merges:

- transport errors
- JSON decoding/encoding errors
- server protocol errors
- client-side coordination failures
- subscription termination signals

`ServerError` is an intermediate structured representation for `error` envelopes before they are converted into the public enum.

## Core Data Flow

### One-Shot Requests

One-shot methods such as `status()`, `variants()`, `size_on_disk()`, and `get_events()` all follow the same path:

1. public method calls `request(message_type, payload)`
2. `request(...)` allocates a fresh numeric id from `next_id`
3. request JSON is built from `RequestMessage<T>`
4. a `oneshot::Sender` is stored in `pending[id]`
5. the JSON message is sent on the shared WebSocket writer
6. the caller awaits the paired `oneshot::Receiver`
7. the background reader receives a response envelope
8. if the envelope has a matching `id`, the pending sender is completed
9. the public method validates the expected response type with `expect_payload(...)`
10. the final typed payload is returned to the caller

This gives the crate a simple RPC-like interface on top of the WebSocket protocol.

### Subscriptions

Subscriptions are handled differently from one-shot requests.

For `subscribe_status()`:

1. a local `mpsc` channel is created
2. the sender is stored in `status_subscribers`
3. a `SubscribeStatus` request is sent to the server
4. the returned `StatusSubscription` owns the receiver and a reference back to the client

For `subscribe_events(key)`:

1. a local `mpsc` channel is created
2. an `EventSubscriber { key, sender }` is stored in `event_subscribers`
3. a `SubscribeEvents` request is sent to the server
4. the returned `EventSubscription` owns the receiver, key, and client reference

When the background reader receives unsolicited messages:

- `status` messages are broadcast to all local status subscribers
- `eventNotification` messages are broadcast only to local event subscribers whose stored `Key` equals the incoming key

This means subscriptions are multiplexed over a single socket and demultiplexed inside the client.

## Concurrency and Shared State

`IndexerClient` is cheap to clone because it is mostly shared pointers:

- `writer: Arc<Mutex<SplitSink<...>>>`
- `pending: Arc<Mutex<HashMap<u64, PendingSender>>>`
- `status_subscribers: Arc<Mutex<HashMap<u64, mpsc::Sender<...>>>>`
- `event_subscribers: Arc<Mutex<HashMap<u64, EventSubscriber>>>`
- `next_id: Arc<AtomicU64>`

The concurrency model is straightforward:

- `AtomicU64` generates ids without taking a mutex
- `Mutex<HashMap<...>>` protects mutable shared registries
- `oneshot` channels pair a single request with a single response
- `mpsc` channels deliver a stream of subscription updates

This keeps the code small and avoids a more complex actor model, while still supporting concurrent callers.

## Incoming Message Routing

The background reader task is implemented by `run_reader(...)` and `handle_message(...)`.

`handle_message(...)` processes each inbound WebSocket frame in this order:

1. normalize the frame into a UTF-8 string
2. deserialize it into `Envelope`
3. if the message has an `id` matching an entry in `pending`, resolve that request first
4. otherwise treat it as an unsolicited protocol message and route by `message_type`

Supported unsolicited message types are:

- `status`
- `eventNotification`
- `subscriptionTerminated`
- `error`

Anything else is currently ignored.

This routing order matters because some message types, especially `error`, can be either request-scoped or connection-scoped depending on whether an `id` is present.

## Error Propagation Model

The crate uses three different error paths.

### Request-scoped failures

If the server responds to a specific request with an envelope containing the same `id` and `type == "error"`, the pending request completes with `IndexerApiError::Server`.

### Subscription failures

If the background reader sees:

- `subscriptionTerminated`, it broadcasts a typed termination error to all status and event subscribers
- unsolicited `error`, it broadcasts the converted error to all subscribers

Subscribers receive errors through their normal `next()` stream.

### Connection/task failure

If the reader loop exits because of a WebSocket error, JSON decoding error, invalid binary frame, or clean socket close:

- all pending requests are failed
- all status subscribers receive an error
- all event subscribers receive an error
- the reader task ends permanently for that client connection

Pending requests are mapped slightly differently depending on cause:

- connection close becomes `RequestCancelled { request_id }`
- other reader-task failures become `BackgroundTaskEnded`

## Subscription Lifetime Semantics

Subscription handles are local receiver wrappers, not exclusive owners of the server-side subscription.

Important behavior:

- dropping a `StatusSubscription` or `EventSubscription` only unregisters that local receiver
- calling `unsubscribe()` also sends the corresponding unsubscribe request to the server
- `unsubscribe_status()` clears all local status subscribers
- `unsubscribe_events(key)` removes all local event subscribers for the same key

This is an important architectural detail: multiple local subscribers may coexist on one connection, but unsubscribe operations act at the shared connection level, not just the handle level.

## Serialization Boundary

The public API is intentionally typed, but it maps very directly to the wire format.

Examples:

- `RequestMessage<T>` serializes as `{ "id": ..., "type": ..., ...payload }`
- `Envelope` deserializes `{ id?, type, data? }`
- `Key` uses tagged serde enums for `Variant` vs `Custom`
- `U64Text` and `U128Text` accept both numeric and string input, but serialize back as strings
- `Bytes32` serializes as `0x`-prefixed hex and accepts prefixed or unprefixed hex on input

This serde-heavy design is what keeps the implementation compact. There is very little manual protocol parsing outside of message dispatch.

## Test Strategy

The project currently relies on unit tests embedded in `types.rs` and `client.rs`.

The tests cover:

- wire-format serialization and deserialization
- wrapper scalar behavior (`Bytes32`, `U64Text`, `U128Text`)
- request envelope validation
- pending request routing
- event and status fanout behavior
- termination and error propagation behavior

There are no integration tests against a live server in this repository. The architecture is therefore validated mainly at the protocol-shape and routing level.

## Design Strengths

- small and easy to understand code surface
- clear separation between transport logic, types, and errors
- one connection can service many concurrent requests and subscriptions
- typed API hides most raw JSON details from callers
- tests exercise the most error-prone serde and routing logic

## Current Constraints

The current design also has a few tradeoffs worth understanding.

- one `IndexerClient` corresponds to one background reader task and one WebSocket connection
- all writes are serialized through a single mutex-protected sink
- subscriber registries are in-memory only
- unsolicited `subscriptionTerminated` and `error` messages are broadcast broadly rather than being narrowed to a single local subscriber
- dropping a subscription handle does not notify the server
- unsubscribe operations are shared-connection operations, so one caller can tear down another caller's server-side subscription for the same client connection
- unknown unsolicited message types are ignored

These are reasonable tradeoffs for a thin high-level client, but they define how consumers should share or isolate `IndexerClient` instances.

## Practical Mental Model

The simplest way to think about the crate is:

- `IndexerClient` is a multiplexed WebSocket session
- `request(...)` turns that session into typed RPC-like calls
- the background reader task is the router
- `StatusSubscription` and `EventSubscription` are local stream receivers fed by that router
- `types.rs` is the protocol contract
- `error.rs` is the failure contract

That model matches the implementation closely and is the right starting point for future changes.
