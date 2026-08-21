# AGENTS.md — aam-rs

## Project overview

`aam-rs` is a Rust library that parses `.aam` (AAML) configuration files: a line-based `key = value` format with directives (`@import`, `@derive`, `@schema`, `@type`), schema-based type validation, bidirectional lookup, and deep reference resolution.

## Architecture

```
src/
  lib.rs               — public re-exports
  aaml/                — core AAML struct (split across several impl files)
    mod.rs             — struct definition, AAML::new/parse/load, register_default_commands
    lookup.rs          — find_obj / find_key / find_deep  (impl AAML)
    validation.rs      — validate_against_schemas / validate_typed_field (impl AAML)
    parsing.rs         — strip_comment, parse_assignment, unwrap_quotes (free functions)
    types_registry.rs  — register_type / validate_value (impl AAML)
    serialize.rs       — serde impl (feature-gated)
  commands/            — directive system; one file per directive
    mod.rs             — Command trait: name() + execute(&mut AAML, args) -> Result
    import.rs          — @import
    derive.rs          — @derive (inheritance; child-wins semantics)
    schema.rs          — @schema, SchemaDef struct
    typecm.rs          — @type, TypeDefinition enum
  types/               — validation type system
    mod.rs             — Type trait + resolve_builtin(path) dispatcher
    primitive_type.rs  — i32, f64, string, bool, color
    math.rs            — math::vector2/3/4, math::matrix4x4
    physics.rs         — physics::kilogram
    time.rs            — time::datetime
    list.rs            — list<T> (homogeneous lists)
  found_value.rs       — FoundValue wrapper (as_str, as_list, as_object, Deref<str>)
  builder.rs           — AAMBuilder fluent API for programmatic .aam generation
  error.rs             — AamlError enum (typed errors)
```

**AAML methods are split across impl blocks in separate files** — `mod.rs`, `lookup.rs`, `validation.rs`, and `types_registry.rs` all `impl AAML`. Check all four when tracing a method.

## Developer workflows

```sh
cargo test                          # run all unit + integration tests
cargo run --example standard        # run an individual example
cargo run --example advanced        # schema/derive/list demo
cargo test --features serde         # test with serde feature
cargo test --features perf-hash     # test with ahash hasher
```

Examples that need `.aam` files call `std::env::set_current_dir` to `examples/` — keep paired `.aam` files there (e.g. `advanced_base.aam` / `advanced_child.aam`).

## Key conventions

### Adding a new directive
1. Create `src/commands/my_cmd.rs`, implement `Command` (`name()` + `execute()`).
2. Register it in `AAML::register_default_commands` inside `src/aaml/mod.rs`.

### Adding a new built-in type
1. Implement `Type` (`from_name`, `base_type`, `validate`) in `src/types/`.
2. Add a match arm to `resolve_builtin` in `src/types/mod.rs`.

### Schema optional fields
Fields suffixed with `*` in `@schema` (`field*: type`) are optional — absence is not an error, but presence is still type-validated. Represented by `SchemaDef::optional_fields: HashSet<String>`.

### Comment parsing quirk
`#` is a comment delimiter **only when surrounded by whitespace**. Hex colors (`tint = #ff6600`) are valid values — see `parsing::strip_comment`.

### `FoundValue` derefs to `&str`
All lookup methods return `Option<FoundValue>`. It implements `Deref<Target = str>`, so `&*val` or `val.as_str()` gives a `&str`. Use `.as_list()` for `[a, b, c]` values and `.as_object()` for `{ k = v }` inline objects.

### Feature flags
| Flag | Effect |
|---|---|
| `perf-hash` | Swaps `HashMap` hasher to `ahash::RandomState` for better throughput |
| `serde` | Derives `Serialize`/`Deserialize` on `FoundValue`, `SchemaDef`, `AAMBuilder`, etc. |

## Integration tests layout

| File | Covers |
|---|---|
| `tests/test_core.rs` | `find_obj`, `find_deep`, loop detection, merge (`+=`) |
| `tests/test_derive.rs` | `@derive` inheritance, schema completeness checks |
| `tests/test_imports.rs` | `@import` file loading |
| `tests/test_parsing.rs` | `parse_assignment`, `strip_comment`, quote handling |
| `tests/test_serde.rs` | serde round-trips (requires `--features serde`) |
| `tests/type_validation_tests.rs` | built-in type validators |

