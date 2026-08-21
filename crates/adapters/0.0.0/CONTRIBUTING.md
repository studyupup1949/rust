# Contributing to Adapters

Thank you for your interest in contributing to **Adapters**! We welcome community contributions, bug reports, feature requests, and documentation improvements.

---

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct. Please maintain a professional, respectful, and friendly atmosphere.

---

## How Can I Contribute?

### 1. Reporting Bugs
- Search existing issues to check if the bug has already been reported.
- If not, open a new issue detailing:
  - The expected behavior vs. actual behavior.
  - Clear steps to reproduce the issue.
  - Your environment details (Rust version, OS, etc.).
  - Code snippets or minimal reproducible examples.

### 2. Suggesting Enhancements
- Open an issue explaining the proposed feature.
- Highlight the use cases, advantages, and alternative approaches considered.

### 3. Submitting Pull Requests
- Fork the repository and create your branch from `main`.
- Install the standard toolchain matching **Rust Edition 2024**.
- Write clear, concise code with comments explaining non-obvious design choices.
- Include unit/integration tests and update relevant benchmarks/examples.
- Run code quality tools and verify that all tests pass:
  ```bash
  # Check style and code quality rules
  cargo clippy --workspace --all-targets -- -D warnings
  
  # Format code
  cargo fmt --all -- --check

  # Run full test suite
  cargo test --workspace --all-targets
  ```
- Write descriptive, clean commit messages.
- Submit your Pull Request against the `main` branch.

---

## Project Structure

- **`src/`**: Core library files containing type definitions, JSON parsing, serializers, and schema builders.
- **`crates/adapters-macros/`**: Proc-macro package implementing declarative validation parsing.
- **`examples/`**: Compilable usage examples.
- **`tests/`**: Functional integration tests.
- **`docs/`**: Native custom `mdBook` documentation layout.

---

## License

By contributing to this repository, you agree that your contributions will be licensed under the project's **MIT License**.
