/// An IPv4 address. (a.b.c.d)
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
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
    //! Construction

    /// Creates a new IPv4 address. [a, b, c, d]
    pub const fn new(address: [u8; 4]) -> Self {
        Self { address }
    }
}

impl From<[u8; 4]> for IPv4Address {
    fn from(address: [u8; 4]) -> Self {
        Self::new(address)
    }
}

impl From<IPv4Address> for [u8; 4] {
    fn from(v4: IPv4Address) -> Self {
        v4.address
    }
}

impl From<(u8, u8, u8, u8)> for IPv4Address {
    fn from(tuple: (u8, u8, u8, u8)) -> Self {
        Self::new([tuple.0, tuple.1, tuple.2, tuple.3])
    }
}

impl From<IPv4Address> for (u8, u8, u8, u8) {
    fn from(v4: IPv4Address) -> Self {
        v4.bytes()
    }
}

impl From<u32> for IPv4Address {
    fn from(value: u32) -> Self {
        Self::new(value.to_be_bytes())
    }
}

impl From<IPv4Address> for u32 {
    fn from(v4: IPv4Address) -> Self {
        u32::from_be_bytes(v4.address)
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

#[cfg(test)]
mod tests {
    use crate::IPv4Address;

    #[test]
    fn default() {
        assert_eq!(IPv4Address::default(), IPv4Address::UNSPECIFIED);
    }

    #[test]
    fn specials() {
        assert_eq!(IPv4Address::UNSPECIFIED.address(), [0, 0, 0, 0]);
        assert_eq!(IPv4Address::LOCALHOST.address(), [127, 0, 0, 1]);
        assert_eq!(IPv4Address::BROADCAST.address(), [255, 255, 255, 255]);
    }

    #[test]
    fn construction() {
        let address: [u8; 4] = [1, 2, 3, 4];
        let tuple: (u8, u8, u8, u8) = (1, 2, 3, 4);
        let value: u32 = 0x01020304u32;

        assert_eq!(IPv4Address::new(address).address, address);
        assert_eq!(IPv4Address::from(address).address, address);
        assert_eq!(IPv4Address::from(tuple).address, address);
        assert_eq!(IPv4Address::from(value).address, address);

        let result: [u8; 4] = IPv4Address::new(address).into();
        assert_eq!(result, address);

        let result: (u8, u8, u8, u8) = IPv4Address::new(address).into();
        assert_eq!(result, tuple);

        let result: u32 = IPv4Address::new(address).into();
        assert_eq!(result, value);
    }

    #[test]
    fn properties() {
        let address: [u8; 4] = [1, 2, 3, 4];
        let tuple: (u8, u8, u8, u8) = (1, 2, 3, 4);

        let v4: IPv4Address = IPv4Address::new(address);
        assert_eq!(v4.address(), address);
        assert_eq!(v4.bytes(), tuple);
    }
}
