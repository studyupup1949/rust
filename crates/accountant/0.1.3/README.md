# Accountant

A simple accounting utilities library for financial calculations in Rust.

## Features

- Basic balance management
- Credit and debit operations
- Simple and lightweight

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
accountant = "0.1.3"
```

## Usage

```rust
use accountant::Accountant;

fn main() {
    let mut acc = Accountant::new();
    acc.credit(100.0);
    acc.debit(25.0);
    println!("Balance: {}", acc.get_balance()); // 75.0
}
```

## License

MIT License
