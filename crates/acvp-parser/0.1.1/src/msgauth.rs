use json::JsonValue;
use rand::Rng;

use crate::{
    parser::{TestCase, TestGroupData, TestResult},
    util::{Direction, IVMode, TestType},
    AcvpError, AcvpResult,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MsgAuth {
    algorithm: String,
    tcid: u32,
    test_type: TestType,
    direction: Direction,
    res_json: JsonValue,
    pub key: Vec<u8>,
    pub key1: Vec<u8>,
    pub key2: Vec<u8>,
    pub key3: Vec<u8>,
    pub iv: Vec<u8>,
    pub ivmode: IVMode,
    pub msg: Vec<u8>,
    pub aad: Vec<u8>,
    pub tag: Vec<u8>,
    pub taglen: usize,
}

impl TestCase for MsgAuth {
    fn new(testjson: &str, tgdata: &TestGroupData) -> AcvpResult<Self> {
        let test = match json::parse(testjson) {
            Ok(test) => test,
            Err(_e) => {
                return Err(AcvpError {
                    code: -libc::EINVAL,
                    message: "Failed to parse testcase JSON for SHash".to_string(),
                });
            }
        };
        let tcid = crate::util::get_acvp_u32("tcId", &test)?;

        let mut msg = Vec::new();
        if test.has_key("msg") {
            let msghex = crate::util::get_acvp_str("msg", &test)?;
            msg = crate::util::hex2bin(&msghex)?;
        } else if test.has_key("message") {
            let msghex = crate::util::get_acvp_str("message", &test)?;
            msg = crate::util::hex2bin(&msghex)?;
        } else if test.has_key("plainText") {
            let msghex = crate::util::get_acvp_str("plainText", &test)?;
            msg = crate::util::hex2bin(&msghex)?;
        } else if test.has_key("cipherText") {
            let msghex = crate::util::get_acvp_str("cipherText", &test)?;
            msg = crate::util::hex2bin(&msghex)?;
        } else if test.has_key("pt") {
            let msghex = crate::util::get_acvp_str("pt", &test)?;
            msg = crate::util::hex2bin(&msghex)?;
        } else if test.has_key("ct") {
            let msghex = crate::util::get_acvp_str("ct", &test)?;
            msg = crate::util::hex2bin(&msghex)?;
        }

        let mut aad = Vec::new();
        if test.has_key("aad") {
            let aadhex = crate::util::get_acvp_str("aad", &test)?;
            aad = crate::util::hex2bin(&aadhex)?;
        }

        let mut key = Vec::new();
        if test.has_key("key") {
            let keyhex = crate::util::get_acvp_str("key", &test)?;
            key = crate::util::hex2bin(&keyhex)?;
        }

        let mut key1 = Vec::new();
        let mut key2 = Vec::new();
        let mut key3 = Vec::new();
        if test.has_key("key1") && test.has_key("key2") && test.has_key("key3") {
            let keyhex = crate::util::get_acvp_str("key1", &test)?;
            key1 = crate::util::hex2bin(&keyhex)?;
            let keyhex = crate::util::get_acvp_str("key2", &test)?;
            key2 = crate::util::hex2bin(&keyhex)?;
            let keyhex = crate::util::get_acvp_str("key3", &test)?;
            key3 = crate::util::hex2bin(&keyhex)?;
        }

        let mut iv = Vec::new();
        if test.has_key("iv") {
            let ivhex = crate::util::get_acvp_str("iv", &test)?;
            iv = crate::util::hex2bin(&ivhex)?;
        } else if tgdata.ivmode == IVMode::Internal {
            let mut rng = rand::thread_rng();
            iv = (0..tgdata.ivlen).map(|_| rng.gen_range(0..255)).collect();
        }

        let mut tag = Vec::new();
        if test.has_key("tag") {
            let taghex = crate::util::get_acvp_str("tag", &test)?;
            tag = crate::util::hex2bin(&taghex)?;
        } else if test.has_key("mac") {
            let taghex = crate::util::get_acvp_str("mac", &test)?;
            tag = crate::util::hex2bin(&taghex)?;
        }

        // We need to special case here where the payload may be empty
        // and the taglength will be set. This issue is seen in ACVP-AES-CCM which
        // does not specify the tag separately in decryption vectors.
        if tag.is_empty() && tgdata.direction == Direction::Decrypt {
            if tgdata.payload_len == 0 {
                tag = msg.clone();
                msg = Vec::new();
            } else {
                tag = msg[tgdata.payload_len..].to_vec();
                msg = msg[..tgdata.payload_len].to_vec();
            }
        }

        Ok(MsgAuth {
            algorithm: tgdata.algorithm.to_string(),
            tcid,
            test_type: tgdata.test_type,
            direction: tgdata.direction,
            res_json: JsonValue::new_object(),
            key,
            key1,
            key2,
            key3,
            iv,
            ivmode: tgdata.ivmode,
            msg,
            aad,
            tag,
            taglen: tgdata.taglen,
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
                message: "The result is not yet set, call set_result API".to_string(),
            });
        }
        Ok(self.res_json.dump())
    }

    fn pretty_result(&self) -> AcvpResult<String> {
        if self.res_json.is_empty() {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: "The result is not yet set, call set_result API".to_string(),
            });
        }
        Ok(self.res_json.pretty(3))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MsgAuthOutput {
    pub out: Vec<u8>,
    pub tag: Vec<u8>,
}

