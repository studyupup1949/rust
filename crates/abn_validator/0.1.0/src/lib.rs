//! Validates the format of Australian Business Numbers (ABNs).
//!
//! An ABN is an 11-digit number: 9 identifying digits with two leading
//! check digits. Validity is determined purely by a checksum over the
//! digits — this crate does not check whether an ABN has actually been
//! issued by the Australian Business Register, only that it is
//! well-formed per the published algorithm:
//! <https://abr.business.gov.au/Help/AbnFormat>.

use std::fmt;

/// Per-position weighting factors used in the checksum, left to right.
const WEIGHTS: [u32; 11] = [10, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];

/// The checksum modulus. A well-formed ABN's weighted digit sum is always
/// a multiple of this value.
const MODULUS: i64 = 89;

/// Why a candidate ABN failed validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbnError {
    /// A character was found that is neither an ASCII digit nor whitespace.
    InvalidCharacter(char),
    /// The input did not contain exactly 11 digits once whitespace
    /// (conventionally used to group digits, e.g. `"51 824 753 556"`) was
    /// removed.
    InvalidLength(usize),
    /// The input had 11 digits but failed the ABN check-digit algorithm.
    ChecksumMismatch,
}

impl fmt::Display for AbnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter(c) => {
                write!(
                    f,
                    "ABN contains a non-digit, non-whitespace character: {c:?}"
                )
            }
            Self::InvalidLength(len) => {
                write!(f, "ABN must contain exactly 11 digits, found {len}")
            }
            Self::ChecksumMismatch => write!(f, "ABN failed check-digit validation"),
        }
    }
}

impl std::error::Error for AbnError {}

/// Validates that `input` is a well-formed Australian Business Number.
///
/// Whitespace is ignored, since ABNs are conventionally displayed in
/// groups (e.g. `"51 824 753 556"`). Any other non-digit character is
/// rejected.
///
/// # Errors
///
/// Returns [`AbnError::InvalidCharacter`] if `input` contains a character
/// that is not an ASCII digit or whitespace, [`AbnError::InvalidLength`]
/// if it does not contain exactly 11 digits, or
/// [`AbnError::ChecksumMismatch`] if the digits fail the ABN check-digit
/// algorithm.
///
/// ```
/// assert!(abn_validator::validate("51 824 753 556").is_ok());
/// assert!(abn_validator::validate("51824753556").is_ok());
/// assert_eq!(
///     abn_validator::validate("51824753557"),
///     Err(abn_validator::AbnError::ChecksumMismatch),
/// );
/// ```
pub fn validate(input: &str) -> Result<(), AbnError> {
    let digits = parse_digits(input)?;

    let weighted_sum: i64 = digits
        .iter()
        .zip(WEIGHTS.iter())
        .map(|(&digit, &weight)| i64::from(digit) * i64::from(weight))
        .sum::<i64>()
        - i64::from(WEIGHTS[0]);

    if weighted_sum % MODULUS == 0 {
        Ok(())
    } else {
        Err(AbnError::ChecksumMismatch)
    }
}

/// Extracts exactly 11 digits from `input`, ignoring whitespace.
fn parse_digits(input: &str) -> Result<[u32; 11], AbnError> {
    let mut digits = [0u32; 11];
    let mut count = 0usize;

    for c in input.chars() {
        if c.is_whitespace() {
            continue;
        }
        let digit = c.to_digit(10).ok_or(AbnError::InvalidCharacter(c))?;
        if let Some(slot) = digits.get_mut(count) {
            *slot = digit;
        }
        count += 1;
    }

    if count == 11 {
        Ok(digits)
    } else {
        Err(AbnError::InvalidLength(count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ABN of Australian Taxation Office (referenced in spec)
    const ATO_ABN: &str = "51824753556";

    #[test]
    fn accepts_known_valid_abn() {
        assert_eq!(validate(ATO_ABN), Ok(()));
    }

    #[test]
    fn ignores_grouping_whitespace() {
        assert_eq!(validate("51 824 753 556"), Ok(()));
    }

    #[test]
    fn ignores_all_whitespace() {
        assert_eq!(validate("51\t824\n753\u{2003}556"), Ok(()));
    }

    #[test]
    fn rejects_empty_and_whitespace_only_input() {
        for input in ["", " \t\n\u{2003}"] {
            assert_eq!(validate(input), Err(AbnError::InvalidLength(0)));
        }
    }

    #[test]
    fn rejects_wrong_check_digits() {
        assert_eq!(validate("51824753557"), Err(AbnError::ChecksumMismatch));
    }

    #[test]
    fn rejects_too_short() {
        assert_eq!(validate("1234567890"), Err(AbnError::InvalidLength(10)));
    }

    #[test]
    fn rejects_too_long() {
        assert_eq!(validate("123456789012"), Err(AbnError::InvalidLength(12)));
    }

    #[test]
    fn rejects_non_digit_character() {
        assert_eq!(
            validate("5182475355X"),
            Err(AbnError::InvalidCharacter('X'))
        );
    }

    #[test]
    fn rejects_punctuation_used_as_grouping() {
        assert_eq!(
            validate("51-824-753-556"),
            Err(AbnError::InvalidCharacter('-'))
        );
    }

    #[test]
    fn rejects_non_ascii_digits() {
        assert_eq!(
            validate("٥1824753556"),
            Err(AbnError::InvalidCharacter('٥'))
        );
    }

    #[test]
    fn error_messages_are_descriptive() {
        assert_eq!(
            AbnError::InvalidCharacter('X').to_string(),
            "ABN contains a non-digit, non-whitespace character: 'X'"
        );
        assert_eq!(
            AbnError::InvalidLength(10).to_string(),
            "ABN must contain exactly 11 digits, found 10"
        );
        assert_eq!(
            AbnError::ChecksumMismatch.to_string(),
            "ABN failed check-digit validation"
        );
    }
}

#[cfg(test)]
mod quickcheck_tests {
    use super::*;
    use quickcheck::{QuickCheck, TestResult};

    fn find_valid_abn(identifier_digits: &str) -> Option<String> {
        (0..10)
            .flat_map(|d0| (0..10).map(move |d1| (d0, d1)))
            .map(|(d0, d1)| format!("{d0}{d1}{identifier_digits}"))
            .find(|candidate| validate(candidate).is_ok())
    }

    #[test]
    fn single_digit_change_always_invalidates_a_valid_abn() {
        fn property(identifier: u32, position: u8, step: u8) -> TestResult {
            let identifier_digits = format!("{:09}", identifier % 1_000_000_000);
            let Some(valid_abn) = find_valid_abn(&identifier_digits) else {
                return TestResult::discard();
            };

            let position = usize::from(position) % valid_abn.len();
            let step = u32::from(step) % 9 + 1;

            let mut digits = valid_abn.into_bytes();
            let original = u32::from(digits[position] - b'0');
            digits[position] = b'0' + u8::try_from((original + step) % 10).expect("< 10");
            let mutated = String::from_utf8(digits).expect("digit bytes are always valid UTF-8");

            TestResult::from_bool(validate(&mutated) == Err(AbnError::ChecksumMismatch))
        }

        QuickCheck::new().quickcheck(property as fn(u32, u8, u8) -> TestResult);
    }
}
