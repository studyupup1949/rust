# AGENTS.md — aam-rs

## Project overview

`aam-rs` is a Rust library that parses `.aam` (AAM) configuration files: a line-based `key = value` format with
directives (`@import`, `@derive`, `@schema`, `@type`), schema-based type validation, bidirectional lookup, and deep
reference resolution. The crate now exposes both the legacy `AAML` API and a newer pipeline-backed `AAM` API with
optional AOT (`.aam.bin`) loading.

## Architecture

```
src/
  lib.rs               — public re-exports
  aaml/                — legacy/core AAML struct (split across several impl files)
    mod.rs             — struct definition, AAML::new/parse/load, register_default_commands
    lookup.rs          — find_obj / find_key / find_deep  (impl AAML)
    validation.rs      — validate_against_schemas / validate_typed_field (impl AAML)
    parsing.rs         — strip_comment, parse_assignment, unwrap_quotes (free functions)
    serialize.rs       — serde impl (feature-gated)
  aam.rs               — newer high-level AAM API backed by pipeline output
  aam_value.rs         — value wrapper/helpers for the AAM API
  pipeline/            — five-stage pipeline (lexer, parser, validator, executer, formatter)
  aot/                 — cook/load `.aam.bin` cache for fast startup (feature-gated)
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
  types_aam/           — pipeline (`AAM`) type validation system (TypeAAM + resolve_builtin)
  found_value.rs       — FoundValue wrapper (as_str, as_list, as_object, Deref<str>)
  builder.rs           — AAMBuilder fluent API for programmatic .aam generation
  error.rs             — AamlError enum (typed errors)
  ffi.rs/python.rs/jni.rs — optional language binding entry points (feature-gated)
```

**AAML methods are split across impl blocks in separate files** — `mod.rs`, `lookup.rs`, and `validation.rs` all `impl AAML`. Check all three when tracing a method.

## Developer workflows

```sh
cargo test                          # run all unit + integration tests
cargo run --example standard        # run an individual example
cargo run --example advanced        # schema/derive/list demo
cargo run --example standard_stress # stress demo / larger input path
cargo test --features serde         # test with serde feature
cargo test --features aot           # explicitly include AOT cache/load tests
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
3. Mirror the same type in `src/types_aam/` + `src/types_aam/mod.rs` so `AAM` pipeline validation stays aligned with `AAML` validation.

### Schema optional fields
Fields suffixed with `*` in `@schema` (`field*: type`) are optional — absence is not an error, but presence is still type-validated. Represented by `SchemaDef::optional_fields: HashSet<String>`.

### Comment parsing quirk
`#` is a comment delimiter **only when surrounded by whitespace**. Hex colors (`tint = #ff6600`) are valid values — see `parsing::strip_comment`.

### `FoundValue` derefs to `&str`
All lookup methods return `Option<FoundValue>`. It implements `Deref<Target = str>`, so `&*val` or `val.as_str()` gives a `&str`. Use `.as_list()` for `[a, b, c]` values and `.as_object()` for `{ k = v }` inline objects.

### Feature flags
| Flag                                                        | Effect                                                                             |
|-------------------------------------------------------------|------------------------------------------------------------------------------------|
| `hash-std`                                                  | Default pipeline hasher (`std::collections::hash_map::RandomState`)                |
| `hash-fx` / `hash-ahash` / `hash-rapidhash` / `hash-ripemd` | Alternate pipeline hashers (mutually exclusive)                                    |
| `perf-hash`                                                 | Swaps `HashMap` hasher to `ahash::RandomState` for better throughput               |
| `aot`                                                       | Enables `src/aot/` cooked binary cache and `AAM::load` fast-path                   |
| `ffi` / `python` / `jni` / `csharp`                         | Enables language binding surfaces                                                  |
| `parallel`                                                  | Enables parallel parse-task pre-validation in pipeline                             |
| `serde`                                                     | Derives `Serialize`/`Deserialize` on `FoundValue`, `SchemaDef`, `AAMBuilder`, etc. |

## Integration tests layout

| File                                   | Covers                                                                         |
|----------------------------------------|--------------------------------------------------------------------------------|
| `tests/test_core.rs`                   | `find_obj`, `find_deep`, loop detection, merge (`+=`)                          |
| `tests/test_derive.rs`                 | `@derive` inheritance, schema completeness checks                              |
| `tests/test_imports.rs`                | `@import` file loading                                                         |
| `tests/test_parsing.rs`                | `parse_assignment`, `strip_comment`, quote handling                            |
| `tests/test_serde.rs`                  | serde round-trips (requires `--features serde`)                                |
| `tests/test_aot.rs`                    | AOT cook/load cache behavior (`AamCompiler`, `AamLoader`, `AAM::load`)         |
| `tests/test_comprehensive_errors.rs`   | large error/diagnostic matrix for parser, directives, and type/schema failures |
| `tests/test_comprehensive_coverage.rs` | broad behavior coverage across parsing, lookups, directives, builder paths     |
| `tests/test_massive_coverage.rs`       | high-volume matrix tests for parser/comment/deep-lookup stability              |
| `tests/type_validation_tests.rs`       | built-in type validators                                                       |

## Public API docs (multi-language)

Keep these files updated when behavior or signatures change:

| Language / Surface | Public API doc path     | Primary source of truth                       |
|--------------------|-------------------------|-----------------------------------------------|
| Rust core          | `PUBLIC_API.md`         | `src/lib.rs`, `src/aaml/`, `src/aam.rs`       |
| C FFI              | `include/PUBLIC_API.md` | `include/aam.h`, `src/ffi.rs`                 |
| C#                 | `csharp/PUBLIC_API.md`  | `csharp/src/AamDocument.cs`                   |
| Go                 | `go/PUBLIC_API.md`      | `go/aam/aam.go`                               |
| Java/Kotlin        | `java/PUBLIC_API.md`    | `java/src/main/kotlin/AamDocument.kt`         |
| Node.js            | `js/PUBLIC_API.md`      | `js/index.d.ts`                               |
| PHP                | `php/PUBLIC_API.md`     | `php/src/AamPhp.php`                          |
| Python             | `python/PUBLIC_API.md`  | `src/python.rs`                               |
| Ruby               | `ruby/PUBLIC_API.md`    | `ruby/ext/aam_rs/src/lib.rs`                  |
| WASM               | `wasm/PUBLIC_API.md`    | `wasm/src/lib.rs`                             |

## API-change workflow

When making a change that can affect users:

1. Update implementation in source code.
2. Update matching `PUBLIC_API.md` file(s).
3. Run language-specific tests or smoke checks for affected bindings.
4. If behavior is intentionally changed, add note to `CHANGELOG.md`.

If you are not sure whether a change is public-facing, treat it as public and update docs.

