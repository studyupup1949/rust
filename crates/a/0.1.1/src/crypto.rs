use crate::error::Hj212Error;

/// Appendix A (HJ 212—2025) describes "data segment encryption" as:
/// - Encrypt every 16 characters as one group
/// - NoPadding
/// - The trailing part shorter than 16 characters is kept in plaintext
/// - CRC is calculated and frame is built first, then the data segment is encrypted
///
/// The on-wire examples show encrypted bytes rendered like:
/// `CP=&&{0xE4,0x3E,...,0xA4}g=N&&...`
/// where the bytes in `{...}` represent encrypted blocks, followed by trailing plaintext.

/// Minimal 16-byte-block cipher interface.
///
/// This keeps `a` usable without forcing a specific algorithm when the standard text
/// does not explicitly mandate one for Appendix A data-segment encryption.
pub trait BlockCipher16 {
    fn encrypt_block(&self, block: &mut [u8; 16]);
    fn decrypt_block(&self, block: &mut [u8; 16]);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedDataSegment {
    /// Encrypted bytes (must be a multiple of 16).
    pub encrypted: Vec<u8>,
    /// Trailing plaintext (length < 16 characters in the Appendix A description).
    pub trailing_plaintext: String,
}

impl EncryptedDataSegment {
    pub fn to_wire_string(&self) -> String {
        if self.encrypted.is_empty() {
            return self.trailing_plaintext.clone();
        }

        let mut out = String::new();
        out.push('{');
        for (i, b) in self.encrypted.iter().enumerate() {
            if i != 0 {
                out.push(',');
            }
            out.push_str(&format!("0x{:02X}", b));
        }
        out.push('}');
        out.push_str(&self.trailing_plaintext);
        out
    }

    pub fn parse_wire_string(input: &str) -> Result<Self, Hj212Error> {
        let s = input.trim();
        if !s.starts_with('{') {
            return Ok(Self {
                encrypted: Vec::new(),
                trailing_plaintext: s.to_string(),
            });
        }

        let close = s.find('}').ok_or(Hj212Error::InvalidEncryptedDataFormat)?;
        let inside = &s[1..close];
        let trailing = &s[close + 1..];

        let mut encrypted = Vec::new();
        for token in inside.split(',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }

            let hex = token
                .strip_prefix("0x")
                .or_else(|| token.strip_prefix("0X"))
                .ok_or(Hj212Error::InvalidEncryptedDataFormat)?;
            if hex.len() != 2 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(Hj212Error::InvalidEncryptedDataFormat);
            }
            let b = u8::from_str_radix(hex, 16).map_err(|_| Hj212Error::InvalidEncryptedDataFormat)?;
            encrypted.push(b);
        }

        if encrypted.len() % 16 != 0 {
            return Err(Hj212Error::InvalidEncryptedDataLength);
        }

        Ok(Self {
            encrypted,
            trailing_plaintext: trailing.to_string(),
        })
    }
}

/// Encrypt a data-segment string using 16-byte blocks and leaving the trailing remainder as plaintext.
///
/// Note: The standard says "16 characters"; HJ212 payloads are typically ASCII.
/// This implementation requires ASCII to avoid splitting inside UTF-8 codepoints.
pub fn encrypt_data_segment<C: BlockCipher16>(plaintext: &str, cipher: &C) -> Result<EncryptedDataSegment, Hj212Error> {
    if !plaintext.is_ascii() {
        return Err(Hj212Error::NonAsciiForEncryption);
    }

    let bytes = plaintext.as_bytes();
    let full_len = (bytes.len() / 16) * 16;
    let (head, tail) = bytes.split_at(full_len);

    let mut encrypted = Vec::with_capacity(head.len());
    for chunk in head.chunks_exact(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        cipher.encrypt_block(&mut block);
        encrypted.extend_from_slice(&block);
    }

    Ok(EncryptedDataSegment {
        encrypted,
        trailing_plaintext: String::from_utf8_lossy(tail).to_string(),
    })
}

/// Decrypt an on-wire data-segment representation produced by [`EncryptedDataSegment::to_wire_string`].
pub fn decrypt_data_segment<C: BlockCipher16>(wire: &str, cipher: &C) -> Result<String, Hj212Error> {
    let parsed = EncryptedDataSegment::parse_wire_string(wire)?;

    let mut out = Vec::with_capacity(parsed.encrypted.len() + parsed.trailing_plaintext.len());
    for chunk in parsed.encrypted.chunks_exact(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        cipher.decrypt_block(&mut block);
        out.extend_from_slice(&block);
    }
    out.extend_from_slice(parsed.trailing_plaintext.as_bytes());
    String::from_utf8(out).map_err(|_| Hj212Error::InvalidEncryptedDataFormat)
}

