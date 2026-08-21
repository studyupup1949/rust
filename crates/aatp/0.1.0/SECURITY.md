# Security Policy

## Supported versions

AATP is pre-1.0. Security fixes are applied to the latest `0.x` release only.

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅        |
| < 0.1   | ❌        |

## Reporting a vulnerability

Please **do not** open a public issue for a security vulnerability. Instead, use GitHub's
private vulnerability reporting:

1. Go to the repository's **Security** tab → **Report a vulnerability**, or
2. Open a [private security advisory](https://github.com/MerlijnW70/aatp/security/advisories/new).

You can expect an initial acknowledgement within a few days. Please include a description,
affected version/commit, and a minimal reproduction if possible.

## Scope — what AATP does and does not protect

AATP is a **transport and framing** layer. Read this before relying on it for security:

- **CRC32C detects accidental corruption, not tampering.** It is not a MAC. An attacker
  who can modify bytes can recompute a valid checksum. For authenticity/integrity against
  a malicious sender, layer a MAC or AEAD on top.
- **The `shadow` module is obfuscation, not encryption.** It is a reversible XOR keystream
  with no confidentiality and no authentication. **Do not use it to protect secrets.** For
  real confidentiality use a vetted authenticated-encryption (AEAD) construction.
- **In scope:** memory safety (`#![forbid(unsafe_code)]`), fail-closed parsing of hostile
  input (no panics, no over-reads, no over-allocation), and correctness of the codec.

Reports about the documented limits of `shadow` or the CRC (i.e. "XOR is breakable",
"CRC can be recomputed") are known and by design — see the module documentation.
