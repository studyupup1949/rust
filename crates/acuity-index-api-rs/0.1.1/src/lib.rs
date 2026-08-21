//! High-level async client for the `acuity-index` WebSocket API.
//!
//! ```no_run
//! use acuity_index_api_rs::{CustomKey, CustomValue, IndexerClient, Key};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = IndexerClient::connect("ws://127.0.0.1:8172").await?;
//!
//!     let spans = client.status().await?;
//!     println!("indexed spans: {spans:?}");
//!
//!     let events = client
//!         .get_events(
//!             Key::Custom(CustomKey {
//!                 name: "ref_index".into(),
//!                 value: CustomValue::U32(42),
//!             }),
//!             Some(100),
//!             None,
//!         )
//!         .await?;
//!     println!("events: {}", events.events.len());
//!     Ok(())
//! }
//! ```
mod client;
mod error;
mod types;

pub use client::{EventSubscription, IndexerClient, StatusSubscription};
pub use error::{IndexerApiError, ServerError};
pub use types::{
    Bytes32, CustomKey, CustomValue, DecodedEvent, EventMatch, EventMeta, EventNotification,
    EventRef, EventsResponse, Key, PalletMeta, Span, StatusUpdate, StoredEvent,
    SubscriptionTarget, U128Text, U64Text,
};
