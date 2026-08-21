use crate::{Ipv4Addr, Ipv4Address};
use core::cmp::Ordering;
use core::fmt;
use core::hash;

/// Describe the internal data structure behavior of `Ipv6Addr`.
///
/// You can implement this trait by yourself or use `ffi` for specific Platform.
///
/// # Examples
///
/// ```rust
/// use addr_hal::Ipv6Address;
///
/// #[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Ord)]
/// struct Ipv6AddrInner {
///     inner: [u16; 8],
/// }
///
/// impl Ipv6Address for Ipv6AddrInner {
///     const LOCALHOST: Self = Self {
///         inner: [0, 0, 0, 0, 0, 0, 0, 1],
///     };
///     const UNSPECIFIED: Self = Self {
///         inner: [0, 0, 0, 0, 0, 0, 0, 0],
///     };
///     fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Self {
///         Self {
///             inner: [a, b, c, d, e, f, g, h],
///         }
///     }
///
///     fn segments(&self) -> [u16; 8] {
///         self.inner.clone()
///     }
/// }
/// ```
pub trait Ipv6Address: Clone + Copy + PartialEq + PartialOrd + Eq + Ord {
    const LOCALHOST: Self;

    const UNSPECIFIED: Self;

    fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Self;

    fn segments(&self) -> [u16; 8];
}

/// Ipv6 address's multicast scope.
#[derive(Copy, PartialEq, Eq, Clone, Hash, Debug)]
pub enum Ipv6MulticastScope {
    InterfaceLocal,
    LinkLocal,
    RealmLocal,
    AdminLocal,
    SiteLocal,
    OrganizationLocal,
    Global,
}

/// An IPv6 address.
///
/// IPv6 addresses are defined as 128-bit integers in [IETF RFC 4291].
/// They are usually represented as eight 16-bit segments.
///
/// See [`IpAddr`] for a type encompassing both IPv4 and IPv6 addresses.
///
/// The size of an `Ipv6Addr` struct may vary depending on the target operating
/// system.
///
/// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
/// [`IpAddr`]: ../../std/net/enum.IpAddr.html
///
/// # Textual representation
///
/// `Ipv6Addr` provides a [`FromStr`] implementation. There are many ways to represent
/// an IPv6 address in text, but in general, each segments is written in hexadecimal
/// notation, and segments are separated by `:`. For more information, see
/// [IETF RFC 5952].
///
/// [`FromStr`]: ../../std/str/trait.FromStr.html
/// [IETF RFC 5952]: https://tools.ietf.org/html/rfc5952
///
/// # Examples
///
/// ```
/// use std::net::Ipv6Addr;
///
/// let localhost = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
/// assert_eq!("::1".parse(), Ok(localhost));
/// assert_eq!(localhost.is_loopback(), true);
/// ```
pub struct Ipv6Addr<IV6: Ipv6Address> {
    inner: IV6,
}

impl<IV6: Ipv6Address> Ipv6Addr<IV6> {
    /// Creates a new IPv6 address from eight 16-bit segments.
    ///
    /// The result will represent the IP address `a:b:c:d:e:f:g:h`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// let addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff);
    /// ```
    pub fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Ipv6Addr<IV6> {
        Ipv6Addr {
            inner: IV6::new(a, b, c, d, e, f, g, h),
        }
    }

    /// An IPv6 address representing localhost: `::1`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// let addr = Ipv6Addr::LOCALHOST;
    /// assert_eq!(addr, Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
    /// ```
    pub const LOCALHOST: Self = Ipv6Addr {
        inner: IV6::LOCALHOST,
    };

    /// An IPv6 address representing the unspecified address: `::`
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// let addr = Ipv6Addr::UNSPECIFIED;
    /// assert_eq!(addr, Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0));
    /// ```
    pub const UNSPECIFIED: Self = Ipv6Addr {
        inner: IV6::UNSPECIFIED,
    };

    /// Returns the eight 16-bit segments that make up this address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).segments(),
    ///            [0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff]);
    /// ```
    pub fn segments(&self) -> [u16; 8] {
        self.inner.segments()
    }

