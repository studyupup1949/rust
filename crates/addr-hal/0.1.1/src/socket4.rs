use crate::Ipv4Addr;
use crate::Ipv4Address;
use core::fmt;
use core::hash;

/// Describe the internal data structure behavior of `SocketAddrV4`.
///
/// You can implement this trait by yourself or use ffi for specific Platform.
pub trait SocketAddressV4: Clone + Copy {
    /// Ipv4Address inner type.
    type IpAddress: Ipv4Address;

    /// Creates a new IPv4 address from ip address and port
    ///
    /// The result will represent the Socket address ip:port.
    fn new(ip: Ipv4Addr<Self::IpAddress>, port: u16) -> Self;

    /// Got ip address.
    fn ip(&self) -> &Ipv4Addr<Self::IpAddress>;

    /// Set ip address.
    fn set_ip(&mut self, ip: Ipv4Addr<Self::IpAddress>);
    /// Got port.
    fn port(&self) -> u16;

    /// Set port.
    fn set_port(&mut self, port: u16);
}

/// An IPv4 socket address.
///
/// IPv4 socket addresses consist of an [IPv4 address] and a 16-bit port number, as
/// stated in [IETF RFC 793].
///
/// See [`SocketAddr`] for a type encompassing both IPv4 and IPv6 socket addresses.
///
/// The size of a `SocketAddrV4` struct may vary depending on the target operating
/// system.
///
/// [IETF RFC 793]: https://tools.ietf.org/html/rfc793
/// [IPv4 address]: ../../std/net/struct.Ipv4Addr.html
/// [`SocketAddr`]: ../../std/net/enum.SocketAddr.html
///
/// # Examples
///
/// ```
/// use std::net::{Ipv4Addr, SocketAddrV4};
///
/// let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
///
/// assert_eq!("127.0.0.1:8080".parse(), Ok(socket));
/// assert_eq!(socket.ip(), &Ipv4Addr::new(127, 0, 0, 1));
/// assert_eq!(socket.port(), 8080);
/// ```
pub struct SocketAddrV4<SA4: SocketAddressV4> {
    inner: SA4,
}

impl<SA4: SocketAddressV4> SocketAddrV4<SA4> {
    /// Creates a new socket address from an [IPv4 address] and a port number.
    ///
    /// [IPv4 address]: ../../std/net/struct.Ipv4Addr.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV4, Ipv4Addr};
    ///
    /// let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// ```
    pub fn new(ip: Ipv4Addr<SA4::IpAddress>, port: u16) -> SocketAddrV4<SA4> {
        SocketAddrV4 {
            inner: SA4::new(ip, port),
        }
    }

    /// Returns the IP address associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV4, Ipv4Addr};
    ///
    /// let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// assert_eq!(socket.ip(), &Ipv4Addr::new(127, 0, 0, 1));
    /// ```
    pub fn ip(&self) -> &Ipv4Addr<SA4::IpAddress> {
        self.inner.ip()
    }

    /// Changes the IP address associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV4, Ipv4Addr};
    ///
    /// let mut socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// socket.set_ip(Ipv4Addr::new(192, 168, 0, 1));
    /// assert_eq!(socket.ip(), &Ipv4Addr::new(192, 168, 0, 1));
    /// ```
    pub fn set_ip(&mut self, new_ip: Ipv4Addr<SA4::IpAddress>) {
        self.inner.set_ip(new_ip)
    }

    /// Returns the port number associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV4, Ipv4Addr};
    ///
    /// let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// assert_eq!(socket.port(), 8080);
    /// ```
    pub fn port(&self) -> u16 {
        self.inner.port()
    }

    /// Changes the port number associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV4, Ipv4Addr};
    ///
    /// let mut socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// socket.set_port(4242);
    /// assert_eq!(socket.port(), 4242);
    /// ```
    pub fn set_port(&mut self, new_port: u16) {
        self.inner.set_port(new_port)
    }
}

impl<SA4: SocketAddressV4> Clone for SocketAddrV4<SA4> {
    fn clone(&self) -> SocketAddrV4<SA4> {
        SocketAddrV4 {
            inner: self.inner.clone(),
        }
    }
}

impl<SA4: SocketAddressV4> Copy for SocketAddrV4<SA4> {}

impl<SA4: SocketAddressV4> fmt::Display for SocketAddrV4<SA4> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip(), self.port())
    }
}

impl<SA4: SocketAddressV4> fmt::Debug for SocketAddrV4<SA4> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl<SA4: SocketAddressV4> Eq for SocketAddrV4<SA4> {}

impl<SA4: SocketAddressV4> PartialEq for SocketAddrV4<SA4> {
    fn eq(&self, other: &SocketAddrV4<SA4>) -> bool {
        let s_ip = self.ip();
        let o_ip = other.ip();

        let s_port = self.port();
        let o_port = other.port();
        (s_ip, s_port).eq(&(o_ip, o_port))
    }
}

impl<SA4: SocketAddressV4> hash::Hash for SocketAddrV4<SA4> {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        let ip = self.ip();
        let port = self.port();
        (ip.octets(), port).hash(s)
    }
}

