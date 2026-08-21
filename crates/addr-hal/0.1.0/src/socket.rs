use crate::{
    IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddressV4, SocketAddressV6,
};
use core::fmt;
use core::hash;
use core::iter::Iterator;
use core::option;

/// An internet socket address, either IPv4 or IPv6.
///
/// Internet socket addresses consist of an [IP address], a 16-bit port number, as well
/// as possibly some version-dependent additional information. See [`SocketAddrV4`]'s and
/// [`SocketAddrV6`]'s respective documentation for more details.
///
/// The size of a `SocketAddr` instance may vary depending on the target operating
/// system.
///
/// [IP address]: ../../std/net/enum.IpAddr.html
/// [`SocketAddrV4`]: ../../std/net/struct.SocketAddrV4.html
/// [`SocketAddrV6`]: ../../std/net/struct.SocketAddrV6.html
///
/// # Examples
///
/// ```
/// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
///
/// let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
///
/// assert_eq!("127.0.0.1:8080".parse(), Ok(socket));
/// assert_eq!(socket.port(), 8080);
/// assert_eq!(socket.is_ipv4(), true);
/// ```
pub enum SocketAddr<SA4: SocketAddressV4, SA6: SocketAddressV6> {
    /// An IPv4 socket address.
    V4(SocketAddrV4<SA4>),
    /// An IPv6 socket address.
    V6(SocketAddrV6<SA6>),
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> SocketAddr<SA4, SA6> {
    /// Creates a new socket address from an [IP address] and a port number.
    ///
    /// [IP address]: ../../std/net/enum.IpAddr.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    ///
    /// let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    /// assert_eq!(socket.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    /// assert_eq!(socket.port(), 8080);
    /// ```
    pub fn new(ip: IpAddr<SA4::IpAddress, SA6::IpAddress>, port: u16) -> SocketAddr<SA4, SA6> {
        match ip {
            IpAddr::V4(a) => SocketAddr::V4(SocketAddrV4::new(a, port)),
            IpAddr::V6(a) => SocketAddr::V6(SocketAddrV6::new(a, port, 0, 0)),
        }
    }

    /// Returns the IP address associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    ///
    /// let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    /// assert_eq!(socket.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    /// ```
    pub fn ip(&self) -> IpAddr<SA4::IpAddress, SA6::IpAddress> {
        match *self {
            SocketAddr::V4(ref a) => IpAddr::V4(*a.ip()),
            SocketAddr::V6(ref a) => IpAddr::V6(*a.ip()),
        }
    }

    /// Changes the IP address associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    ///
    /// let mut socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    /// socket.set_ip(IpAddr::V4(Ipv4Addr::new(10, 10, 0, 1)));
    /// assert_eq!(socket.ip(), IpAddr::V4(Ipv4Addr::new(10, 10, 0, 1)));
    /// ```
    pub fn set_ip(&mut self, new_ip: IpAddr<SA4::IpAddress, SA6::IpAddress>) {
        // `match (*self, new_ip)` would have us mutate a copy of self only to throw it away.
        match (self, new_ip) {
            (&mut SocketAddr::V4(ref mut a), IpAddr::V4(new_ip)) => a.set_ip(new_ip),
            (&mut SocketAddr::V6(ref mut a), IpAddr::V6(new_ip)) => a.set_ip(new_ip),
            (self_, new_ip) => *self_ = Self::new(new_ip, self_.port()),
        }
    }

    /// Returns the port number associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    ///
    /// let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    /// assert_eq!(socket.port(), 8080);
    /// ```
    pub fn port(&self) -> u16 {
        match *self {
            SocketAddr::V4(ref a) => a.port(),
            SocketAddr::V6(ref a) => a.port(),
        }
    }

    /// Changes the port number associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    ///
    /// let mut socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    /// socket.set_port(1025);
    /// assert_eq!(socket.port(), 1025);
    /// ```
    pub fn set_port(&mut self, new_port: u16) {
        match *self {
            SocketAddr::V4(ref mut a) => a.set_port(new_port),
            SocketAddr::V6(ref mut a) => a.set_port(new_port),
        }
    }

    /// Returns [`true`] if the [IP address] in this `SocketAddr` is an
    /// [IPv4 address], and [`false`] otherwise.
    ///
    /// [`true`]: ../../std/primitive.bool.html
    /// [`false`]: ../../std/primitive.bool.html
    /// [IP address]: ../../std/net/enum.IpAddr.html
    /// [IPv4 address]: ../../std/net/enum.IpAddr.html#variant.V4
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    ///
    /// let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    /// assert_eq!(socket.is_ipv4(), true);
    /// assert_eq!(socket.is_ipv6(), false);
    /// ```
    pub fn is_ipv4(&self) -> bool {
        matches!(*self, SocketAddr::V4(_))
    }

    /// Returns [`true`] if the [IP address] in this `SocketAddr` is an
    /// [IPv6 address], and [`false`] otherwise.
    ///
    /// [`true`]: ../../std/primitive.bool.html
    /// [`false`]: ../../std/primitive.bool.html
    /// [IP address]: ../../std/net/enum.IpAddr.html
    /// [IPv6 address]: ../../std/net/enum.IpAddr.html#variant.V6
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv6Addr, SocketAddr};
    ///
    /// let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 65535, 0, 1)), 8080);
    /// assert_eq!(socket.is_ipv4(), false);
    /// assert_eq!(socket.is_ipv6(), true);
    /// ```
    pub fn is_ipv6(&self) -> bool {
        matches!(*self, SocketAddr::V6(_))
    }
}

