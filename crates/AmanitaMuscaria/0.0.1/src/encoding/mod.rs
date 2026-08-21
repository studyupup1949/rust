use hex::*;
use base64::*;
use blake2_rfc::blake2b::*;

pub struct EncodingAPI;

impl EncodingAPI {
    pub fn encode_into_hex<T: AsRef<[u8]>>(input: T) -> String {
        return hex::encode_upper(input.as_ref())
    }
    pub fn decode_from_hex<T: AsRef<str>>(s: T) -> Vec<u8> {
        return hex::decode(s.as_ref()).expect("[Error] Failed To Decode From Hex");
    }
    pub fn decode_from_base64<T: AsRef<str>>(s: T) -> Vec<u8> {
        return base64::decode(s.as_ref()).expect("[Error] Failed To Decode From Base64");
    }
    pub fn encode_into_base64<T: AsRef<[u8]>>(s: T) -> String {
        return base64::encode(s.as_ref())
    }
}

pub struct HashEncoderAPI;

impl HashEncoderAPI {
    pub fn return_48_byte_blake2b_hash_as_hex<T: AsRef<[u8]>>(input: T) -> String {
        let hash = blake2b(48, &[], input.as_ref());
        return EncodingAPI::encode_into_hex(hash)
    }
    pub fn return_48_byte_blake2b_hash_with_key_as_hex<T: AsRef<[u8]>>(input: T, key: T) -> String {
        let hash = blake2b(48, key.as_ref(), input.as_ref());
        return EncodingAPI::encode_into_hex(hash)
    }
}