/// Appendix H.1.2 specifies SM4-ECB with PKCS7 padding and Base64 encoding for HTTPS Authorization.
#[cfg(feature = "sm4")]
pub mod sm4_auth {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};
    use sm4::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
    use sm4::Sm4;

    pub fn sm4_ecb_pkcs7_encrypt_base64(plaintext: &str, key_16: &[u8; 16]) -> String {
        let cipher = Sm4::new_from_slice(key_16).expect("SM4 key length is 16");
        let mut buf = plaintext.as_bytes().to_vec();
        let pad = 16 - (buf.len() % 16);
        buf.extend(std::iter::repeat(pad as u8).take(pad));

        for chunk in buf.chunks_exact_mut(16) {
            let mut block = sm4::cipher::Block::<Sm4>::clone_from_slice(chunk);
            cipher.encrypt_block(&mut block);
            chunk.copy_from_slice(&block);
        }

        general_purpose::STANDARD.encode(buf)
    }

    pub fn sm4_ecb_pkcs7_decrypt_base64(ciphertext_b64: &str, key_16: &[u8; 16]) -> Result<String, Hj212Error> {
        let cipher = Sm4::new_from_slice(key_16).map_err(|_| Hj212Error::InvalidEncryptedDataFormat)?;
        let mut buf = general_purpose::STANDARD
            .decode(ciphertext_b64.trim())
            .map_err(|_| Hj212Error::InvalidEncryptedDataFormat)?;
        if buf.len() % 16 != 0 {
            return Err(Hj212Error::InvalidEncryptedDataLength);
        }

        for chunk in buf.chunks_exact_mut(16) {
            let mut block = sm4::cipher::Block::<Sm4>::clone_from_slice(chunk);
            cipher.decrypt_block(&mut block);
            chunk.copy_from_slice(&block);
        }

        let pad = *buf.last().ok_or(Hj212Error::InvalidEncryptedDataFormat)? as usize;
        if pad == 0 || pad > 16 || pad > buf.len() {
            return Err(Hj212Error::InvalidEncryptedDataFormat);
        }
        if !buf[buf.len() - pad..].iter().all(|&b| b as usize == pad) {
            return Err(Hj212Error::InvalidEncryptedDataFormat);
        }
        buf.truncate(buf.len() - pad);
        String::from_utf8(buf).map_err(|_| Hj212Error::InvalidEncryptedDataFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct XorCipher(u8);
    impl BlockCipher16 for XorCipher {
        fn encrypt_block(&self, block: &mut [u8; 16]) {
            for b in block.iter_mut() {
                *b ^= self.0;
            }
        }
        fn decrypt_block(&self, block: &mut [u8; 16]) {
            for b in block.iter_mut() {
                *b ^= self.0;
            }
        }
    }

    #[test]
    fn wire_format_roundtrip() {
        let seg = EncryptedDataSegment {
            encrypted: (0u8..16u8).collect(),
            trailing_plaintext: "g=N".to_string(),
        };
        let wire = seg.to_wire_string();
        assert!(wire.starts_with('{') && wire.contains("0x00") && wire.contains("0x0F") && wire.ends_with("g=N"));
        let parsed = EncryptedDataSegment::parse_wire_string(&wire).unwrap();
        assert_eq!(parsed.encrypted, seg.encrypted);
        assert_eq!(parsed.trailing_plaintext, "g=N");
    }

    #[test]
    fn encrypt_decrypt_keeps_trailing_plaintext() {
        // 16 bytes encrypted + 3 bytes plaintext tail
        let plaintext = "1234567890ABCDEFxyz";
        let cipher = XorCipher(0x5A);
        let enc = encrypt_data_segment(plaintext, &cipher).unwrap();
        assert_eq!(enc.encrypted.len(), 16);
        assert_eq!(enc.trailing_plaintext, "xyz");
        let wire = enc.to_wire_string();
        let dec = decrypt_data_segment(&wire, &cipher).unwrap();
        assert_eq!(dec, plaintext);
    }

    #[test]
    fn parse_rejects_non_multiple_of_16() {
        let err = EncryptedDataSegment::parse_wire_string("{0x01,0x02}").unwrap_err();
        match err {
            Hj212Error::InvalidEncryptedDataLength => {}
            _ => panic!("unexpected error: {err:?}"),
        }
    }

    #[cfg(feature = "sm4")]
    #[test]
    fn appendix_h_sm4_auth_example_matches() {
        // From Appendix H.1.2 example:
        // user=testuser, pass=AezF0nZRs4kypuGO
        // key=msbE74gsiSHSEh5e
        // base64=LG/m7lsFU/taig+vbQsDJXy4NGAaE2j2lplPwhZM7iM=
        let plaintext = "testuser:AezF0nZRs4kypuGO";
        let key: [u8; 16] = *b"msbE74gsiSHSEh5e";
        let b64 = crate::crypto::sm4_auth::sm4_ecb_pkcs7_encrypt_base64(plaintext, &key);
        assert_eq!(b64, "LG/m7lsFU/taig+vbQsDJXy4NGAaE2j2lplPwhZM7iM=");
        let back = crate::crypto::sm4_auth::sm4_ecb_pkcs7_decrypt_base64(&b64, &key).unwrap();
        assert_eq!(back, plaintext);
    }
}
