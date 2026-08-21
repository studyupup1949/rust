use crate::{error::AcnError, flags::Flags};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Length {
    Standard(u16),                      // Value is first 12 bits
    Extended { upper: u8, lower: u16 }, // Value is first 20 bits
}

impl Length {
    pub const EXTENDED_LENGTH_THRESHOLD: usize = 4096;

    pub fn new(length: usize, flags: Flags) -> Self {
        // Add the byte length of Length::Standard to the lengths of each segment
        let mut pdu_length = length + 2;

        if pdu_length >= Self::EXTENDED_LENGTH_THRESHOLD || flags.is_extended_length() {
            // Length::Extended requires an extra byte
            pdu_length += 1;

            Length::Extended {
                upper: (pdu_length >> 16 & 0xf) as u8,
                lower: pdu_length as u16,
            }
        } else {
            Length::Standard(pdu_length as u16 & 0xfff)
        }
    }

    pub fn new_standard_from_u16(length: u16) -> Self {
        let pdu_length = length + 2;
        Length::Standard(pdu_length & 0xfff)
    }

    pub fn new_extended_from_u32(length: u32) -> Self {
        let pdu_length = length + 3;

        Length::Extended {
            upper: (pdu_length >> 16 & 0xf) as u8,
            lower: pdu_length as u16,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Length::Standard(_) => 2,
            Length::Extended { .. } => 3,
        }
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            Length::Standard(length) => *length as u32,
            Length::Extended { upper, lower } => ((upper & 0xf) as u32) << 16 | *lower as u32,
        }
    }

    pub fn as_usize(&self) -> usize {
        self.as_u32() as usize
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, AcnError> {
        let size = self.size();

        if buf.len() < size {
            return Err(AcnError::InvalidBufferLength(buf.len()));
        }

        match self {
            Length::Standard(length) => {
                let bytes = length.to_be_bytes();
                buf[0] |= bytes[0];
                buf[1] = bytes[1];
            }
            Length::Extended { upper, lower } => {
                buf[0] |= *upper;

                let bytes = lower.to_be_bytes();
                buf[1] = bytes[0];
                buf[2] = bytes[1];
            }
        }

        Ok(size)
    }

    pub fn decode(buf: &[u8]) -> Result<Self, AcnError> {
        if buf.len() < 2 {
            return Err(AcnError::InvalidBufferLength(buf.len()));
        }

        let flags = Flags::decode(buf)?;

        let length = if flags.is_extended_length() {
            if buf.len() < 3 {
                return Err(AcnError::InvalidBufferLength(buf.len()));
            }

            Length::Extended {
                upper: buf[0],
                lower: u16::from_be_bytes([buf[1], buf[2]]),
            }
        } else {
            Length::Standard(u16::from_be_bytes([buf[0] & 0xf, buf[1]]))
        };

        Ok(length)
    }
}

impl From<Length> for u32 {
    fn from(length: Length) -> u32 {
        length.as_u32()
    }
}

impl From<Length> for usize {
    fn from(length: Length) -> usize {
        length.as_usize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flags::Flags;

    #[test]
    fn test_length_standard() {
        let length = Length::new(100, Flags::empty());
        assert_eq!(length, Length::Standard(102));
        assert_eq!(length.size(), 2);
        assert_eq!(length.as_usize(), 102);
    }

    #[test]
    fn test_length_extended_from_extended_length() {
        let length = Length::new(65536, Flags::empty());
        assert_eq!(length, Length::Extended { upper: 1, lower: 3 });
        assert_eq!(length.size(), 3);
        assert_eq!(length.as_usize(), 65539);
    }

    #[test]
    fn test_length_extended_from_flags() {
        let length = Length::new(1024, Flags::EXTENDED_LENGTH);
        assert_eq!(
            length,
            Length::Extended {
                upper: 0,
                lower: 1027
            }
        );
        assert_eq!(length.size(), 3);
        assert_eq!(length.as_usize(), 1027);
    }

    #[test]
    fn test_length_encode_standard() {
        let length = Length::Standard(102);
        let mut buf = [0; 2];
        length.encode(&mut buf).unwrap();
        assert_eq!(buf, [0, 102]);
    }

    #[test]
    fn test_length_encode_extended() {
        let length = Length::Extended { upper: 1, lower: 3 };
        let mut buf = [0; 3];
        length.encode(&mut buf).unwrap();
        assert_eq!(buf, [1, 0, 3]);
    }
}
