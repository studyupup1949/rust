use crate::error::AcnError;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Flags(u8);

impl Flags {
    /// If bit 8 is set, the length is extended by an extra byte.
    pub const EXTENDED_LENGTH_BIT_MASK: u8 = 0b10000000;
    /// If bit 7 is set, the pdu contains a vector segment
    pub const INCLUDES_VECTOR_BIT_MASK: u8 = 0b01000000;
    /// If bit 6 is set, the pdu contains a header segment
    pub const INCLUDES_HEADER_BIT_MASK: u8 = 0b00100000;
    /// If bit 5 is set, the pdu contains a data segment
    pub const INCLUDES_DATA_BIT_MASK: u8 = 0b00010000;

    pub const ALL_BIT_MASK: u8 = Self::EXTENDED_LENGTH_BIT_MASK
        | Self::INCLUDES_VECTOR_BIT_MASK
        | Self::INCLUDES_HEADER_BIT_MASK
        | Self::INCLUDES_DATA_BIT_MASK;

    pub const EXTENDED_LENGTH: Self = Self(Self::EXTENDED_LENGTH_BIT_MASK);

    pub const INCLUDES_VECTOR: Self = Self(Self::INCLUDES_VECTOR_BIT_MASK);

    pub const INCLUDES_HEADER: Self = Self(Self::INCLUDES_HEADER_BIT_MASK);

    pub const INCLUDES_DATA: Self = Self(Self::INCLUDES_DATA_BIT_MASK);

    pub const ALL: Self = Self(Self::ALL_BIT_MASK);

    pub const fn new(flags: u8) -> Self {
        Self(flags & Self::ALL_BIT_MASK)
    }

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn bits(&self) -> u8 {
        self.0
    }

    pub const fn is_extended_length(&self) -> bool {
        self.0 & Self::EXTENDED_LENGTH_BIT_MASK != 0
    }

    pub const fn is_includes_vector(&self) -> bool {
        self.0 & Self::INCLUDES_VECTOR_BIT_MASK != 0
    }

    pub const fn is_includes_header(&self) -> bool {
        self.0 & Self::INCLUDES_HEADER_BIT_MASK != 0
    }

    pub const fn is_includes_data(&self) -> bool {
        self.0 & Self::INCLUDES_DATA_BIT_MASK != 0
    }

    pub const fn is_all(&self) -> bool {
        self.0 == Self::ALL_BIT_MASK
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, AcnError> {
        if buf.is_empty() {
            return Err(AcnError::InvalidBufferLength(buf.len()));
        }

        buf[0] |= self.bits();

        Ok(1)
    }

    pub fn decode(buf: &[u8]) -> Result<Self, AcnError> {
        if buf.is_empty() {
            return Err(AcnError::InvalidBufferLength(0));
        }

        Ok(Self::new(buf[0]))
    }
}

impl BitAnd for Flags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::new(self.bits() & rhs.bits())
    }
}

impl BitAnd<u8> for Flags {
    type Output = Self;

    fn bitand(self, rhs: u8) -> Self::Output {
        Self::new(self.bits() & rhs)
    }
}

impl BitAnd<Flags> for u8 {
    type Output = Flags;

    fn bitand(self, rhs: Flags) -> Self::Output {
        Flags::new(self & rhs.bits())
    }
}

impl BitAndAssign for Flags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.bits();
    }
}

impl BitAndAssign<u8> for Flags {
    fn bitand_assign(&mut self, rhs: u8) {
        self.0 &= rhs;
    }
}

impl BitAndAssign<Flags> for u8 {
    fn bitand_assign(&mut self, rhs: Flags) {
        *self &= rhs.bits();
    }
}

impl BitOr for Flags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::new(self.bits() | rhs.bits())
    }
}

impl BitOr<u8> for Flags {
    type Output = Self;

    fn bitor(self, rhs: u8) -> Self::Output {
        Self::new(self.bits() | rhs)
    }
}

impl BitOr<Flags> for u8 {
    type Output = Flags;

    fn bitor(self, rhs: Flags) -> Self::Output {
        Flags::new(self | rhs.bits())
    }
}

impl BitOrAssign for Flags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.bits();
    }
}

impl BitOrAssign<u8> for Flags {
    fn bitor_assign(&mut self, rhs: u8) {
        self.0 |= rhs;
    }
}

impl BitOrAssign<Flags> for u8 {
    fn bitor_assign(&mut self, rhs: Flags) {
        *self |= rhs.bits();
    }
}

impl From<u8> for Flags {
    fn from(flags: u8) -> Self {
        Self::new(flags)
    }
}

impl From<Flags> for u8 {
    fn from(flags: Flags) -> Self {
        flags.bits()
    }
}
