# Introduction

Adapters is a high-performance, developer-friendly, and schema-driven data validation, serialization, and transformation library designed for modern Rust.

By bridging the gap between dynamic formats (like JSON or raw key-value objects) and Rust's strictly typed domain models, Adapters allows you to validate dynamic data payloads before instantiation, serialize/deserialize with full bounds checking, and easily map types between different API boundaries.

---

## Key Pillars of the Library

The library is designed around three distinct, yet highly interconnected layers:

- **Unified Schema & Validation Layer**:
  Instead of validating data after parsing it into structured models (which can cause panics or silent errors on invalid types), Adapters defines a declarative, dynamic schema tree. Inbound payloads are verified at the dynamic level first, matching strict type names, number ranges, string lengths, custom regexes, and complex formats (e.g., Email or URLs).

- **High-Performance Serialization & Deserialization**:
  Adapters implements a zero-dependency serialization and deserialization model. Every primitive type (including newly added `char`, `i128`, `u128`, and standard network addresses like `IpAddr`, `Ipv4Addr`, and `Ipv6Addr`), option, vector, and map is fully supported, allowing seamless, safe round-trips from dynamic values back into complex Rust structures.

- **Functional Data Transformation**:
  Domain models often diverge between different contexts (e.g., Database Models vs. API Presentation Models). Using Pipeline and FieldMapper classes, you can map, rename, and transform data trees programmatically in a highly functional manner.

---

## Why Use Adapters?

- **Zero-Dependency Core Parser**: Comes with a built-in recursive-descent JSON engine, avoiding bloated dependencies and guaranteeing high compilation speed.
- **Detailed Error Accumulation**: The validator evaluates all constraint rules rather than short-circuiting, gathering all errors across your nested fields into a comprehensive error list in a single pass.
- **Safe Native Numeric Types**: Distinct floating-point and integer tracking prevents type coercion bugs before they make it into your business logic.
- **Ergonomic Macros**: Instantly implement all necessary traits for your types using simple struct tags via the `#[derive(Schema)]` proc-macro.

---

## Authors & Sponsorship

Adapters is maintained by **[Muhammad Fiaz](https://muhammadfiaz.com)**.

- **GitHub Profile**: [@muhammad-fiaz](https://github.com/muhammad-fiaz)
- **Official Website**: [muhammadfiaz.com](https://muhammadfiaz.com)
- **Support & Sponsor**: [pay.muhammadfiaz.com](https://pay.muhammadfiaz.com) or via [GitHub Sponsors](https://github.com/sponsors/muhammad-fiaz)

