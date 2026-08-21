use crate::Domain;

impl Domain {
    //! Validation

    /// The maximum length of a domain label.
    pub const MAX_LABEL_LEN: usize = 63;

    /// The maximum length of a domain name.
    pub const MAX_NAME_LEN: usize = 253;

    /// Checks if the char is valid. (excludes dots and dashes)
    #[inline(always)]
    const fn is_valid_char(c: u8, ignore_case: bool) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_digit() || (ignore_case && c.is_ascii_uppercase())
    }

    /// Checks if the domain label is valid.
    pub fn is_valid_label(label: &[u8], ignore_case: bool) -> bool {
        if label.is_empty() || label.len() > Self::MAX_LABEL_LEN {
            false
        } else if label[0] == b'-' || label[label.len() - 1] == b'-' {
            false
        } else {
            for (i, c) in label.iter().enumerate() {
                if !Self::is_valid_char(*c, ignore_case) {
                    if *c == b'-' {
                        if label[i - 1] == b'-' {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
            true
        }
    }

    /// Checks if the domain label is valid.
    pub fn is_valid_label_str(label: &str, ignore_case: bool) -> bool {
        Self::is_valid_label(label.as_bytes(), ignore_case)
    }

    /// Checks if the domain name is valid.
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

    /// Checks if the domain name is valid.
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
            ("az", false, true),
            ("AZ", false, false),
            ("AZ", true, true),
            ("09", false, true),
            ("-", false, false),
            ("-a", false, false),
            ("a-", false, false),
            ("a--a", false, false),
            ("a-a", false, true),
            ("a-a-a", false, true),
            ("a-a-a.a-a-a.a-a-a", false, false),
        ];
        for (label, ignore_case, expected) in test_cases {
            let result: bool = Domain::is_valid_label_str(*label, *ignore_case);
            assert_eq!(result, *expected, "label={}", label);
        }
    }

    #[test]
    fn is_valid_label_len() {
        let mut label: String = String::default();
        for _ in 0..63 {
            label.push('a');
        }
        assert!(Domain::is_valid_label_str(label.as_str(), false));
        label.push('a');
        assert!(!Domain::is_valid_label_str(label.as_str(), false));
    }

    #[test]
    fn is_valid_name() {
        let test_cases: &[(&str, bool, bool)] = &[
            ("", false, false),
            ("az", false, true),
            ("AZ", false, false),
            ("AZ", true, true),
            ("09", false, true),
            (".", false, false),
            (".a", false, false),
            ("a.", false, false),
            ("a..a", false, false),
            ("a.a", false, true),
            ("a.a.a", false, true),
            ("a-a-a.a-a-a.a-a-a", false, true),
        ];
        for (name, ignore_case, expected) in test_cases {
            let result: bool = Domain::is_valid_name_str(*name, *ignore_case);
            assert_eq!(result, *expected, "name={}", name);
        }
    }

    #[test]
    fn is_valid_name_len() {
        let mut name: String = String::default();
        for i in 0..253 {
            if i != 0 && i % 50 == 0 {
                name.push('.');
            } else {
                name.push('a');
            }
        }
        assert!(Domain::is_valid_name_str(name.as_str(), false));
        name.push('a');
        assert!(!Domain::is_valid_name_str(name.as_str(), false));
    }
}
