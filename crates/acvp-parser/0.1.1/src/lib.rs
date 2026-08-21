use std::fmt;

pub type AcvpResult<T> = std::result::Result<T, AcvpError>;

#[derive(Debug, Clone)]
pub struct AcvpError {
    pub code: i32,
    pub message: String,
}

impl fmt::Display for AcvpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", &self.message, &self.code)
    }
}

pub mod blkcipher;
pub mod drbg;
pub mod hash;
pub mod msgauth;
pub mod parser;
pub mod util;

#[cfg(test)]
#[test]
fn test_init() {
    use hash::SecureHash;

    use crate::parser::{AcvpTest, TestCase, TestGroupData, TestResult};
    let tgdata = TestGroupData {
        algorithm: "sha1".to_string(),
        test_type: util::TestType::AFT,
        taglen: 0,
        payload_len: 0,
        ivmode: util::IVMode::Nil,
        ivlen: 0,
        direction: util::Direction::Nil,
        drbgmode: drbg::DrbgMode::Nil,
        prediction_resistance: false,
        der_func: false,
        reseed: false,
        returned_bits_len: 0,
    };

    let mut t =
        AcvpTest::<SecureHash>::new(r#"{ "tcId": 10, "msg": "abcdef" }"#, &tgdata).expect("Failed");
    t.set_result(vec![0xa, 0xb, 0xc, 0xd, 0xe, 0xf])
        .expect("Failed to set result");
    println!("{}", t.pretty_result().expect("Failed dump"));
}
