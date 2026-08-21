#![no_std]
#![feature(const_fn)]

/* mod error; */
/* pub use error::AddrParseError; */

pub mod parser;

mod ipv4;
pub use ipv4::Ipv4Addr;
pub use ipv4::Ipv4Address;

mod ipv6;
pub use ipv6::Ipv6Addr;
pub use ipv6::Ipv6Address;
pub use ipv6::Ipv6MulticastScope;

mod ip;
pub use ip::IpAddr;

mod socket4;
pub use socket4::SocketAddrV4;
pub use socket4::SocketAddressV4;

mod socket6;
pub use socket6::SocketAddrV6;
pub use socket6::SocketAddressV6;

mod socket;
pub use socket::SocketAddr;
pub use socket::ToSocketAddrError;
pub use socket::ToSocketAddrs;
