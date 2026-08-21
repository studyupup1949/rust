use crate::compiler::ir::{CompileResult, QueryIR};

/// Trait for SQL dialect implementations.
/// Each database backend (e.g. ClickHouse) implements this
/// to compile a QueryIR into its native SQL syntax.
pub trait SqlDialect: Send + Sync {
    /// Compile a QueryIR into a parameterized SQL string, binding values,
    /// and an alias remapping table for columns aliased in SELECT for HAVING support.
    fn compile(&self, ir: &QueryIR) -> CompileResult;

    /// Quote a column or table identifier for this dialect (e.g. backticks for ClickHouse).
    fn quote_identifier(&self, name: &str) -> String;

    /// Whether the dialect supports `COUNT(DISTINCT col)`.
    fn supports_count_distinct(&self) -> bool {
        true
    }

    /// The placeholder character for parameterized queries (e.g. `?` for ClickHouse, `$1` for PostgreSQL).
    fn placeholder(&self) -> &str {
        "?"
    }

    /// Dialect name for logging/debugging.
    fn name(&self) -> &str;
}
