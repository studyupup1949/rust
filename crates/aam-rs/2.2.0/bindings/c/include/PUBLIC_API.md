# Public API (C FFI)

Source of truth: `include/aam.h`.

## Handle Lifecycle

- `AamHandle *aam_new(void)`
- `void aam_free(AamHandle *handle)`

## Parse, Load, and Format

- `int aam_parse(AamHandle *handle, const char *content)`
- `int aam_load(AamHandle *handle, const char *path)`
- `char *aam_format(AamHandle *handle, const char *content)`

Return convention: `0` on success, non-zero on error.

## Lookup

- `char *aam_get(const AamHandle *handle, const char *key)`
- `char *aam_find(const AamHandle *handle, const char *query)`
- `char *aam_deep_search(const AamHandle *handle, const char *pattern)`
- `char *aam_reverse_search(const AamHandle *handle, const char *value)`
- `char *aam_schema_names(const AamHandle *handle)`
- `char *aam_type_names(const AamHandle *handle)`

Returned strings are owned by Rust and must be released via `aam_string_free`.

## Memory and Errors

- `void aam_string_free(char *s)`
- `const char *aam_last_error(const AamHandle *handle)`

`aam_last_error` returns an internal borrowed pointer. Do not free it.
