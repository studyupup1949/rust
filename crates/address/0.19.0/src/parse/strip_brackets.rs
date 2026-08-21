/// Strips the surrounding brackets from the `address`.
///
/// Returns `None` if the address is not bracketed.
///
/// # Examples
/// [::1]   -> `Some("::1")`
/// []      -> `Some("")`
/// ::1     -> `None`
/// [::1    -> `None`
pub(crate) fn strip_brackets(address: &[u8]) -> Option<&[u8]> {
    if address.len() >= 2 && address[0] == b'[' && address[address.len() - 1] == b']' {
        Some(&address[1..address.len() - 1])
    } else {
        None
    }
}
