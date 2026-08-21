//! RPC layer. Wraps `solana-rpc-client` (blocking + nonblocking) with batched fetches.

pub mod blocking;

#[cfg(feature = "nonblocking")]
pub mod nonblocking;

/// RPC server limit for `getMultipleAccounts`. Larger requests are chunked.
pub const MAX_GMA_BATCH: usize = 100;
