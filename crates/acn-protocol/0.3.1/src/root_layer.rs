use crate::AcnError;
use core::error::Error;

pub trait RootLayerCodec {
    type Error: Error + From<AcnError>;

    fn size(&self) -> usize {
        self.preamble_length() + self.pdu_block_length() + self.postamble_length()
    }

    fn preamble_length(&self) -> usize {
        0
    }

    fn encode_preamble(&self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(self.preamble_length())
    }

    fn pdu_block_length(&self) -> usize {
        0
    }

    fn encode_pdu_block(&self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(self.pdu_block_length())
    }

    fn postamble_length(&self) -> usize {
        0
    }

    fn encode_postamble(&self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(self.postamble_length())
    }

    fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let buffer_len = buf.len();
        let expected_len = self.size();

        if buffer_len < expected_len {
            return Err(AcnError::InvalidBufferLength {
                actual: buffer_len,
                expected: expected_len,
            }.into());
        }

        let mut offset = 0;

        offset += self.encode_preamble(&mut buf[offset..])?;

        offset += self.encode_pdu_block(&mut buf[offset..])?;

        offset += self.encode_postamble(&mut buf[offset..])?;

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

    struct TestRootLayerPacket {
        pdu1: TestPdu,
        pdu2: TestPdu,
    }

    impl RootLayerCodec for TestRootLayerPacket {
        type Error = AcnError;

        fn preamble_length(&self) -> usize {
            4
        }

        fn encode_preamble(&self, buf: &mut [u8]) -> Result<usize, AcnError> {
            buf[0..4].copy_from_slice(&[0x01, 0x02, 0x03, 0x04]);

            Ok(self.preamble_length())
        }

        fn pdu_block_length(&self) -> usize {
            12
        }

        fn encode_pdu_block(&self, buf: &mut [u8]) -> Result<usize, AcnError> {
            buf[0] = self.pdu1.target_id;
            buf[1] = self.pdu1.source_id;
            buf[2..6].copy_from_slice(&self.pdu1.data.to_be_bytes());
            buf[6] = self.pdu2.target_id;
            buf[7] = self.pdu2.source_id;
            buf[8..12].copy_from_slice(&self.pdu2.data.to_be_bytes());

            Ok(self.pdu_block_length())
        }

        fn postamble_length(&self) -> usize {
            4
        }

        fn encode_postamble(&self, buf: &mut [u8]) -> Result<usize, AcnError> {
            buf[0..4].copy_from_slice(&[0x05, 0x06, 0x07, 0x08]);

            Ok(self.postamble_length())
        }

        fn decode(buf: &[u8]) -> Result<Self, Self::Error> {
            if buf.len() < 4 {
                return Err(AcnError::InvalidBufferLength {
                    actual: buf.len(),
                    expected: 4,
                });
            }

            if buf[0..4] != [0x01, 0x02, 0x03, 0x04] {
                return Err(AcnError::InvalidPreamble);
            }

            let pdu1 = TestPdu {
                target_id: buf[0],
                source_id: buf[1],
                data: u32::from_be_bytes(buf[2..6].try_into()?),
            };

            let pdu2 = TestPdu {
                target_id: buf[6],
                source_id: buf[7],
                data: u32::from_be_bytes(buf[8..12].try_into()?),
            };

            if buf[12..16] != [0x05, 0x06, 0x07, 0x08] {
                return Err(AcnError::InvalidPostamble);
            }

            Ok(Self { pdu1, pdu2 })
        }
    }

    #[test]
    fn should_encode_root_layer_packet() {
        let buf = &mut [0u8; 20];

        let root_layer_packet = TestRootLayerPacket {
            pdu1: TestPdu {
                target_id: 0x01,
                source_id: 0x02,
                data: 0x12345678,
            },
            pdu2: TestPdu {
                target_id: 0x03,
                source_id: 0x04,
                data: 0x87654321,
            },
        };

        let bytes_written = root_layer_packet.encode(buf).unwrap();

        assert_eq!(bytes_written, 20);

        assert_eq!(
            buf,
            &[
                0x01, // Preamble Byte 1
                0x02, // Preamble Byte 2
                0x03, // Preamble Byte 3
                0x04, // Preamble Byte 4
                0x01, // PDU1 Target ID
                0x02, // PDU1 Source ID
                0x12, // PDU1 Data Byte 1
                0x34, // PDU1 Data Byte 2
                0x56, // PDU1 Data Byte 3
                0x78, // PDU1 Data Byte 4
                0x03, // PDU2 Target ID
                0x04, // PDU2 Source ID
                0x87, // PDU2 Data Byte 1
                0x65, // PDU2 Data Byte 2
                0x43, // PDU2 Data Byte 3
                0x21, // PDU2 Data Byte 4
                0x05, // Postamble Byte 1
                0x06, // Postamble Byte 2
                0x07, // Postamble Byte 3
                0x08, // Postamble Byte 4
            ]
        );
    }
}
