# ABN Validator

`abn_validator` validates the format and checksum of Australian Business
Numbers (ABNs).

It accepts both compact ABNs and the conventionally grouped form:

```rust
use abn_validator::{AbnError, validate};

assert_eq!(validate("51 824 753 556"), Ok(()));
assert_eq!(validate("51824753556"), Ok(()));
assert_eq!(
    validate("51824753557"),
    Err(AbnError::ChecksumMismatch),
);
```

This crate only verifies that an ABN is well-formed. It does not query the
Australian Business Register or confirm that the ABN has been issued or is
currently active.

## Installation

Add the crate to your project:

```toml
[dependencies]
abn_validator = "0.1"
```

Then call `validate` with an ABN supplied as a string:

```rust
fn process_abn(abn: &str) -> Result<(), abn_validator::AbnError> {
    abn_validator::validate(abn)?;

    // The ABN has the correct format and checksum. Registration status
    // requires a separate lookup with the Australian Business Register.
    Ok(())
}
```

## Accepted input

An input is valid when, after removing whitespace, it contains exactly 11
ASCII digits and passes the ABN checksum.

- Whitespace is ignored, including spaces, tabs, line breaks, and Unicode
  whitespace.
- Other separators are rejected. For example, `51-824-753-556` contains an
  invalid `-` character.
- Non-ASCII numerals are rejected.

Validation failures identify the reason:

| Error | Meaning |
| --- | --- |
| `AbnError::InvalidCharacter(character)` | A character was neither an ASCII digit nor whitespace. |
| `AbnError::InvalidLength(length)` | The input did not contain exactly 11 digits after whitespace was removed. |
| `AbnError::ChecksumMismatch` | The input had 11 digits but failed the checksum. |

## Validation algorithm

ABNs contain a nine-digit identifier and two leading check digits. Validation
subtracts one from the first digit, applies the weighting factors defined by
the Australian Business Register, and checks that the weighted sum is evenly
divisible by 89.

See the Australian Business Register's
[ABN format documentation](https://abr.business.gov.au/Help/AbnFormat) for the
authoritative algorithm and worked example.

## Minimum supported Rust version

The minimum supported Rust version (MSRV) is **Rust 1.85**. The crate uses the
Rust 2024 edition and tests this toolchain in CI.

## License

This project is licensed under the MIT License.
