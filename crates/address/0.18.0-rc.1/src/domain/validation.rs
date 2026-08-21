use crate::Domain;

impl Domain {
    //! Label Validation

    /// The maximum length of a domain label.
    pub const MAX_LABEL_LEN: usize = 63;

    /// Checks if the char `c` is valid. (excludes dots and dashes)
    const fn is_valid_char(c: u8, ignore_case: bool) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_digit() || (ignore_case && c.is_ascii_uppercase())
    }

    /// Checks if the domain `label` is valid.
    pub fn is_valid_label(label: &[u8], ignore_case: bool) -> bool {
        if (label.is_empty() || label.len() > Self::MAX_LABEL_LEN)
            || (label[0] == b'-' || label[label.len() - 1] == b'-')
        {
            false
        } else {
            for (i, c) in label.iter().enumerate() {
                if !Self::is_valid_char(*c, ignore_case) && (*c != b'-' || label[i - 1] == b'-') {
                    return false;
                }
            }
            true
        }
    }

    /// Checks if the domain `label` is valid.
    pub fn is_valid_label_str(label: &str, ignore_case: bool) -> bool {
        Self::is_valid_label(label.as_bytes(), ignore_case)
    }
}

impl Domain {
    //! Domain Validation

    /// The maximum length of a domain name.
    pub const MAX_NAME_LEN: usize = 253;

    /// Checks if the domain `name` is valid.
    pub fn is_valid_name(name: &[u8], ignore_case: bool) -> bool {
        if name.is_empty() || name.len() > Self::MAX_NAME_LEN {
            false
        } else {
            let mut rem: &[u8] = name;
            while let Some(dot) = rem.iter().position(|c| *c == b'.') {
                if !Self::is_valid_label(&rem[..dot], ignore_case) {
                    return false;
                }
                rem = &rem[dot + 1..];
            }
            Self::is_valid_label(rem, ignore_case)
        }
    }

    /// Checks if the domain `name` is valid.
    pub fn is_valid_name_str(name: &str, ignore_case: bool) -> bool {
        Self::is_valid_name(name.as_bytes(), ignore_case)
    }
}

#[cfg(test)]
mod tests {
    use crate::Domain;

    #[test]
    fn is_valid_label() {
        let test_cases: &[(&str, bool, bool)] = &[
            ("", false, false),
            ("09", true, true),
            ("az", true, true),
            ("AZ", false, true),
            ("-a", false, false),
            ("a-", false, false),
            ("a--a", false, false),
            ("a-a", true, true),
            ("a-a-a", true, true),
        ];
        for (label, expected, expected_ignore_case) in test_cases {
            let result: bool = Domain::is_valid_label_str(label, false);
            assert_eq!(result, *expected, "label={}", label);

            let result: bool = Domain::is_valid_label_str(label, true);
            assert_eq!(result, *expected_ignore_case, "label={}", label);
        }
    }

    #[test]
    fn is_valid_name() {
        let test_cases: &[(&str, bool, bool)] = &[
            ("", false, false),
            ("09", true, true),
            ("az", true, true),
            ("AZ", false, true),
            (".a", false, false),
            ("a.", false, false),
            ("a..a", false, false),
            ("a.a", true, true),
            ("a.a.a", true, true),
            ("a-a.a-a.a-a", true, true),
        ];
        for (label, expected, expected_ignore_case) in test_cases {
            let result: bool = Domain::is_valid_name_str(label, false);
            assert_eq!(result, *expected, "label={}", label);

            let result: bool = Domain::is_valid_name_str(label, true);
            assert_eq!(result, *expected_ignore_case, "label={}", label);
        }
    }
}
