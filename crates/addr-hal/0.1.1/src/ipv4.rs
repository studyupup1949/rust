use crate::{Ipv6Addr, Ipv6Address};
use core::cmp::Ordering;
use core::fmt;
use core::hash;

/// Describe the internal data structure behavior of `Ipv4Addr`.
///
/// You can implement this trait by yourself or use `ffi` for specific Platform.
///
/// # Examples
///
/// ```rust
///
/// use addr_hal::Ipv4Address;
/// use addr_hal::Ipv4Addr;
///
/// #[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Ord)]
/// struct Ipv4AddrInner {
///     inner: [u8; 4],
/// }
///
/// impl Ipv4Address for Ipv4AddrInner {
///     const LOCALHOST: Self = Self {
///         inner: [127, 0, 0, 1],
///     };
///     const UNSPECIFIED: Self = Self {
///         inner: [0, 0, 0, 0],
///     };
///
///     const BROADCAST: Self = Self {
///         inner: [255, 255, 255, 255],
///     };
///
///     fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
///         Self {
///             inner: [a, b, c, d],
///         }
///     }
///
///     fn octets(&self) -> [u8; 4] {
///         self.inner.clone()
///     }
/// }
///
/// let ip: Ipv4Addr<Ipv4AddrInner> = "127.0.0.1".parse().unwrap();
/// assert_eq!("127.0.0.1".parse(), Ok(ip));
/// ```
pub trait Ipv4Address: Clone + Copy + PartialEq + Ord {
    /// An IPv4 address with the address pointing to localhost, usually is `127.0.0.1`.
    const LOCALHOST: Self;

    /// An IPv4 address with the address pointing to unspecified, usually is `0.0.0.0`.
    const UNSPECIFIED: Self;

    /// An IPv4 address with the address pointing to broadcast, usually is `255.255.255.255`.
    const BROADCAST: Self;

    /// Creates a new IPv4 address from four eight-bit octets.
    ///
    /// The result will represent the IP address `a`.`b`.`c`.`d`.
    fn new(a: u8, b: u8, c: u8, d: u8) -> Self;

    /// Returns the four eight-bit integers that make up this address.
    fn octets(&self) -> [u8; 4];
}

/// An IPv4 address.
///
/// IPv4 addresses are defined as 32-bit integers in [IETF RFC 791].
/// They are usually represented as four octets.
///
/// See [`IpAddr`] for a type encompassing both IPv4 and IPv6 addresses.
///
/// The size of an `Ipv4Addr` struct may vary depending on the target operating
/// system.
///
/// [IETF RFC 791]: https://tools.ietf.org/html/rfc791
/// [`IpAddr`]: ../addr_hal/enum.IpAddr.html
///
/// # Textual representation
///
/// `Ipv4Addr` provides a [`FromStr`] implementation. The four octets are in decimal
/// notation, divided by `.` (this is called "dot-decimal notation").
///
/// [`FromStr`]: https://doc.rust-lang.org/core/str/trait.FromStr.html
///
/// # Examples
///
/// ```
/// use addr_hal::Ipv4Addr;
/// use addr_mock::Ipv4AddrInner;
///
/// let localhost = Ipv4Addr::<Ipv4AddrInner>::new(127, 0, 0, 1);
/// assert_eq!("127.0.0.1".parse(), Ok(localhost));
/// //assert_eq!(localhost.is_loopback(), true);
/// ```
pub struct Ipv4Addr<IV4: Ipv4Address> {
    inner: IV4,
}

impl<IV4: Ipv4Address> Ipv4Addr<IV4> {
    /// An IPv4 address with the address pointing to localhost: 127.0.0.1.
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// let addr: Ipv4Addr<Ipv4AddrInner> = Ipv4Addr::LOCALHOST;
    /// assert_eq!(addr, Ipv4Addr::new(127, 0, 0, 1));
    /// ```
    pub const LOCALHOST: Self = Self {
        inner: IV4::LOCALHOST,
    };

    /// An IPv4 address representing an unspecified address: 0.0.0.0
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// let addr: Ipv4Addr<Ipv4AddrInner> = Ipv4Addr::UNSPECIFIED;
    /// assert_eq!(addr, Ipv4Addr::new(0, 0, 0, 0));
    /// ```
    pub const UNSPECIFIED: Self = Self {
        inner: IV4::UNSPECIFIED,
    };

