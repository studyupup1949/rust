//! `accache` — Solana account fetch / cache / persist library for in-process tests.
//!
//! See the crate-level types: [`Accache`], [`KeyedAccount`], [`CacheConfig`], [`Source`].
//!
//! # Quick start
//!
//! Persist a fixture to disk, then load it back offline and feed it to
//! [`mollusk-svm`](https://crates.io/crates/mollusk-svm):
//!
//! ```no_run
//! use accache::{Accache, Account, Compression, KeyedAccount, Pubkey, RefreshPolicy};
//! use accache::format::{Entry, write_file};
//!
//! // Build a fixture file (in real use this would come from `Accache::builder().with_rpc(...)`).
//! let fixture = std::env::temp_dir().join("example.acc");
//! let ka = KeyedAccount::new(
//!     Pubkey::new_from_array([1; 32]),
//!     Account { lamports: 1_000_000, data: vec![], owner: Pubkey::new_from_array([0; 32]),
//!               executable: false, rent_epoch: 0 },
//! );
//! write_file(&fixture, &[Entry::from_keyed(&ka, None)], Compression::Zstd { level: 3 }).unwrap();
//!
//! // Load it back offline and pass to mollusk via the (Pubkey, Account) tuple shape.
//! let acc = Accache::builder()
//!     .with_files([&fixture])
//!     .refresh(RefreshPolicy::Offline)
//!     .build()
//!     .unwrap();
//! let loaded: Vec<(Pubkey, Account)> = acc.get_multiple(&[ka.key]).unwrap()
//!     .into_iter().map(Into::into).collect();
//! assert_eq!(loaded.len(), 1);
//! ```

pub mod account;
pub mod config;
pub mod error;
pub mod format;
pub mod source;

mod accache;

#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "nonblocking")]
pub mod nonblocking;

pub use accache::Accache;
pub use account::KeyedAccount;
pub use config::{CacheConfig, Compression, RefreshPolicy};
pub use error::{AccacheError, Result};
pub use source::{AccacheBuilder, Source};

pub use solana_account::Account;
pub use solana_commitment_config::CommitmentConfig;
pub use solana_pubkey::Pubkey;
