use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Error, ItemFn, Token,
};

struct Args {
    _vars: HashSet<Ident>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            _vars: vars.into_iter().collect(),
        })
    }
}

fn async_to_sync(mut input: syn::ItemFn) -> Result<ItemFn, Error> {
    if input.sig.asyncness.take().is_none() {
        let msg = "the `async` keyword is missing from the function declaration";
        return Err(syn::Error::new_spanned(input.sig.fn_token, msg));
    }
    let (last_stmt_start_span, last_stmt_end_span) = {
        let mut last_stmt = input
            .block
            .stmts
            .last()
            .map(ToTokens::into_token_stream)
            .unwrap_or_default()
            .into_iter();
        // `Span` on stable Rust has a limitation that only points to the first
        // token, not the whole tokens. We can work around this limitation by
        // using the first/last span of the tokens like
        // `syn::Error::new_spanned` does.
        let start = last_stmt.next().map_or_else(Span::call_site, |t| t.span());
        let end = last_stmt.last().map_or(start, |t| t.span());
        (start, end)
    };

    let body = &input.block;
    let brace_token = input.block.brace_token;
    input.block = syn::parse2(quote_spanned! {last_stmt_end_span=>
        {
            futures::executor::block_on(async #body);
        }
    })
    .expect("Parsing failure");
    input.block.brace_token = brace_token;
    Ok(input)
}

#[proc_macro_attribute]
pub fn rust_decorator_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut func: ItemFn = syn::parse(input).unwrap();
    let mut func = async_to_sync(func).unwrap();
    let ident = &func.sig.ident;
    let test_path = syn::LitStr::new(&format!("::{}", ident), proc_macro2::Span::call_site());
    let mut allow_fail = false;
    let mut ignore = false;
    func.attrs.retain(|ref attr| {
        match attr.path.clone().into_token_stream().to_string().as_ref() {
            "allow_fail" => allow_fail = true,
            "ignore" => ignore = true,
            _ => return true,
        }
        false
    });

    let x = func.clone().block;

    let func_span = func.span();
    let res = quote_spanned!(func_span=>
        #[allow(dead_code,non_upper_case_globals)]
        #[test_case]
        pub const #ident:  admiral_types::RussTestDescAndFn =  admiral_types::RussTestDescAndFn {
            desc: rustc_test::TestDesc {
                allow_fail: #allow_fail,
                ignore: #ignore,
                should_panic: rustc_test::ShouldPanic::Yes,
                name: rustc_test::StaticTestName(concat!(module_path!(), #test_path)),
            },
            testfn: || {
                #x
            },
        };
    );
    res.into()
}