    /// An IPv4 address representing the broadcast address: 255.255.255.255
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// let addr: Ipv4Addr<Ipv4AddrInner> = Ipv4Addr::BROADCAST;
    /// assert_eq!(addr, Ipv4Addr::new(255, 255, 255, 255));
    /// ```
    pub const BROADCAST: Self = Self {
        inner: IV4::BROADCAST,
    };

    /// Creates a new IPv4 address from four eight-bit octets.
    ///
    /// The result will represent the IP address `a`.`b`.`c`.`d`.
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// let addr: Ipv4Addr<Ipv4AddrInner> = Ipv4Addr::new(127, 0, 0, 1);
    /// ```
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4Addr {
            inner: IV4::new(a, b, c, d),
        }
    }

    /// Returns the four eight-bit integers that make up this address.
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// let addr: Ipv4Addr<Ipv4AddrInner> = Ipv4Addr::new(127, 0, 0, 1);
    /// assert_eq!(addr.octets(), [127, 0, 0, 1]);
    /// ```
    pub fn octets(&self) -> [u8; 4] {
        self.inner.octets()
    }

    /// Returns [`true`] if this address part of the `198.18.0.0/15` range, which is reserved for
    /// network devices benchmarking. This range is defined in [IETF RFC 2544] as `192.18.0.0`
    /// through `198.19.255.255` but [errata 423] corrects it to `198.18.0.0/15`.
    ///
    /// [IETF RFC 2544]: https://tools.ietf.org/html/rfc2544
    /// [errata 423]: https://www.rfc-editor.org/errata/eid423
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    ///
    /// assert_eq!(Ipv4::new(198, 17, 255, 255).is_benchmarking(), false);
    /// assert_eq!(Ipv4::new(198, 18, 0, 0).is_benchmarking(), true);
    /// assert_eq!(Ipv4::new(198, 19, 255, 255).is_benchmarking(), true);
    /// assert_eq!(Ipv4::new(198, 20, 0, 0).is_benchmarking(), false);
    /// ```
    pub fn is_benchmarking(&self) -> bool {
        self.octets()[0] == 198 && (self.octets()[1] & 0xfe) == 18
    }

    /// Returns [`true`] if this is a broadcast address (255.255.255.255).
    ///
    /// A broadcast address has all octets set to 255 as defined in [IETF RFC 919].
    ///
    /// [IETF RFC 919]: https://tools.ietf.org/html/rfc919
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    ///
    /// assert_eq!(Ipv4::new(255, 255, 255, 255).is_broadcast(), true);
    /// assert_eq!(Ipv4::new(236, 168, 10, 65).is_broadcast(), false);
    /// ```
    pub fn is_broadcast(&self) -> bool {
        self == &Self::BROADCAST
    }

    /// Returns [`true`] if this address is in a range designated for documentation.
    ///
    /// This is defined in [IETF RFC 5737]:
    ///
    /// - 192.0.2.0/24 (TEST-NET-1)
    /// - 198.51.100.0/24 (TEST-NET-2)
    /// - 203.0.113.0/24 (TEST-NET-3)
    ///
    /// [IETF RFC 5737]: https://tools.ietf.org/html/rfc5737
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    ///
    /// assert_eq!(Ipv4::new(192, 0, 2, 255).is_documentation(), true);
    /// assert_eq!(Ipv4::new(198, 51, 100, 65).is_documentation(), true);
    /// assert_eq!(Ipv4::new(203, 0, 113, 6).is_documentation(), true);
    /// assert_eq!(Ipv4::new(193, 34, 17, 19).is_documentation(), false);
    /// ```
    pub fn is_documentation(&self) -> bool {
        match self.octets() {
            [192, 0, 2, _] => true,
            [198, 51, 100, _] => true,
            [203, 0, 113, _] => true,
            _ => false,
        }
    }

    /// Returns [`true`] if the address appears to be globally routable.
    /// See [iana-ipv4-special-registry][ipv4-sr].
    ///
    /// The following return false:
    ///
    /// - private addresses (see [`is_private()`](#method.is_private))
    /// - the loopback address (see [`is_loopback()`](#method.is_loopback))
    /// - the link-local address (see [`is_link_local()`](#method.is_link_local))
    /// - the broadcast address (see [`is_broadcast()`](#method.is_broadcast))
    /// - addresses used for documentation (see [`is_documentation()`](#method.is_documentation))
    /// - the unspecified address (see [`is_unspecified()`](#method.is_unspecified)), and the whole
    ///   0.0.0.0/8 block
    /// - addresses reserved for future protocols (see
    /// [`is_ietf_protocol_assignment()`](#method.is_ietf_protocol_assignment), except
    /// `192.0.0.9/32` and `192.0.0.10/32` which are globally routable
    /// - addresses reserved for future use (see [`is_reserved()`](#method.is_reserved)
    /// - addresses reserved for networking devices benchmarking (see
    /// [`is_benchmarking`](#method.is_benchmarking))
    ///
    /// [ipv4-sr]: https://www.iana.org/assignments/iana-ipv4-special-registry/iana-ipv4-special-registry.xhtml
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    ///
    /// // private addresses are not global
    /// assert_eq!(Ipv4::new(10, 254, 0, 0).is_global(), false);
    /// assert_eq!(Ipv4::new(192, 168, 10, 65).is_global(), false);
    /// assert_eq!(Ipv4::new(172, 16, 10, 65).is_global(), false);
    ///
    /// // the 0.0.0.0/8 block is not global
    /// assert_eq!(Ipv4::new(0, 1, 2, 3).is_global(), false);
    /// // in particular, the unspecified address is not global
    /// assert_eq!(Ipv4::new(0, 0, 0, 0).is_global(), false);
    ///
    /// // the loopback address is not global
    /// assert_eq!(Ipv4::new(127, 0, 0, 1).is_global(), false);
    ///
    /// // link local addresses are not global
    /// assert_eq!(Ipv4::new(169, 254, 45, 1).is_global(), false);
    ///
    /// // the broadcast address is not global
    /// assert_eq!(Ipv4::new(255, 255, 255, 255).is_global(), false);
    ///
    /// // the address space designated for documentation is not global
    /// assert_eq!(Ipv4::new(192, 0, 2, 255).is_global(), false);
    /// assert_eq!(Ipv4::new(198, 51, 100, 65).is_global(), false);
    /// assert_eq!(Ipv4::new(203, 0, 113, 6).is_global(), false);
    ///
    /// // shared addresses are not global
    /// assert_eq!(Ipv4::new(100, 100, 0, 0).is_global(), false);
    ///
    /// // addresses reserved for protocol assignment are not global
    /// assert_eq!(Ipv4::new(192, 0, 0, 0).is_global(), false);
    /// assert_eq!(Ipv4::new(192, 0, 0, 255).is_global(), false);
    ///
    /// // addresses reserved for future use are not global
    /// assert_eq!(Ipv4::new(250, 10, 20, 30).is_global(), false);
    ///
    /// // addresses reserved for network devices benchmarking are not global
    /// assert_eq!(Ipv4::new(198, 18, 0, 0).is_global(), false);
    ///
    /// // All the other addresses are global
    /// assert_eq!(Ipv4::new(1, 1, 1, 1).is_global(), true);
    /// assert_eq!(Ipv4::new(80, 9, 12, 3).is_global(), true);
    /// ```
    pub fn is_global(&self) -> bool {
        match self.octets() {
            [192, 0, 0, 9] | [192, 0, 0, 10] => return true,
            _ => {
                return {
                    !self.is_private()
                        && !self.is_loopback()
                        && !self.is_link_local()
                        && !self.is_broadcast()
                        && !self.is_documentation()
                        && !self.is_shared()
                        && !self.is_ietf_protocol_assignment()
                        && !self.is_reserved()
                        && !self.is_benchmarking()
                        && self.octets()[0] != 0
                }
            }
        };
    }

    /// Returns [`true`] if this address is part of `192.0.0.0/24`, which is reserved to
    /// IANA for IETF protocol assignments, as documented in [IETF RFC 6890].
    ///
    /// Note that parts of this block are in use:
    ///
    /// - `192.0.0.8/32` is the "IPv4 dummy address" (see [IETF RFC 7600])
    /// - `192.0.0.9/32` is the "Port Control Protocol Anycast" (see [IETF RFC 7723])
    /// - `192.0.0.10/32` is used for NAT traversal (see [IETF RFC 8155])
    ///
    /// [IETF RFC 6890]: https://tools.ietf.org/html/rfc6890
    /// [IETF RFC 7600]: https://tools.ietf.org/html/rfc7600
    /// [IETF RFC 7723]: https://tools.ietf.org/html/rfc7723
    /// [IETF RFC 8155]: https://tools.ietf.org/html/rfc8155
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    ///
    /// assert_eq!(Ipv4::new(192, 0, 0, 0).is_ietf_protocol_assignment(), true);
    /// assert_eq!(Ipv4::new(192, 0, 0, 8).is_ietf_protocol_assignment(), true);
    /// assert_eq!(Ipv4::new(192, 0, 0, 9).is_ietf_protocol_assignment(), true);
    /// assert_eq!(Ipv4::new(192, 0, 0, 255).is_ietf_protocol_assignment(), true);
    /// assert_eq!(Ipv4::new(192, 0, 1, 0).is_ietf_protocol_assignment(), false);
    /// assert_eq!(Ipv4::new(191, 255, 255, 255).is_ietf_protocol_assignment(), false);
    /// ```
    pub fn is_ietf_protocol_assignment(&self) -> bool {
        self.octets()[0] == 192 && self.octets()[1] == 0 && self.octets()[2] == 0
    }

    /// Returns [`true`] if the address is link-local (169.254.0.0/16).
    ///
    /// This property is defined by [IETF RFC 3927].
    ///
    /// [IETF RFC 3927]: https://tools.ietf.org/html/rfc3927
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    ///
    /// assert_eq!(Ipv4::new(169, 254, 0, 0).is_link_local(), true);
    /// assert_eq!(Ipv4::new(169, 254, 10, 65).is_link_local(), true);
    /// assert_eq!(Ipv4::new(16, 89, 10, 65).is_link_local(), false);
    /// ```
    pub fn is_link_local(&self) -> bool {
        match self.octets() {
            [169, 254, ..] => true,
            _ => false,
        }
    }

    /// Returns [`true`] if this is a loopback address (127.0.0.0/8).
    ///
    /// This property is defined by [IETF RFC 1122].
    ///
    /// [IETF RFC 1122]: https://tools.ietf.org/html/rfc1122
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    ///
    /// assert_eq!(Ipv4::new(127, 0, 0, 1).is_loopback(), true);
    /// assert_eq!(Ipv4::new(45, 22, 13, 197).is_loopback(), false);
    /// ```
    pub fn is_loopback(&self) -> bool {
        self.octets()[0] == 127
    }

    /// Returns [`true`] if this is a multicast address (224.0.0.0/4).
    ///
    /// Multicast addresses have a most significant octet between 224 and 239,
    /// and is defined by [IETF RFC 5771].
    ///
    /// [IETF RFC 5771]: https://tools.ietf.org/html/rfc5771
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    ///
    /// assert_eq!(Ipv4::new(224, 254, 0, 0).is_multicast(), true);
    /// assert_eq!(Ipv4::new(236, 168, 10, 65).is_multicast(), true);
    /// assert_eq!(Ipv4::new(172, 16, 10, 65).is_multicast(), false);
    /// ```
    pub fn is_multicast(&self) -> bool {
        self.octets()[0] >= 224 && self.octets()[0] <= 239
    }

    /// Returns [`true`] if this is a private address.
    ///
    /// The private address ranges are defined in [IETF RFC 1918] and include:
    ///
    ///  - 10.0.0.0/8
    ///  - 172.16.0.0/12
    ///  - 192.168.0.0/16
    ///
    /// [IETF RFC 1918]: https://tools.ietf.org/html/rfc1918
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    ///
    /// assert_eq!(Ipv4::new(10, 0, 0, 1).is_private(), true);
    /// assert_eq!(Ipv4::new(10, 10, 10, 10).is_private(), true);
    /// assert_eq!(Ipv4::new(172, 16, 10, 10).is_private(), true);
    /// assert_eq!(Ipv4::new(172, 29, 45, 14).is_private(), true);
    /// assert_eq!(Ipv4::new(172, 32, 0, 2).is_private(), false);
    /// assert_eq!(Ipv4::new(192, 168, 0, 2).is_private(), true);
    /// assert_eq!(Ipv4::new(192, 169, 0, 2).is_private(), false);
    /// ```
    pub fn is_private(&self) -> bool {
        match self.octets() {
            [10, ..] => true,
            [172, b, ..] if b >= 16 && b <= 31 => true,
            [192, 168, ..] => true,
            _ => false,
        }
    }

    /// Returns [`true`] if this address is reserved by IANA for future use. [IETF RFC 1112]
    /// defines the block of reserved addresses as `240.0.0.0/4`. This range normally includes the
    /// broadcast address `255.255.255.255`, but this implementation explicitely excludes it, since
    /// it is obviously not reserved for future use.
    ///
    /// [IETF RFC 1112]: https://tools.ietf.org/html/rfc1112
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Warning
    ///
    /// As IANA assigns new addresses, this method will be
    /// updated. This may result in non-reserved addresses being
    /// treated as reserved in code that relies on an outdated version
    /// of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    ///
    /// assert_eq!(Ipv4::new(240, 0, 0, 0).is_reserved(), true);
    /// assert_eq!(Ipv4::new(255, 255, 255, 254).is_reserved(), true);
    ///
    /// assert_eq!(Ipv4::new(239, 255, 255, 255).is_reserved(), false);
    /// // The broadcast address is not considered as reserved for future use by this implementation
    /// assert_eq!(Ipv4::new(255, 255, 255, 255).is_reserved(), false);
    /// ```
    pub fn is_reserved(&self) -> bool {
        self.octets()[0] & 240 == 240 && !self.is_broadcast()
    }

    /// Returns [`true`] if this address is part of the Shared Address Space defined in
    /// [IETF RFC 6598] (`100.64.0.0/10`).
    ///
    /// [IETF RFC 6598]: https://tools.ietf.org/html/rfc6598
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    ///
    /// assert_eq!(Ipv4::new(100, 64, 0, 0).is_shared(), true);
    /// assert_eq!(Ipv4::new(100, 127, 255, 255).is_shared(), true);
    /// assert_eq!(Ipv4::new(100, 128, 0, 0).is_shared(), false);
    /// ```
    pub fn is_shared(&self) -> bool {
        self.octets()[0] == 100 && (self.octets()[1] & 0b1100_0000 == 0b0100_0000)
    }

    /// Returns [`true`] for the special 'unspecified' address (0.0.0.0).
    ///
    /// This property is defined in _UNIX Network Programming, Second Edition_,
    /// W. Richard Stevens, p. 891; see also [ip7].
    ///
    /// [ip7]: http://man7.org/linux/man-pages/man7/ip.7.html
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    ///
    /// assert_eq!(Ipv4::new(0, 0, 0, 0).is_unspecified(), true);
    /// assert_eq!(Ipv4::new(45, 22, 13, 197).is_unspecified(), false);
    /// ```
    pub fn is_unspecified(&self) -> bool {
        self == &Self::UNSPECIFIED
    }

    /// Converts this address to an IPv4-compatible [IPv6 address].
    ///
    /// a.b.c.d becomes ::a.b.c.d
    ///
    /// [IPv6 address]: ../addr_hal/struct.Ipv6Addr.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// use addr_hal::Ipv6Addr;
    /// use addr_mock::Ipv6AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    /// type Ipv6 = Ipv6Addr<Ipv6AddrInner>;
    ///
    /// assert_eq!(
    ///     Ipv4::new(192, 0, 2, 255).to_ipv6_compatible(),
    ///     Ipv6::new(0, 0, 0, 0, 0, 0, 49152, 767)
    /// );
    /// ```
    pub fn to_ipv6_compatible<IV6: Ipv6Address>(&self) -> Ipv6Addr<IV6> {
        let octets = self.octets();
        Ipv6Addr::from([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, octets[0], octets[1], octets[2], octets[3],
        ])
    }

    /// Converts this address to an IPv4-mapped [IPv6 address].
    ///
    /// a.b.c.d becomes ::ffff:a.b.c.d
    ///
    /// [IPv6 address]: ../addr_hal/struct.Ipv6Addr.html
    ///
    /// # Examples
    ///
    /// ```
    /// use addr_hal::Ipv4Addr;
    /// use addr_mock::Ipv4AddrInner;
    ///
    /// use addr_hal::Ipv6Addr;
    /// use addr_mock::Ipv6AddrInner;
    ///
    /// type Ipv4 = Ipv4Addr<Ipv4AddrInner>;
    /// type Ipv6 = Ipv6Addr<Ipv6AddrInner>;
    ///
    /// assert_eq!(Ipv4::new(192, 0, 2, 255).to_ipv6_mapped(),
    ///            Ipv6::new(0, 0, 0, 0, 0, 65535, 49152, 767));
    /// ```
    pub fn to_ipv6_mapped<IV6: Ipv6Address>(&self) -> Ipv6Addr<IV6> {
        let octets = self.octets();
        Ipv6Addr::from([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, octets[0], octets[1], octets[2], octets[3],
        ])
    }
}