    /// Returns [`true`] for the special 'unspecified' address (::).
    ///
    /// This property is defined in [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_unspecified(), false);
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).is_unspecified(), true);
    /// ```
    pub fn is_unspecified(&self) -> bool {
        self.segments() == [0, 0, 0, 0, 0, 0, 0, 0]
    }

    /// Returns [`true`] if this is a loopback address (::1).
    ///
    /// This property is defined in [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_loopback(), false);
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0x1).is_loopback(), true);
    /// ```
    pub fn is_loopback(&self) -> bool {
        self.segments() == [0, 0, 0, 0, 0, 0, 0, 1]
    }

    /// Returns [`true`] if the address appears to be globally routable.
    ///
    /// The following return [`false`]:
    ///
    /// - the loopback address
    /// - link-local and unique local unicast addresses
    /// - interface-, link-, realm-, admin- and site-local multicast addresses
    ///
    /// [`true`]: ../../std/primitive.bool.html
    /// [`false`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_global(), true);
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0x1).is_global(), false);
    /// assert_eq!(Ipv6Addr::new(0, 0, 0x1c9, 0, 0, 0xafc8, 0, 0x1).is_global(), true);
    /// ```
    pub fn is_global(&self) -> bool {
        match self.multicast_scope() {
            Some(Ipv6MulticastScope::Global) => true,
            None => self.is_unicast_global(),
            _ => false,
        }
    }

    /// Returns [`true`] if this is a unique local address (`fc00::/7`).
    ///
    /// This property is defined in [IETF RFC 4193].
    ///
    /// [IETF RFC 4193]: https://tools.ietf.org/html/rfc4193
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_unique_local(), false);
    /// assert_eq!(Ipv6Addr::new(0xfc02, 0, 0, 0, 0, 0, 0, 0).is_unique_local(), true);
    /// ```
    pub fn is_unique_local(&self) -> bool {
        (self.segments()[0] & 0xfe00) == 0xfc00
    }

    /// Returns [`true`] if the address is a unicast link-local address (`fe80::/64`).
    ///
    /// A common mis-conception is to think that "unicast link-local addresses start with
    /// `fe80::`", but the [IETF RFC 4291] actually defines a stricter format for these addresses:
    ///
    /// ```no_rust
    /// |   10     |
    /// |  bits    |         54 bits         |          64 bits           |
    /// +----------+-------------------------+----------------------------+
    /// |1111111010|           0             |       interface ID         |
    /// +----------+-------------------------+----------------------------+
    /// ```
    ///
    /// This method validates the format defined in the RFC and won't recognize the following
    /// addresses such as `fe80:0:0:1::` or `fe81::` as unicast link-local addresses for example.
    /// If you need a less strict validation use [`is_unicast_link_local()`] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// let ip = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0);
    /// assert!(ip.is_unicast_link_local_strict());
    ///
    /// let ip = Ipv6Addr::new(0xfe80, 0, 0, 0, 0xffff, 0xffff, 0xffff, 0xffff);
    /// assert!(ip.is_unicast_link_local_strict());
    ///
    /// let ip = Ipv6Addr::new(0xfe80, 0, 0, 1, 0, 0, 0, 0);
    /// assert!(!ip.is_unicast_link_local_strict());
    /// assert!(ip.is_unicast_link_local());
    ///
    /// let ip = Ipv6Addr::new(0xfe81, 0, 0, 0, 0, 0, 0, 0);
    /// assert!(!ip.is_unicast_link_local_strict());
    /// assert!(ip.is_unicast_link_local());
    /// ```
    ///
    /// # See also
    ///
    /// - [IETF RFC 4291 section 2.5.6]
    /// - [RFC 4291 errata 4406]
    /// - [`is_unicast_link_local()`]
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    /// [IETF RFC 4291 section 2.5.6]: https://tools.ietf.org/html/rfc4291#section-2.5.6
    /// [`true`]: ../../std/primitive.bool.html
    /// [RFC 4291 errata 4406]: https://www.rfc-editor.org/errata/eid4406
    /// [`is_unicast_link_local()`]: ../../std/net/struct.Ipv6Addr.html#method.is_unicast_link_local
    ///
    pub fn is_unicast_link_local_strict(&self) -> bool {
        (self.segments()[0] & 0xffff) == 0xfe80
            && (self.segments()[1] & 0xffff) == 0
            && (self.segments()[2] & 0xffff) == 0
            && (self.segments()[3] & 0xffff) == 0
    }

    /// Returns [`true`] if the address is a unicast link-local address (`fe80::/10`).
    ///
    /// This method returns [`true`] for addresses in the range reserved by [RFC 4291 section 2.4],
    /// i.e. addresses with the following format:
    ///
    /// ```no_rust
    /// |   10     |
    /// |  bits    |         54 bits         |          64 bits           |
    /// +----------+-------------------------+----------------------------+
    /// |1111111010|    arbitratry value     |       interface ID         |
    /// +----------+-------------------------+----------------------------+
    /// ```
    ///
    /// As a result, this method consider addresses such as `fe80:0:0:1::` or `fe81::` to be
    /// unicast link-local addresses, whereas [`is_unicast_link_local_strict()`] does not. If you
    /// need a strict validation fully compliant with the RFC, use
    /// [`is_unicast_link_local_strict()`].
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// let ip = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0);
    /// assert!(ip.is_unicast_link_local());
    ///
    /// let ip = Ipv6Addr::new(0xfe80, 0, 0, 0, 0xffff, 0xffff, 0xffff, 0xffff);
    /// assert!(ip.is_unicast_link_local());
    ///
    /// let ip = Ipv6Addr::new(0xfe80, 0, 0, 1, 0, 0, 0, 0);
    /// assert!(ip.is_unicast_link_local());
    /// assert!(!ip.is_unicast_link_local_strict());
    ///
    /// let ip = Ipv6Addr::new(0xfe81, 0, 0, 0, 0, 0, 0, 0);
    /// assert!(ip.is_unicast_link_local());
    /// assert!(!ip.is_unicast_link_local_strict());
    /// ```
    ///
    /// # See also
    ///
    /// - [IETF RFC 4291 section 2.4]
    /// - [RFC 4291 errata 4406]
    ///
    /// [IETF RFC 4291 section 2.4]: https://tools.ietf.org/html/rfc4291#section-2.4
    /// [`true`]: ../../std/primitive.bool.html
    /// [RFC 4291 errata 4406]: https://www.rfc-editor.org/errata/eid4406
    /// [`is_unicast_link_local_strict()`]: ../../std/net/struct.Ipv6Addr.html#method.is_unicast_link_local_strict
    ///
    pub fn is_unicast_link_local(&self) -> bool {
        (self.segments()[0] & 0xffc0) == 0xfe80
    }

    /// Returns [`true`] if this is a deprecated unicast site-local address (fec0::/10). The
    /// unicast site-local address format is defined in [RFC 4291 section 2.5.7] as:
    ///
    /// ```no_rust
    /// |   10     |
    /// |  bits    |         54 bits         |         64 bits            |
    /// +----------+-------------------------+----------------------------+
    /// |1111111011|        subnet ID        |       interface ID         |
    /// +----------+-------------------------+----------------------------+
    /// ```
    ///
    /// [`true`]: ../../std/primitive.bool.html
    /// [RFC 4291 section 2.5.7]: https://tools.ietf.org/html/rfc4291#section-2.5.7
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(
    ///     Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_unicast_site_local(),
    ///     false
    /// );
    /// assert_eq!(Ipv6Addr::new(0xfec2, 0, 0, 0, 0, 0, 0, 0).is_unicast_site_local(), true);
    /// ```
    ///
    /// # Warning
    ///
    /// As per [RFC 3879], the whole `FEC0::/10` prefix is
    /// deprecated. New software must not support site-local
    /// addresses.
    ///
    /// [RFC 3879]: https://tools.ietf.org/html/rfc3879
    pub fn is_unicast_site_local(&self) -> bool {
        (self.segments()[0] & 0xffc0) == 0xfec0
    }

    /// Returns [`true`] if this is an address reserved for documentation
    /// (2001:db8::/32).
    ///
    /// This property is defined in [IETF RFC 3849].
    ///
    /// [IETF RFC 3849]: https://tools.ietf.org/html/rfc3849
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_documentation(), false);
    /// assert_eq!(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0).is_documentation(), true);
    /// ```
    pub fn is_documentation(&self) -> bool {
        (self.segments()[0] == 0x2001) && (self.segments()[1] == 0xdb8)
    }

    /// Returns [`true`] if the address is a globally routable unicast address.
    ///
    /// The following return false:
    ///
    /// - the loopback address
    /// - the link-local addresses
    /// - unique local addresses
    /// - the unspecified address
    /// - the address range reserved for documentation
    ///
    /// This method returns [`true`] for site-local addresses as per [RFC 4291 section 2.5.7]
    ///
    /// ```no_rust
    /// The special behavior of [the site-local unicast] prefix defined in [RFC3513] must no longer
    /// be supported in new implementations (i.e., new implementations must treat this prefix as
    /// Global Unicast).
    /// ```
    ///
    /// [`true`]: ../../std/primitive.bool.html
    /// [RFC 4291 section 2.5.7]: https://tools.ietf.org/html/rfc4291#section-2.5.7
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0).is_unicast_global(), false);
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_unicast_global(), true);
    /// ```
    pub fn is_unicast_global(&self) -> bool {
        !self.is_multicast()
            && !self.is_loopback()
            && !self.is_unicast_link_local()
            && !self.is_unique_local()
            && !self.is_unspecified()
            && !self.is_documentation()
    }

    /// Returns the address's multicast scope if the address is multicast.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::{Ipv6Addr, Ipv6MulticastScope};
    ///
    /// assert_eq!(
    ///     Ipv6Addr::new(0xff0e, 0, 0, 0, 0, 0, 0, 0).multicast_scope(),
    ///     Some(Ipv6MulticastScope::Global)
    /// );
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).multicast_scope(), None);
    /// ```
    pub fn multicast_scope(&self) -> Option<Ipv6MulticastScope> {
        if self.is_multicast() {
            match self.segments()[0] & 0x000f {
                1 => Some(Ipv6MulticastScope::InterfaceLocal),
                2 => Some(Ipv6MulticastScope::LinkLocal),
                3 => Some(Ipv6MulticastScope::RealmLocal),
                4 => Some(Ipv6MulticastScope::AdminLocal),
                5 => Some(Ipv6MulticastScope::SiteLocal),
                8 => Some(Ipv6MulticastScope::OrganizationLocal),
                14 => Some(Ipv6MulticastScope::Global),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Returns [`true`] if this is a multicast address (ff00::/8).
    ///
    /// This property is defined by [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0).is_multicast(), true);
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).is_multicast(), false);
    /// ```
    pub fn is_multicast(&self) -> bool {
        (self.segments()[0] & 0xff00) == 0xff00
    }

    /// Converts this address to an [IPv4 address]. Returns [`None`] if this address is
    /// neither IPv4-compatible or IPv4-mapped.
    ///
    /// ::a.b.c.d and ::ffff:a.b.c.d become a.b.c.d
    ///
    /// [IPv4 address]: ../../std/net/struct.Ipv4Addr.html
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0).to_ipv4(), None);
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff).to_ipv4(),
    ///            Some(Ipv4Addr::new(192, 10, 2, 255)));
    /// assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).to_ipv4(),
    ///            Some(Ipv4Addr::new(0, 0, 0, 1)));
    /// ```
    pub fn to_ipv4<IV4: Ipv4Address>(&self) -> Option<Ipv4Addr<IV4>> {
        match self.segments() {
            [0, 0, 0, 0, 0, f, g, h] if f == 0 || f == 0xffff => Some(Ipv4Addr::new(
                (g >> 8) as u8,
                g as u8,
                (h >> 8) as u8,
                h as u8,
            )),
            _ => None,
        }
    }

    /// Returns the sixteen eight-bit integers the IPv6 address consists of.
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// assert_eq!(Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0).octets(),
    ///            [255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    /// ```
    pub fn octets(&self) -> [u8; 16] {
        let segments = self.segments();
        let a = segments[0].to_be_bytes();
        let b = segments[1].to_be_bytes();
        let c = segments[2].to_be_bytes();
        let d = segments[3].to_be_bytes();
        let e = segments[4].to_be_bytes();
        let f = segments[5].to_be_bytes();
        let g = segments[6].to_be_bytes();
        let h = segments[7].to_be_bytes();

        [
            a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1], e[0], e[1], f[0], f[1], g[0], g[1],
            h[0], h[1],
        ]
    }
}

