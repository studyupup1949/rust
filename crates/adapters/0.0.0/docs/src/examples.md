# Usage Examples

Welcome to the examples section for **Adapters**! These compilable walkthroughs showcase different core features and capabilities of the library:

## 1. Basic Schema Validation & Serialization
Demonstrates dynamic validation, struct mapping via simple tags, and round-trips from JSON using `#[derive(Schema)]`.
- **Go to page:** [Basic Schema](examples/basic_schema.md)

## 2. Explicit Schema Builder
Learn how to define structural constraints dynamically at runtime without derive macros.
- **Go to page:** [Explicit Schema](examples/explicit_schema.md)

## 3. Advanced Validation Constraints
Explore full constraint capabilities like non-empty, alphanumeric, positive/negative bounds, and non-zero check rules.
- **Go to page:** [Advanced Validators](examples/advanced_validators.md)

## 4. Native JSON Parsing Engine
Showcases native, highly-optimized zero-dependency tokenization, unicode escapes, and pretty-print JSON formatting.
- **Go to page:** [JSON Parsing](examples/json_parsing.md)

## 5. Deep Nested Object Models
Validate recursive children structures and gather accurate validation error paths (e.g., `address.city`).
- **Go to page:** [Nested Models](examples/nested_models.md)

## 6. Optional Fields & Default Values
Leverage Option fields and handle default fallback values programmatically.
- **Go to page:** [Optional & Defaults](examples/optional_defaults.md)

## 7. Strict Type Coercion Mode
Prevent implicit casting or conversion of numeric and string types by opting into strict mode.
- **Go to page:** [Strict Mode](examples/strict_mode.md)

## 8. Data Transformation & Pipelines
Transform and adapt domain structures between database layouts and web responses.
- **Go to page:** [Data Transformation](examples/transformation.md)