impl<IV4: Ipv4Address> Clone for Ipv4Addr<IV4> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<IV4: Ipv4Address> Copy for Ipv4Addr<IV4> {}

impl<IV4: Ipv4Address> fmt::Debug for Ipv4Addr<IV4> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl<IV4: Ipv4Address> fmt::Display for Ipv4Addr<IV4> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let octets = self.octets();
        write!(
            fmt,
            "{}.{}.{}.{}",
            octets[0], octets[1], octets[2], octets[3]
        )
    }
}

impl<IV4: Ipv4Address> Eq for Ipv4Addr<IV4> {}

impl<IV4: Ipv4Address> PartialEq for Ipv4Addr<IV4> {
    fn eq(&self, other: &Ipv4Addr<IV4>) -> bool {
        self.inner == other.inner
    }
}

impl<IV4: Ipv4Address> From<[u8; 4]> for Ipv4Addr<IV4> {
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::from([13u8, 12u8, 11u8, 10u8]);
    /// assert_eq!(Ipv4Addr::new(13, 12, 11, 10), addr);
    /// ```
    fn from(octets: [u8; 4]) -> Ipv4Addr<IV4> {
        Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3])
    }
}

impl<IV4: Ipv4Address> From<u32> for Ipv4Addr<IV4> {
    /// Converts a host byte order `u32` into an `Ipv4Addr`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::from(0x0d0c0b0au32);
    /// assert_eq!(Ipv4Addr::new(13, 12, 11, 10), addr);
    /// ```
    fn from(ip: u32) -> Ipv4Addr<IV4> {
        Ipv4Addr::from(ip.to_be_bytes())
    }
}

