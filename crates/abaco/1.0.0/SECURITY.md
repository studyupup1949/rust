# Security Policy

## Scope

Abaco is a pure Rust math library providing expression evaluation and unit conversion. It has no network I/O in the core library (the `ai` feature adds reqwest for planned currency rates).

## Attack Surface

| Area | Risk | Mitigation |
|------|------|------------|
| Expression parsing | Stack overflow via deep nesting | Recursive descent with bounded token stream |
| Division by zero | Panic or undefined | Explicit zero checks, returns `EvalError` |
| NaN/Infinity | Silent propagation | `check_result()` validates every function output |
| Unit lookup | Timing side-channel | HashMap-based, constant-time per lookup |
| Serialization | Malformed input | serde_json validation |
| Numeric overflow | Silent wrap | f64 range; i64 safe-range check before cast |

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.22.x | Yes |
| < 0.22 | No |

## Reporting Vulnerabilities

- Email: security@agnos.dev
- Do **not** open public issues for security vulnerabilities
- 48-hour acknowledgment SLA
- 90-day coordinated disclosure timeline

## Design Principles

- Zero `unsafe` code
- No raw pointer manipulation
- All public types are `Send + Sync` compatible
- Minimal dependency surface
