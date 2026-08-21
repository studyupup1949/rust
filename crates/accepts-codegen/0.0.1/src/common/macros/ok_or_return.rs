macro_rules! ok_or_return {
    ( $expr:expr) => {{
        match $expr {
            ::core::result::Result::Ok(v) => v,
            ::core::result::Result::Err(err) => return ::core::convert::From::from(err),
        }
    }};
}
pub(crate) use ok_or_return;
