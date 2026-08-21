# address

This library aids in processing network addresses.

## Features & Dependencies

```toml
address = "0.19.0-rc.2"
```

This crate has no dependencies by default.

- `idna`: Adds `Domain::from_unicode` & `to_unicode` for international domain names. Uses the `idna` crate.
- `serde`: Adds `Serialize` & `Deserialize` implementations via the `serde` crate. Human-readable formats use the
  `Display` & `FromStr` strings. Binary formats use compact binary forms for the IP & socket address types, matching
  the wire format of the standard library types. The `Ref` types deserialize by borrowing from the input.

## Address Types

There are 6 core address types:

- `IPAddress`: Either an IPv4 address or an IPv6 address.
    - Includes the `IPAddress` enum along with the `IPv4Address` & `IPv6Address` struct types.
- `SocketAddress`: An IP address with an associated port.
    - Includes the `SocketAddress`, `SocketAddressV4` & `SocketAddressV6` struct types.
- `Domain`: A domain name.
    - Includes: the `Domain` & `DomainRef` struct types.
- `Endpoint`: A domain with an associated port.
    - Includes: the `Endpoint` & `EndpointRef` struct types.
- `Host`: Either a domain or an IP address.
    - Includes: the `Host` & `HostRef` enum types.
- `Authority`: A host with an associated port.
    - Includes: the `Authority` & `AuthorityRef` struct types.

## Owned & Reference Types

Address types that are not `Copy` have owned and Ref types (example: `Domain` & `DomainRef`). This allows both owned
types and types that do not require allocation. These types can be easily converted between one another.

## Domain Names

Domain names are restricted to lowercase ASCII letters, digits, and dashes: dot-separated labels of up to 63 bytes
that do not start or end with a dash, with a total name length of up to 253 bytes. Mixed-case input is normalized to
lowercase when parsing owned types. Underscores, empty labels, and the trailing root dot are invalid. Unicode names
can be converted to their ASCII form with the `idna` feature.

## Standard Library Types

IP addresses and socket addresses are different from their standard library counterparts. They can be easily converted
between each other. There is a difference in IPv6 socket addresses: the `flow_info` and `scope_id` are not included as
part of the address.
