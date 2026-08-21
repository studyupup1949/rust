macro_rules! take_until {
    ($input:expr, $stop:tt) => {{
        let mut out = proc_macro2::TokenStream::new();
        while !$input.is_empty() && !$input.peek(syn::Token![$stop]) {
            let tt: proc_macro2::TokenTree = $input.parse()?;
            out.extend(std::iter::once(tt.into()));
        }
        out
    }};
}
pub(crate) use take_until;
