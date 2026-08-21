/// A row returned from a SQL query, represented as an ordered map
/// of column name → JSON value. Database-agnostic.
pub type RowMap = indexmap::IndexMap<String, serde_json::Value>;

/// Result of a cube query execution.
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub rows: Vec<RowMap>,
    pub row_count: usize,
    pub sql: String,
}

impl QueryResult {
    pub fn new(rows: Vec<RowMap>, sql: String) -> Self {
        let row_count = rows.len();
        Self { rows, row_count, sql }
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}
