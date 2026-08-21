use json::JsonValue;

use crate::{AcvpError, AcvpResult};

const HASHES: &[&str; 11] = &[
    "SHA-1",
    "SHA-224",
    "SHA-256",
    "SHA-384",
    "SHA-512",
    "SHA3-224",
    "SHA3-256",
    "SHA3-384",
    "SHA3-512",
    "SHAKE-128",
    "SHAKE-256",
];

const BLKCIPHERS: &[&str; 6] = &[
    "ACVP-AES-CBC",
    "ACVP-AES-CTR",
    "ACVP-AES-ECB",
    "ACVP-AES-GCM",
    "ACVP-TDES-ECB",
    "ACVP-TDES-CBC",
];

const MACS: &[&str; 13] = &[
    "HMAC-SHA-1",
    "HMAC-SHA2-224",
    "HMAC-SHA2-256",
    "HMAC-SHA2-384",
    "HMAC-SHA2-512",
    "HMAC-SHA3-224",
    "HMAC-SHA3-256",
    "HMAC-SHA3-384",
    "HMAC-SHA3-512",
    "CMAC-AES",
    "CMAC-TDES",
    "ACVP-AES-GCM",
    "ACVP-AES-GMAC",
];

const RNGS: &[&str; 3] = &["hashDRBG", "ctrDRBG", "hmacDRBG"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcvpAlgorithm {
    Hash,
    MsgAuth,
    BlockCipher,
    Rng,
    Nil,
}

impl AcvpAlgorithm {
    pub fn alg_type(alg: &str) -> AcvpResult<Self> {
        if str_lookup(alg, HASHES) {
            return Ok(Self::Hash);
        }
        if str_lookup(alg, MACS) {
            return Ok(Self::MsgAuth);
        }
        if str_lookup(alg, BLKCIPHERS) {
            return Ok(Self::BlockCipher);
        }
        if str_lookup(alg, RNGS) {
            return Ok(Self::Rng);
        }
        Err(AcvpError {
            code: -libc::EINVAL,
            message: format!("Uknown type for algorithm '{}'", alg),
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TestType {
    AFT,
    CTR,
    MCT,
    LDT,
    Nil,
}

impl TestType {
    pub fn from_string(ttype: &str) -> AcvpResult<Self> {
        let test_type = match ttype {
            "AFT" => TestType::AFT,
            "CTR" => TestType::CTR,
            "MCT" => TestType::MCT,
            "LDT" => TestType::LDT,
            _ => {
                return Err(AcvpError {
                    code: -libc::EINVAL,
                    message: format!("Invalid test type '{}'", ttype),
                });
            }
        };
        Ok(test_type)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Encrypt,
    Decrypt,
    Generate,
    Verify,
    Nil,
}

impl Direction {
    pub fn from_string(direction: &str) -> AcvpResult<Self> {
        let dir = match direction {
            "encrypt" => Direction::Encrypt,
            "decrypt" => Direction::Decrypt,
            "gen" => Direction::Generate,
            "ver" => Direction::Verify,
            _ => {
                return Err(AcvpError {
                    code: -libc::EINVAL,
                    message: format!("Invalid direction '{}'", direction),
                });
            }
        };
        Ok(dir)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum IVMode {
    Internal,
    External,
    Nil,
}

impl IVMode {
    pub fn from_string(mode: &str) -> AcvpResult<Self> {
        let ivmode = match mode {
            "internal" => IVMode::Internal,
            "external" => IVMode::External,
            _ => {
                return Err(AcvpError {
                    code: -libc::EINVAL,
                    message: format!("Invalid IVGenMode '{}'", mode),
                });
            }
        };
        Ok(ivmode)
    }
}

fn str_lookup(key: &str, arr: &[&str]) -> bool {
    if let Some(_str) = arr.iter().find(|&s| *s == key) {
        return true;
    }
    false
}

pub fn get_acvp_str(key: &str, json: &JsonValue) -> AcvpResult<String> {
    let value = match json[key].as_str() {
        Some(val) => val,
        None => {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: format!("Failed to obtain str value associated with key '{}'", key,),
            });
        }
    };
    Ok(value.to_string())
}

pub fn get_acvp_u32(key: &str, json: &JsonValue) -> AcvpResult<u32> {
    let value = match json[key].as_u32() {
        Some(val) => val,
        None => {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: format!("Failed to obtain u32 value associated with key '{}'", key,),
            });
        }
    };
    Ok(value)
}

pub fn get_acvp_bool(key: &str, json: &JsonValue) -> AcvpResult<bool> {
    let value = match json[key].as_bool() {
        Some(val) => val,
        None => {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: format!(
                    "Failed to obtain boolean value associated with key '{}'",
                    key,
                ),
            });
        }
    };
    Ok(value)
}

pub(crate) fn hex2bin(hex: &str) -> AcvpResult<Vec<u8>> {
    let bin = match hex::decode(hex) {
        Ok(bin) => bin,
        Err(_e) => {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: format!("Invalid hex string '{}'", hex),
            });
        }
    };
    Ok(bin)
}

pub fn get_algorithm_type(vector: &str) -> AcvpResult<AcvpAlgorithm> {
    let vec = match json::parse(vector) {
        Ok(vec) => vec,
        Err(_e) => {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: "Invalid input vector, failed to get algorithm type".to_string(),
            });
        }
    };

    let mut alg_type = AcvpAlgorithm::Nil;
    for v in vec.members() {
        if v.has_key("algorithm") {
            let alg = get_acvp_str("algorithm", v)?;
            alg_type = AcvpAlgorithm::alg_type(&alg)?;
        }
    }
    if alg_type == AcvpAlgorithm::Nil {
        return Err(AcvpError {
            code: -libc::EINVAL,
            message: "No 'algorithm' key present in input vector".to_string(),
        });
    }

    Ok(alg_type)
}
