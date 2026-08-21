use crate::{IPAddress, IPv4Address, IPv6Address, SocketAddress, SocketAddressV4, SocketAddressV6};

/// Implements `Serialize` and `Deserialize` for an owned type that serializes as its `Display` string in
/// human-readable formats and as the binary type in other formats.
macro_rules! impl_serde_string_or_binary {
    ($ty:ident, $expecting:literal, $bin:ty, $to_bin:expr, $from_bin:expr) => {
        impl ::serde::Serialize for crate::$ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                if serializer.is_human_readable() {
                    serializer.collect_str(self)
                } else {
                    let to_bin = $to_bin;
                    let binary: $bin = to_bin(*self);
                    ::serde::Serialize::serialize(&binary, serializer)
                }
            }
        }

        impl<'de> ::serde::Deserialize<'de> for crate::$ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    deserializer.deserialize_str(crate::serde::FromStrVisitor::new($expecting))
                } else {
                    <$bin as ::serde::Deserialize>::deserialize(deserializer).map($from_bin)
                }
            }
        }
    };
}

impl_serde_string_or_binary!(
    IPv4Address,
    "an IPv4 address string",
    [u8; 4],
    IPv4Address::address,
    IPv4Address::new
);

impl_serde_string_or_binary!(
    IPv6Address,
    "an IPv6 address string",
    [u8; 16],
    IPv6Address::address,
    IPv6Address::new
);

impl_serde_string_or_binary!(
    SocketAddress,
    "a socket address string",
    (IPAddress, u16),
    |socket: SocketAddress| (socket.ip(), socket.port()),
    |(ip, port)| SocketAddress::new(ip, port)
);

impl_serde_string_or_binary!(
    SocketAddressV4,
    "an IPv4 socket address string",
    (IPv4Address, u16),
    |socket: SocketAddressV4| (socket.ip(), socket.port()),
    |(ip, port)| SocketAddressV4::new(ip, port)
);

impl_serde_string_or_binary!(
    SocketAddressV6,
    "an IPv6 socket address string",
    (IPv6Address, u16),
    |socket: SocketAddressV6| (socket.ip(), socket.port()),
    |(ip, port)| SocketAddressV6::new(ip, port)
);
