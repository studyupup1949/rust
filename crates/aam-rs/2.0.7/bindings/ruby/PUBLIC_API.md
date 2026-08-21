# Public API (Ruby)

Source of truth: `ruby/ext/aam_rs/src/lib.rs`.

## Main Namespace

- `AamRb::AAM`
- `AamRb::AAMBuilder`
- `AamRb::SchemaField`

## Class Methods

- `AamRb::AAM.new`
- `AamRb::AAM.parse(content)`
- `AamRb::AAM.load(path)`
- `AamRb::AAMBuilder.new`
- `AamRb::AAMBuilder.with_capacity(capacity)`
- `AamRb::SchemaField.required(name, type_name)`
- `AamRb::SchemaField.optional(name, type_name)`

## Instance Methods

- `get(key)`
- `keys`
- `to_map`
- `find(query)`
- `deep_search(pattern)`
- `reverse_search(value)`
- `schema_names`
- `type_names`

## AAMBuilder Instance Methods

- `add_line(key, value)`
- `comment(text)`
- `schema(name, fields)`
- `schema_multiline(name, fields)`
- `derive(path, schemas)`
- `import(path)`
- `type_alias(alias, type_name)`
- `as_string`

## Error Model

- Parse and load failures raise runtime exceptions from native layer.

## Compatibility Note

- Existing README examples use a lightweight helper style.
- Native extension API is centered on `AamRb::AAM`.
