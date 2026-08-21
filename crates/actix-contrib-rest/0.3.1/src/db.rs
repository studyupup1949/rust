//! Types to handle connections and transactions.
//!
//! Module only available when the `sqlx-postgres` feature is activated.

use sqlx::{Postgres, Transaction};

/// Alias for SQLx Transaction for Postgres.
pub type Tx<'a> = Transaction<'a, Postgres>;
