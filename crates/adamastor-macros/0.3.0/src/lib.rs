//! Proc macros for the Adamastor LLM framework.
//!
//! This crate provides the `#[schema]` attribute macro that simplifies
//! defining types for structured LLM outputs.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Attribute macro that adds all required derives for Gemini schema types.
///
/// This macro expands to add `Debug`, `Serialize`, `Deserialize`, and `JsonSchema`
/// derives to your struct, making it ready for use with Adamastor's structured output API.
///
/// # Example
///
/// ```ignore
/// use adamastor::schema;
///
/// #[schema]
/// struct Recipe {
///     name: String,
///     ingredients: Vec<String>,
///     prep_time_minutes: u32,
/// }
/// ```
///
/// This expands to:
///
/// ```ignore
/// #[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
/// struct Recipe {
///     name: String,
///     ingredients: Vec<String>,
///     prep_time_minutes: u32,
/// }
/// ```
#[proc_macro_attribute]
pub fn schema(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let expanded = quote! {
        #[derive(
            Debug,
            adamastor::Serialize,
            adamastor::Deserialize,
            adamastor::JsonSchema
        )]
        #input
    };

    TokenStream::from(expanded)
}
