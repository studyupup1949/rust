#![deny(missing_docs)]

//! # Actix Sled Cache
//! _A caching system built on top of Sled and Actix_
//!
//! This project is designed to allow easy caching in actix-based systems. Specifically, it was
//! developed to hold image files and metadata in a media cache for a simple web application.
//!
//! ### Example:
//! ```rust
//! use actix::{Actor, System};
//! use actix_sled_cache::bincode::Cache;
//! use sled_extensions::{Db, ConfigBuilder};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let sys = System::new("cache-example");
//! let db = Db::start(ConfigBuilder::default().temporary(true).build())?;
//!
//! let mut cache_builder = Cache::builder(db, "simple-cache");
//! cache_builder
//!     .as_mut()
//!     .extend_on_update()
//!     .extend_on_fetch()
//!     .expiration_length(chrono::Duration::seconds(1));
//!
//! let cache: Cache<usize> = cache_builder
//!     .frequency(std::time::Duration::from_secs(1))
//!     .build()?;
//!
//! // Clone the tree out of the cache before starting the cache actor
//! let tree = cache.tree();
//!
//! cache.start();
//!
//! // If un-accessed, this record will be deleted after one second
//! tree.insert("some-key", 5)?;
//!
//! // Commented to prevent a never-ending test
//! // sys.run()?;
//! # Ok(())
//! # }
//! ```
//!
//! The cache created here has a frequency of one second, which means every second it will check
//! for expired records. In many cases, the frequency can be much less often. This cache also has
//! an expiration_length of one second, which means after one second has passed since the last
//! interaction with the record, it is eligible for deletion.

use actix::{Message, Recipient};
use std::sync::Arc;

mod cache;
mod error;

pub use self::{
    cache::{Cache, CacheBuilder},
    error::Error,
};

/// A bincode-backed cache for structured data
pub mod bincode {
    use sled_extensions::bincode::BincodeEncoding;

    use crate::cache;

    /// A cache that stores structured data encoded as bincode
    pub type Cache<V> = cache::Cache<V, BincodeEncoding, BincodeEncoding>;

    /// A builder for the cache that stores structured data encoded as bincode
    pub type CacheBuilder<V> = cache::CacheBuilder<V, BincodeEncoding, BincodeEncoding>;
}

/// A pure bytes-backed cache for binary data
pub mod bin {
    use sled_extensions::{bincode::BincodeEncoding, expiring::plain::PlainEncoding, IVec};

    use crate::cache;

    /// A cache that stores unstructured data encoded as bytes
    pub type Cache = cache::Cache<IVec, BincodeEncoding, PlainEncoding>;

    /// A builder for thate cache that stores unstructured data encoded as bytes
    pub type CacheBuilder = cache::CacheBuilder<IVec, BincodeEncoding, PlainEncoding>;
}

#[derive(Message)]
/// A message containing the key ov an item that has been deleted from a cache
///
/// This message will be sent to all subscribers upon deletion
pub struct Deleted(pub Arc<[u8]>);

#[derive(Message)]
/// A message that registers an actor's interest in cache expiry
pub struct Subscribe(pub Recipient<Deleted>);
