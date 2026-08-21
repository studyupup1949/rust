//! Binary writer implementing TypeScript SDK compatible encoding
//!
//! This module provides exact binary encoding compatibility with the TypeScript SDK,
//! including identical varint/uvarint encoding, length prefixes, and field encoding.

// Allow unwrap in this module - write operations to Vec<u8> cannot fail
#![allow(clippy::unwrap_used)]

use thiserror::Error;

/// Errors that can occur during binary encoding
#[derive(Error, Debug)]
pub enum EncodingError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Field number out of range [1, 32]: {0}")]
    InvalidFieldNumber(u32),

    #[error("Value exceeds maximum safe integer")]
    ValueTooLarge,

    #[error("Hash must be exactly 32 bytes, got {0}")]
    InvalidHashLength(usize),

    #[error("Cannot marshal negative bigint")]
    NegativeBigInt,

    #[error("Invalid UTF-8 string")]
    InvalidUtf8,
}

/// Binary writer that matches TypeScript SDK encoding exactly
#[derive(Debug, Clone)]
pub struct BinaryWriter {
    buffer: Vec<u8>,
}

impl BinaryWriter {
    /// Create a new binary writer
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Create a binary writer with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    /// Get the accumulated bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    /// Get a reference to the accumulated bytes
    pub fn bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Write raw bytes to the buffer
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodingError> {
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    /// Encode a field with its number and value
    /// Matches TS: fieldMarshalBinary(field: number, val: Uint8Array)
    pub fn write_field(&mut self, field: u32, value: &[u8]) -> Result<(), EncodingError> {
        if field < 1 || field > 32 {
            return Err(EncodingError::InvalidFieldNumber(field));
        }
        self.write_uvarint(field as u64)?;
        self.write_bytes(value)?;
        Ok(())
    }

    /// Encode an unsigned varint using Go's canonical encoding/binary algorithm
    /// Matches Go: binary.PutUvarint(buf, x)
    pub fn write_uvarint(&mut self, mut value: u64) -> Result<(), EncodingError> {
        // Use Go's canonical varint algorithm - no special cases needed
        while value >= 0x80 {
            self.buffer.push((value as u8) | 0x80);
            value >>= 7;
        }
        self.buffer.push(value as u8);
        Ok(())
    }

    /// Encode an unsigned varint with field number
    pub fn write_uvarint_field(&mut self, value: u64, field: u32) -> Result<(), EncodingError> {
        let mut temp_writer = BinaryWriter::new();
        temp_writer.write_uvarint(value)?;
        self.write_field(field, temp_writer.bytes())?;
        Ok(())
    }

    /// Encode a signed varint using Go's canonical zigzag encoding
    /// Matches Go: binary.PutVarint(buf, x)
    pub fn write_varint(&mut self, value: i64) -> Result<(), EncodingError> {
        // Go's canonical zigzag encoding algorithm
        let unsigned = ((value as u64) << 1) ^ ((value >> 63) as u64);
        self.write_uvarint(unsigned)
    }

    /// Encode a signed varint with field number
    pub fn write_varint_field(&mut self, value: i64, field: u32) -> Result<(), EncodingError> {
        let mut temp_writer = BinaryWriter::new();
        temp_writer.write_varint(value)?;
        self.write_field(field, temp_writer.bytes())?;
        Ok(())
    }

    /// Encode a big number (unsigned big integer)
    /// Matches TS: bigNumberMarshalBinary(bn: bigint, field?: number)
    pub fn write_big_number(&mut self, value: &num_bigint::BigUint) -> Result<(), EncodingError> {
        let hex_string = value.to_str_radix(16);

        // Ensure even number of hex digits
        let padded_hex = if hex_string.len() % 2 == 1 {
            format!("0{}", hex_string)
        } else {
            hex_string
        };

        // Convert hex string to bytes
        let bytes: Result<Vec<u8>, _> = (0..padded_hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&padded_hex[i..i + 2], 16))
            .collect();

        let bytes = bytes.map_err(|_| EncodingError::InvalidUtf8)?;
        self.write_bytes_with_length(&bytes)?;
        Ok(())
    }

