# actix-rate-limiter

`actix-rate-limiter` is a simple yet powerful per-route rate limiter for
[Actix](https://docs.rs/actix-web/latest/actix_web/) with support for regex.

### Available backends

Right now, only in-memory storage is supported officially. But you can
create your own backend using the `BackendProvider` trait. You can use
`MemoryBackendProvider` as an example implementation.

We plan to add support for some other backends in the future, such as Redis.
If you want to help with their development, please open a PR.

### Examples

Check the [examples](./examples/) folder of our repository to see the available
code samples.

### License

This library is [licensed](./LICENSE) under MIT License.
