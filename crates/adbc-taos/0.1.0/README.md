# ADBC-Taos

[![Crates.io](https://img.shields.io/crates/v/adbc-taos)](https://crates.io/crates/adbc-taos)
[![Documentation](https://docs.rs/adbc-taos/badge.svg)](https://docs.rs/adbc-taos)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

**ADBC-Taos** is a high-performance Rust driver for [TDengine](https://www.tdengine.com/) time-series database, implementing the [Arrow Database Connectivity (ADBC)](https://arrow.apache.org/docs/format/ADBC.html) standard. It provides zero-copy Arrow-native data access, eliminating unnecessary data conversion overhead for analytical workloads.

## Features

- **Full ADBC Core API** - Complete implementation of Driver, Database, Connection, and Statement interfaces
- **Zero-Copy Data Access** - Direct Arrow RecordBatch streaming using columnar block-based access
- **TDengine 3.3.0+ Support** - Compatible with latest TDengine features including JSON, GEOMETRY, DECIMAL types
- **Connection Pooling** - Built-in connection pooling via `deadpool` for high-concurrency scenarios
- **Prepared Statements** - Parameter binding support for secure and efficient query execution
- **Supertable Metadata** - TDengine-specific introspection for supertables and tags
- **Comprehensive Type Coverage** - Full support for all TDengine data types including unsigned integers
- **Async Runtime Bridging** - Seamless integration between async TDengine client and synchronous ADBC API

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Connection String Format](#connection-string-format)
- [Usage Examples](#usage-examples)
  - [Basic Query](#basic-query)
  - [Data Insertion](#data-insertion)
  - [Prepared Statements](#prepared-statements)
  - [Connection Pooling](#connection-pooling)
  - [Metadata Query](#metadata-query)
- [Type Mappings](#type-mappings)
- [Performance](#performance)
- [Testing](#testing)
- [Documentation](#documentation)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [License](#license)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
adbc-taos = "0.1"
adbc-core = "0.21"
arrow-array = "57"
```

### Requirements

- **Rust**: 1.75 or later
- **TDengine**: 3.3.0.0 or later ([Installation Guide](https://docs.tdengine.com/3.3/taos-sql/))

## Quick Start

```rust
use adbc_core::{Database, Connection, Statement, Driver, Optionable};
use adbc_core::options::{OptionDatabase, OptionValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create driver and configure database
    let driver = adbc_taos::TaosDriver::default();
    let mut db = driver.new_database()?;
    db.set_option(
        OptionDatabase::Uri,
        OptionValue::String("taos://root:taosdata@127.0.0.1:6030".into())
    )?;

    // 2. Establish connection
    let mut conn = db.new_connection()?;

    // 3. Execute query
    let mut stmt = conn.new_statement()?;
    stmt.set_sql_query("SELECT * FROM my_db.my_table LIMIT 10")?;
    let reader = stmt.execute()?;

    // 4. Process results as Arrow RecordBatches
    for batch in reader {
        let batch = batch?;
        println!("Received batch with {} rows", batch.num_rows());

        // Access columns by index
        for col_idx in 0..batch.num_columns() {
            let col = batch.column(col_idx);
            println!("  Column {}: {} rows, type: {:?}",
                     col_idx,
                     col.len(),
                     col.data_type());
        }

        // Example: Access first column as string if it's NCHAR/VARCHAR
        use arrow_array::StringArray;
        if let Some(str_col) = batch.column(0).as_any().downcast_ref::<StringArray>() {
            for row_idx in 0..str_col.len() {
                if let Some(value) = str_col.get(row_idx) {
                    println!("  Row[{}]: {}", row_idx, value);
                }
            }
        }
    }

    Ok(())
}
```

## Connection String Format

The TDengine DSN (Data Source Name) format:

```
taos://[user:password@]host[:port][/database][?params]
```

### Examples

```rust
// Default credentials (root/taosdata)
"taos://127.0.0.1:6030"

// With credentials
"taos://admin:pass@localhost:6030"

// With database
"taos://root:taosdata@localhost:6030/mydb"

// All options
"taos://admin:secret@192.168.1.100:6030/production?timezone=UTC"
```

### Configuration Options

You can also set options programmatically:

```rust
let mut db = adbc_taos::TaosDatabase::default();

// Set individual options
db.set_option(OptionDatabase::Uri, OptionValue::String("taos://127.0.0.1:6030".into()))?;
db.set_option(OptionDatabase::User, OptionValue::String("admin".into()))?;
db.set_option(OptionDatabase::Password, OptionValue::String("pass".into()))?;
```

## Usage Examples

### Basic Query

```rust
use adbc_core::{Connection, Statement};
use arrow_array::{Float64Array, TimestampMillisecondArray, StringArray};

let mut conn = db.new_connection()?;
let mut stmt = conn.new_statement()?;
stmt.set_sql_query("SELECT ts, temperature, location FROM sensors WHERE location = 'room-1' LIMIT 100")?;
let reader = stmt.execute()?;

for batch in reader {
    let batch = batch?;

    // Access column 0: timestamp
    let ts_col = batch.column(0)
        .as_any()
        .downcast_ref::<TimestampMillisecondArray>()
        .expect("Column 0 should be timestamp");

    // Access column 1: temperature (double)
    let temp_col = batch.column(1)
        .as_any()
        .downcast_ref::<Float64Array>()
        .expect("Column 1 should be float");

    // Access column 2: location (string)
    let loc_col = batch.column(2)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("Column 2 should be string");

    // Iterate through rows
    for i in 0..batch.num_rows() {
        if let Some(ts) = ts_col.get(i) {
            if let Some(temp) = temp_col.get(i) {
                if let Some(location) = loc_col.get(i) {
                    println!("{} - {} - {}°C",
                        ts,
                        location,
                        temp);
                }
            }
        }
    }
}
```

### Data Insertion

```rust
let mut stmt = conn.new_statement()?;

// Single row insertion
stmt.set_sql_query("INSERT INTO my_db.data VALUES (NOW, 25.5, 'active')")?;
let affected_rows = stmt.execute_update()?;
println!("Inserted {} rows", affected_rows);

// Batch insertion
let insert_sql = "
    INSERT INTO my_db.data VALUES
    (NOW, 25.5, 'active'),
    (NOW + 1s, 26.1, 'active'),
    (NOW + 2s, 24.8, 'idle')
";
stmt.set_sql_query(insert_sql)?;
let affected_rows = stmt.execute_update()?;
```

### Prepared Statements

```rust
let mut stmt = conn.new_statement()?;

// Prepare statement with parameters
stmt.set_sql_query("SELECT * FROM sensors WHERE temperature > ? AND location = ?")?;
stmt.prepare()?;

// Note: Parameter binding uses Arrow RecordBatches
// See examples/prepared.rs for complete implementation
```

### Connection Pooling

For high-concurrency scenarios, enable connection pooling:

```rust
use adbc_taos::TaosDatabase;

let mut db = TaosDatabase::default();
db.uri = "taos://root:taosdata@127.0.0.1:6030".into();

// Enable pooling with max 10 connections
db.set_pool_size(10);

// All subsequent connections use the pool
for i in 0..5 {
    let conn = db.new_connection()?;
    // Use connection in parallel tasks...
}

// Pool is disabled by default (pool_size = 0)
```

**Benefits:**
- Reduced connection overhead
- Automatic connection recycling
- Health checks and reconnection
- Better resource utilization

### Metadata Query

```rust
use adbc_core::Connection;

// Get table schema
let schema = conn.get_table_schema(Some("my_db"), None, "my_table")?;
println!("Table schema:");
for field in schema.fields() {
    println!("  - {}: {:?}", field.name(), field.data_type());
}

// Check if table is a supertable
let is_super = conn.is_supertable(Some("my_db"), "my_supertable")?;
println!("Is supertable: {}", is_super);

// Get supertable tags
if is_super {
    let tags = conn.get_table_tags(Some("my_db"), "my_supertable")?;
    for batch in tags {
        let batch = batch?;
        let tag_name_col = batch.column(0)
            .as_any()
            .downcast_ref::<arrow_array::StringArray>();

        let tag_type_col = batch.column(1)
            .as_any()
            .downcast_ref::<arrow_array::StringArray>();

        if let (Some(names), Some(types)) = (tag_name_col, tag_type_col) {
            for i in 0..names.len() {
                println!("  Tag: {} ({})",
                    names.get(i).unwrap_or(""),
                    types.get(i).unwrap_or(""));
            }
        }
    }
}

// Get server version
println!("TDengine version: {}", conn.server_version());
```

## Type Mappings

### TDengine to Arrow

| TDengine Type | Arrow Type | Notes |
|--------------|------------|-------|
| `BOOL` | `Boolean` | |
| `TINYINT` | `Int8` | Signed 8-bit |
| `SMALLINT` | `Int16` | Signed 16-bit |
| `INT` | `Int32` | Signed 32-bit |
| `BIGINT` | `Int64` | Signed 64-bit |
| `TINYINT UNSIGNED` | `UInt8` | Unsigned 8-bit |
| `SMALLINT UNSIGNED` | `UInt16` | Unsigned 16-bit |
| `INT UNSIGNED` | `UInt32` | Unsigned 32-bit |
| `BIGINT UNSIGNED` | `UInt64` | Unsigned 64-bit |
| `FLOAT` | `Float32` | IEEE 754 single precision |
| `DOUBLE` | `Float64` | IEEE 754 double precision |
| `VARCHAR` | `Binary` | Variable-length binary |
| `VARBINARY` | `Binary` | Variable-length binary |
| `NCHAR` | `Utf8` | Unicode string |
| `JSON` | `Utf8` | JSON as string |
| `TIMESTAMP` | `Timestamp(Millisecond, None)` | Millisecond precision |
| `GEOMETRY` | `Binary` | Spatial data type |
| `DECIMAL` | `Decimal128(38, 0)` | Maximum precision |
| `BLOB` | `Binary` | Large binary objects |

### Arrow to TDengine (Parameter Binding)

Supported Arrow types for prepared statement parameters:

- `BooleanArray`, `Int8Array`, `Int16Array`, `Int32Array`, `Int64Array`
- `UInt8Array`, `UInt16Array`, `UInt32Array`, `UInt64Array`
- `Float32Array`, `Float64Array`
- `StringArray`, `BinaryArray`
- `TimestampMillisecondArray`, `TimestampMicrosecondArray`, `TimestampNanosecondArray`

## Performance

ADBC-Taos is optimized for high-throughput data processing:

### Key Optimizations

1. **Columnar Block Access** - Uses TDengine's `RawBlock` API instead of row iteration
2. **Zero-Copy Conversion** - Direct Arrow format conversion without intermediate buffers
3. **Connection Pooling** - Efficient connection reuse for concurrent operations
4. **Async Runtime Bridging** - Optimal async/sync integration with Tokio

### Benchmark Results

See `benches/conversion_benchmark.rs` for performance measurements. Recent optimizations (commit `6606c74`) achieved significant improvements by replacing row-based iteration with columnar block access.

Run benchmarks:

```bash
cargo bench
```

## Testing

### Run Tests

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

Configure TDengine connection in `.env` (copy from `.env.example`):

```bash
TAOS_HOST=127.0.0.1
TAOS_PORT=6030
TAOS_USER=root
TAOS_PASSWORD=taosdata
TAOS_DATABASE=demo  # Optional
```

### Test Structure

```
tests/
├── common/
│   └── mod.rs              # Test utilities and fixtures
├── integration_tests.rs    # Main integration test suite
└── readme_examples.rs      # README example validation
```

## Documentation

### Generate Documentation

```bash
# Generate and open API documentation
cargo doc --open

# Generate documentation without dependencies
cargo doc --no-deps

# Include private items
cargo doc --document-private-items
```

### Online Documentation

- [API Reference](https://docs.rs/adbc-taos)
- [TDengine Documentation](https://docs.tdengine.com/)
- [ADBC Specification](https://arrow.apache.org/docs/format/ADBC.html)
- [Arrow Rust Documentation](https://docs.rs/arrow/)

### Examples

Explore the `examples/` directory for complete working examples:

```bash
cargo run --example basic      # Basic query
cargo run --example query      # Query execution
cargo run --example insert     # Data insertion
cargo run --example metadata   # Schema introspection
cargo run --example table_ops  # Table operations
```

## Project Structure

```
adbc-taos/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── driver.rs           # TaosDriver: Entry point
│   ├── database.rs         # TaosDatabase: Configuration and pooling
│   ├── connection.rs       # TaosConnection: Active connections
│   ├── statement.rs        # TaosStatement: SQL execution
│   ├── reader.rs           # RecordBatch readers (optimized)
│   ├── pool.rs             # Connection pooling
│   ├── error.rs            # Error types
│   ├── types.rs            # Type conversion utilities
│   └── utils.rs            # Async runtime handling
├── examples/               # Usage examples
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
├── CLAUDE.md               # Development guidelines
├── PROJECT_INDEX.md        # Detailed architecture docs
└── README.md               # This file
```

### Architecture Highlights

- **ADBC Standard Compliance** - Implements `adbc_core` traits for interoperability
- **Columnar Data Access** - Reader uses block-based columnar access for optimal performance
- **Type Safety** - Comprehensive TDengine-to-Arrow type mapping
- **Error Handling** - Detailed error types with ADBC status codes
- **Async Runtime Management** - Custom runtime wrapper for seamless async/sync bridging

## Contributing

We welcome contributions! Please see our development guidelines:

1. Read `CLAUDE.md` for development workflow
2. Read `PROJECT_INDEX.md` for architecture details
3. Follow Rust best practices (run `cargo clippy` and `cargo fmt`)
4. Add tests for new features
5. Update documentation as needed

### Development Setup

```bash
# Clone repository
git clone https://github.com/your-org/adbc-taos.git
cd adbc-taos

# Install dependencies
cargo build

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt
```

## Related Projects

- [adbc-rs](https://github.com/alexandreyc/adbc-rs) - ADBC core implementations
- [adbc-clickhouse](https://github.com/if0ne/adbc-clickhouse) - ADBC driver for ClickHouse
- [taos-client](https://github.com/taosdata/taos-rust-driver) - TDengine Rust client library

## References

- [TDengine v3.3+ Documentation](https://docs.tdengine.com/)
- [TDengine Rust Client](https://docs.tdengine.com/3.3/taos-sql/)
- [ADBC Specification](https://arrow.apache.org/docs/format/ADBC.html)
- [Arrow Rust Implementation](https://arrow.apache.org/docs/rust/)

## License

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

---

**Project Status:** Phase 2 Complete - Full ADBC implementation with optimized data access

For detailed architecture and API reference, see [PROJECT_INDEX.md](PROJECT_INDEX.md)
