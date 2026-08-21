# Issues

## Cargo

- Decide whether to declare an MSRV: add `rust-version` to `Cargo.toml`, measure it, and enforce it in CI.
- Test the feature matrix: no features, `idna` alone, and `serde` alone, not just `--all-features`.

## API

- Add `From` impls for std concrete types to crate aggregates: `Ipv4Addr`/`Ipv6Addr` -> `IPAddress` and
  `SocketAddrV4`/`SocketAddrV6` -> `SocketAddress`, matching the std `IpAddr`/`SocketAddr` conversions.
- Add direct `to_authority()` for `SocketAddressV4` & `SocketAddressV6`, avoiding `.to_socket().to_authority()`.
- Align error accessor naming: `InvalidDomainName` exposes `name()` & `into_name()` while the coding rules say
  `value()` & `into_value()`. Rename the accessors or amend the rule.
- Add `PartialEq<&str>` for `Domain` & `DomainRef` so `domain == "localhost"` works, matching the std `String`/`str`
  comparisons.
- Add a `labels()` iterator to `Domain` & `DomainRef`, avoiding manual `name().split('.')`.

## Validation

- `Domain` allows an all-numeric final label (e.g. `999.1.1.1`), so malformed IPv4 strings parse as domains in
  `Host`/`Authority`. RFC 1123/3696 hostname rules forbid this; raw DNS allows it. Decide which profile to follow.
- Socket parsing rejects the zone syntax std accepts (`[fe80::1%1]:80` parses via `SocketAddrV6::from_str`), since
  scope ids are not modeled. Decide whether to document or support it.

## Testing

- Coverage is representative, not exhaustive: most items get a single happy-path case. Edge cases could be much
  deeper across the crate.
- Test the README claim that version-specific types match the std wire format: assert binary-serialized
  `IPv4Address`/`IPv6Address`/`SocketAddressV4`/`SocketAddressV6` bytes equal their std counterparts.

## Performance

- Revisit the `parse` byte -> str conversions with `[u8]::as_ascii()` when nightly `ascii_char` (rust#110998)
  stabilizes; safe `from_utf8` currently relies on its ASCII fast path being equivalent.
