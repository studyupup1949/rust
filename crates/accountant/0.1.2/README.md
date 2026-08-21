# Accountant - Security Research Placeholder

⚠️ **This is a security research placeholder package** ⚠️

## Purpose

This crate name was found to be referenced in production code but was **not registered on crates.io**, making it vulnerable to **dependency confusion attacks**.

This placeholder was registered to:
1. Prevent malicious actors from claiming this package name
2. Demonstrate the vulnerability as part of responsible security research
3. Alert the legitimate project owners to register their internal dependencies

## What is Dependency Confusion?

Dependency confusion (also known as namespace confusion) is a supply chain attack technique where:

1. An organization uses internal/private package names in their code
2. These package names are not registered on public registries (npm, PyPI, crates.io, etc.)
3. An attacker registers the same package name on the public registry
4. When developers or CI/CD systems build the project, they may fetch the attacker's malicious package

## Affected Project

This package name was found referenced in:
- **Repository**: [wormhole-foundation/wormhole](https://github.com/wormhole-foundation/wormhole)
- **File**: `cosmwasm/contracts/global-accountant/Cargo.toml`
- **Reference**: `accountant = "0.1.0"`

## No Malicious Code

This package contains **NO malicious code**. It is a harmless placeholder with:
- A simple struct definition
- Documentation about the security issue
- No network calls, file access, or build scripts

## For Wormhole Team

If you are from the Wormhole Foundation and wish to claim this package name:
- Please contact the author through the repository
- Ownership can be transferred to your organization

## Security Research

This placeholder was registered as part of responsible security disclosure practices.

## License

MIT License - This is a placeholder package for security research.
