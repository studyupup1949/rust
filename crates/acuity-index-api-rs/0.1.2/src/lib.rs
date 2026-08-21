//! High-level async client for the `acuity-index` WebSocket API.
//!
//! ```no_run
//! use acuity_index_api_rs::{CustomKey, CustomScalarValue, IndexerClient, Key};
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
//!             Key::Custom(CustomKey::composite(
//!                 "item_revision",
//!                 [
//!                     CustomScalarValue::Bytes32([0x11; 32].into()),
//!                     CustomScalarValue::U32(7),
//!                 ],
//!             )),
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
    Bytes32, CustomKey, CustomScalarValue, CustomValue, DecodedEvent, EventMatch, EventMeta,
    EventNotification, EventRef, EventsResponse, Key, PalletMeta, Span, StatusUpdate,
    StoredEvent, SubscriptionTarget, U128Text, U64Text,
};
