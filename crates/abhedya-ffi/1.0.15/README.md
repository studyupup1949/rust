# Abhedya FFI

This crate provides **Foreign Function Interface (FFI)** bindings for the [Abhedya](https://github.com/ParamTatva-org/abhedyam) Post-Quantum Cryptography library.

It exposes a C-ABI compatible API for:

- Key Generation (`abhedya_keygen`)
- LWE Encryption (`abhedya_encrypt`)
- LWE Decryption (`abhedya_decrypt`)

This library is used as the core for Abhedya's Python, Node.js, and Go SDKs.

## Usage

This is a low-level library. See the [main repository](https://github.com/ParamTatva-org/abhedyam) for high-level SDK usage.
