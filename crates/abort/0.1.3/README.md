# abort

This crate defines a lone function `abort`, which has one job: terminate the calling process (abnormally).

It works on stable Rust by default, and has an optional "nightly" feature flag to rather use the unstable `core::intrinsics::abort`.
