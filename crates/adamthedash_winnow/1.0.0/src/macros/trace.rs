/// `trace!("parser_name", parser)` expands to:
/// `trace("module::name::parser_name", parser)`
#[macro_export]
macro_rules! trace {
    ($name:literal, $parser:expr) => {
        $crate::combinator::trace($crate::trace_name!($name), $parser)
    };
}
