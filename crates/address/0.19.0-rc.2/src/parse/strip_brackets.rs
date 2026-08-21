/// Strips the surrounding brackets from the `address` string.
///
/// Returns `None` if the address is not bracketed.
///
/// # Examples
/// [::1]   -> `Some("::1")`
/// []      -> `Some("")`
/// ::1     -> `None`
/// [::1    -> `None`
pub(crate) fn strip_brackets(address: &str) -> Option<&str> {
    let bytes: &[u8] = address.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'[' && bytes[bytes.len() - 1] == b']' {
        Some(&address[1..address.len() - 1])
    } else {
        None
    }
}
