# Accelerate-General: Matrix and Vector General Operations using Apple's Accelerate Framework in Rust

This project provides Rust **FFI bindings** to interact with **Apple's Accelerate framework**, enabling efficient matrix and vector operations with both single-precision (`f32`) and double-precision (`f64`) floating-point numbers, as well as complex numbers (`Complex<f32>`, `Complex<f64>`).

## Features

- **Vector and Matrix Operations**: Provides efficient routines for basic linear algebra operations.
- **Single and Double Precision**: Support for both single (`f32`) and double (`f64`) precision operations.
- **Complex Number Support**: Complex arithmetic with both single-precision and double-precision complex numbers.
- **Optimized for Apple Platforms**: Uses the Accelerate framework for high performance on macOS and iOS.

## Requirements

- **Rust** (1.60 or higher)
- **Apple's Accelerate Framework** (iOS 16.4+ ,iPadOS 16.4+ ,Mac Catalyst 16.4+ ,macOS 13.3+ ,tvOS 16.4+ ,visionOS 1.0+ ,watchOS 9.4+)
- **FFI** for interfacing with C functions

### Dependencies

The project uses the following dependencies:

- `num-complex`: To support complex number operations in Rust.
- `std::ffi`: For calling C functions via FFI.

```toml
[dependencies]
num-complex = "0.4"
```

## Getting Started

### Installation

Clone the repository and include it in your project by adding the following to your Cargo.toml:
```toml
[dependencies]
accelerate-general = { path = "/path/to/your/cloned/repo" }
```

### Usage
1. Import the required modules and types from the library.
2. Use FFI functions for matrix and vector operations.

## Safety

All functions in this library are marked as unsafe since they directly interface with C libraries via FFI. It is the responsibility of the caller to ensure that:

- Memory pointers passed to the functions are valid and aligned.
- vectors and matrices are correctly dimensioned and non-null.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgements

This project uses Apple’s Accelerate framework for optimized matrix and vector operations. The Accelerate framework provides high-performance BLAS routines that are used via FFI in this project.