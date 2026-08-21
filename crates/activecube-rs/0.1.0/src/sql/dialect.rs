use crate::compiler::ir::{QueryIR, SqlValue};

/// Trait for SQL dialect implementations.
/// Each database backend (StarRocks, ClickHouse, etc.) implements this
/// to compile a QueryIR into its native SQL syntax.
pub trait SqlDialect: Send + Sync {
    /// Compile a QueryIR into a parameterized SQL string and binding values.
    fn compile(&self, ir: &QueryIR) -> (String, Vec<SqlValue>);

    /// Quote a column or table identifier for this dialect (e.g. backticks for MySQL/StarRocks).
    fn quote_identifier(&self, name: &str) -> String;

    /// Whether the dialect supports `COUNT(DISTINCT col)`.
    fn supports_count_distinct(&self) -> bool {
        true
    }

    /// The placeholder character for parameterized queries (e.g. `?` for MySQL, `$1` for PostgreSQL).
    fn placeholder(&self) -> &str {
        "?"
    }

    /// Dialect name for logging/debugging.
    fn name(&self) -> &str;
}
