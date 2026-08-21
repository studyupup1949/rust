use fancy_regex::Regex;

#[derive(Debug)]
pub struct UserPassword(String);

impl UserPassword {
    pub fn parse(s: String) -> Result<UserPassword, String> {
        // The following regex ensures at least one lowercase, uppercase, number,
        // and symbol exist in 8+ character length password
        let re =
            Regex::new(r"^(?=\P{Ll}*\p{Ll})(?=\P{Lu}*\p{Lu})(?=\P{N}*\p{N})(?=[\p{L}\p{N}]*[^\p{L}\p{N}])[\s\S]{8,}$").unwrap();

        if re.is_match(&s).unwrap() {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid password.", s))
        }
    }
}

impl AsRef<str> for UserPassword {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::UserPassword;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_valid_password_is_accepted() {
        let password = "AcidHouse@303".to_string();
        assert_ok!(UserPassword::parse(password));
    }
    #[test]
    fn a_password_shorter_than_8_characters_is_rejected() {
        let password = "abcdefg".to_string();
        assert_err!(UserPassword::parse(password));
    }
    #[test]
    fn a_password_missing_uppercase_letter_rejected() {
        let password = "acidhouse@303".to_string();
        assert_err!(UserPassword::parse(password));
    }
    #[test]
    fn a_password_missing_lowercase_letter_is_rejected() {
        let password = "ACIDHOUSE@303".to_string();
        assert_err!(UserPassword::parse(password));
    }
    #[test]
    fn a_password_missing_symbol_is_rejected() {
        let password = "AcidHouse303".to_string();
        assert_err!(UserPassword::parse(password));
    }
}
