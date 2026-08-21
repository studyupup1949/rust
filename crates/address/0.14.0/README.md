# address

This library aids in processing network addresses.

## Usage

There are currently no features. You can use all address types with the following dependency:

    [dependencies]
    address = "0.14.0"

## Address Types

There are 6 core address types:

- **IPAddress**: Either an IPv4 address or an IPv6 address.
    - Includes the `IPAddress` enum along with the `IPv4Address` & `IPv6Address` struct types.
- **SocketAddress**: An IP address with an associated port.
    - Includes the `SocketAddress`, `SocketAddressV4` & `SocketAddressV6` struct types.
- **Domain**: A domain name.
    - Includes: the `Domain` & `DomainRef` struct types.
- **Endpoint**: A domain with an associated port.
    - Includes: the `Endpoint` & `EndpointRef` struct types.
- **Host**: Either a domain or an IP address.
    - Includes: the `Host` & `HostRef` enum types.
- **Authority**: A host with an associated port.
    - Includes: the `Authority` & `AuthorityRef` struct types.

## Owned & Reference Types

Address types that are not `Copy` have owned and `Ref` types (example: `Domain` & `DomainRef`). This allows both owned
types and types that do not require allocation. These types can be easily converted between one another.

## Standard Library Types

IP addresses and socket addresses are different from their standard library counterparts. They can be easily converted
between each other. There is a difference in IPv6 socket addresses: the `flow_info` and `scope_id` are not included as
part of the address.
