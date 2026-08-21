use std::str::FromStr;

use json::JsonValue;

use crate::{
    blkcipher::BlkCipherMCTOutput,
    drbg::DrbgMode,
    msgauth::MsgAuthOutput,
    util::{AcvpAlgorithm, Direction, IVMode, TestType},
    AcvpError, AcvpResult,
};

pub trait TestGroup {
    fn new(algorithm: &str, tgjson: &str) -> AcvpResult<Self>
    where
        Self: Sized;
    fn dump(&self) -> String;
    fn pretty(&self) -> String;
}

pub trait TestCase {
    fn new(test: &str, tgdata: &TestGroupData) -> AcvpResult<Self>
    where
        Self: Sized;
    fn get_result(&self) -> AcvpResult<JsonValue>;
    fn dump_result(&self) -> AcvpResult<String>;
    fn pretty_result(&self) -> AcvpResult<String>;
}

pub trait TestResult<T> {
    fn set_result(&mut self, res: T) -> AcvpResult<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestGroupData {
    pub algorithm: String,
    pub test_type: TestType,
    // For AEAD
    pub taglen: usize,
    pub payload_len: usize,
    pub ivmode: IVMode,
    pub ivlen: usize,
    // For SKCipher
    pub direction: Direction,
    // For AKCipher
    // pub hash_alg: String,
    // pub sig_type: String,
    // pub n: Vec<u8>,
    // pub e: Vec<u8>,
    // For DRBG
    pub drbgmode: DrbgMode,
    pub prediction_resistance: bool,
    pub reseed: bool,
    pub der_func: bool,
    pub returned_bits_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcvpTest<T> {
    pub tcid: u32,
    pub tgdata: TestGroupData,
    pub test: T,
    test_json: JsonValue,
}

impl<T: TestCase> TestCase for AcvpTest<T> {
    fn new(test: &str, tgdata: &TestGroupData) -> AcvpResult<Self> {
        let tc = match json::parse(test) {
            Ok(tc) => tc,
            Err(_e) => {
                return Err(AcvpError {
                    code: -libc::EINVAL,
                    message: "Failed to parse testcase JSON".to_string(),
                });
            }
        };

        if !tc.has_key("tcId") {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: "Required field tcID missing from testcase JSON".to_string(),
            });
        }
        let tcid = crate::util::get_acvp_u32("tcId", &tc)?;
        let test = T::new(test, tgdata)?;

        Ok(AcvpTest {
            tcid,
            tgdata: tgdata.clone(),
            test,
            test_json: tc,
        })
    }

    fn get_result(&self) -> AcvpResult<JsonValue> {
        self.test.get_result()
    }

    fn dump_result(&self) -> AcvpResult<String> {
        self.test.dump_result()
    }

    fn pretty_result(&self) -> AcvpResult<String> {
        self.test.pretty_result()
    }
}

impl<T: TestResult<bool>> TestResult<bool> for AcvpTest<T> {
    fn set_result(&mut self, res: bool) -> AcvpResult<()> {
        self.test.set_result(res)
    }
}

impl<T: TestResult<Vec<u8>>> TestResult<Vec<u8>> for AcvpTest<T> {
    fn set_result(&mut self, res: Vec<u8>) -> AcvpResult<()> {
        self.test.set_result(res)
    }
}

impl<T: TestResult<Vec<Vec<u8>>>> TestResult<Vec<Vec<u8>>> for AcvpTest<T> {
    fn set_result(&mut self, res: Vec<Vec<u8>>) -> AcvpResult<()> {
        self.test.set_result(res)
    }
}

impl<T: TestResult<Vec<BlkCipherMCTOutput>>> TestResult<Vec<BlkCipherMCTOutput>> for AcvpTest<T> {
    fn set_result(&mut self, res: Vec<BlkCipherMCTOutput>) -> AcvpResult<()> {
        self.test.set_result(res)
    }
}

