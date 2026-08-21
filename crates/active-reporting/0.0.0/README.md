# Report

`Report` helps with issue reporting by providing a typed interface into error chain management.

## What problem it solves

`Result<T, E>` can only carry one error but we want to show multiple (and possibly with different metadata) to the user.

## Usage

```rust
use libutils_report::{
    Report,
    Same,
    Name
};

// creates a report
// there should only be one report per program
let report = Report::default();

// report anything that implements Into<Issue>
report.issue("an error happened");

// closure with same chain
let closure = |report: Report<Same>| {};

// closure with added chain name
// when this report is dropped, the chain node will be popped
// the chain works effectively as a stack
let another = |report: Report<Name<"Another">>| {
    // pass on the report
    closure(report.to());
};
```

This type allows for management of the error chain (where issues are originated) based on a RAII pattern. The `Issue` type comes from [`libutils-issue`](https://crates.io/crates/libutils-issue).

There are three modes for the report:
- `Main`: just created
- `Same`: no node added
- `Name<&'static str>`: add a node, the pop when this report is dropped

And includes three functions:
- `.to()`: passes on the report to a function, and modifies/maintains error chain,
- `.issue(impl Into<Issue>)`: reports an issue,
- `.eat<Type>(Result<Type, Issue>) -> Option<Type>`: reports the issue if it exists

## When to use it

This type is useful in application with complex error reporting. Specially, for programs in which the *amount* of errors is not as simplistic as `Result<T, E>`, and they must be shown to the user.

It is not well-suited for:
- simple libraries: should return `Result<T, Issue>` instead,
- simple binaries: should use `Result<T, E>`,
- `#![no_std]` environments

It **is** well-suited for:
- CLI tools,
- compilers,
- user-facing applications