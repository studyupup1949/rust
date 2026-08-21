# absperf-minilzo-sys

Unsafe wrappers around minilzo.  This is a pretty simple rust-bindgen generated
wrapper.

## Why not the existing one?

That one doesn't actually cover the entire library, and it doesn't statically
link the library.  We could make those fixes upstream, but we need this library
functioning ASAP.