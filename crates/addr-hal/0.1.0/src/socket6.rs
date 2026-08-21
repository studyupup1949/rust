use crate::{Ipv6Addr, Ipv6Address};
use core::fmt;
use core::hash;

/// Describe the internal data structure behavior of `SocketAddrV6`.
///
/// You can implement this trait by yourself or use `ffi` for specific Platform.
pub trait SocketAddressV6: Clone + Copy {
    /// Ipv6Address inner type.
    type IpAddress: Ipv6Address;

    /// Creates a new IPv6 socket address from ip address and port
    ///
    /// The result will represent the Socket address ip:port.
    fn new(ip: Ipv6Addr<Self::IpAddress>, port: u16, flowinfo: u32, scope_id: u32) -> Self;

    /// Got ip address.
    fn ip(&self) -> &Ipv6Addr<Self::IpAddress>;

    /// Set ip address.
    fn set_ip(&mut self, ip: Ipv6Addr<Self::IpAddress>);

    /// Got port.
    fn port(&self) -> u16;

    /// Set port.
    fn set_port(&mut self, port: u16);

    /// Set flowinfo.
    fn set_flowinfo(&mut self, new_flowinfo: u32);

    /// Got flowinfo.
    fn flowinfo(&self) -> u32;

    /// Set scope id.
    fn set_scope_id(&mut self, new_scope_id: u32);

    /// Got scope id.
    fn scope_id(&self) -> u32;
}

/// An IPv6 socket address.
///
/// IPv6 socket addresses consist of an [Ipv6 address], a 16-bit port number, as well
/// as fields containing the traffic class, the flow label, and a scope identifier
/// (see [IETF RFC 2553, Section 3.3] for more details).
///
/// See [`SocketAddr`] for a type encompassing both IPv4 and IPv6 socket addresses.
///
/// The size of a `SocketAddrV6` struct may vary depending on the target operating
/// system.
///
/// [IETF RFC 2553, Section 3.3]: https://tools.ietf.org/html/rfc2553#section-3.3
/// [IPv6 address]: ../../std/net/struct.Ipv6Addr.html
/// [`SocketAddr`]: ../../std/net/enum.SocketAddr.html
///
/// # Examples
///
/// ```
/// use std::net::{Ipv6Addr, SocketAddrV6};
///
/// let socket = SocketAddrV6::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1), 8080, 0, 0);
///
/// assert_eq!("[2001:db8::1]:8080".parse(), Ok(socket));
/// assert_eq!(socket.ip(), &Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
/// assert_eq!(socket.port(), 8080);
/// ```
pub struct SocketAddrV6<SA6: SocketAddressV6> {
    inner: SA6,
}

impl<SA6: SocketAddressV6> SocketAddrV6<SA6> {
    /// Creates a new socket address from an [IPv6 address], a 16-bit port number,
    /// and the `flowinfo` and `scope_id` fields.
    ///
    /// For more information on the meaning and layout of the `flowinfo` and `scope_id`
    /// parameters, see [IETF RFC 2553, Section 3.3].
    ///
    /// [IETF RFC 2553, Section 3.3]: https://tools.ietf.org/html/rfc2553#section-3.3
    /// [IPv6 address]: ../../std/net/struct.Ipv6Addr.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV6, Ipv6Addr};
    ///
    /// let socket = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 8080, 0, 0);
    /// ```
    pub fn new(
        ip: Ipv6Addr<SA6::IpAddress>,
        port: u16,
        flowinfo: u32,
        scope_id: u32,
    ) -> SocketAddrV6<SA6> {
        SocketAddrV6 {
            inner: SA6::new(ip, port, flowinfo, scope_id),
        }
    }

    /// Returns the IP address associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV6, Ipv6Addr};
    ///
    /// let socket = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 8080, 0, 0);
    /// assert_eq!(socket.ip(), &Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
    /// ```
    pub fn ip(&self) -> &Ipv6Addr<SA6::IpAddress> {
        self.inner.ip()
    }

    /// Changes the IP address associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV6, Ipv6Addr};
    ///
    /// let mut socket = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 8080, 0, 0);
    /// socket.set_ip(Ipv6Addr::new(76, 45, 0, 0, 0, 0, 0, 0));
    /// assert_eq!(socket.ip(), &Ipv6Addr::new(76, 45, 0, 0, 0, 0, 0, 0));
    /// ```
    pub fn set_ip(&mut self, new_ip: Ipv6Addr<SA6::IpAddress>) {
        self.inner.set_ip(new_ip)
    }

