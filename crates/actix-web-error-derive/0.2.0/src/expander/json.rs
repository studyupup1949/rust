use crate::{
    expand::BodyExpander,
    input::{Enum, Struct},
};
use proc_macro2::TokenStream;
use quote::quote;

pub struct Json;

impl BodyExpander for Json {
    fn expand_struct(_: &Struct) -> TokenStream {
        json_expand()
    }

    fn expand_enum(_: &Enum) -> TokenStream {
        json_expand()
    }
}

fn json_expand() -> TokenStream {
    quote! {
        fn error_response(&self) -> ::actix_web::HttpResponse<::actix_web::body::BoxBody> {
            ::actix_web::HttpResponseBuilder::new(self.status_code()).json(::actix_web_error::__private::JsonErrorSerialize(&self))
        }
    }
}
