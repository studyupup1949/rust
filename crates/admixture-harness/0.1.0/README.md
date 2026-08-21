# admixture-harness

Test harness for integration tests with admixture contexts.

## Overview

`admixture-harness` provides the `#[admixture_test]` macro for writing integration tests with automatic context lifecycle management. It uses the `inventory` crate to collect tests at compile-time and run them sequentially with proper setup and teardown.

## Features

- **`#[admixture_test]` macro**: Automatically manages context lifecycle for each test
- **Context reuse**: Tests sharing the same context type reuse the same context instance (major performance improvement!)
- **Inventory-based test collection**: Tests are collected at compile-time and grouped by context type
- **Sequential execution within context groups**: Tests run sequentially per context type
- **Flexible return types**: Supports both `Result<(), E>` and `()` return types
- **Works with Docker**: Integrates seamlessly with `admixture-docker` for container-based tests

## Usage

### Basic Example

```rust
use admixture::context;
use admixture_harness::prelude::*;

// Define your test context with inline configuration
context! {
    MyTestContext {
        database: MockDatabaseSetup = MockDatabaseSetup::new(),
        cache: MockCacheSetup = MockCacheSetup::new(),
    }
}

// Write tests with the harness
#[admixture_test(context = MyTestContext)]
async fn test_with_result(ctx: &MyTestContext) -> Result<(), TestError> {
    let db = ctx.database().client().await?;
    // Test logic here
    Ok(())
}

#[admixture_test(context = MyTestContext)]
async fn test_with_panics(ctx: &MyTestContext) {
    let cache = ctx.cache().client().await.unwrap();
    assert_eq!(cache.get("key").await, Some("value"));
}

// Generate the test runner
admixture_harness::test_runner!();
```

### Docker/PostgreSQL Example

```rust
use admixture::context;
use admixture_docker::SqlxPostgresServiceSetup;
use admixture_harness::prelude::*;
use testcontainers_modules::postgres::Postgres;

context! {
    PostgresTestContext {
        postgres: SqlxPostgresServiceSetup = Postgres::default(),
    }
}

#[admixture_test(context = PostgresTestContext)]
async fn test_database_query(ctx: &PostgresTestContext) -> Result<(), TestError> {
    let client = ctx.postgres().client().await
        .map_err(|e| TestError::Other(e.to_string()))?;
    
    let result: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&client)
        .await
        .map_err(|e| TestError::Other(e.to_string()))?;
    
    assert_eq!(result, 1);
    Ok(())
}

admixture_harness::test_runner!();
```

## How It Works

1. **`#[admixture_test]` macro**:
   - Registers each test with the `inventory` crate
   - Includes the context type information for grouping
   - Generates a wrapper that can receive a context reference
   - Calls the test function with the context

2. **`test_runner!()` macro**:
   - Generates a single `#[tokio::test]` function
   - Collects all registered tests and groups them by context type
   - For each context type:
     - Starts the context once
     - Runs all tests for that context sequentially
     - Stops the context
   - Reports results for all tests

3. **Test execution with context reuse**:
   - **Context is started once per context type** (not per test!)
   - All tests sharing the same context type reuse the same context instance
   - This dramatically improves performance, especially with Docker containers
   - Tests still run sequentially within each context group
   - Context is always torn down after all tests in the group complete

### Example: Context Reuse in Action

If you have 3 tests all using `PostgresTestContext`:
```rust
#[admixture_test(context = PostgresTestContext)]
async fn test_one(ctx: &PostgresTestContext) { /* ... */ }

#[admixture_test(context = PostgresTestContext)]
async fn test_two(ctx: &PostgresTestContext) { /* ... */ }

#[admixture_test(context = PostgresTestContext)]
async fn test_three(ctx: &PostgresTestContext) { /* ... */ }
```

The harness will:
1. Start PostgresTestContext once
2. Run test_one with the context
3. Run test_two with the **same** context
4. Run test_three with the **same** context
5. Stop the context once

This means the PostgreSQL Docker container is started **only once** for all three tests!

### Visual Output Example

When you run tests with `--nocapture`, you'll see clear visual output showing context reuse:

```
================================================================================
🚀 Starting Integration Test Run
   Total tests: 7
   Context types: 3
================================================================================

📦 Context: CacheContext
   Tests in group: 2
   ⏳ Starting context...
      🔧 Starting Cache service
   ✅ Context started successfully

   [1/2] Running: test_cache_get_1 ... ✅ PASSED
   [2/2] Running: test_cache_get_2 ... ✅ PASSED

   ⏳ Stopping context... ✅ Stopped

📦 Context: DatabaseContext
   Tests in group: 2
   ⏳ Starting context...
      🔧 Starting Database service
   ✅ Context started successfully

   [1/2] Running: test_database_query_1 ... ✅ PASSED
   [2/2] Running: test_database_query_2 ... ✅ PASSED

   ⏳ Stopping context... ✅ Stopped

📦 Context: FullContext
   Tests in group: 3
   ⏳ Starting context...
      🔧 Starting Database service
      🔧 Starting Cache service
   ✅ Context started successfully

   [1/3] Running: test_full_integration_1 ... ✅ PASSED
   [2/3] Running: test_full_integration_2 ... ✅ PASSED
   [3/3] Running: test_full_integration_3 ... ✅ PASSED

   ⏳ Stopping context... ✅ Stopped

================================================================================
📊 Test Results Summary
================================================================================
   ✅ All tests passed!

   Total:  7
   ✅ Passed: 7
   ❌ Failed: 0
================================================================================
```

Notice how:
- Each context type is started **once**
- All tests for that context run sequentially
- The context is stopped **once** after all tests complete
- You can clearly see the progress: `[1/3]`, `[2/3]`, `[3/3]`

## Requirements

- Test functions must be `async`
- Test functions must have exactly one parameter: `ctx: &<ContextType>`
- The context must provide service configurations either:
  - Using inline config syntax: `service: ServiceType = config_expr`
  - Or service configs must implement `Default`
- Tests must return either `()` or `Result<(), E>`

## Future Enhancements

- **Test isolation options**: Allow opting into fresh context per test when needed
- **Lifecycle hooks**: `before_each`, `after_each`, `before_all`, `after_all`
- **Parallel context groups**: Run different context groups in parallel
- **Test filtering**: Run specific tests by name or pattern
- **Custom context setup**: Allow non-Default context initialization
- **Test dependencies**: Specify test execution order within a context group

## Running Tests

```bash
# Run all harness tests
cargo test --package admixture-harness

# Run specific test file
cargo test --test harness_basic_tests

# Run with output
cargo test --test harness_basic_tests -- --nocapture
```

## License

Same as parent `admixture` project.