    /// Returns the port number associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV6, Ipv6Addr};
    ///
    /// let socket = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 8080, 0, 0);
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
    /// use std::net::{SocketAddrV6, Ipv6Addr};
    ///
    /// let mut socket = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 8080, 0, 0);
    /// socket.set_port(4242);
    /// assert_eq!(socket.port(), 4242);
    /// ```
    pub fn set_port(&mut self, new_port: u16) {
        self.inner.set_port(new_port)
    }

    /// Returns the flow information associated with this address.
    ///
    /// This information corresponds to the `sin6_flowinfo` field in C's `netinet/in.h`,
    /// as specified in [IETF RFC 2553, Section 3.3].
    /// It combines information about the flow label and the traffic class as specified
    /// in [IETF RFC 2460], respectively [Section 6] and [Section 7].
    ///
    /// [IETF RFC 2553, Section 3.3]: https://tools.ietf.org/html/rfc2553#section-3.3
    /// [IETF RFC 2460]: https://tools.ietf.org/html/rfc2460
    /// [Section 6]: https://tools.ietf.org/html/rfc2460#section-6
    /// [Section 7]: https://tools.ietf.org/html/rfc2460#section-7
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV6, Ipv6Addr};
    ///
    /// let socket = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 8080, 10, 0);
    /// assert_eq!(socket.flowinfo(), 10);
    /// ```
    pub fn flowinfo(&self) -> u32 {
        self.inner.flowinfo()
    }

    /// Changes the flow information associated with this socket address.
    ///
    /// See the [`flowinfo`] method's documentation for more details.
    ///
    /// [`flowinfo`]: #method.flowinfo
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV6, Ipv6Addr};
    ///
    /// let mut socket = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 8080, 10, 0);
    /// socket.set_flowinfo(56);
    /// assert_eq!(socket.flowinfo(), 56);
    /// ```
    pub fn set_flowinfo(&mut self, new_flowinfo: u32) {
        self.inner.set_flowinfo(new_flowinfo)
    }

    /// Returns the scope ID associated with this address.
    ///
    /// This information corresponds to the `sin6_scope_id` field in C's `netinet/in.h`,
    /// as specified in [IETF RFC 2553, Section 3.3].
    ///
    /// [IETF RFC 2553, Section 3.3]: https://tools.ietf.org/html/rfc2553#section-3.3
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV6, Ipv6Addr};
    ///
    /// let socket = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 8080, 0, 78);
    /// assert_eq!(socket.scope_id(), 78);
    /// ```
    pub fn scope_id(&self) -> u32 {
        self.inner.scope_id()
    }

    /// Changes the scope ID associated with this socket address.
    ///
    /// See the [`scope_id`] method's documentation for more details.
    ///
    /// [`scope_id`]: #method.scope_id
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{SocketAddrV6, Ipv6Addr};
    ///
    /// let mut socket = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 8080, 0, 78);
    /// socket.set_scope_id(42);
    /// assert_eq!(socket.scope_id(), 42);
    /// ```
    pub fn set_scope_id(&mut self, new_scope_id: u32) {
        self.inner.set_scope_id(new_scope_id)
    }
}

impl<SA6: SocketAddressV6> Copy for SocketAddrV6<SA6> {}

impl<SA6: SocketAddressV6> Clone for SocketAddrV6<SA6> {
    fn clone(&self) -> SocketAddrV6<SA6> {
        SocketAddrV6 {
            inner: self.inner.clone(),
        }
    }
}

impl<SA6: SocketAddressV6> fmt::Display for SocketAddrV6<SA6> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]:{}", self.ip(), self.port())
    }
}

impl<SA6: SocketAddressV6> fmt::Debug for SocketAddrV6<SA6> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl<SA6: SocketAddressV6> Eq for SocketAddrV6<SA6> {}

impl<SA6: SocketAddressV6> PartialEq for SocketAddrV6<SA6> {
    fn eq(&self, other: &SocketAddrV6<SA6>) -> bool {
        let s_ip = self.ip();
        let o_ip = other.ip();

        let s_port = self.port();
        let o_port = other.port();

        let s_flowinfo = self.flowinfo();
        let o_flowinfo = other.flowinfo();

        let s_scope_id = self.scope_id();
        let o_scope_id = other.scope_id();
        (s_ip, s_port, s_flowinfo, s_scope_id).eq(&(o_ip, o_port, o_flowinfo, o_scope_id))
    }
}

impl<SA6: SocketAddressV6> hash::Hash for SocketAddrV6<SA6> {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        (
            self.inner.ip(),
            self.inner.port(),
            self.inner.flowinfo(),
            self.inner.scope_id(),
        )
            .hash(s)
    }
}