impl<T: TestResult<MsgAuthOutput>> TestResult<MsgAuthOutput> for AcvpTest<T> {
    fn set_result(&mut self, res: MsgAuthOutput) -> AcvpResult<()> {
        self.test.set_result(res)
    }
}

impl<T: Clone + TestCase> AcvpTest<T> {
    pub fn get_test_data(&self) -> T {
        self.test.clone()
    }

    pub fn dump(&self) -> String {
        self.test_json.dump()
    }

    pub fn pretty(&self) -> String {
        self.test_json.pretty(3)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcvpTestGroup<T> {
    test_type: TestType,
    tgid: u32,
    pub tests: Vec<AcvpTest<T>>,
    testgroup_json: JsonValue,
}

impl<T: TestCase> TestGroup for AcvpTestGroup<T> {
    fn new(algorithm: &str, tgjson: &str) -> AcvpResult<Self> {
        let tg = match json::parse(tgjson) {
            Ok(tg) => tg,
            Err(_e) => {
                return Err(AcvpError {
                    code: -libc::EINVAL,
                    message: "Failed to parse testgroup JSON".to_string(),
                });
            }
        };
        if !tg.has_key("tgId") || !tg.has_key("testType") {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: "Provided testgroup JSON does not have required fields".to_string(),
            });
        }
        let tgid = crate::util::get_acvp_u32("tgId", &tg)?;
        let test_type = TestType::from_string(&crate::util::get_acvp_str("testType", &tg)?)?;

        let mut direction = Direction::Nil;
        if tg.has_key("direction") {
            direction = Direction::from_string(&crate::util::get_acvp_str("direction", &tg)?)?;
        }

        let mut taglen = 0;
        if tg.has_key("tagLen") {
            let taglen_bits = crate::util::get_acvp_u32("tagLen", &tg)? as usize;
            taglen = taglen_bits / 8;
        } else if tg.has_key("macLen") {
            let taglen_bits = crate::util::get_acvp_u32("macLen", &tg)? as usize;
            taglen = taglen_bits / 8;
        }

        let mut payload_len = 0;
        if tg.has_key("payloadLen") {
            let payloadlen_bits = crate::util::get_acvp_u32("payloadLen", &tg)? as usize;
            payload_len = payloadlen_bits / 8;
        }

        let mut ivmode = IVMode::Nil;
        if tg.has_key("ivGen") {
            let ivmode_str = crate::util::get_acvp_str("ivGen", &tg)?;
            ivmode = IVMode::from_string(&ivmode_str)?;
        }

        let mut ivlen = 0;
        if tg.has_key("ivLen") {
            let ivlen_bits = crate::util::get_acvp_u32("ivLen", &tg)? as usize;
            ivlen = ivlen_bits / 8;
        }

        let mut drbgmode = DrbgMode::Nil;
        if tg.has_key("mode") {
            let mode = crate::util::get_acvp_str("mode", &tg)?;
            drbgmode = DrbgMode::from_str(&mode)?;
        }

        let mut prediction_resistance = false;
        if tg.has_key("predResistance") {
            prediction_resistance = crate::util::get_acvp_bool("predResistance", &tg)?;
        }

        let mut der_func = false;
        if tg.has_key("derFunc") {
            der_func = crate::util::get_acvp_bool("derFunc", &tg)?;
        }

        let mut reseed = false;
        if tg.has_key("reSeed") {
            reseed = crate::util::get_acvp_bool("reSeed", &tg)?;
        }

        let mut returned_bits_len: usize = 0;
        if tg.has_key("returnedBitsLen") {
            let returned_bits = crate::util::get_acvp_u32("returnedBitsLen", &tg)? as usize;
            returned_bits_len = returned_bits / 8;
        }

        let tgdata = TestGroupData {
            algorithm: algorithm.to_string(),
            test_type,
            taglen,
            payload_len,
            ivmode,
            ivlen,
            direction,
            drbgmode,
            prediction_resistance,
            der_func,
            reseed,
            returned_bits_len,
        };

        let tcs = &tg["tests"];
        let mut tests = Vec::new();
        for tc in tcs.members() {
            let test = AcvpTest::<T>::new(&tc.dump(), &tgdata)?;
            tests.push(test)
        }

        Ok(AcvpTestGroup {
            test_type,
            tgid,
            tests,
            testgroup_json: tg,
        })
    }

