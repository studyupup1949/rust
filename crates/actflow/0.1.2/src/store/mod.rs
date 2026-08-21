//! Storage layer for persisting workflows, processes, and events.
//!
//! Provides an abstraction over different storage backends:
//! - `MemStore`: In-memory storage for testing
//! - `PostgresStore`: PostgreSQL for production persistence

pub mod data;
mod db;
pub mod query;
mod store;

use std::error::Error;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter};

use crate::{ActflowError, Result};

use query::*;

pub use db::{MemStore, PostgresStore};
pub use store::Store;

/// Maps database errors to ActflowError.
fn map_db_err(err: impl Error) -> ActflowError {
    ActflowError::Store(err.to_string())
}

/// Identifiers for different storage collections.
#[derive(Debug, Clone, AsRefStr, PartialEq, Hash, Eq, EnumIter)]
pub enum StoreIden {
    /// Workflow definitions.
    #[strum(serialize = "workflows")]
    Workflows,
    /// Execution events.
    #[strum(serialize = "events")]
    Events,
    /// Process instances.
    #[strum(serialize = "procs")]
    Procs,
    /// Node execution records.
    #[strum(serialize = "nodes")]
    Nodes,
    /// Log entries.
    #[strum(serialize = "logs")]
    Logs,
}

/// Paginated query result.
#[derive(Debug, Deserialize, Serialize)]
pub struct PageData<T> {
    /// Total number of matching records.
    pub count: usize,
    /// Current page number (1-based).
    pub page_num: usize,
    /// Total number of pages.
    pub page_count: usize,
    /// Number of records per page.
    pub page_size: usize,
    /// Records in the current page.
    pub rows: Vec<T>,
}

/// Trait for types that can identify their storage collection.
pub trait DbCollectionIden {
    /// Returns the collection identifier for this type.
    fn iden() -> StoreIden;
}

/// Trait for database collection operations.
pub trait DbCollection: Send + Sync {
    /// The type of items stored in this collection.
    type Item;

    /// Checks if a record with the given ID exists.
    fn exists(
        &self,
        id: &str,
    ) -> Result<bool>;

    /// Finds a record by ID.
    fn find(
        &self,
        id: &str,
    ) -> Result<Self::Item>;

    /// Queries records with pagination and filtering.
    fn query(
        &self,
        query: &Query,
    ) -> Result<PageData<Self::Item>>;

    /// Creates a new record.
    fn create(
        &self,
        data: &Self::Item,
    ) -> Result<bool>;

    /// Updates an existing record.
    fn update(
        &self,
        data: &Self::Item,
    ) -> Result<bool>;

    /// Deletes a record by ID.
    fn delete(
        &self,
        id: &str,
    ) -> Result<bool>;
}

/// Trait for database store initialization.
pub trait DbStore {
    /// Initializes the database and registers collections with the store.
    fn init(
        &self,
        s: &Store,
    );
}
