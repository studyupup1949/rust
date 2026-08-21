# Issues

## Cargo

- Decide whether to declare an MSRV: add `rust-version` to `Cargo.toml`, measure it, and enforce it in CI.
- Test the feature matrix: no features, `idna` alone, and `serde` alone, not just `--all-features`.

## API

- Add `From` impls for std concrete types to crate aggregates: `Ipv4Addr`/`Ipv6Addr` -> `IPAddress` and
  `SocketAddrV4`/`SocketAddrV6` -> `SocketAddress`, matching the std `IpAddr`/`SocketAddr` conversions.
- Add direct `to_authority()` for `SocketAddressV4` & `SocketAddressV6`, avoiding `.to_socket().to_authority()`.

## Validation

- `Domain` allows an all-numeric final label (e.g. `999.1.1.1`), so malformed IPv4 strings parse as domains in
  `Host`/`Authority`. RFC 1123/3696 hostname rules forbid this; raw DNS allows it. Decide which profile to follow.

## Testing

- Coverage is representative, not exhaustive: most items get a single happy-path case. Edge cases could be much
  deeper across the crate.
