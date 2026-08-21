# Claude Configuration for acai Project

This file contains persistent directives for Claude when working on the acai project.

> **Note:** The name "acai" is lowercase in all contexts, not an acronym. Always use lowercase "acai" in documentation, code, and comments.

## Words of Wisdom

- "You missed something there and I had to catch it. Let's reflect on why that is and update the documentation and CLAUDE.md to make sure it doesn't happen again."
- "Generalize our learnings so we learn faster" - Look for patterns in issues to develop broader, more effective checks
- Focus on creating tightly-scoped, single-purpose tools that solve the right problem

## ALWAYS means ALWAYS

- Always run `cargo clippy --all-targets -- -D clippy::all` before committing to check for linting issues
- Always run `cargo fmt` immediately after a successful run of clippy
- Always use type-specialized constructors instead of wrapper functions that are rearrangements of their arguments.
- Always provide a positive signal that the code is working as intended, not just absence of failure

## NEVER means NEVER

- Never write `fn test_...` when a function is already within the `tests` module (it's redundant)

## Special Commands

- When the user types `make lint`: Run both `cargo clippy --all-targets -- -D clippy::all && cargo fmt --check` and report success or issues found
- When the user types `make lint --verbose`: Run the linting checks and tests (`cargo clippy`, `cargo fmt --check`, and `cargo test`) and give a detailed report with check-marks for each category of guidelines in this file

## Documentation

- Use comprehensive examples in documentation
- Document all public API methods
- Include clear usage examples for all request types
- Always have a blank line before and after Markdown headers
  - Specifically check for patterns like `/// Text\n/// # Header` which would indicate a missing blank line
  - Use grep or similar tools to systematically check for headers that might be missing proper spacing
  - Headers in Rust doc comments look like `/// # Header` or `/// ## Subheader`

## Testing

- Write tests for all public API functionality
- Include serialization/deserialization tests for JSON structures
- Name test functions descriptively without redundant prefixes
- Design systems that can verify themselves - functions that check their own output
- Test at transient extremes, not just typical cases

## System Design

- Minimize the number of decision points and branching paths
- Make choices that compound and enable, rather than restrict future options
- Allocate strategy between "actual implementation" and "observable behavior"
- Follow a consistent philosophy in design decisions
- Create systems where new features come from new code rather than modifying existing code
- Build a verifier component that validates the system's behavior and output

## Git Practices

- Write comprehensive commit messages explaining the why, not just the what
- Include examples in commit messages when relevant
- Run linters before committing

## Rust Idioms

- Follow Rust naming conventions
- Use explicit typing where it improves readability
- Implement specialized constructor methods for enum variants when appropriate
- Provide builder-style methods with meaningful names (like `with_timeout`) for configurable types
- Always place test modules at the bottom of the file
- Always organize module declarations with the following pattern:
  - Group public modules first (`pub mod bar;`), sorted alphabetically
  - Add a blank line
  - Group private modules next (`mod foo;`)
  - Add descriptive comments for each group
- Quality compounds - invest in it early
- Return `Option<&T>` rather than `&Option<T>` for accessor methods, using `.as_ref()`
- Re-export necessary types for doctest examples instead of marking them as `ignore`
- Prefix imports in doctests with `# ` to hide them in documentation while still testing them
- Maintain semantic accuracy in error conversions (e.g., map HttpError to HttpError when possible)
- Don't wrap types that already use Arc internally (like ReqwestClient) in another Arc
- Remove unnecessary wrapper methods that don't add value beyond the methods they wrap
- Don't add comments explaining what was removed or not implemented - just remove the code
- Remove commented-out code and imports rather than leaving them in the codebase
- Don't add explanatory comments about why a function or operation isn't needed - simply implement the code correctly without calling attention to alternatives
- Prefer a single pattern for methods that do the same thing - avoid aliases like both `register` and `add_handler`
- When parameterizing a type with generics, implement methods for the generic type, not just the concrete type
- Prefer using built-in API methods like `error_for_status()` instead of writing custom error handling logic
- Use appropriate error variants (e.g., `HttpError` for HTTP errors) rather than generic error types
- Create specific error types for common error cases instead of using a generic error with a message
- Follow Rust naming conventions for acronyms: use JsonRpc, not JSONRPC or JSONRpc
- Don't use `unwrap()` or `expect()` in public APIs; instead, return `Result` to properly propagate errors
- Re-export utility types/methods at the crate root with `pub use` to make them visible to users and silence unused method warnings