impl<IV6: Ipv6Address> fmt::Display for Ipv6Addr<IV6> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.segments() {
            // We need special cases for :: and ::1, otherwise they're formatted
            // as ::0.0.0.[01]
            [0, 0, 0, 0, 0, 0, 0, 0] => write!(fmt, "::"),
            [0, 0, 0, 0, 0, 0, 0, 1] => write!(fmt, "::1"),
            // Ipv4 Compatible address
            [0, 0, 0, 0, 0, 0, g, h] => write!(
                fmt,
                "::{}.{}.{}.{}",
                (g >> 8) as u8,
                g as u8,
                (h >> 8) as u8,
                h as u8
            ),
            // Ipv4-Mapped address
            [0, 0, 0, 0, 0, 0xffff, g, h] => write!(
                fmt,
                "::ffff:{}.{}.{}.{}",
                (g >> 8) as u8,
                g as u8,
                (h >> 8) as u8,
                h as u8
            ),
            _ => {
                fn find_zero_slice(segments: &[u16; 8]) -> (usize, usize) {
                    let mut longest_span_len = 0;
                    let mut longest_span_at = 0;
                    let mut cur_span_len = 0;
                    let mut cur_span_at = 0;

                    for i in 0..8 {
                        if segments[i] == 0 {
                            if cur_span_len == 0 {
                                cur_span_at = i;
                            }

                            cur_span_len += 1;

                            if cur_span_len > longest_span_len {
                                longest_span_len = cur_span_len;
                                longest_span_at = cur_span_at;
                            }
                        } else {
                            cur_span_len = 0;
                            cur_span_at = 0;
                        }
                    }

                    (longest_span_at, longest_span_len)
                }

                let (zeros_at, zeros_len) = find_zero_slice(&self.segments());

                if zeros_len > 1 {
                    fn fmt_subslice(segments: &[u16], fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                        if !segments.is_empty() {
                            write!(fmt, "{:x}", segments[0])?;
                            for &seg in &segments[1..] {
                                write!(fmt, ":{:x}", seg)?;
                            }
                        }
                        Ok(())
                    }

                    fmt_subslice(&self.segments()[..zeros_at], fmt)?;
                    fmt.write_str("::")?;
                    fmt_subslice(&self.segments()[zeros_at + zeros_len..], fmt)
                } else {
                    let &[a, b, c, d, e, f, g, h] = &self.segments();
                    write!(
                        fmt,
                        "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                        a, b, c, d, e, f, g, h
                    )
                }
            }
        }
    }
}

