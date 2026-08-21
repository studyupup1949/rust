use crate::Domain;

impl Domain {
    //! Label Validation

    /// The maximum length of a domain label.
    pub const MAX_LABEL_LEN: usize = 63;

    /// Checks if the char is valid.
    #[inline(always)]
    fn is_valid_char(c: u8, ignore_case: bool) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_digit() || (ignore_case && c.is_ascii_uppercase())
    }

    /// Checks if the domain label is valid.
    pub fn is_valid_label(label: &[u8], ignore_case: bool) -> bool {
        if label.is_empty() || label.len() > Self::MAX_LABEL_LEN {
            false
        } else {
            for (i, c) in label.iter().enumerate() {
                if !Self::is_valid_char(*c, ignore_case) {
                    if *c != b'-' || i == 0 || i == label.len() - 1 || label[i - 1] == b'-' {
                        return false;
                    }
                }
            }
            true
        }
    }

    /// Checks if the domain label string is valid.
    pub fn is_valid_label_str(label: &str, ignore_case: bool) -> bool {
        Self::is_valid_label(label.as_bytes(), ignore_case)
    }
}

impl Domain {
    //! Name Validation

    /// The maximum length of a domain name.
    pub const MAX_NAME_LEN: usize = 253;

    /// Checks if the domain name is valid.
    pub fn is_valid_name(name: &[u8], ignore_case: bool) -> bool {
        if name.is_empty() || name.len() > Self::MAX_NAME_LEN {
            false
        } else {
            match name.iter().position(|c| *c == b'.') {
                Some(dot) => {
                    Self::is_valid_label(&name[..dot], ignore_case)
                        && Self::is_valid_name(&name[dot + 1..], ignore_case)
                }
                None => Self::is_valid_label(name, ignore_case),
            }
        }
    }

    /// Checks if the domain name string is valid.
    pub fn is_valid_name_str(name: &str, ignore_case: bool) -> bool {
        Self::is_valid_name(name.as_bytes(), ignore_case)
    }
}
