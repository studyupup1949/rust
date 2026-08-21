#[derive(Debug)]
pub struct UserName(String);

impl UserName {
    pub fn parse(s: String) -> Result<UserName, String> {
        let is_too_long = s.trim().chars().count() > 30;
        let is_too_short = s.trim().chars().count() < 2;
        let allowed_characters = ['_'];
        let has_valid_characters = s
            .trim()
            .chars()
            .all(|x| x.is_alphanumeric() || allowed_characters.contains(&x));

        if !has_valid_characters {
            return Err("Username contains invalid characters".to_string());
        }

        if is_too_long || is_too_short {
            return Err("Username length is incorrect".to_string());
        }

        Ok(Self(s.trim().to_lowercase()))
    }
}

impl AsRef<str> for UserName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::UserName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_30_character_long_name_is_valid() {
        let name = "a".repeat(30);
        assert_ok!(UserName::parse(name));
    }
    #[test]
    fn a_name_longer_than_30_characters_is_rejected() {
        let name = "a".repeat(31);
        assert_err!(UserName::parse(name));
    }
    #[test]
    fn a_name_shorter_than_2_characters_is_rejected() {
        let name = "a".to_string();
        assert_err!(UserName::parse(name));
    }
    #[test]
    fn a_name_containing_invalid_characters_is_rejected() {
        let name = "acid%house".to_string();
        assert_err!(UserName::parse(name));
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "acid_house_303".to_string();
        assert_ok!(UserName::parse(name));
    }
    #[test]
    fn a_name_containing_spaces_is_rejected() {
        let name = "acid house".to_string();
        assert_err!(UserName::parse(name));
    }
}
