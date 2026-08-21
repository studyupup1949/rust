# Ada SDK for Rust

`AdaClient` represents one authenticated namespace. Principal facades are cached and share the channel plus one lazy, persistent namespace stream for events, signals, and jobs.

## Installation

```sh
cargo add ada-sdk
```

Rust 1.88 or newer is required. The API key authenticates an entire namespace, so construct `AdaClient` only in trusted backend code.

## Backend client

```rust
use ada_sdk::{AdaClient, ClientConfig, StreamConfig};

let ada = AdaClient::connect(ClientConfig {
    endpoint: "https://memory.example.com".into(),
    api_key: std::env::var("ADA_API_KEY")?,
    insecure: false,
    streams: StreamConfig::default(),
}).await?;

let alice = ada.principal("alice");
let bob = ada.principal("bob");

let stop_alice = alice.events().on_memory_ingest_finished(|event| {
    println!("{}", event.document_id);
});
let stop_bob = bob.signals().on_routine_broken(|signal| {
    println!("{signal:?}");
});
let stop_job = alice.jobs().on_progressed(|event| {
    println!("{event:?}");
});

stop_alice.unsubscribe();
stop_alice.unsubscribe();
stop_bob.unsubscribe();
stop_job.unsubscribe();
ada.close().await;
```

Every handler method names one event and accepts its exact generated protobuf payload type. `Unsubscribe` is synchronous and idempotent. There is no public `open`, stream handle, async iterator, or generator API, and removing listeners does not close the upstream namespace stream.

Ingest, recall, document, job, data, and summary calls are available only through `Principal` and its typed modules. Compile-fail tests enforce unknown-handler, wrong-payload, no-`open`, and no-root-data-plane constraints.

Unary calls accept concrete protobuf request types and overwrite their principal:

```rust
let response = alice.ingest(IngestRequest {
    document: Some(document),
    ..Default::default()
}).await?;
let status = alice.documents().get_status(GetDocumentStatusRequest {
    document_id,
    ..Default::default()
}).await?;
```

Configure `after_event_id`, `replay_limit`, and bounded reconnect behavior independently in `StreamConfig.events`, `.signals`, and `.jobs`. Root lifecycle callbacks are also concrete:

```rust
let stop_cursor = ada.lifecycle().on_cursor(|info| {
    save_cursor(info.stream, &info.event_id, &info.principal_id);
});
let stop_terminal = ada.lifecycle().on_terminal(|info| {
    report_failure(info.stream, &info.error);
});
```

Namespace catalog methods are `get_public_event_catalog` and `get_signal_catalog`. `close().await` is idempotent and stops every shared stream and reconnect attempt.

This crate is backend-only. Never embed the namespace API key in a browser. Mint a short-lived principal session with `mint_browser_session` and use the TypeScript gRPC-Web client for browser access.
