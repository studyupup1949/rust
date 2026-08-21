/// Parse an address at runtime and panic on failure.
///
/// Accepts a string literal or any expression returning `&str`. For
/// compile-time validation with span-aware errors, use the proc macro
/// `addrezz_macros::addr!` instead (enabled via the `macros` feature on
/// the facade crate).
///
/// ```ignore
/// use addrezz_core::addr;
/// let a = addr!("https://github.com/");
/// ```
#[macro_export]
macro_rules! addr {
    ($s:expr) => {{
        match $crate::Addr::parse($s) {
            ::core::result::Result::Ok(a) => a,
            ::core::result::Result::Err(e) => {
                ::core::panic!("addr!: failed to parse {:?}: {}", $s, e)
            }
        }
    }};
}

/// Try-parse variant of [`addr!`] that returns `Result<Addr, ParseError>`.
#[macro_export]
macro_rules! try_addr {
    ($s:expr) => {
        $crate::Addr::parse($s)
    };
}
