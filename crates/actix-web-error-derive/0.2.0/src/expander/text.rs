use crate::{
    expand::BodyExpander,
    input::{Enum, Struct},
};
use proc_macro2::TokenStream;
use quote::quote;

pub struct Text;

impl BodyExpander for Text {
    fn expand_struct(_: &Struct) -> TokenStream {
        expand_text()
    }

    fn expand_enum(_: &Enum) -> TokenStream {
        expand_text()
    }
}

fn expand_text() -> TokenStream {
    quote!()
}
