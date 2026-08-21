# active-uuid-registry

A functional interface for managing sets of UUIDs organized by named contexts in a global registry.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
active-uuid-registry = "0.1.0"
```

For concurrent access with DashMap, enable the `concurrent` feature:

```toml
[dependencies]
active-uuid-registry = { version = "0.1.0", features = ["concurrent"] }
```

## Usage

```rust
use active_uuid_registry::interface::{reserve, add, remove, try_remove, replace, clear};
use active_uuid_registry::UuidPoolError;
use uuid::Uuid;

// Reserve a new UUID in a named context
let reserve_res: Result<Uuid, UuidPoolError> = reserve("server");

// Add an existing UUID to a context
let custom_uuid = Uuid::...; // create your UUID here
let add_res: Result<(), UuidPoolError> = add("client", custom_uuid);

// Remove a UUID from a context
let remove_res: Result<(), UuidPoolError> = remove("client", uuid);

// Try to remove the UUID from the `client` context
let removed: bool = try_remove("client", custom_uuid);

// Replace one UUID with another within the same context
let old = reserve("server").unwrap();
let new = Uuid::...; // create your UUID here
let replace_res: Result<(), UuidPoolError> = replace("server", old, new);

// Clear all UUIDs from a certain context
let clear_res: Result<(), UuidPoolError> = clear_context("server");

// Clear all UUIDs from all contexts
let clear_res: Result<(), UuidPoolError> = clear_all();
```

## API

| Function | Description |
|----------|-------------|
| `reserve(context)` | Generate and register a new UUID in the given context |
| `reserve_with_base(context, base)` | Reserve with custom base parameter |
| `reserve_with(context, base, max_retries)` | Reserve with custom base and retry count |
| `add(context, uuid)` | Register an existing UUID in a context |
| `remove(context, uuid)` | Remove a UUID from a context (returns `Result`) |
| `try_remove(context, uuid)` | Remove a UUID from a context (returns `bool`) |
| `replace(context, old, new)` | Replace one UUID with another in a context |
| `clear_context(context)` | Remove all UUIDS from a specific context |
| `clear_all()` | Remove all UUIDs from all contexts |

## License

MIT
