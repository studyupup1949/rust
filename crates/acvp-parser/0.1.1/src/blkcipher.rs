use json::JsonValue;

use crate::{
    parser::{TestCase, TestGroupData, TestResult},
    util::{Direction, TestType},
    AcvpError, AcvpResult,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BlockCipher {
    algorithm: String,
    test_type: TestType,
    direction: Direction,
    tcid: u32,
    res_json: JsonValue,
    pub iv: Vec<u8>,
    pub input: Vec<u8>,
    pub key: Vec<u8>,
    pub key1: Vec<u8>,
    pub key2: Vec<u8>,
    pub key3: Vec<u8>,
}

impl TestCase for BlockCipher {
    fn new(testjson: &str, tgdata: &TestGroupData) -> AcvpResult<Self> {
        let test = match json::parse(testjson) {
            Ok(test) => test,
            Err(_e) => {
                return Err(AcvpError {
                    code: -libc::EINVAL,
                    message: "Failed to parse testcase JSON for SKCipher".to_string(),
                });
            }
        };
        let tcid = crate::util::get_acvp_u32("tcId", &test)?;

        let mut iv = Vec::new();
        if test.has_key("iv") {
            let ivhex = crate::util::get_acvp_str("iv", &test)?;
            iv = crate::util::hex2bin(&ivhex)?;
        } else if test.has_key("tweakValue") {
            let ivhex = crate::util::get_acvp_str("tweakValue", &test)?;
            iv = crate::util::hex2bin(&ivhex)?;
        }

        let mut key = Vec::new();
        let mut key1 = Vec::new();
        let mut key2 = Vec::new();
        let mut key3 = Vec::new();
        if test.has_key("key1") && test.has_key("key2") && test.has_key("key3") {
            let key1hex = crate::util::get_acvp_str("key1", &test)?;
            key1 = crate::util::hex2bin(&key1hex)?;
            key.extend(key1.iter().copied());

            let key2hex = crate::util::get_acvp_str("key2", &test)?;
            key2 = crate::util::hex2bin(&key2hex)?;
            key.extend(key2.iter().copied());

            let key3hex = crate::util::get_acvp_str("key3", &test)?;
            key3 = crate::util::hex2bin(&key3hex)?;
            key.extend(key3.iter().copied());
        } else if test.has_key("key") {
            let keyhex = crate::util::get_acvp_str("key", &test)?;
            key = crate::util::hex2bin(&keyhex)?;
        }

        let inphex = match tgdata.direction {
            Direction::Encrypt => crate::util::get_acvp_str("pt", &test)?,
            Direction::Decrypt => crate::util::get_acvp_str("ct", &test)?,
            _ => {
                return Err(AcvpError {
                    code: -libc::EINVAL,
                    message: "Invalid direction for block cipher operaion".to_string(),
                });
            }
        };
        let input = crate::util::hex2bin(&inphex)?;

        Ok(BlockCipher {
            algorithm: tgdata.algorithm.to_string(),
            test_type: tgdata.test_type,
            direction: tgdata.direction,
            tcid,
            res_json: JsonValue::new_object(),
            iv,
            input,
            key,
            key1,
            key2,
            key3,
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

impl TestResult<Vec<u8>> for BlockCipher {
    fn set_result(&mut self, res: Vec<u8>) -> AcvpResult<()> {
        self.set_aft_result(res);
        Ok(())
    }
}

impl TestResult<Vec<BlkCipherMCTOutput>> for BlockCipher {
    fn set_result(&mut self, res: Vec<BlkCipherMCTOutput>) -> AcvpResult<()> {
        self.set_mct_result(res)
    }
}

#[derive(Debug, Clone)]
pub struct BlkCipherMCTOutput {
    pub key: Vec<u8>,
    pub key1: Vec<u8>,
    pub key2: Vec<u8>,
    pub key3: Vec<u8>,
    pub iv: Vec<u8>,
    pub inp: Vec<u8>,
    pub out: Vec<u8>,
}

impl BlkCipherMCTOutput {
    pub fn new_aes(key: Vec<u8>, iv: Vec<u8>, inp: Vec<u8>, out: Vec<u8>) -> Self {
        BlkCipherMCTOutput {
            key,
            key1: Vec::new(),
            key2: Vec::new(),
            key3: Vec::new(),
            iv,
            inp,
            out,
        }
    }

    pub fn new_tdes(
        key1: Vec<u8>,
        key2: Vec<u8>,
        key3: Vec<u8>,
        iv: Vec<u8>,
        inp: Vec<u8>,
        out: Vec<u8>,
    ) -> Self {
        BlkCipherMCTOutput {
            key: Vec::new(),
            key1,
            key2,
            key3,
            iv,
            inp,
            out,
        }
    }
}

impl BlockCipher {
    pub fn set_aft_result(&mut self, out: Vec<u8>) {
        let mut res = JsonValue::new_object();
        res["tcId"] = self.tcid.into();
        match self.direction {
            Direction::Decrypt => {
                res["pt"] = hex::encode(out).to_ascii_uppercase().into();
            }
            Direction::Encrypt => {
                res["ct"] = hex::encode(out).to_ascii_uppercase().into();
            }
            _ => {}
        }
        self.res_json = res;
    }

    pub fn set_mct_result(&mut self, outvec: Vec<BlkCipherMCTOutput>) -> AcvpResult<()> {
        let mut results = JsonValue::new_array();
        for out in outvec {
            let mut res = JsonValue::new_object();
            if self.algorithm.contains("AES") {
                res["key"] = hex::encode(out.key).to_ascii_uppercase().into();
            }
            if !out.iv.is_empty() {
                res["iv"] = hex::encode(out.iv).to_ascii_uppercase().into();
            }
            match self.direction {
                Direction::Decrypt => {
                    res["pt"] = hex::encode(out.out).to_ascii_uppercase().into();
                    res["ct"] = hex::encode(out.inp).to_ascii_uppercase().into();
                }
                Direction::Encrypt => {
                    res["pt"] = hex::encode(out.inp).to_ascii_uppercase().into();
                    res["ct"] = hex::encode(out.out).to_ascii_uppercase().into();
                }
                _ => {}
            }
            if self.algorithm.contains("TDES") {
                res["key1"] = hex::encode(out.key1).to_ascii_uppercase().into();
                res["key2"] = hex::encode(out.key2).to_ascii_uppercase().into();
                res["key3"] = hex::encode(out.key3).to_ascii_uppercase().into();
            }
            match results.push(res) {
                Ok(()) => {}
                Err(_e) => {
                    return Err(AcvpError {
                        code: -1,
                        message: "Unexpected error pushing to JsonValue array".to_string(),
                    });
                }
            }
        }
        self.res_json = json::object! {
            tcId: self.tcid,
            resultsArray: results
        };
        Ok(())
    }
}
