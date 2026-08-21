# Contributing to AAML

First off, thank you for considering contributing to AAML! It's people like you that make open source software such an amazing place to learn, inspire, and create.

We welcome contributions of all forms:
* 🐛 **Bug Reports**: Found a parser error? Let us know.
* 💡 **Feature Requests**: Need a new method in `AAMBuilder`? Open an issue.
* 👩‍💻 **Code**: Fixes, optimizations, or new features.
* 📝 **Documentation**: Typos, unclear examples, or better explanations.

## 🛠 Development Setup

To contribute to AAML, you will need **Rust** and **Cargo** installed.

1. **Fork the repository** on GitHub.
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/aaml.git
   cd aaml
   ```

3. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/my-new-feature
   # or
   git checkout -b fix/issue-number
   ```

## 🧪 Testing & Code Style

We want to keep AAML stable and idiomatic. Please follow these steps before submitting a PR:

### 1. Run Tests
Ensure all existing tests pass and add new tests for your features.
```bash
cargo test
```

### 2. Format Code
We use `rustfmt` to ensure consistent code style.
```bash
cargo fmt
```

### 3. Run Clippy
Rust's linter helps catch common mistakes.
```bash
cargo clippy -- -D warnings
```

## 📥 Submitting a Pull Request (PR)

1. **Push your changes** to your fork.
2. Go to the original AAML repository and click **"Compare & pull request"**.
3. **Description**:
   - Clearly describe what you changed and why.
   - If it fixes an open issue, reference it (e.g., `Fixes #12`).
   - If you added a new feature (like a new parsing rule), please provide a small example configuration snippet.
4. **Review**: We will review your code as soon as possible. We might ask for small changes to fit the project's architecture.

## 🐛 Reporting Bugs

If you find a bug, please create an issue including:

- **Version**: Which version of AAML are you using?
- **Reproduction**: A minimal `.aam` file content that causes the error.
- **Error Message**: The output from `AamlError` (e.g., `Parse Error at line 5...`).

## 📜 License

By contributing, you agree that your contributions will be licensed under the same License that covers the project (see the [LICENSE](LICENSE) file).
