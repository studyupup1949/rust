use crate::generate_crud::generate_crud_internal;
use crate::generate_endpoint::generate_endpoint_internal;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod generate_crud;
mod generate_endpoint;

#[proc_macro_error]
#[proc_macro]
/// Input to the `generate_endpoint` macro
///
/// To use the validation injection it assumes the type has a method called `into_inner` and calls it right before validation,
/// this means that custom extractors should follow the same pattern as built in if they desire to be used with the validator
///
/// # Example
///
/// ```rust
/// # use actix_helper_utils_macros::generate_endpoint;
/// generate_endpoint! {
///     fn login;
///     method: get;
///     path: "/health";
///     docs: {
///         tag: "health",
///         context_path: "/",
///         responses: {
///             (status = 200, description = "Everything works just fine!")
///         }
///     }
///     {
///         Ok(HttpResponse::Ok().body("Everything works just fine!"))
///     }
/// }
///```
///
pub fn generate_endpoint(input: TokenStream) -> TokenStream {
    generate_endpoint_internal(input.into())
}

#[proc_macro_error]
#[proc_macro]
pub fn generate_crud(input: TokenStream) -> TokenStream {
    generate_crud_internal(input.into())
}
