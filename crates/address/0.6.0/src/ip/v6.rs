/// Represents an IPv6 address. (a:b:c:d:e:f:g:h)
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct IPv6Address {
    address: [u8; 16],
}

impl IPv6Address {
    //! Special Addresses

    /// The unspecified address. (::)
    pub const UNSPECIFIED: Self = Self::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    /// The localhost address. (::1)
    pub const LOCALHOST: Self = Self::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
}

impl IPv6Address {
    //! Constructors

    /// Creates a new IPv6 address. [a-high, a-low, b-high, b-low, ..., h-high, h-low]
    pub const fn new(address: [u8; 16]) -> Self {
        Self { address }
    }
}

impl Default for IPv6Address {
    fn default() -> Self {
        Self::UNSPECIFIED
    }
}

impl IPv6Address {
    //! Properties

    /// Gets the address. [a-high, a-low, b-high, b-low, ..., h-high, h-low]
    pub const fn address(&self) -> &[u8; 16] {
        &self.address
    }

    /// Gets the segments. [a, b, c, d, e, f, g, h]
    pub const fn segments(&self) -> [u16; 8] {
        [
            (self.address[0] as u16) << 8 | (self.address[1] as u16),
            (self.address[2] as u16) << 8 | (self.address[3] as u16),
            (self.address[4] as u16) << 8 | (self.address[5] as u16),
            (self.address[6] as u16) << 8 | (self.address[7] as u16),
            (self.address[8] as u16) << 8 | (self.address[9] as u16),
            (self.address[10] as u16) << 8 | (self.address[11] as u16),
            (self.address[12] as u16) << 8 | (self.address[13] as u16),
            (self.address[14] as u16) << 8 | (self.address[15] as u16),
        ]
    }
}

impl IPv6Address {
    //! Classifications

    /// Checks if the address is an IPv4 compatible address (::a.b.c.d).
    pub const fn is_v4_compatible(&self) -> bool {
        matches!(
            self.address,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, _, _, _, _]
        )
    }

    /// Checks if the address is an IPv4 mapped address (::ffff:a.b.c.d).
    pub const fn is_v4_mapped(&self) -> bool {
        matches!(
            self.address,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, _, _, _, _]
        )
    }

    /// Checks if the address is an IPv4 compatible address (::a.b.c.d) or an IPv4 mapped address (::ffff:a.b.c.d).
    pub const fn is_v4_convertable(&self) -> bool {
        self.is_v4_compatible() || self.is_v4_mapped()
    }
}
