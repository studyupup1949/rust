// adminx-core/src/storage.rs
//
// The database abstraction. Every backend (SeaORM for SQL, Mongo, ...) is a
// `Storage` implementation registered once, globally. Resource default CRUD is
// written against this trait and never names a concrete database.

use crate::error::CoreError;
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use serde_json::{Map, Value};

/// A single column filter applied to a list query.
#[derive(Debug, Clone)]
pub struct FilterClause {
    pub field: String,
    pub op: FilterOp,
    pub value: String,
}

/// How a [`FilterClause`] value is matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    /// Exact match (`col = value`).
    Eq,
    /// Case-insensitive substring match (`col LIKE %value%`).
    Contains,
    /// Greater than or equal (`col >= value`), used for date-range "from".
    Gte,
    /// Less than or equal (`col <= value`), used for date-range "to".
    Lte,
}

/// Pagination + ordering + filters distilled from a request query string.
#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub page: u64,
    pub per_page: u64,
    pub sort_by: Option<String>,
    pub sort_desc: bool,
    /// Active column filters (empty when none requested).
    pub filters: Vec<FilterClause>,
}

impl QueryOptions {
    pub fn offset(&self) -> u64 {
        (self.page.max(1) - 1) * self.per_page
    }
}

/// One page of rows plus the total count for pagination.
#[derive(Debug, Clone)]
pub struct ListPage {
    pub rows: Vec<Value>,
    pub total: u64,
}

/// Result of an insert. `last_insert_id` is backend-dependent (MySQL yields an
/// autoincrement id; Postgres a RETURNING id when available).
#[derive(Debug, Clone, Default)]
pub struct CreateOutcome {
    pub last_insert_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum StorageError {
    NotFound,
    Backend(String),
}

impl From<StorageError> for CoreError {
    fn from(e: StorageError) -> Self {
        match e {
            StorageError::NotFound => CoreError::NotFound,
            StorageError::Backend(m) => CoreError::Internal(m),
        }
    }
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn list(&self, table: &str, opts: &QueryOptions) -> Result<ListPage, StorageError>;

    async fn get(&self, table: &str, pk: &str, id: &str) -> Result<Option<Value>, StorageError>;

    /// Fetch the first row where `column = value`. Used by auth to look up an
    /// admin user by email. Backends that don't support it inherit the default
    /// (unsupported); SQL/Mongo backends override it.
    async fn find_one_by(
        &self,
        _table: &str,
        _column: &str,
        _value: &str,
    ) -> Result<Option<Value>, StorageError> {
        Err(StorageError::Backend(
            "find_one_by not supported by this storage backend".into(),
        ))
    }

    async fn create(
        &self,
        table: &str,
        data: Map<String, Value>,
    ) -> Result<CreateOutcome, StorageError>;

    /// Returns the number of affected rows.
    async fn update(
        &self,
        table: &str,
        pk: &str,
        id: &str,
        data: Map<String, Value>,
    ) -> Result<u64, StorageError>;

    /// `soft = true` should set a `deleted` flag rather than removing the row.
    /// Returns the number of affected rows.
    async fn delete(
        &self,
        table: &str,
        pk: &str,
        id: &str,
        soft: bool,
    ) -> Result<u64, StorageError>;

    /// Execute a raw, backend-specific statement — **SQL** for SeaORM (e.g. an
    /// `INSERT`/`CREATE TABLE`), or a **JSON command document** for Mongo (e.g.
    /// `{"insert":"products","documents":[{...}]}`). Intended for seeding and
    /// migrations. Returns the number of affected records where the backend
    /// reports it. Backends that don't support it inherit this error default.
    async fn execute_raw(&self, _statement: &str) -> Result<u64, StorageError> {
        Err(StorageError::Backend(
            "execute_raw not supported by this storage backend".into(),
        ))
    }

    async fn health(&self) -> bool;
}

static STORAGE: OnceCell<Box<dyn Storage>> = OnceCell::new();

/// Register the global storage backend. Call once during startup.
pub fn set_storage(storage: Box<dyn Storage>) {
    if STORAGE.set(storage).is_err() {
        tracing::warn!("adminx storage backend was already initialized; ignoring reset");
    }
}

/// Access the global storage backend. Panics if never initialized.
pub fn storage() -> &'static dyn Storage {
    STORAGE
        .get()
        .expect("adminx storage backend not initialized; call set_storage() first")
        .as_ref()
}

/// Seed the database by running a batch of raw statements against the active
/// backend, in order, stopping at the first error. Write **SQL** when using a
/// SeaORM backend, or **JSON command documents** when using Mongo — the same
/// call works for both, you just author for whichever backend you registered.
///
/// ```ignore
/// // SeaORM / Postgres:
/// adminx::seed(&[
///     "INSERT INTO categories (name, slug) VALUES ('Books','books') ON CONFLICT DO NOTHING",
/// ]).await?;
///
/// // Mongo:
/// adminx::seed(&[
///     r#"{"insert":"categories","documents":[{"name":"Books","slug":"books"}]}"#,
/// ]).await?;
/// ```
pub async fn seed(statements: &[&str]) -> Result<(), StorageError> {
    let s = storage();
    for stmt in statements {
        s.execute_raw(stmt).await?;
    }
    Ok(())
}
