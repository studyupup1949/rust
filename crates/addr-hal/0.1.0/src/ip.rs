use crate::{Ipv4Addr, Ipv4Address, Ipv6Addr, Ipv6Address};
use core::cmp::Ordering;
use core::fmt;
use core::hash;

/// An IP address, either IPv4 or IPv6.
///
/// This enum can contain either an [`Ipv4Addr`] or an [`Ipv6Addr`], see their
/// respective documentation for more details.
///
/// The size of an `IpAddr` instance may vary depending on the target operating
/// system.
///
/// [`Ipv4Addr`]: ../addr_hal/struct.Ipv4Addr.html
/// [`Ipv6Addr`]: ../addr_hal/struct.Ipv6Addr.html
///
/// # Examples
///
/// ```
/// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
///
/// let localhost_v4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
/// let localhost_v6 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
///
/// assert_eq!("127.0.0.1".parse(), Ok(localhost_v4));
/// assert_eq!("::1".parse(), Ok(localhost_v6));
///
/// assert_eq!(localhost_v4.is_ipv6(), false);
/// assert_eq!(localhost_v4.is_ipv4(), true);
/// ```
// #[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum IpAddr<IV4: Ipv4Address, IV6: Ipv6Address> {
    /// An IPv4 address.
    V4(Ipv4Addr<IV4>),
    /// An IPv6 address.
    V6(Ipv6Addr<IV6>),
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> IpAddr<IV4, IV6> {
    /// Returns [`true`] for the special 'unspecified' address.
    ///
    /// See the documentation for [`Ipv4Addr::is_unspecified`][IPv4] and
    /// [`Ipv6Addr::is_unspecified`][IPv6] for more details.
    ///
    /// [IPv4]: ../addr_hal/struct.Ipv4Addr.html#method.is_unspecified
    /// [IPv6]: ../addr_hal/struct.Ipv6Addr.html#method.is_unspecified
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)).is_unspecified(), true);
    /// assert_eq!(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)).is_unspecified(), true);
    /// ```
    pub fn is_unspecified(&self) -> bool {
        match self {
            IpAddr::V4(ip) => ip.is_unspecified(),
            IpAddr::V6(ip) => ip.is_unspecified(),
        }
    }

    /// Returns [`true`] if this is a loopback address.
    ///
    /// See the documentation for [`Ipv4Addr::is_loopback`][IPv4] and
    /// [`Ipv6Addr::is_loopback`][IPv6] for more details.
    ///
    /// [IPv4]: ../../std/net/struct.Ipv4Addr.html#method.is_loopback
    /// [IPv6]: ../../std/net/struct.Ipv6Addr.html#method.is_loopback
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).is_loopback(), true);
    /// assert_eq!(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0x1)).is_loopback(), true);
    /// ```
    pub fn is_loopback(&self) -> bool {
        match self {
            IpAddr::V4(ip) => ip.is_loopback(),
            IpAddr::V6(ip) => ip.is_loopback(),
        }
    }

    /// Returns [`true`] if the address appears to be globally routable.
    ///
    /// See the documentation for [`Ipv4Addr::is_global`][IPv4] and
    /// [`Ipv6Addr::is_global`][IPv6] for more details.
    ///
    /// [IPv4]: ../../std/net/struct.Ipv4Addr.html#method.is_global
    /// [IPv6]: ../../std/net/struct.Ipv6Addr.html#method.is_global
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(IpAddr::V4(Ipv4Addr::new(80, 9, 12, 3)).is_global(), true);
    /// assert_eq!(IpAddr::V6(Ipv6Addr::new(0, 0, 0x1c9, 0, 0, 0xafc8, 0, 0x1)).is_global(), true);
    /// ```
    pub fn is_global(&self) -> bool {
        match self {
            IpAddr::V4(ip) => ip.is_global(),
            IpAddr::V6(ip) => ip.is_global(),
        }
    }

    /// Returns [`true`] if this is a multicast address.
    ///
    /// See the documentation for [`Ipv4Addr::is_multicast`][IPv4] and
    /// [`Ipv6Addr::is_multicast`][IPv6] for more details.
    ///
    /// [IPv4]: ../../std/net/struct.Ipv4Addr.html#method.is_multicast
    /// [IPv6]: ../../std/net/struct.Ipv6Addr.html#method.is_multicast
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(IpAddr::V4(Ipv4Addr::new(224, 254, 0, 0)).is_multicast(), true);
    /// assert_eq!(IpAddr::V6(Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0)).is_multicast(), true);
    /// ```
    pub fn is_multicast(&self) -> bool {
        match self {
            IpAddr::V4(ip) => ip.is_multicast(),
            IpAddr::V6(ip) => ip.is_multicast(),
        }
    }

    /// Returns [`true`] if this address is in a range designated for documentation.
    ///
    /// See the documentation for [`Ipv4Addr::is_documentation`][IPv4] and
    /// [`Ipv6Addr::is_documentation`][IPv6] for more details.
    ///
    /// [IPv4]: ../../std/net/struct.Ipv4Addr.html#method.is_documentation
    /// [IPv6]: ../../std/net/struct.Ipv6Addr.html#method.is_documentation
    /// [`true`]: ../../std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(ip)]
    ///
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 6)).is_documentation(), true);
    /// assert_eq!(
    ///     IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0)).is_documentation(),
    ///     true
    /// );
    /// ```
    pub fn is_documentation(&self) -> bool {
        match self {
            IpAddr::V4(ip) => ip.is_documentation(),
            IpAddr::V6(ip) => ip.is_documentation(),
        }
    }

    /// Returns [`true`] if this address is an [IPv4 address], and [`false`] otherwise.
    ///
    /// [`true`]: ../../std/primitive.bool.html
    /// [`false`]: ../../std/primitive.bool.html
    /// [IPv4 address]: #variant.V4
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 6)).is_ipv4(), true);
    /// assert_eq!(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0)).is_ipv4(), false);
    /// ```
    pub fn is_ipv4(&self) -> bool {
        match self {
            IpAddr::V4(_) => true,
            IpAddr::V6(_) => false,
        }
    }

    /// Returns [`true`] if this address is an [IPv6 address], and [`false`] otherwise.
    ///
    /// [`true`]: ../../std/primitive.bool.html
    /// [`false`]: ../../std/primitive.bool.html
    /// [IPv6 address]: #variant.V6
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    ///
    /// assert_eq!(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 6)).is_ipv6(), false);
    /// assert_eq!(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0)).is_ipv6(), true);
    /// ```
    pub fn is_ipv6(&self) -> bool {
        match self {
            IpAddr::V4(_) => false,
            IpAddr::V6(_) => true,
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> Clone for IpAddr<IV4, IV6> {
    fn clone(&self) -> Self {
        match self {
            IpAddr::V4(ref a) => IpAddr::V4(a.clone()),
            IpAddr::V6(ref a) => IpAddr::V6(a.clone()),
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> Copy for IpAddr<IV4, IV6> {}

impl<IV4: Ipv4Address, IV6: Ipv6Address> fmt::Debug for IpAddr<IV4, IV6> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> fmt::Display for IpAddr<IV4, IV6> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpAddr::V4(ip) => ip.fmt(fmt),
            IpAddr::V6(ip) => ip.fmt(fmt),
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> From<Ipv4Addr<IV4>> for IpAddr<IV4, IV6> {
    fn from(ipv4: Ipv4Addr<IV4>) -> IpAddr<IV4, IV6> {
        IpAddr::V4(ipv4)
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> From<Ipv6Addr<IV6>> for IpAddr<IV4, IV6> {
    fn from(ipv6: Ipv6Addr<IV6>) -> IpAddr<IV4, IV6> {
        IpAddr::V6(ipv6)
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> Eq for IpAddr<IV4, IV6> {}

impl<IV4: Ipv4Address, IV6: Ipv6Address> PartialEq<IpAddr<IV4, IV6>> for IpAddr<IV4, IV6> {
    fn eq(&self, other: &IpAddr<IV4, IV6>) -> bool {
        match self {
            IpAddr::V4(v4) => v4 == other,
            IpAddr::V6(v6) => v6 == other,
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> PartialEq<Ipv4Addr<IV4>> for IpAddr<IV4, IV6> {
    fn eq(&self, other: &Ipv4Addr<IV4>) -> bool {
        match self {
            IpAddr::V4(v4) => v4 == other,
            IpAddr::V6(_) => false,
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> PartialEq<IpAddr<IV4, IV6>> for Ipv4Addr<IV4> {
    fn eq(&self, other: &IpAddr<IV4, IV6>) -> bool {
        match other {
            IpAddr::V4(v4) => self == v4,
            IpAddr::V6(_) => false,
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> PartialEq<Ipv6Addr<IV6>> for IpAddr<IV4, IV6> {
    fn eq(&self, other: &Ipv6Addr<IV6>) -> bool {
        match self {
            IpAddr::V4(_) => false,
            IpAddr::V6(v6) => v6 == other,
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> PartialEq<IpAddr<IV4, IV6>> for Ipv6Addr<IV6> {
    fn eq(&self, other: &IpAddr<IV4, IV6>) -> bool {
        match other {
            IpAddr::V4(_) => false,
            IpAddr::V6(v6) => self == v6,
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> hash::Hash for IpAddr<IV4, IV6> {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        match self {
            IpAddr::V4(ref a) => a.hash(s),
            IpAddr::V6(ref a) => a.hash(s),
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> Ord for IpAddr<IV4, IV6> {
    fn cmp(&self, other: &IpAddr<IV4, IV6>) -> Ordering {
        match (self, other) {
            (IpAddr::V4(s_v4), IpAddr::V4(o_v4)) => s_v4.cmp(o_v4),
            (IpAddr::V6(s_v6), IpAddr::V6(o_v6)) => s_v6.cmp(o_v6),
            _ => Ordering::Equal,
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> PartialOrd<IpAddr<IV4, IV6>> for IpAddr<IV4, IV6> {
    fn partial_cmp(&self, other: &IpAddr<IV4, IV6>) -> Option<Ordering> {
        match self {
            IpAddr::V4(v4) => v4.partial_cmp(other),
            IpAddr::V6(v6) => v6.partial_cmp(other),
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> PartialOrd<Ipv4Addr<IV4>> for IpAddr<IV4, IV6> {
    fn partial_cmp(&self, other: &Ipv4Addr<IV4>) -> Option<Ordering> {
        match self {
            IpAddr::V4(v4) => v4.partial_cmp(other),
            IpAddr::V6(_) => Some(Ordering::Greater),
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> PartialOrd<IpAddr<IV4, IV6>> for Ipv4Addr<IV4> {
    fn partial_cmp(&self, other: &IpAddr<IV4, IV6>) -> Option<Ordering> {
        match other {
            IpAddr::V4(v4) => self.partial_cmp(v4),
            IpAddr::V6(_) => Some(Ordering::Less),
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> PartialOrd<Ipv6Addr<IV6>> for IpAddr<IV4, IV6> {
    fn partial_cmp(&self, other: &Ipv6Addr<IV6>) -> Option<Ordering> {
        match self {
            IpAddr::V4(_) => Some(Ordering::Less),
            IpAddr::V6(v6) => v6.partial_cmp(other),
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> PartialOrd<IpAddr<IV4, IV6>> for Ipv6Addr<IV6> {
    fn partial_cmp(&self, other: &IpAddr<IV4, IV6>) -> Option<Ordering> {
        match other {
            IpAddr::V4(_) => Some(Ordering::Greater),
            IpAddr::V6(v6) => self.partial_cmp(v6),
        }
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> From<[u8; 4]> for IpAddr<IV4, IV6> {
    /// Creates an `IpAddr::V4` from a four element byte array.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv4Addr};
    ///
    /// let addr = IpAddr::from([13u8, 12u8, 11u8, 10u8]);
    /// assert_eq!(IpAddr::V4(Ipv4Addr::new(13, 12, 11, 10)), addr);
    /// ```
    fn from(octets: [u8; 4]) -> IpAddr<IV4, IV6> {
        IpAddr::V4(Ipv4Addr::from(octets))
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> From<[u8; 16]> for IpAddr<IV4, IV6> {
    /// Creates an `IpAddr::V6` from a sixteen element byte array.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv6Addr};
    ///
    /// let addr = IpAddr::from([
    ///     25u8, 24u8, 23u8, 22u8, 21u8, 20u8, 19u8, 18u8,
    ///     17u8, 16u8, 15u8, 14u8, 13u8, 12u8, 11u8, 10u8,
    /// ]);
    /// assert_eq!(
    ///     IpAddr::V6(Ipv6Addr::new(
    ///         0x1918, 0x1716,
    ///         0x1514, 0x1312,
    ///         0x1110, 0x0f0e,
    ///         0x0d0c, 0x0b0a
    ///     )),
    ///     addr
    /// );
    /// ```
    fn from(octets: [u8; 16]) -> IpAddr<IV4, IV6> {
        IpAddr::V6(Ipv6Addr::from(octets))
    }
}

impl<IV4: Ipv4Address, IV6: Ipv6Address> From<[u16; 8]> for IpAddr<IV4, IV6> {
    /// Creates an `IpAddr::V6` from an eight element 16-bit array.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{IpAddr, Ipv6Addr};
    ///
    /// let addr = IpAddr::from([
    ///     525u16, 524u16, 523u16, 522u16,
    ///     521u16, 520u16, 519u16, 518u16,
    /// ]);
    /// assert_eq!(
    ///     IpAddr::V6(Ipv6Addr::new(
    ///         0x20d, 0x20c,
    ///         0x20b, 0x20a,
    ///         0x209, 0x208,
    ///         0x207, 0x206
    ///     )),
    ///     addr
    /// );
    /// ```
    fn from(segments: [u16; 8]) -> IpAddr<IV4, IV6> {
        IpAddr::V6(Ipv6Addr::from(segments))
    }
}