    fn dump(&self) -> String {
        self.testgroup_json.dump()
    }

    fn pretty(&self) -> String {
        self.testgroup_json.pretty(3)
    }
}

impl<T: TestCase> AcvpTestGroup<T> {
    pub fn get_result(&self) -> AcvpResult<JsonValue> {
        let mut results = JsonValue::new_array();
        for test in &self.tests {
            let res = test.get_result()?;
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
        Ok(json::object! {
            tgId: self.tgid,
            tests: results,
        })
    }

    pub fn dump_result(&self) -> AcvpResult<String> {
        let res = self.get_result()?;
        Ok(res.dump())
    }

    pub fn pretty_result(&self) -> AcvpResult<String> {
        let res = self.get_result()?;
        Ok(res.pretty(3))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcvpRequest<T> {
    pub version: String,
    pub revision: String,
    pub vsid: u32,
    pub algorithm: String,
    pub alg_type: AcvpAlgorithm,
    pub is_sample: bool,
    pub testgroups: Vec<AcvpTestGroup<T>>,
    request_json: JsonValue,
}

impl<T: TestCase> AcvpRequest<T> {
    pub fn new(vector: &str) -> AcvpResult<Self> {
        let request = match json::parse(vector) {
            Ok(req) => req,
            Err(_e) => {
                return Err(AcvpError {
                    code: -libc::EINVAL,
                    message: "Invalid ACVP Request JSON".to_string(),
                });
            }
        };

        if !request.is_array() {
            return Err(AcvpError {
                code: -libc::EINVAL,
                message: "ACVP Request vector must be a JSON Array".to_string(),
            });
        }

        let mut testgroups = Vec::new();
        let mut algorithm = "".to_string();
        let mut alg_type = AcvpAlgorithm::Nil;
        let mut revision = "".to_string();
        let mut vsid = 0;
        let mut is_sample = false;
        let mut version = "".to_string();
        for req in request.members() {
            if req.has_key("acvVersion") {
                version = crate::util::get_acvp_str("acvVersion", req)?;
                continue;
            }
            algorithm = crate::util::get_acvp_str("algorithm", req)?;
            alg_type = AcvpAlgorithm::alg_type(&algorithm)?;
            revision = crate::util::get_acvp_str("revision", req)?;
            vsid = crate::util::get_acvp_u32("vsId", req)?;
            is_sample = crate::util::get_acvp_bool("isSample", req)?;
            let tgs = &req["testGroups"];

            for tg in tgs.members() {
                let testgroup = AcvpTestGroup::<T>::new(&algorithm, &tg.dump())?;
                testgroups.push(testgroup);
            }
        }
        Ok(AcvpRequest {
            version,
            revision,
            vsid,
            algorithm,
            alg_type,
            is_sample,
            testgroups,
            request_json: request,
        })
    }

    pub fn get_result(&self) -> AcvpResult<JsonValue> {
        let mut results = JsonValue::new_array();
        for tg in &self.testgroups {
            let res = tg.get_result()?;
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
        let vers = json::object! {
            acvVersion: self.version.clone()
        };
        let resp = json::object! {
            vsId: self.vsid,
            algorithm: self.algorithm.clone(),
            revision: self.revision.clone(),
            isSample: self.is_sample,
            testGroups: results,
        };
        Ok(json::array![vers, resp])
    }

    pub fn dump_result(&self) -> AcvpResult<String> {
        let res = self.get_result()?;
        Ok(res.dump())
    }

    pub fn pretty_result(&self) -> AcvpResult<String> {
        let res = self.get_result()?;
        Ok(res.pretty(3))
    }

    pub fn dump(&self) -> String {
        self.request_json.dump()
    }

    pub fn pretty(&self) -> String {
        self.request_json.pretty(3)
    }
}
