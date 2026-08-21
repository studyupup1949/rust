# ADBC-Taos Project Index

**Version:** 0.1.0
**Status:** Active Development
**Last Updated:** 2026-01-03

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture](#architecture)
3. [API Reference](#api-reference)
4. [Module Structure](#module-structure)
5. [Type Mappings](#type-mappings)
6. [Examples](#examples)
7. [Testing](#testing)
8. [Benchmarks](#benchmarks)
9. [Dependencies](#dependencies)
10. [Development Workflow](#development-workflow)

---

## Project Overview

ADBC-Taos is a Rust implementation of the Arrow Database Connectivity (ADBC) driver for TDengine time-series database. It provides Arrow-native data access without unnecessary data copies, enabling high-performance analytics.

### Key Features

- **Full ADBC Core API Implementation**: Complete support for ADBC driver, database, connection, and statement interfaces
- **TDengine 3.3.0+ Support**: Compatible with latest TDengine features and data types
- **Connection Pooling**: Built-in connection pooling via `deadpool` for high-concurrency scenarios
- **Prepared Statements**: Parameter binding support for efficient query execution
- **Supertable Metadata**: TDengine-specific metadata introspection for supertables and tags
- **Comprehensive Type Support**: All TDengine types including JSON, GEOMETRY, DECIMAL, BLOB
- **Zero-Copy Data Access**: Direct Arrow format conversion using columnar block-based access

### Documentation

- **[CLAUDE.md](CLAUDE.md)** - Development guidelines and build instructions
- **[README.md](README.md)** - User-facing documentation and quick start guide
- **[TODO.md](TODO.md)** - Implementation roadmap and task tracking

---

## Architecture

```
adbc-taos/
├── src/
│   ├── lib.rs              # Public API exports and module declarations
│   ├── driver.rs           # TaosDriver: Entry point for creating databases
│   ├── database.rs         # TaosDatabase: Configuration and connection factory
│   ├── connection.rs       # TaosConnection: Active database connection
│   ├── statement.rs        # TaosStatement: SQL execution and parameter binding
│   ├── reader.rs           # RecordBatch readers for streaming results
│   ├── pool.rs             # Connection pooling implementation
│   ├── error.rs            # Error types and conversions
│   ├── types.rs            # Type definitions and utilities
│   └── utils.rs            # Runtime handling for async/sync bridging
├── examples/               # Usage examples
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
└── Cargo.toml              # Package configuration
```

### Design Patterns

1. **Async Runtime Management** (`utils.rs`): Custom `Runtime` enum wraps Tokio runtime with automatic detection of existing runtime via `Handle::try_current()`

2. **Arc-Based Sharing**: Connections and shared state use `Arc<T>` for thread-safe reference counting

3. **Option Pattern**: Components implement `Optionable` trait from `adbc_core` for standardized configuration management

4. **Columnar Data Access**: Reader implementation uses block-based columnar access instead of row iteration for optimal performance

---

## API Reference

### Core Types

#### `TaosDriver`
**Location:** `src/driver.rs:6`

Entry point for creating database instances. Implements `adbc_core::Driver` trait.

```rust
pub struct TaosDriver {}

impl Driver for TaosDriver {
    type DatabaseType = TaosDatabase;

    fn new_database(&mut self) -> Result<Self::DatabaseType>;
    fn new_database_with_opts(&mut self, opts: ...) -> Result<Self::DatabaseType>;
}
```

**Usage:**
```rust
let driver = TaosDriver::default();
let db = driver.new_database()?;
```

---

#### `TaosDatabase`
**Location:** `src/database.rs:21`

Configuration holder that stores connection parameters and creates connections with optional pooling.

**Public Fields:**
- `uri: String` - TDengine DSN (Data Source Name)
- `user: String` - Username
- `password: String` - Password

**Key Methods:**
- `set_pool_size(&mut self, size: usize)` - Configure connection pool (0 = no pooling)
- `pool_size(&self) -> usize` - Get current pool size
- `new_connection()` - Create new connection (ADBC trait)

**Usage:**
```rust
let mut db = TaosDatabase::default();
db.set_option(OptionDatabase::Uri, OptionValue::String("taos://127.0.0.1:6030".into()))?;
db.set_pool_size(10);  // Enable pooling with max 10 connections
let conn = db.new_connection()?;
```

---

#### `TaosConnection`
**Location:** `src/connection.rs:23`

Wraps a TDengine connection and provides metadata query capabilities.

**Key Methods:**
- `server_version(&self) -> &str` - Get TDengine server version
- `inner(&self) -> &Taos` - Access underlying TDengine connection
- `get_table_tags(&self, catalog, table_name) -> RecordBatchReader` - Query supertable tags
- `is_supertable(&self, catalog, table_name) -> bool` - Check if table is supertable
- `new_statement(&mut self) -> Result<TaosStatement>` - Create statement (ADBC trait)
- `get_info(&self, codes) -> RecordBatchReader` - Get driver/database info
- `get_objects(&self, depth, ...) -> RecordBatchReader` - Introspect database objects
- `get_table_schema(&self, catalog, db_schema, table_name) -> Schema` - Get table schema

**Usage:**
```rust
let mut conn = db.new_connection()?;
println!("Server version: {}", conn.server_version());

let tags = conn.get_table_tags(Some("my_db"), "my_supertable")?;
for batch in tags {
    // Process tag metadata
}
```

---

#### `TaosStatement`
**Location:** `src/statement.rs:30`

Handles SQL query preparation, parameter binding, and execution.

**Key Methods:**
- `set_sql_query(&mut self, query: String)` - Set SQL query string
- `execute(&mut self) -> Result<RecordBatchReader>` - Execute query and stream results
- `execute_update(&mut self) -> Result<i64>` - Execute INSERT/UPDATE/DELETE
- `bind_batch(&mut self, batch: RecordBatch)` - Bind Arrow batch as parameters
- `prepare(&mut self)` - Prepare statement for parameter binding

**Usage:**
```rust
let mut stmt = conn.new_statement()?;
stmt.set_sql_query("SELECT * FROM my_table WHERE ts > ?")?;
let reader = stmt.execute()?;

for batch in reader {
    let batch = batch?;
    // Process RecordBatch
}
```

---

#### RecordBatch Readers
**Location:** `src/reader.rs`

- **`TaosRecordBatchReader`**: Streams TDengine query results using block-based columnar access for optimal performance
- **`VecRecordBatchReader`**: Iterator wrapper for pre-loaded RecordBatches

**Optimization:** Uses columnar block access (`RawBlock` API) instead of row-by-row iteration, significantly reducing conversion overhead.

---

### Error Handling

#### `TaosError`
**Location:** `src/error.rs`

Custom error type with ADBC status code mapping.

**Error Categories:**
- `Connection` - Connection establishment failures
- `InvalidOption` - Invalid configuration options
- `Query` - SQL execution errors
- `Conversion` - TDengine-to-Arrow type conversion failures

**Usage:**
```rust
match result {
    Ok(data) => process(data),
    Err(TaosError::Connection(msg)) => eprintln!("Connection failed: {}", msg),
    Err(TaosError::Query(msg)) => eprintln!("Query error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

---

## Module Structure

### Public API (Re-exports)

**File:** `src/lib.rs:75-80`

```rust
pub use connection::TaosConnection;
pub use database::TaosDatabase;
pub use driver::TaosDriver;
pub use error::TaosError;
pub use pool::{TaosPool, TaosPoolConfig, PooledTaosConnection};
pub use utils::{Runtime, block_on_on_runtime};
```

### Module Dependencies

```
driver.rs
  └── database.rs
        ├── connection.rs
        │     ├── statement.rs
        │     │     └── reader.rs
        │     └── error.rs
        ├── pool.rs
        ├── error.rs
        └── utils.rs
```

---

## Type Mappings

### TDengine to Arrow Type Conversion

**Function:** `map_taos_ty_to_arrow()` in `src/connection.rs:568`

| TDengine Type | Arrow Type | Notes |
|--------------|------------|-------|
| `BOOL` | `Boolean` | |
| `TINYINT` | `Int8` | Signed 8-bit |
| `SMALLINT` | `Int16` | Signed 16-bit |
| `INT` | `Int32` | Signed 32-bit |
| `BIGINT` | `Int64` | Signed 64-bit |
| `FLOAT` | `Float32` | IEEE 754 single precision |
| `DOUBLE` | `Float64` | IEEE 754 double precision |
| `VARCHAR` | `Binary` | Variable-length binary |
| `VARBINARY` | `Binary` | Variable-length binary |
| `NCHAR` | `Utf8` | Unicode string |
| `JSON` | `Utf8` | JSON as string |
| `TIMESTAMP` | `Timestamp(Millisecond, None)` | Millisecond precision |
| `TINYINT UNSIGNED` | `UInt8` | Unsigned 8-bit |
| `SMALLINT UNSIGNED` | `UInt16` | Unsigned 16-bit |
| `INT UNSIGNED` | `UInt32` | Unsigned 32-bit |
| `BIGINT UNSIGNED` | `UInt64` | Unsigned 64-bit |
| `GEOMETRY` | `Binary` | Spatial data type |
| `DECIMAL` | `Decimal128(38, 0)` | Maximum precision |

### Arrow to TDengine Parameter Binding

**Function:** `arrow_array_to_column_view()` in `src/statement.rs:66`

Supported Arrow types for parameter binding:
- `BooleanArray`, `Int8Array`...`Int64Array`
- `UInt8Array`...`UInt64Array`
- `Float32Array`, `Float64Array`
- `StringArray`, `BinaryArray`
- `TimestampMillisecondArray`, `TimestampMicrosecondArray`, `TimestampNanosecondArray`

---

## Examples

### Basic Query
**File:** `examples/basic.rs`

```rust
use adbc_core::{Database, Connection, Statement, Driver};

let driver = TaosDriver::default();
let mut db = driver.new_database()?;
db.set_option(OptionDatabase::Uri, OptionValue::String("taos://127.0.0.1:6030".into()))?;

let mut conn = db.new_connection()?;
let mut stmt = conn.new_statement()?;
stmt.set_sql_query("SELECT * FROM my_table LIMIT 10")?;
let reader = stmt.execute()?;

for batch in reader {
    let batch = batch?;
    println!("Got {} rows", batch.num_rows());
}
```

### Data Insertion
**File:** `examples/insert.rs`

```rust
let mut stmt = conn.new_statement()?;
stmt.set_sql_query("INSERT INTO my_db.data VALUES (NOW, 25.5)")?;
let affected_rows = stmt.execute_update()?;
println!("Inserted {} rows", affected_rows);
```

### Metadata Query
**File:** `examples/metadata.rs`

```rust
// Get table schema
let schema = conn.get_table_schema(Some("my_db"), None, "my_table")?;
for field in schema.fields() {
    println!("{}: {:?}", field.name(), field.data_type());
}

// Get supertable tags
let tags = conn.get_table_tags(Some("my_db"), "my_supertable")?;
for batch in tags {
    // Process tag metadata
}
```

### Connection Pooling
**File:** `examples/query.rs` (modified)

```rust
let mut db = TaosDatabase::default();
db.uri = "taos://root:taosdata@127.0.0.1:6030".into();
db.set_pool_size(10);  // Pool with max 10 connections

// Create multiple connections efficiently
for i in 0..5 {
    let conn = db.new_connection()?;
    // Use connection
}
```

---

## Testing

### Test Structure

```
tests/
├── common/
│   └── mod.rs              # Test utilities and fixtures
├── integration_tests.rs    # Main integration test suite
└── readme_examples.rs      # README example validation
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_basic_query

# Run integration tests only
cargo test --test integration_tests
```

### Test Environment

Configure connection in `.env`:
```
TAOS_HOST="127.0.0.1"
TAOS_PORT="6030"
TAOS_USER="root"
TAOS_PASSWORD="taosdata"
```

### Unit Tests

**File:** `src/database.rs:286-363`

Database configuration and DSN parsing tests:
- `test_default_database()` - Default configuration
- `test_build_dsn_simple()` - Basic DSN construction
- `test_build_dsn_with_credentials_in_uri()` - URI with credentials
- `test_build_dsn_special_characters()` - Special character encoding
- `test_build_dsn_invalid_uri()` - Error handling
- `test_set_option_*` - Option setting/getting
- `test_pool_size_*` - Pool configuration

---

## Benchmarks

### Benchmark Suite
**File:** `benches/conversion_benchmark.rs`

Performance benchmarks for TDengine-to-Arrow data conversion using Criterion.

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench conversion_benchmark conversion_functions

# Generate detailed report
cargo bench -- --save-baseline main
```

### Benchmark Categories

1. **Type Conversion Benchmarks**: Measure conversion performance for each TDengine data type
2. **Columnar vs Row-based**: Compare block-based columnar access with row iteration
3. **Batch Size Scaling**: Test performance across different RecordBatch sizes
4. **Throughput Metrics**: Rows/second and MB/second for conversion pipeline

### Recent Optimization

**Commit:** `6606c74` - perf(reader): optimize TDengine-to-Arrow conversion with columnar access

Replaced row-by-row iteration with block-based columnar access using `RawBlock` API, achieving significant performance improvements.

---

## Dependencies

### Runtime Dependencies

```toml
[dependencies]
adbc_core = "0.21.0"           # ADBC trait definitions
arrow-array = "57.1.0"         # Arrow data structures
arrow-schema = "57.1.0"        # Arrow schema types
taos-client = { version = "0.12.3", package = "taos" }  # TDengine driver
deadpool = "0.10.0"            # Connection pooling
thiserror = "2.0.17"           # Error derivation
tokio = { version = "1.45.1", features = ["full"] }  # Async runtime
urlencoding = "2.1.3"          # URL encoding for DSN
serde = { version = "1.0.228", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"
```

### Development Dependencies

```toml
[dev-dependencies]
dotenvy = "0.15.7"             # Environment variable loading
anyhow = "1.0.100"             # Error handling in examples/tests
criterion = "0.5"              # Benchmarking framework
```

### System Requirements

- **Rust:** 1.75+ (edition 2024)
- **TDengine:** 3.3.0.0 or higher
- **Operating System:** Linux, macOS, Windows (via TDengine client libraries)

---

## Development Workflow

### Build Commands

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Quick type check
cargo check

# Lint code
cargo clippy

# Format code
cargo fmt
```

### Documentation Generation

```bash
# Generate documentation
cargo doc --no-deps

# Generate and open in browser
cargo doc --open

# Document with private items
cargo doc --document-private-items
```

### Git Workflow

Recent commits:
- `bd0f3dc` - feat: add benchmark infrastructure for TDengine-to-Arrow conversion
- `6606c74` - perf(reader): optimize TDengine-to-Arrow conversion with columnar access
- `4678d9d` - feat: implement ADBC Phase 2 - Statement execution and query results

### Code Quality

- **Rust Guidelines:** Microsoft-style Rust development discipline (see `rust-guidelines` skill)
- **LSP Integration:** rust-analyzer for code intelligence and refactoring
- **Testing:** Comprehensive unit and integration test coverage
- **Benchmarks:** Criterion-based performance tracking
- **Documentation:** M-CANONICAL-DOCS format for public APIs

---

## Reference Implementations

When implementing features, refer to:
- [adbc-clickhouse](https://github.com/if0ne/adbc-clickhouse.git) - Similar ADBC driver for ClickHouse
- [adbc-rs](https://github.com/alexandreyc/adbc-rs.git) - ADBC core implementations

## TDengine Resources

- [Official Documentation](https://docs.tdengine.com/)
- [Rust Client Library](https://docs.tdengine.com/tdengine-reference/client-libraries/rust/)
- [Data Types](https://docs.tdengine.com/tdengine-reference/sql-manual/data-types/)
- [Supertables](https://docs.tdengine.com/tdengine-reference/sql-manual/manage-supertables/)

---

## License

Apache License 2.0

---

**Generated:** 2026-01-03
**Project Status:** Phase 2 Complete (Statement execution and query results)
