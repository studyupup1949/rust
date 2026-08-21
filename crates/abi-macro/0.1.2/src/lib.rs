use proc_macro::TokenStream;

mod derive;
mod expand;

#[proc_macro_attribute]
pub fn ffi(args: TokenStream, input: TokenStream) -> TokenStream {
    expand::expand_ffi(args.into(), input.into())
        .unwrap()
        .into()
}

#[proc_macro_derive(CheckTypeId)]
pub fn derive_check_type_id(item: TokenStream) -> TokenStream {
    derive::check_type_id(item.into()).unwrap().into()
}
