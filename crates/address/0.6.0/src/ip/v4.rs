/// Represents an IPv4 address. (a.b.c.d)
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct IPv4Address {
    address: [u8; 4],
}

impl IPv4Address {
    //! Special Addresses

    /// The unspecified address. (0.0.0.0)
    pub const UNSPECIFIED: Self = Self::new([0, 0, 0, 0]);

    /// The localhost address. (127.0.0.1)
    pub const LOCALHOST: Self = Self::new([127, 0, 0, 1]);

    /// The broadcast address. (255.255.255.255)
    pub const BROADCAST: Self = Self::new([255, 255, 255, 255]);
}

impl IPv4Address {
    //! Constructors

    /// Creates a new IPv4 address. [a, b, c, d]
    pub const fn new(address: [u8; 4]) -> Self {
        Self { address }
    }
}

impl Default for IPv4Address {
    fn default() -> Self {
        Self::UNSPECIFIED
    }
}

impl IPv4Address {
    //! Properties

    /// Gets the address. [a, b, c, d]
    pub const fn address(&self) -> [u8; 4] {
        self.address
    }

    /// Gets the bytes. (a, b, c, d)
    pub const fn bytes(&self) -> (u8, u8, u8, u8) {
        (
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
        )
    }
}
