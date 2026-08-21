use crate::error::AcnError;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Vector {
    U8(u8),
    U16(u16),
    U32(u32),
}

impl Vector {
    pub fn size(&self) -> usize {
        match self {
            Vector::U8(_) => 1,
            Vector::U16(_) => 2,
            Vector::U32(_) => 4,
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, AcnError> {
        let size = self.size();

        if buf.len() < size {
            return Err(AcnError::InvalidBufferLength(buf.len()));
        }

        match self {
            Vector::U8(value) => {
                buf[0] = *value;
            }
            Vector::U16(value) => {
                buf[0..2].copy_from_slice(&value.to_be_bytes());
            }
            Vector::U32(value) => {
                buf[0..4].copy_from_slice(&value.to_be_bytes());
            }
        };

        Ok(size)
    }
}

impl From<u8> for Vector {
    fn from(value: u8) -> Self {
        Vector::U8(value)
    }
}

impl From<u16> for Vector {
    fn from(value: u16) -> Self {
        Vector::U16(value)
    }
}

impl From<u32> for Vector {
    fn from(value: u32) -> Self {
        Vector::U32(value)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_vector_u8() {
        let vector = Vector::U8(0x01);
        let mut buf = [0; 1];
        assert_eq!(vector.encode(&mut buf).unwrap(), 1);
        assert_eq!(buf[0], 0x01);
    }

    #[test]
    fn test_vector_u16() {
        let vector = Vector::U16(0x0102);
        let mut buf = [0; 2];
        assert_eq!(vector.encode(&mut buf).unwrap(), 2);
        assert_eq!(buf, [0x01, 0x02]);
    }

    #[test]
    fn test_vector_u32() {
        let vector = Vector::U32(0x01020304);
        let mut buf = [0; 4];
        assert_eq!(vector.encode(&mut buf).unwrap(), 4);
        assert_eq!(buf, [0x01, 0x02, 0x03, 0x04]);
    }
}
