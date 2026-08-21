# admixture

A Rust framework for managing service lifecycles in integration tests. Admixture lets you declaratively define test contexts composed of services (databases, caches, HTTP servers, etc.) with automatic setup and teardown.

## Crates

| Crate | Description |
|-------|-------------|
| [`admixture`](https://crates.io/crates/admixture) | Core traits and macros for defining services and contexts |
| [`admixture-macros`](https://crates.io/crates/admixture-macros) | Proc macros for declaratively defining test contexts |
| [`admixture-docker`](https://crates.io/crates/admixture-docker) | Docker container services via testcontainers |
| [`admixture-docker-macros`](https://crates.io/crates/admixture-docker-macros) | Proc macro for defining Docker container services |
| [`admixture-harness`](https://crates.io/crates/admixture-harness) | Test harness with context reuse and grouped execution |
| [`admixture-harness-macros`](https://crates.io/crates/admixture-harness-macros) | Attribute macro for registering tests with contexts |

## Quick Start

Add `admixture` to your dev-dependencies:

```toml
[dev-dependencies]
admixture = "0.1"
```

Define a test context with services:

```rust
use admixture::context;

context! {
    MyTestContext {
        database: MockDatabaseSetup,
        cache: MockCacheSetup,
    }
}
```

### With Docker containers

Use `admixture-docker` for real database/cache containers in tests:

```toml
[dev-dependencies]
admixture = "0.1"
admixture-docker = { version = "0.1", features = ["sqlx-postgres"] }
```

```rust
use admixture::context;
use admixture_docker::SqlxPostgresServiceSetup;
use testcontainers_modules::postgres::Postgres;

context! {
    PostgresTestContext {
        postgres: SqlxPostgresServiceSetup,
    }
}
```

### With the test harness

Use `admixture-harness` for automatic context lifecycle management and context reuse across tests:

```toml
[dev-dependencies]
admixture = "0.1"
admixture-harness = "0.1"
```

```rust
use admixture::context;
use admixture_harness::prelude::*;

#[admixture_test(context = MyTestContext)]
async fn test_something(ctx: &MyTestContext) -> Result<(), TestError> {
    // Context is automatically started and shared across tests
    Ok(())
}

admixture_harness::test_runner!();
```

The harness starts each context type **once** and reuses it across all tests that share that context, significantly reducing test execution time.

## License

Licensed under the [MIT License](LICENSE).
