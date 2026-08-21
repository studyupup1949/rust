use json::JsonValue;

use crate::{
    parser::{TestCase, TestResult},
    util::TestType,
    AcvpError, AcvpResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrbgMode {
    SHA1,
    SHA256,
    SHA384,
    SHA512,
    AES128,
    AES192,
    AES256,
    Nil,
}

impl std::str::FromStr for DrbgMode {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SHA-1" => Ok(Self::SHA1),
            "SHA2-256" => Ok(Self::SHA256),
            "SHA2-384" => Ok(Self::SHA384),
            "SHA2-512" => Ok(Self::SHA512),
            "AES-128" => Ok(Self::AES128),
            "AES-192" => Ok(Self::AES192),
            "AES-256" => Ok(Self::AES256),
            _ => {
                return Err(AcvpError {
                    code: -libc::EINVAL,
                    message: format!("Mode '{}' is not valid", s),
                });
            }
        }
    }

    type Err = AcvpError;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DrbgOtherInput {
    pub iuse: String,
    pub addtl_input: Vec<u8>,
    pub entropy_input: Vec<u8>,
}

impl DrbgOtherInput {
    pub fn new(testjson: &JsonValue) -> AcvpResult<Self> {
        let iuse = crate::util::get_acvp_str("intendedUse", testjson)?;
        let addtlhex = crate::util::get_acvp_str("additionalInput", testjson)?;
        let addtl_input = crate::util::hex2bin(&addtlhex)?;

        let eihex = crate::util::get_acvp_str("entropyInput", testjson)?;
        let entropy_input = crate::util::hex2bin(&eihex)?;

        Ok(DrbgOtherInput {
            iuse,
            addtl_input,
            entropy_input,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Drbg {
    algorithm: String,
    tcid: u32,
    test_type: TestType,
    res_json: JsonValue,
    pub entropy_input: Vec<u8>,
    pub nonce: Vec<u8>,
    pub perso_string: Vec<u8>,
    pub other_input: Vec<DrbgOtherInput>,
}

impl TestCase for Drbg {
    fn new(test: &str, tgdata: &crate::parser::TestGroupData) -> AcvpResult<Self>
    where
        Self: Sized,
    {
        let testjson = match json::parse(test) {
            Ok(t) => t,
            Err(_e) => {
                return Err(AcvpError {
                    code: -libc::EINVAL,
                    message: String::from("Failed to parse JSON data for DRBG"),
                });
            }
        };

        let tcid = crate::util::get_acvp_u32("tcId", &testjson)?;

        let eihex = crate::util::get_acvp_str("entropyInput", &testjson)?;
        let entropy_input = crate::util::hex2bin(&eihex)?;

        let noncehex = crate::util::get_acvp_str("nonce", &testjson)?;
        let nonce = crate::util::hex2bin(&noncehex)?;

        let pshex = crate::util::get_acvp_str("persoString", &testjson)?;
        let perso_string = crate::util::hex2bin(&pshex)?;

        let oi = &testjson["otherInput"];

        if !oi.is_array() {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: String::from("other input for DRBG vector is not an array"),
            });
        }

        let mut other_input = Vec::new();
        for inp in oi.members() {
            other_input.push(DrbgOtherInput::new(inp)?);
        }

        Ok(Drbg {
            algorithm: tgdata.algorithm.to_string(),
            tcid,
            test_type: tgdata.test_type,
            res_json: JsonValue::new_object(),
            entropy_input,
            nonce,
            perso_string,
            other_input,
        })
    }

    fn get_result(&self) -> AcvpResult<JsonValue> {
        if self.res_json.is_empty() {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: "The result is not yet set, call set_*_result APIs".to_string(),
            });
        }
        Ok(self.res_json.clone())
    }

    fn dump_result(&self) -> AcvpResult<String> {
        if self.res_json.is_empty() {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: "The result is not yet set, call set_*_result APIs".to_string(),
            });
        }
        Ok(self.res_json.dump())
    }

    fn pretty_result(&self) -> AcvpResult<String> {
        if self.res_json.is_empty() {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: "The result is not yet set, call set_*_result APIs".to_string(),
            });
        }
        Ok(self.res_json.pretty(3))
    }
}

impl TestResult<Vec<u8>> for Drbg {
    fn set_result(&mut self, res: Vec<u8>) -> AcvpResult<()> {
        let rethex = hex::encode(res).to_ascii_uppercase();
        self.res_json = json::object! {
            tcId: self.tcid,
            returnedBits: rethex,
        };
        Ok(())
    }
}
