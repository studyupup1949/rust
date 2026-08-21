use std::fmt::{Display, Formatter, Write};

use crate::{IPAddress, IPv4Address, IPv6Address};

impl Display for IPv4Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (a, b, c, d) = self.bytes();
        write!(f, "{}.{}.{}.{}", a, b, c, d)
    }
}

impl Display for IPv6Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self == &Self::UNSPECIFIED {
            f.write_str("::")
        } else if self == &Self::LOCALHOST {
            f.write_str("::1")
        } else if let Some(v4) = self.to_v4() {
            let address: &[u8; 16] = self.address();
            match (address[10], address[11]) {
                (0, 0) => write!(f, "::{}", v4),
                (0xFF, 0xFF) => write!(f, "::ffff:{}", v4),
                _ => unreachable!(),
            }
        } else {
            #[derive(Copy, Clone, Default)]
            struct Zeros {
                index: usize,
                len: usize,
            }
            let segments: [u16; 8] = self.segments();
            let longest_zeros: Zeros = {
                let mut longest: Zeros = Zeros::default();
                let mut current: Zeros = Zeros::default();
                for (i, &segment) in segments.iter().enumerate() {
                    if segment == 0 {
                        if current.len == 0 {
                            current.index = i;
                        }
                        current.len += 1;
                        if current.len > longest.len {
                            longest = current;
                        }
                    } else {
                        current = Zeros::default();
                    }
                }
                longest
            };
            #[inline(always)]
            fn format(f: &mut std::fmt::Formatter<'_>, chunk: &[u16]) -> std::fmt::Result {
                if let Some((first, tail)) = chunk.split_first() {
                    write!(f, "{:x}", first)?;
                    for segment in tail {
                        f.write_char(':')?;
                        write!(f, "{:x}", segment)?;
                    }
                }
                Ok(())
            }
            if longest_zeros.len > 1 {
                format(f, &segments[..longest_zeros.index])?;
                f.write_str("::")?;
                format(f, &segments[longest_zeros.index + longest_zeros.len..])
            } else {
                format(f, &segments)
            }
        }
    }
}

impl Display for IPAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V4(v4) => write!(f, "{}", v4),
            Self::V6(v6) => write!(f, "{}", v6),
        }
    }
}