impl<IV4: Ipv4Address> From<Ipv4Addr<IV4>> for u32 {
    /// Converts an `Ipv4Addr` into a host byte order `u32`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::new(13, 12, 11, 10);
    /// assert_eq!(0x0d0c0b0au32, u32::from(addr));
    /// ```
    fn from(ip: Ipv4Addr<IV4>) -> u32 {
        let ip = ip.octets();
        u32::from_be_bytes(ip)
    }
}

impl<IV4: Ipv4Address> hash::Hash for Ipv4Addr<IV4> {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        self.octets().hash(s)
    }
}

impl<IV4: Ipv4Address> PartialOrd for Ipv4Addr<IV4> {
    fn partial_cmp(&self, other: &Ipv4Addr<IV4>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<IV4: Ipv4Address> Ord for Ipv4Addr<IV4> {
    fn cmp(&self, other: &Ipv4Addr<IV4>) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::Ipv4Address;
    #[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Ord)]
    struct Ipv4AddrInner {
        inner: [u8; 4],
    }

    impl Ipv4Address for Ipv4AddrInner {
        const LOCALHOST: Self = Self {
            inner: [127, 0, 0, 1],
        };
        const UNSPECIFIED: Self = Self {
            inner: [0, 0, 0, 0],
        };

        const BROADCAST: Self = Self {
            inner: [255, 255, 255, 255],
        };

        fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
            Self {
                inner: [a, b, c, d],
            }
        }

        fn octets(&self) -> [u8; 4] {
            self.inner.clone()
        }
    }
}
