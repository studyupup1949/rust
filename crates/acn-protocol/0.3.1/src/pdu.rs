use crate::{error::AcnError, flags::Flags, length::Length, vector::Vector};
use core::error::Error;

pub trait PduCodec {
    type Error: Error + From<AcnError>;

    fn flags(&self) -> Flags {
        let mut flags = Flags::default();

        if self.vector_length() + self.header_length() + self.data_length()
            >= Length::EXTENDED_LENGTH_THRESHOLD
        {
            flags |= Flags::EXTENDED_LENGTH;
        }

        if self.vector_length() > 0 {
            flags |= Flags::INCLUDES_VECTOR;
        }

        if self.header_length() > 0 {
            flags |= Flags::INCLUDES_HEADER;
        }

        if self.data_length() > 0 {
            flags |= Flags::INCLUDES_DATA;
        }

        flags
    }

    fn length(&self) -> Length {
        Length::new(
            self.vector_length() + self.header_length() + self.data_length(),
            self.flags(),
        )
    }

    fn vector(&self) -> Option<Vector> {
        None
    }

    fn vector_length(&self) -> usize {
        self.vector().map_or(0, |v| v.size())
    }

    fn header_length(&self) -> usize {
        0
    }

    fn encode_header(&self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn data_length(&self) -> usize {
        0
    }

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // It's possible the buffer is not zeroed out and it is being overwritten
        // flags and length OR the first byte so we must zero it first
        buf[0] = 0;

        self.flags().encode(buf)?;

        let mut offset = self.length().encode(buf)?;

        if let Some(vector) = self.vector() {
            offset += vector.encode(&mut buf[offset..])?;
        }

        offset += self.encode_header(&mut buf[offset..])?;

        offset += self.encode_data(&mut buf[offset..])?;

        Ok(offset)
    }

    fn decode(buf: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPdu {
        target_id: u8,
        source_id: u8,
        data: u32,
    }

    impl PduCodec for TestPdu {
        type Error = AcnError;

        fn vector(&self) -> Option<Vector> {
            Some(Vector::U8(0x01))
        }

        fn header_length(&self) -> usize {
            2
        }

        fn encode_header(&self, buf: &mut [u8]) -> Result<usize, AcnError> {
            buf[0] = self.target_id;
            buf[1] = self.source_id;
            Ok(2)
        }

        fn data_length(&self) -> usize {
            4
        }

        fn encode_data(&self, buf: &mut [u8]) -> Result<usize, AcnError> {
            buf[0..4].copy_from_slice(&self.data.to_be_bytes());
            Ok(4)
        }

        fn decode(buf: &[u8]) -> Result<Self, Self::Error> {
            let length = Length::decode(buf)?;

            let buffer_len = buf.len();
            let expected_len = length.as_usize();

            if buffer_len < expected_len {
                return Err(AcnError::InvalidBufferLength {
                    actual: buffer_len,
                    expected: expected_len,
                });
            }

            let vector = buf[3];

            if vector != 0x01 {
                return Err(AcnError::InvalidVector(vector.into()));
            }

            let target_id = buf[4];
            let source_id = buf[5];
            let data = u32::from_be_bytes(buf[6..10].try_into()?);

            Ok(TestPdu {
                target_id,
                source_id,
                data,
            })
        }
    }

    #[test]
    fn should_encode_pdu() {
        let buf = &mut [0u8; 9];

        let pdu = TestPdu {
            target_id: 0x01,
            source_id: 0x02,
            data: 0x12345678,
        };

        let flags = pdu.flags();

        let expected_flags = Flags::INCLUDES_VECTOR | Flags::INCLUDES_HEADER | Flags::INCLUDES_DATA;

        assert_eq!(flags, expected_flags);

        let length = pdu.length();
        assert_eq!(length.size(), 2);
        assert_eq!(length.as_u32(), 9);

        pdu.encode(buf).unwrap();

        assert_eq!(
            buf,
            &[
                expected_flags.bits(), // Flags |= Length MSB
                0x09,                  // Length LSB
                0x01,                  // Vector
                0x01,                  // Header Byte 1 = Target ID
                0x02,                  // Header Byte 2 = Source ID
                0x12,                  // Data Byte 1
                0x34,                  // Data Byte 2
                0x56,                  // Data Byte 3
                0x78,                  // Data Byte 4
            ]
        );
    }
}
