use base64::{Engine as _, prelude::BASE64_STANDARD};
use infer::Type;

pub trait StringUtil {
    /// Check whether given `s` is a hex string.
    fn is_hex(&self) -> bool;

    /// Escape `like` SQL in `s`.
    /// Replace `\` to `\\`, `%` to `\%` and `_` to `\_`.
    fn like_escape(&self) -> String;
}

impl<S> StringUtil for S
where
    S: AsRef<str>,
{
    /// Check whether given string is a hex string.
    fn is_hex(&self) -> bool {
        self.as_ref().chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Escape `like` SQL in string.
    fn like_escape(&self) -> String {
        let mut s = self.as_ref().replace('\\', "\\\\"); // first replace \
        s = s.replace('%', "\\%");
        s.replace('_', "\\_")
    }
}

pub struct DataUrl {
    pub data: Vec<u8>,
    pub mime: Type,
}

impl DataUrl {
    pub fn encode(&self) -> String {
        let data = BASE64_STANDARD.encode(&self.data);
        format!("data:{};base64,{data}", self.mime)
    }

    pub fn decode<S: AsRef<str>>(data: S) -> Option<Self> {
        let data: Vec<_> = data.as_ref().split(',').collect();
        if data.len() == 2 {
            BASE64_STANDARD.decode(data[1]).map_or(None, Self::new)
        } else {
            None
        }
    }

    pub fn new(data: Vec<u8>) -> Option<Self> {
        let mime = infer::get(&data);
        mime.map(|mime| Self { data, mime })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_hex_valid() {
        assert!("0123456789abcdefABCDEF".is_hex());
        assert!("".is_hex());
    }

    #[test]
    fn is_hex_invalid() {
        assert!(!"0x123".is_hex());
        assert!(!"hello".is_hex());
        assert!(!"12g4".is_hex());
    }

    #[test]
    fn like_escape_basic() {
        assert_eq!("plain".like_escape(), "plain");
    }

    #[test]
    fn like_escape_special() {
        assert_eq!("a%b_c\\d".like_escape(), "a\\%b\\_c\\\\d");
    }

    #[test]
    fn like_escape_backslash_first() {
        // `\` must be escaped first so it does not double-escape later inserts.
        assert_eq!("%\\".like_escape(), "\\%\\\\");
    }

    #[test]
    fn dataurl_roundtrip() {
        // 8-byte PNG signature is enough for `infer` to detect image/png.
        let png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let url = DataUrl::new(png.clone()).unwrap();
        let encoded = url.encode();
        let decoded = DataUrl::decode(&encoded).unwrap();
        assert_eq!(decoded.data, png);
        assert_eq!(decoded.mime.mime_type(), "image/png");
    }

    #[test]
    fn dataurl_decode_bad() {
        assert!(DataUrl::decode("no-comma-here").is_none());
    }

    #[test]
    fn dataurl_new_unknown() {
        // Empty / random bytes have no detectable mime.
        assert!(DataUrl::new(Vec::new()).is_none());
        assert!(DataUrl::new(vec![0x01, 0x02, 0x03]).is_none());
    }
}
