# address

This library aids in processing network addresses.

## Usage

There are currently no features. You can use all address types with the following dependency:

    [dependencies]
    address = "0.11.0"

## Address Types

There are 6 core address types:

- **IPAddress**: Either an IPv4 address or an IPv6 address.
    - Includes `IPAddress` which is an enum, and the `IPv4Address` & `IPv6Address` struct types.
- **SocketAddress**: An IP address with an associated port.
    - Includes the `SocketAddress`, `SocketAddressV4` & `SocketAddressV6` struct types.
- **Domain**: A domain name.
    - Includes: the `Domain` & `DomainRef` types.
- **Endpoint**: A domain with an associated port.
    - Includes: the `Endpoint` & `EndpointRef` types.
- **Host**: Either a domain or an IP address.
    - Includes: the `Host` & `HostRef` types.
- **Authority**: A host with an associated port.
    - Includes: the `Authority` & `AuthorityRef` types.

## Owned & Reference Types

Address types that are not `Copy` have owned and `Ref` types (example: `Domain` & `DomainRef`).
This allows both owned types and types that do not require allocation. They can be easily converted
between each other. `Cow` is not used to avoid complicated ownership.

## Standard Library Types

IP addresses and socket addresses are different from the standard library counterparts. They can be
easily converted between each other. There is a difference in IPv6 socket addresses where the
`flow_info` and `scope_id` are not included as part of the address.
