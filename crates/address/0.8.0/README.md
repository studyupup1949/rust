# address
This library aids in processing network addresses.

## Address Types
There are 6 core address types:

- **IP Address**: An IPv4 or IPv6 address. Includes IPv4, IPv6 and enum types.
- **Socket Address**: An IP address with an associated port. Include IPv4, IPv6, and IP (non-enum) types.
- **Domain**: A validated domain name. Includes owned and reference types.
- **Endpoint**: A domain name with an associated port. Includes owned and reference types.
- **Host**: A domain name or an IP address. Includes owned and reference types.
- **Authority**: A host with an associated port. Includes owned and reference types.

## Owned & Reference Types
Address types that are not `Copy` have owned and `Ref` types (example: `Domain` & `DomainRef`). This allows both owned
types and types that do not require allocation. They can be easily converted between each other.

## Standard Library Types
IP addresses and socket addresses are different from the standard library counterparts. They can be easily converted
between each other. There is a difference in IPv6 socket addresses where the `flow_info` and `scope_id` are not
included as part of the address.

