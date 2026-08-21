# Ad Hoc Result

A Rust library providing an extension to Rust's standard `Result` type that allows for "ad hoc" values to be provided alongside errors.

This is useful in scenarios where a computation may fail but can still recommend a reasonable value to use despite the failure.

For example, this is needed when you can solve a linear system, but the accuracy is poor due to large condition numbers: you may want to return the computed solution as a recommendation, even though the operation is technically a failure.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ad-hoc-result = "0.1.0"
```

## Usage

The `AdHocResult` type extends the standard `Result` with a third variant:

```rust
enum AdHocResult<T, E> {
    Ok(T),          // Operation succeeded
    AdHoc(T, E),    // Operation failed, but here's a recommended value
    Err(E)          // Operation failed completely
}
```

### Basic Example

```rust
use ad_hoc_result::AdHocResult;

// Function that returns AdHocResult
fn divide(a: f64, b: f64) -> AdHocResult<f64, String> {
    if b == 0.0 {
        // Operation failed, but we recommend using infinity
        AdHocResult::AdHoc(f64::INFINITY, "Division by zero".to_string())
    } else {
        AdHocResult::Ok(a / b)
    }
}

// Normal successful case
let result = divide(10.0, 2.0);
assert_eq!(result.unwrap(), 5.0);

// Ad-hoc case - failed but with a recommendation
let ad_hoc_result = divide(10.0, 0.0);
assert_eq!(ad_hoc_result.unwrap_adhoc(), f64::INFINITY);
```

### Handling AdHocResult

The library provides several ways to handle the different variants:

```rust
use ad_hoc_result::AdHocResult;

let ok: AdHocResult<i32, &str> = AdHocResult::Ok(42);
let adhoc: AdHocResult<i32, &str> = AdHocResult::AdHoc(42, "Not ideal");
let err: AdHocResult<i32, &str> = AdHocResult::Err("Error");

// Extract only strictly successful values
assert_eq!(ok.unwrap(), 42);
// ok_value would panic on adhoc or err variants

// Extract success or recommended values
assert_eq!(ok.unwrap_adhoc(), 42);
assert_eq!(adhoc.unwrap_adhoc(), 42);
// adhoc_value would panic on err variant

// Convert to standard Result
assert_eq!(ok.to_result(), Ok(42));
assert_eq!(adhoc.to_result(), Err("Not ideal")); // Discards the recommended value
assert_eq!(err.to_result(), Err("Error"));

// Convert to Result treating AdHoc as success
assert_eq!(ok.to_result_with_adhoc(), Ok(42));
assert_eq!(adhoc.to_result_with_adhoc(), Ok(42)); // Uses the recommended value
assert_eq!(err.to_result_with_adhoc(), Err("Error"));
```

### Creating AdHocResult

Use the `new_*` methods for cleaner creation:

```rust
let ok = AdHocResult::new_ok(42);
let adhoc = AdHocResult::new_adhoc(42, "Not ideal");
let err = AdHocResult::new_err("Error occurred");
```

### Conversion To/From Result

Seamless conversion between `Result` and `AdHocResult`:

```rust
// From Result to AdHocResult
let ok_result: Result<i32, &str> = Ok(42);
let adhoc_ok: AdHocResult<i32, &str> = ok_result.into();

let err_result: Result<i32, &str> = Err("Error");
let adhoc_err: AdHocResult<i32, &str> = err_result.into();

// From AdHocResult to Result
let ok: AdHocResult<i32, &str> = AdHocResult::Ok(42);
let ok_result: Result<i32, &str> = ok.into(); // Result::Ok(42)

let adhoc: AdHocResult<i32, &str> = AdHocResult::AdHoc(42, "Not ideal");
let adhoc_result: Result<i32, &str> = adhoc.into(); // Result::Err("Not ideal")
```

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).