impl MsgAuthOutput {
    pub fn new(out: Vec<u8>, tag: Vec<u8>) -> Self {
        MsgAuthOutput { out, tag }
    }
}

impl TestResult<Vec<u8>> for MsgAuth {
    fn set_result(&mut self, result: Vec<u8>) -> AcvpResult<()> {
        let mut res = JsonValue::new_object();
        res["tcId"] = self.tcid.into();
        if self.algorithm.contains("HMAC")
            || self.algorithm.contains("TDES")
            || self.algorithm.contains("CMAC")
        {
            res["mac"] = hex::encode(result).to_ascii_uppercase().into();
        } else if self.algorithm.contains("GMAC") {
            res["tag"] = hex::encode(result).to_ascii_uppercase().into();
        } else if self.algorithm.contains("GCM") {
            res["pt"] = hex::encode(result).to_ascii_uppercase().into();
        } else {
            match self.direction {
                Direction::Decrypt | Direction::Verify => {
                    res["pt"] = hex::encode(result).to_ascii_uppercase().into();
                }
                Direction::Encrypt | Direction::Generate => {
                    res["ct"] = hex::encode(result).to_ascii_uppercase().into();
                }
                Direction::Nil => {
                    return Err(AcvpError {
                        code: -libc::EINVAL,
                        message: "Invalid direction for MsgAuth algorithm".to_string(),
                    });
                }
            };
        }
        self.res_json = res;
        Ok(())
    }
}

impl TestResult<MsgAuthOutput> for MsgAuth {
    fn set_result(&mut self, result: MsgAuthOutput) -> AcvpResult<()> {
        let mut res = JsonValue::new_array();
        res["tcId"] = self.tcid.into();
        if self.ivmode == IVMode::Internal {
            res["iv"] = hex::encode(&self.iv).to_ascii_uppercase().into();
        }
        if self.algorithm.contains("CCM") {
            let mut ct = result.out;
            ct.extend(result.tag.iter());
            res["ct"] = hex::encode(ct).to_ascii_uppercase().into();
        } else {
            res["ct"] = hex::encode(result.out).to_ascii_uppercase().into();
            res["tag"] = hex::encode(result.tag).to_ascii_uppercase().into();
        }
        self.res_json = res;
        Ok(())
    }
}

impl TestResult<bool> for MsgAuth {
    fn set_result(&mut self, result: bool) -> AcvpResult<()> {
        self.res_json = json::object! {
            tcId: self.tcid,
            testPassed: result,
        };
        Ok(())
    }
}
