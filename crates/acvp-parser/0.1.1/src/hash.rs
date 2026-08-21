use json::JsonValue;

use crate::{
    parser::{TestCase, TestGroupData, TestResult},
    util::TestType,
    AcvpError, AcvpResult,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SecureHash {
    algorithm: String,
    tcid: u32,
    test_type: TestType,
    res_json: JsonValue,
    pub msg: Vec<u8>,
}

impl TestCase for SecureHash {
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

        let msghex = crate::util::get_acvp_str("msg", &test)?;
        let msg = crate::util::hex2bin(&msghex)?;

        Ok(SecureHash {
            algorithm: tgdata.algorithm.to_string(),
            tcid,
            test_type: tgdata.test_type,
            res_json: JsonValue::new_object(),
            msg,
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

impl TestResult<Vec<u8>> for SecureHash {
    fn set_result(&mut self, res: Vec<u8>) -> AcvpResult<()> {
        self.set_aft_result(res);
        Ok(())
    }
}

impl TestResult<Vec<Vec<u8>>> for SecureHash {
    fn set_result(&mut self, res: Vec<Vec<u8>>) -> AcvpResult<()> {
        self.set_mct_result(res)
    }
}

impl SecureHash {
    fn set_aft_result(&mut self, md: Vec<u8>) {
        let mdhex = hex::encode(md).to_ascii_uppercase();
        self.res_json = json::object! {
            tcId: self.tcid,
            md: mdhex,
        };
    }

    fn set_mct_result(&mut self, mdvec: Vec<Vec<u8>>) -> AcvpResult<()> {
        let mut results = JsonValue::new_array();
        for md in mdvec {
            let mdhex = hex::encode(md).to_ascii_uppercase();
            let res = json::object! { md: mdhex };
            match results.push(res) {
                Ok(()) => {}
                Err(_e) => {
                    return Err(AcvpError {
                        code: -1,
                        message: "Unexpected failure pushing to JsonValue array".to_string(),
                    });
                }
            };
        }
        self.res_json = json::object! {
            tcId: self.tcid,
            resultsArray: results,
        };
        Ok(())
    }
}
