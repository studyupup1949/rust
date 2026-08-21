0.2.0 - 25 May 2025

## Breaking Changes
* **API Change**: `RequestIDMiddleware::new()` no longer takes parameters
  - Old: `RequestIDMiddleware::new(36)`
  - New: `RequestIDMiddleware::new()` (uses default 36-character UUID)

## New Features
* **Added `with_id_length(length: usize)` method** for customizing request ID length
  - Allows generating IDs from 1 to 36 characters
  - Panics if length is 0 for safety
  - Example: `RequestIDMiddleware::new().with_id_length(16)`

## Improvements
* **Simplified API**: Default constructor now uses standard UUID length (36 characters)
* **Better method chaining**: All configuration methods can be chained fluently
* **Enhanced documentation**: Added comprehensive usage guide and examples
* **Updated examples**: All examples now use the new API

## Migration Guide
```rust
// Before (0.1.x)
RequestIDMiddleware::new(36)
RequestIDMiddleware::new(16)

// After (0.2.x)
RequestIDMiddleware::new()                    // Default 36 characters
RequestIDMiddleware::new().with_id_length(16) // Custom length
```

0.1.0 - 25 May 2025

* First release
