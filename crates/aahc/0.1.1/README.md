## Overview

Agnostic Aysynchronous HTTP Client (aahc) is a lightweight, pure Rust
asynchronous HTTP/1.1 client that does no dynamic memory allocation, is not
tied to any specific asynchronous executor or executors, and does not spawn
additional threads.

## Design goals

### Agnostic to choice of executor

Most other asynchronous HTTP client crates depend on a specific asynchronous
runtime environment (usually tokio or async-std, or less frequently fibers).
Most of the rest have feature flags that let them work with two or three
specific choices of executor. aahc does not depend on any particular
asynchronous executor or runtime environment and should work with any executor
that provides socket capabilities. This is achieved by requiring the user to
provide the transport socket over which the HTTP request should be sent, which
must be appropriate to the asynchronous runtime environment used by the
application—an application running in tokio would pass in tokio sockets, an
application running in async-std would pass in async-std sockets, an
application running in GTK would pass in GLib sockets, and so on.

### Threadless

The only other asynchronous HTTP client crate I am aware of that is truly
executor-agnostic is [Isahc](https://crates.io/crates/isahc). Isahc
accomplishes this goal by spawning an auxiliary thread which performs I/O.
aahc does not spawn any threads.

### Safe

aahc is `#![forbid(unsafe_code)]`. Its dependencies are
[httparse](https://crates.io/crates/httparse) and
[futures-io](https://crates.io/crates/futures-io), which both have relatively
little unsafe code.

### Zero allocations

aahc does not perform any heap allocations during normal operation. Neither
does its underlying parser, [httparse](https://crates.io/crates/httparse). Note
that if the `detailed-errors` feature is enabled (which is the default) then
some errors are returned with heap-allocated detailed messages; if the feature
is disabled, then these errors are returned with less informative data but with
no heap allocation.

## Limitations

### HTTP/2+

aahc is an HTTP/1.x client. It does not currently support HTTP/2+.

### Compression

aahc does not currently support HTTP compression in the `Transfer-Encoding`
header.

Compression via the `Content-Encoding` header can be used; as an end-to-end
header (rather than a hop-by-hop header), aahc takes the perspective that
`Content-Encoding` is the application’s responsibility.

### HTTP proxies

SOCKS proxies work entirely at the transport layer, below the HTTP layer. Thus,
an application can connect to a SOCKS proxy, perform the initial SOCKS
negotiation, and pass the resulting socket into aahc.

HTTP proxies using the `CONNECT` method work similarly to SOCKS proxies. Thus,
an application can similarly connect to an HTTP proxy, perform the initial
`CONNECT` request, and pass the resulting socket into aahc. However, aahc does
not support making `CONNECT` requests itself, and thus cannot be used to
provide tunnels through `CONNECT`-style proxies for other consumers (or other
instances of itself).

HTTP proxies not using `CONNECT` are supported by aahc. As always, the
application is responsible for creating and connecting the transport socket, in
this case to the proxy rather than to the origin server. The application can
then provide a request target in absolute form rather than origin form, as is
appropriate when communicating with a proxy server.

### The Upgrade header

aahc does not support the `Upgrade` header. It therefore cannot speak
WebSocket.

### 1xx status codes

aahc currently ignores 1xx (informational) status codes. Only the final ≥2xx
response is returned to the application.

### Trailers

aahc does not currently support the use of trailers following a response body
using chunked transfer encoding. Applications must not set `TE: trailers` in
their request headers.

## Features

### detailed-errors

If this feature is enabled, error returns of kind `InvalidData` contain a
nested error object with more details about the exact error; however, because
of the implementation of `std::io::Error`, such additional details must be
allocated on the heap. If this feature is disabled, error returns of kind
`InvalidData` do not contain a nested error object.
