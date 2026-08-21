macro_rules! parse_macro_input2 {
    ($tokens:expr => $T:ty) => {{
        match syn::parse2::<$T>($tokens) {
            Ok(v) => v,
            Err(e) => return e.to_compile_error().into(),
        }
    }};
    ($tokens:expr) => {{ $crate::parse_macro_input2!($tokens => syn::parse::Nothing) }};
}
pub(crate) use parse_macro_input2;
