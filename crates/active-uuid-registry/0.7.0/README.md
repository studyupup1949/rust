# active-uuid-registry

A library for managing in-process, namespace- and context-aware UUIDs for liveness tracking. UUIDs are organized in a two-level global registry (`namespace -> context -> UUID set`), making it straightforward to track running components across logical scopes in dynamic systems.

The underlying registry data structure is a mutable global state using a singleton pattern, thus it is necessitated that the developer implements proper management and implementation practices for their particular process/use-case.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
active-uuid-registry = "0.7.0"
```

By default, the registry uses a mutex-protected `HashMap` with `Arc<str>` keys and `HashSet<Uuid>` values.
For high-concurrency workloads, enable the `concurrent-map` feature to use `DashMap`/`DashSet` instead:

```toml
[dependencies]
active-uuid-registry = { version = "0.7.0", features = ["concurrent-map"] }
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
let ctx_entries = get_context_entries("my_app", "server")?;  // Vec<(NamespaceString, ContextString, Uuid)>
let ns_entries = get_namespace_entries("my_app")?;            // all contexts in namespace
let all_entries = get_all_namespace_entries()?;               // all namespaces
let ids = list_ids("my_app", "server");                       // Vec<Uuid>

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
| `remove_namespace(ns)` | Remove a namespace and all of its data |
| `replace_namespace(old, new)` | Rename a namespace |

### Context-UUID Operations

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
| `get_context_entries(ns, ctx)` | All UUIDs for a specific context in a namespace |
| `get_namespace_entries(ns)` | All UUIDs across all contexts in a namespace |
| `get_all_namespace_entries()` | All UUIDs across all contexts across all namespaces |
| `list_namespaces()` | All registered namespace names |
| `list_contexts(ns)` | All context names within a namespace |
| `list_ids(ns, ctx)` | All UUIDs within a context in a namespace |

### Clear / Drain

| Function | Description |
|---|---|
| `clear_context(ns, ctx)` | Drop all UUIDs from a context |
| `clear_namespace(ns)` | Remove a namespace and all its contexts from the registry |
| `clear_all_namespaces()` | Drop everything from the registry |
| `clear_all_contexts(ns)` | Drop all contexts within a namespace, retaining the namespace entry |
| `drain_context(ns, ctx)` | Remove and return all UUIDs from a context |
| `drain_all_contexts(ns)` | Remove and return all contexts in a namespace |
| `drain_namespace(ns)` | Remove and return an entire namespace |
| `drain_all_namespaces()` | Remove and return all namespaces |

## License

[MIT](LICENSE)