    /// Encode a big number with field number
    pub fn write_big_number_field(
        &mut self,
        value: &num_bigint::BigUint,
        field: u32,
    ) -> Result<(), EncodingError> {
        let mut temp_writer = BinaryWriter::new();
        temp_writer.write_big_number(value)?;
        self.write_field(field, temp_writer.bytes())?;
        Ok(())
    }

    /// Encode a boolean value
    /// Matches TS: booleanMarshalBinary(b: boolean, field?: number)
    pub fn write_bool(&mut self, value: bool) -> Result<(), EncodingError> {
        self.buffer.push(if value { 1 } else { 0 });
        Ok(())
    }

    /// Encode a boolean with field number
    pub fn write_bool_field(&mut self, value: bool, field: u32) -> Result<(), EncodingError> {
        let mut temp_writer = BinaryWriter::new();
        temp_writer.write_bool(value)?;
        self.write_field(field, temp_writer.bytes())?;
        Ok(())
    }

    /// Encode a string as UTF-8 bytes with length prefix
    /// Matches TS: stringMarshalBinary(val: string, field?: number)
    pub fn write_string(&mut self, value: &str) -> Result<(), EncodingError> {
        let bytes = value.as_bytes();
        self.write_bytes_with_length(bytes)?;
        Ok(())
    }

    /// Encode a string with field number
    pub fn write_string_field(&mut self, value: &str, field: u32) -> Result<(), EncodingError> {
        let mut temp_writer = BinaryWriter::new();
        temp_writer.write_string(value)?;
        self.write_field(field, temp_writer.bytes())?;
        Ok(())
    }

    /// Encode bytes with length prefix
    /// Matches TS: bytesMarshalBinary(val: Uint8Array, field?: number)
    pub fn write_bytes_with_length(&mut self, bytes: &[u8]) -> Result<(), EncodingError> {
        self.write_uvarint(bytes.len() as u64)?;
        self.write_bytes(bytes)?;
        Ok(())
    }

    /// Encode bytes with length prefix and field number
    pub fn write_bytes_field(&mut self, bytes: &[u8], field: u32) -> Result<(), EncodingError> {
        let mut temp_writer = BinaryWriter::new();
        temp_writer.write_bytes_with_length(bytes)?;
        self.write_field(field, temp_writer.bytes())?;
        Ok(())
    }

    /// Encode a 32-byte hash without length prefix
    /// Matches TS: hashMarshalBinary(val: Uint8Array, field?: number)
    pub fn write_hash(&mut self, hash: &[u8; 32]) -> Result<(), EncodingError> {
        self.write_bytes(hash)?;
        Ok(())
    }

    /// Encode a hash with field number
    pub fn write_hash_field(&mut self, hash: &[u8; 32], field: u32) -> Result<(), EncodingError> {
        self.write_field(field, hash)?;
        Ok(())
    }

    /// Encode a variable-length hash with validation
    pub fn write_hash_bytes(&mut self, hash: &[u8]) -> Result<(), EncodingError> {
        if hash.len() != 32 {
            return Err(EncodingError::InvalidHashLength(hash.len()));
        }
        self.write_bytes(hash)?;
        Ok(())
    }

    /// Encode a variable-length hash with field number
    pub fn write_hash_bytes_field(&mut self, hash: &[u8], field: u32) -> Result<(), EncodingError> {
        if hash.len() != 32 {
            return Err(EncodingError::InvalidHashLength(hash.len()));
        }
        self.write_field(field, hash)?;
        Ok(())
    }

    /// Write an optional value (None = skip, Some = encode)
    pub fn write_optional<T, F>(
        &mut self,
        value: Option<&T>,
        _field: u32,
        writer_fn: F,
    ) -> Result<(), EncodingError>
    where
        T: Clone,
        F: FnOnce(&mut Self, &T) -> Result<(), EncodingError>,
    {
        if let Some(val) = value {
            writer_fn(self, val)?;
        }
        Ok(())
    }

    /// Write an array/slice with element encoding
    pub fn write_array<T, F>(
        &mut self,
        items: &[T],
        _field: u32,
        writer_fn: F,
    ) -> Result<(), EncodingError>
    where
        F: Fn(&mut Self, &T) -> Result<(), EncodingError>,
    {
        for item in items {
            writer_fn(self, item)?;
        }
        Ok(())
    }
}