impl<
        SA4: SocketAddressV4,
        SA6: SocketAddressV6,
        I: Into<IpAddr<SA4::IpAddress, SA6::IpAddress>>,
    > From<(I, u16)> for SocketAddr<SA4, SA6>
{
    /// Converts a tuple struct (Into<[`IpAddr`]>, `u16`) into a [`SocketAddr`].
    ///
    /// This conversion creates a [`SocketAddr::V4`] for a [`IpAddr::V4`]
    /// and creates a [`SocketAddr::V6`] for a [`IpAddr::V6`].
    ///
    /// `u16` is treated as port of the newly created [`SocketAddr`].
    ///
    /// [`IpAddr`]: ../../std/net/enum.IpAddr.html
    /// [`IpAddr::V4`]: ../../std/net/enum.IpAddr.html#variant.V4
    /// [`IpAddr::V6`]: ../../std/net/enum.IpAddr.html#variant.V6
    /// [`SocketAddr`]: ../../std/net/enum.SocketAddr.html
    /// [`SocketAddr::V4`]: ../../std/net/enum.SocketAddr.html#variant.V4
    /// [`SocketAddr::V6`]: ../../std/net/enum.SocketAddr.html#variant.V6
    fn from(pieces: (I, u16)) -> SocketAddr<SA4, SA6> {
        SocketAddr::new(pieces.0.into(), pieces.1)
    }
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> From<SocketAddrV4<SA4>> for SocketAddr<SA4, SA6> {
    /// Converts a [`SocketAddrV4`] into a [`SocketAddr::V4`].
    ///
    /// [`SocketAddrV4`]: ../../std/net/struct.SocketAddrV4.html
    /// [`SocketAddr::V4`]: ../../std/net/enum.SocketAddr.html#variant.V4
    fn from(sock4: SocketAddrV4<SA4>) -> SocketAddr<SA4, SA6> {
        SocketAddr::V4(sock4)
    }
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> From<SocketAddrV6<SA6>> for SocketAddr<SA4, SA6> {
    /// Converts a [`SocketAddrV6`] into a [`SocketAddr::V6`].
    ///
    /// [`SocketAddrV6`]: ../../std/net/struct.SocketAddrV6.html
    /// [`SocketAddr::V6`]: ../../std/net/enum.SocketAddr.html#variant.V6
    fn from(sock6: SocketAddrV6<SA6>) -> SocketAddr<SA4, SA6> {
        SocketAddr::V6(sock6)
    }
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> Clone for SocketAddr<SA4, SA6> {
    fn clone(&self) -> Self {
        match self {
            SocketAddr::V4(a) => SocketAddr::V4(a.clone()),
            SocketAddr::V6(a) => SocketAddr::V6(a.clone()),
        }
    }
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> Copy for SocketAddr<SA4, SA6> {}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> fmt::Display for SocketAddr<SA4, SA6> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip(), self.port())
    }
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> fmt::Debug for SocketAddr<SA4, SA6> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> Eq for SocketAddr<SA4, SA6> {}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> PartialEq for SocketAddr<SA4, SA6> {
    fn eq(&self, other: &SocketAddr<SA4, SA6>) -> bool {
        match (self, other) {
            (SocketAddr::V4(s), SocketAddr::V4(o)) => s == o,
            (SocketAddr::V6(s), SocketAddr::V6(o)) => s == o,
            _ => false,
        }
    }
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> hash::Hash for SocketAddr<SA4, SA6> {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        let ip = self.ip();
        let port = self.port();
        (ip, port).hash(s)
    }
}

pub enum ToSocketAddrError {}

/// A trait for objects which can be converted or resolved to one or more
/// [`SocketAddr`] values.
///
/// This trait is used for generic address resolution when constructing network
/// objects. By default it is implemented for the following types:
///
///  * [`SocketAddr`]: [`to_socket_addrs`] is the identity function.
///
///  * [`SocketAddrV4`], [`SocketAddrV6`], `(`[`IpAddr`]`, `[`u16`]`)`,
///    `(`[`Ipv4Addr`]`, `[`u16`]`)`, `(`[`Ipv6Addr`]`, `[`u16`]`)`:
///    [`to_socket_addrs`] constructs a [`SocketAddr`] trivially.
///
///  * `(`[`&str`]`, `[`u16`]`)`: the string should be either a string representation
///    of an [`IpAddr`] address as expected by [`FromStr`] implementation or a host
///    name.
///
///  * [`&str`]: the string should be either a string representation of a
///    [`SocketAddr`] as expected by its [`FromStr`] implementation or a string like
///    `<host_name>:<port>` pair where `<port>` is a [`u16`] value.
pub trait ToSocketAddrs<SA4: SocketAddressV4, SA6: SocketAddressV6> {
    /// Returned iterator over socket addresses which this type may correspond
    /// to.
    type Iter: Iterator<Item = SocketAddr<SA4, SA6>>;

    /// Converts this object to an iterator of resolved `SocketAddr`s.
    ///
    /// The returned iterator may not actually yield any values depending on the
    /// outcome of any resolution performed.
    ///
    /// Note that this function may block the current thread while resolution is
    /// performed.
    fn to_socket_addrs(&self) -> Result<Self::Iter, ToSocketAddrError>;
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> ToSocketAddrs<SA4, SA6> for SocketAddr<SA4, SA6> {
    type Iter = option::IntoIter<SocketAddr<SA4, SA6>>;
    fn to_socket_addrs(&self) -> Result<option::IntoIter<SocketAddr<SA4, SA6>>, ToSocketAddrError> {
        Ok(Some(*self).into_iter())
    }
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> ToSocketAddrs<SA4, SA6> for SocketAddrV4<SA4> {
    type Iter = option::IntoIter<SocketAddr<SA4, SA6>>;
    fn to_socket_addrs(&self) -> Result<option::IntoIter<SocketAddr<SA4, SA6>>, ToSocketAddrError> {
        SocketAddr::V4(*self).to_socket_addrs()
    }
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> ToSocketAddrs<SA4, SA6> for SocketAddrV6<SA6> {
    type Iter = option::IntoIter<SocketAddr<SA4, SA6>>;
    fn to_socket_addrs(&self) -> Result<option::IntoIter<SocketAddr<SA4, SA6>>, ToSocketAddrError> {
        SocketAddr::V6(*self).to_socket_addrs()
    }
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> ToSocketAddrs<SA4, SA6>
    for (IpAddr<SA4::IpAddress, SA6::IpAddress>, u16)
{
    type Iter = option::IntoIter<SocketAddr<SA4, SA6>>;
    fn to_socket_addrs(&self) -> Result<option::IntoIter<SocketAddr<SA4, SA6>>, ToSocketAddrError> {
        let (ip, port) = *self;
        match ip {
            IpAddr::V4(ref a) => (*a, port).to_socket_addrs(),
            IpAddr::V6(ref a) => (*a, port).to_socket_addrs(),
        }
    }
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> ToSocketAddrs<SA4, SA6>
    for (Ipv4Addr<SA4::IpAddress>, u16)
{
    type Iter = option::IntoIter<SocketAddr<SA4, SA6>>;
    fn to_socket_addrs(&self) -> Result<option::IntoIter<SocketAddr<SA4, SA6>>, ToSocketAddrError> {
        let (ip, port) = *self;
        SocketAddrV4::new(ip, port).to_socket_addrs()
    }
}

impl<SA4: SocketAddressV4, SA6: SocketAddressV6> ToSocketAddrs<SA4, SA6>
    for (Ipv6Addr<SA6::IpAddress>, u16)
{
    type Iter = option::IntoIter<SocketAddr<SA4, SA6>>;
    fn to_socket_addrs(&self) -> Result<option::IntoIter<SocketAddr<SA4, SA6>>, ToSocketAddrError> {
        let (ip, port) = *self;
        SocketAddrV6::new(ip, port, 0, 0).to_socket_addrs()
    }
}

// fn resolve_socket_addr(lh: LookupHost) -> io::Result<vec::IntoIter<SocketAddr>> {
//     let p = lh.port();
//     let v: Vec<_> = lh
//         .map(|mut a| {
//             a.set_port(p);
//             a
//         })
//         .collect();
//     Ok(v.into_iter())
// }
//
// impl<SA4: SocketAddressV4, SA6: SocketAddressV6> ToSocketAddrs for (&str, u16) {
//     type Iter = vec::IntoIter<SocketAddr>;
//     fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
//         let (host, port) = *self;
//
//         // try to parse the host as a regular IP address first
//         if let Ok(addr) = host.parse::<Ipv4Addr>() {
//             let addr = SocketAddrV4::new(addr, port);
//             return Ok(vec![SocketAddr::V4(addr)].into_iter());
//         }
//         if let Ok(addr) = host.parse::<Ipv6Addr>() {
//             let addr = SocketAddrV6::new(addr, port, 0, 0);
//             return Ok(vec![SocketAddr::V6(addr)].into_iter());
//         }
//
//         resolve_socket_addr((host, port).try_into()?)
//     }
// }
//
// // accepts strings like 'localhost:12345'
// impl<SA4: SocketAddressV4, SA6: SocketAddressV6> ToSocketAddrs for str {
//     type Iter = vec::IntoIter<SocketAddr>;
//     fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
//         // try to parse as a regular SocketAddr first
//         if let Some(addr) = self.parse().ok() {
//             return Ok(vec![addr].into_iter());
//         }
//
//         resolve_socket_addr(self.try_into()?)
//     }
// }
//
// impl<'a, SA4: SocketAddressV4, SA6: SocketAddressV6> ToSocketAddrs for &'a [SocketAddr] {
//     type Iter = iter::Cloned<slice::Iter<'a, SocketAddr>>;
//
//     fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
//         Ok(self.iter().cloned())
//     }
// }
//
// impl<T: ToSocketAddrs + ?Sized> ToSocketAddrs for &T {
//     type Iter = T::Iter;
//     fn to_socket_addrs(&self) -> io::Result<T::Iter> {
//         (**self).to_socket_addrs()
//     }
/* } */
