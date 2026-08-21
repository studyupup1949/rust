use proc_macro2::TokenStream;
use syn::parse::{ParseStream, Parser};

use crate::common::{
    context::CodegenContext, function::map_err_to_compile_error, macros::ok_or_return,
};

use super::{builder::build_linear_accepts_spec, config::default_config_list_parser};

pub fn expand(ctx: &CodegenContext, input: TokenStream) -> TokenStream {
    let configs = ok_or_return!(map_err_to_compile_error(
        (|i: ParseStream| { default_config_list_parser(i) }).parse2(input)
    ));

    let mut tokens = TokenStream::new();

    for config in configs {
        let spec = ok_or_return!(map_err_to_compile_error(build_linear_accepts_spec(
            ctx, &config
        )));
        spec.into_tokens(ctx, &mut tokens);
    }

    tokens
}
