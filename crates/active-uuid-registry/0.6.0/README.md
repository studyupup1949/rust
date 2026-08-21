# active-uuid-registry

A library for managing in-process, namespace- and context-aware UUIDs for liveness tracking. UUIDs are organized in a two-level global registry (`namespace -> context -> UUID set`), making it straightforward to track running components across logical scopes in dynamic systems.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
active-uuid-registry = "0.6.0"
```

By default, the registry uses a mutex-protected `HashMap` with `Arc<str>` keys and `HashSet<Uuid>` values.
For high-concurrency workloads, enable the `concurrent-map` feature to use `DashMap`/`DashSet` instead:

```toml
[dependencies]
active-uuid-registry = { version = "0.6.0", features = ["concurrent-map"] }
```

## Usage

```rust
use active_uuid_registry::interface::*;
use active_uuid_registry::UuidPoolError;

// Pre-create a namespace (optional — auto-created on first insert)
reserve_namespace("my_app");

// Reserve a new UUID in a namespace + context
let id = reserve_id("my_app", "server")?;

// Add an existing UUID
add_id("my_app", "client", some_uuid)?;

// Remove a UUID
remove_id("my_app", "client", some_uuid)?;

// Try-remove (returns bool instead of Result)
let removed: bool = try_remove_id("my_app", "client", some_uuid);

// Replace a UUID within a context
replace_id("my_app", "server", old_uuid, new_uuid)?;

// Query UUIDs
let pairs = get_pairs("my_app", "server")?;  // Vec<(String, Uuid)>
let ns_pairs = get_namespace_pairs("my_app")?;  // all contexts in namespace
let all_pairs = get_all_pairs()?;  // all namespaces

// List registered namespaces and contexts
let namespaces = list_namespaces();
let contexts = list_contexts("my_app");

// Clear (non-returning)
clear_context("my_app", "server");
clear_namespace("my_app");
clear_all_namespaces();
clear_all_contexts("my_app");

// Drain (returns removed entries and clears them)
let drained_ctx = drain_context("my_app", "server")?;  // Vec<(String, Uuid)>
let drained_ctxs = drain_all_contexts("my_app")?;  // Vec<(String, String, Uuid)>
let drained_ns = drain_namespace("my_app")?;  // Vec<(String, String, Uuid)>
let drained_all = drain_all_namespaces()?;  // Vec<(String, String, Uuid)>
```

## API

### Namespace Management

| Function | Description |
|---|---|
| `reserve_namespace(ns)` | Pre-create a namespace entry |
| `remove_namespace(ns)` | Remove a namespace and all its data |
| `replace_namespace(old, new)` | Rename a namespace |

### UUID Operations

| Function | Description |
|---|---|
| `reserve_id(ns, ctx)` | Generate and register a new UUID |
| `reserve_id_with_base(ns, ctx, base)` | Reserve with a custom base parameter |
| `reserve_id_with(ns, ctx, base, retries)` | Reserve with custom base and retry limit |
| `add_id(ns, ctx, uuid)` | Register an existing UUID |
| `remove_id(ns, ctx, uuid)` | Remove a UUID (returns `Result`) |
| `try_remove_id(ns, ctx, uuid)` | Remove a UUID (returns `bool`) |
| `replace_id(ns, ctx, old, new)` | Replace one UUID with another in a context |

### Query / Inspect

| Function | Description |
|---|---|
| `get_pairs(ns, ctx)` | All UUIDs for a specific context |
| `get_namespace_pairs(ns)` | All UUIDs across all contexts in a namespace |
| `get_all_pairs()` | All UUIDs across all namespaces |
| `list_namespaces()` | All registered namespace names |
| `list_contexts(ns)` | All context names within a namespace |

### Clear / Drain

| Function | Description |
|---|---|
| `clear_context(ns, ctx)` | Drop all UUIDs from a context |
| `clear_namespace(ns)` | Remove a namespace and all its contexts from the registry |
| `clear_all_namespaces()` | Drop everything from the registry |
| `clear_all_contexts(ns)` | Drop all contexts within a namespace, retaining the namespace entry |
| `drain_context(ns, ctx)` | Remove and return all UUIDs from a context — `Vec<(String, Uuid)>` |
| `drain_all_contexts(ns)` | Remove and return all contexts in a namespace — `Vec<(String, String, Uuid)>` |
| `drain_namespace(ns)` | Remove and return an entire namespace — `Vec<(String, String, Uuid)>` |
| `drain_all_namespaces()` | Remove and return all namespaces — `Vec<(String, String, Uuid)>` |

## License

[MIT](LICENSE)
