use std::os::raw::c_char;

include!(concat!(env!("OUT_DIR"), "/ascii.rs"));

pub fn escape(esc_chr: char, src: &str, cntl_set: &str) -> String {
    let len = unsafe {
        ascii_escape_len(
            esc_chr as c_char,
            src.as_ptr() as *const c_char,
            src.len(),
            cntl_set.as_ptr() as *const c_char,
            cntl_set.len(),
        )
    };
    let mut dst = Vec::with_capacity(len);
    unsafe {
        ascii_escape(
            esc_chr as c_char,
            src.as_ptr() as *const c_char,
            src.len(),
            dst.as_mut_ptr() as *mut c_char,
            cntl_set.as_ptr() as *const c_char,
            cntl_set.len(),
        );
        dst.set_len(len);
        String::from_utf8_unchecked(dst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        assert_eq!(escape('\\', "", "\""), "");
        assert_eq!(escape('\\', "abc", "\""), "abc");
        assert_eq!(escape('\\', "a\\b", "\""), "a\\\\b");
        assert_eq!(escape('\\', "a\"b", "\""), "a\\\"b");
        assert_eq!(escape('\\', "a\\\"b", "\""), "a\\\\\\\"b");
        assert_eq!(escape('\\', "😄\"😄😄", "\""), "😄\\\"😄😄");
    }
}
