use addr_hal::{Ipv4Address, Ipv6Address};

#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Ord)]
pub struct Ipv4AddrInner {
    inner: [u8; 4],
}

impl Ipv4Address for Ipv4AddrInner {
    const LOCALHOST: Self = Ipv4AddrInner {
        inner: [127, 0, 0, 1],
    };

    const UNSPECIFIED: Self = Ipv4AddrInner {
        inner: [0, 0, 0, 0],
    };

    const BROADCAST: Self = Ipv4AddrInner {
        inner: [255, 255, 255, 255],
    };

    fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4AddrInner {
            inner: [a, b, c, d],
        }
    }

    fn octets(&self) -> [u8; 4] {
        self.inner.clone()
    }
}

#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Ord)]
pub struct Ipv6AddrInner {
    inner: [u16; 8],
}

impl Ipv6Address for Ipv6AddrInner {
    const LOCALHOST: Ipv6AddrInner = Ipv6AddrInner {
        inner: [0, 0, 0, 0, 0, 0, 0, 1],
    };

    const UNSPECIFIED: Ipv6AddrInner = Ipv6AddrInner {
        inner: [0, 0, 0, 0, 0, 0, 0, 0],
    };

    fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Self {
        Ipv6AddrInner {
            inner: [a, b, c, d, e, f, g, h],
        }
    }

    fn segments(&self) -> [u16; 8] {
        self.inner.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::Ipv4AddrInner;
    use addr_hal::Ipv4Addr;
    // use addr_hal::Ipv4Address;

    #[test]
    fn test_ipv4() {
        let localhost = Ipv4Addr::<Ipv4AddrInner>::new(127, 0, 0, 1);
        assert_eq!("127.0.0.1".parse(), Ok(localhost));
    }
}