impl Default for BinaryWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions that match TypeScript SDK exactly
impl BinaryWriter {
    /// Create field-encoded data (helper matching TS withFieldNumber)
    pub fn with_field_number(data: &[u8], field: Option<u32>) -> Result<Vec<u8>, EncodingError> {
        match field {
            Some(field_num) => {
                let mut writer = BinaryWriter::new();
                writer.write_field(field_num, data)?;
                Ok(writer.into_bytes())
            }
            None => Ok(data.to_vec()),
        }
    }

    /// Encode uvarint as standalone function
    pub fn encode_uvarint(value: u64) -> Vec<u8> {
        let mut writer = BinaryWriter::new();
        writer.write_uvarint(value).unwrap(); // Should never fail for u64
        writer.into_bytes()
    }

    /// Encode varint as standalone function
    pub fn encode_varint(value: i64) -> Vec<u8> {
        let mut writer = BinaryWriter::new();
        writer.write_varint(value).unwrap(); // Should never fail for i64
        writer.into_bytes()
    }

    /// Encode string as standalone function
    pub fn encode_string(value: &str) -> Vec<u8> {
        let mut writer = BinaryWriter::new();
        writer.write_string(value).unwrap(); // Should never fail for valid UTF-8
        writer.into_bytes()
    }

    /// Encode bytes with length as standalone function
    pub fn encode_bytes(bytes: &[u8]) -> Vec<u8> {
        let mut writer = BinaryWriter::new();
        writer.write_bytes_with_length(bytes).unwrap(); // Should never fail
        writer.into_bytes()
    }

    /// Encode boolean as standalone function
    pub fn encode_bool(value: bool) -> Vec<u8> {
        vec![if value { 1 } else { 0 }]
    }

    /// Encode hash as standalone function
    pub fn encode_hash(hash: &[u8; 32]) -> Vec<u8> {
        hash.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uvarint_encoding() {
        // Test cases matching TypeScript implementation
        let test_cases = vec![
            (0u64, vec![0]),
            (1u64, vec![1]),
            (127u64, vec![127]),
            (128u64, vec![128, 1]),
            (256u64, vec![128, 2]),
            (16384u64, vec![128, 128, 1]),
        ];

        for (input, expected) in test_cases {
            let result = BinaryWriter::encode_uvarint(input);
            assert_eq!(result, expected, "uvarint({}) failed", input);
        }
    }

    #[test]
    fn test_varint_encoding() {
        // Test zigzag encoding
        let test_cases = vec![
            (0i64, vec![0]),
            (-1i64, vec![1]),
            (1i64, vec![2]),
            (-2i64, vec![3]),
            (2i64, vec![4]),
        ];

        for (input, expected) in test_cases {
            let result = BinaryWriter::encode_varint(input);
            assert_eq!(result, expected, "varint({}) failed", input);
        }
    }

    #[test]
    fn test_string_encoding() {
        let result = BinaryWriter::encode_string("hello");
        // Length (5) + "hello"
        let expected = vec![5, b'h', b'e', b'l', b'l', b'o'];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_bytes_encoding() {
        let input = &[1, 2, 3, 4];
        let result = BinaryWriter::encode_bytes(input);
        // Length (4) + [1, 2, 3, 4]
        let expected = vec![4, 1, 2, 3, 4];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_bool_encoding() {
        assert_eq!(BinaryWriter::encode_bool(true), vec![1]);
        assert_eq!(BinaryWriter::encode_bool(false), vec![0]);
    }

    #[test]
    fn test_field_encoding() {
        let mut writer = BinaryWriter::new();
        writer.write_field(1, &[42]).unwrap();
        // Field 1 (encoded as uvarint) + value [42]
        let expected = vec![1, 42];
        assert_eq!(writer.bytes(), &expected);
    }

    #[test]
    fn test_hash_validation() {
        let mut writer = BinaryWriter::new();

        // Valid 32-byte hash should work
        let valid_hash = [0u8; 32];
        assert!(writer.write_hash(&valid_hash).is_ok());

        // Invalid length should fail
        let invalid_hash = [0u8; 31];
        assert!(writer.write_hash_bytes(&invalid_hash).is_err());
    }
}
