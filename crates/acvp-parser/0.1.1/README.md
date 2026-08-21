# Automated Cryptographic Validation Protocol (ACVP) Parser

![CI Badge](https://github.com/puru1761/acvp-parser/actions/workflows/main.yml/badge.svg)
![License](https://img.shields.io/github/license/puru1761/acvp-parser)
![Crate Badge](https://img.shields.io/crates/v/acvp-parser.svg)

This repository contains the source code for an ACVP Parser crate implemented in
the Rust programming language. This library is meant to be used for parsing
ACVP style test vectors for use in Cryptographic Algorithm Validation System
(CAVS) testing to be performed for obtaining FIPS 140-{2,3} CAVS certificates.

## Usage

Add the following to your `Cargo.toml` in order to use this crate:

```
acvp-parser = "*"
```

## Build

To build this crate for development purposes, do:

```
cargo build
```

## Test

To test the APIs provided by this crate, do:

```
cargo test
```

## Author

* Purushottam A. Kulkarni <<puruk@protonmail.com>>