impl<IV6: Ipv6Address> fmt::Debug for Ipv6Addr<IV6> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl<IV6: Ipv6Address> Clone for Ipv6Addr<IV6> {
    fn clone(&self) -> Ipv6Addr<IV6> {
        Ipv6Addr {
            inner: self.inner.clone(),
        }
    }
}

impl<IV6: Ipv6Address> Copy for Ipv6Addr<IV6> {}

impl<IV6: Ipv6Address> PartialEq for Ipv6Addr<IV6> {
    fn eq(&self, other: &Ipv6Addr<IV6>) -> bool {
        self.inner == other.inner
    }
}

impl<IV6: Ipv6Address> Eq for Ipv6Addr<IV6> {}

impl<IV6: Ipv6Address> hash::Hash for Ipv6Addr<IV6> {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        self.inner.segments().hash(s)
    }
}

impl<IV6: Ipv6Address> PartialOrd for Ipv6Addr<IV6> {
    fn partial_cmp(&self, other: &Ipv6Addr<IV6>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<IV6: Ipv6Address> Ord for Ipv6Addr<IV6> {
    fn cmp(&self, other: &Ipv6Addr<IV6>) -> Ordering {
        self.segments().cmp(&other.segments())
    }
}

impl<IV6: Ipv6Address> From<Ipv6Addr<IV6>> for u128 {
    /// Convert an `Ipv6Addr` into a host byte order `u128`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// let addr = Ipv6Addr::new(
    ///     0x1020, 0x3040, 0x5060, 0x7080,
    ///     0x90A0, 0xB0C0, 0xD0E0, 0xF00D,
    /// );
    /// assert_eq!(0x102030405060708090A0B0C0D0E0F00D_u128, u128::from(addr));
    /// ```
    fn from(ip: Ipv6Addr<IV6>) -> u128 {
        let ip = ip.octets();
        u128::from_be_bytes(ip)
    }
}

impl<IV6: Ipv6Address> From<u128> for Ipv6Addr<IV6> {
    /// Convert a host byte order `u128` into an `Ipv6Addr`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    ///
    /// let addr = Ipv6Addr::from(0x102030405060708090A0B0C0D0E0F00D_u128);
    /// assert_eq!(
    ///     Ipv6Addr::new(
    ///         0x1020, 0x3040, 0x5060, 0x7080,
    ///         0x90A0, 0xB0C0, 0xD0E0, 0xF00D,
    ///     ),
    ///     addr);
    /// ```
    fn from(ip: u128) -> Ipv6Addr<IV6> {
        Ipv6Addr::from(ip.to_be_bytes())
    }
}

impl<IV6: Ipv6Address> From<[u8; 16]> for Ipv6Addr<IV6> {
    fn from(o: [u8; 16]) -> Ipv6Addr<IV6> {
        let a = u16::from_be_bytes([o[0], o[1]]);
        let b = u16::from_be_bytes([o[2], o[3]]);
        let c = u16::from_be_bytes([o[4], o[5]]);
        let d = u16::from_be_bytes([o[6], o[7]]);
        let e = u16::from_be_bytes([o[8], o[9]]);
        let f = u16::from_be_bytes([o[10], o[11]]);
        let g = u16::from_be_bytes([o[12], o[13]]);
        let h = u16::from_be_bytes([o[14], o[15]]);
        Ipv6Addr::new(a, b, c, d, e, f, g, h)
    }
}

impl<IV6: Ipv6Address> From<[u16; 8]> for Ipv6Addr<IV6> {
    fn from(segments: [u16; 8]) -> Ipv6Addr<IV6> {
        let [a, b, c, d, e, f, g, h] = segments;
        Ipv6Addr::new(a, b, c, d, e, f, g, h)
    }
}

