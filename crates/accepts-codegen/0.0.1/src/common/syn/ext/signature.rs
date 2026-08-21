use proc_macro2::Span;
use syn::{
    Abi, FnArg, Generics, Ident, Path, ReturnType, Signature, Token, Type, Variadic,
    punctuated::Punctuated,
    token::{self, Comma, Fn, Paren},
};

use super::IdentConstructExt;

pub trait SignatureConstructExt {
    fn from_parts(
        constness: Option<Token![const]>,
        asyncness: Option<Token![async]>,
        unsafety: Option<Token![unsafe]>,
        abi: Option<Abi>,
        fn_token: Token![fn],
        ident: Ident,
        generics: Generics,
        paren_token: token::Paren,
        inputs: Punctuated<FnArg, Token![,]>,
        variadic: Option<Variadic>,
        output: ReturnType,
    ) -> Signature;

    fn from_ident_generics_inputs_output(
        ident: Ident,
        generics: Generics,
        inputs: Punctuated<FnArg, Token![,]>,
        output: ReturnType,
    ) -> Signature;

    fn from_ident_inputs_output(
        ident: Ident,
        inputs: Punctuated<FnArg, Token![,]>,
        output: ReturnType,
    ) -> Signature;

    fn new_signature(inputs: Punctuated<FnArg, Comma>) -> Signature;
}

impl SignatureConstructExt for Signature {
    fn from_parts(
        constness: Option<Token![const]>,
        asyncness: Option<Token![async]>,
        unsafety: Option<Token![unsafe]>,
        abi: Option<Abi>,
        fn_token: Token![fn],
        ident: Ident,
        generics: Generics,
        paren_token: token::Paren,
        inputs: Punctuated<FnArg, Token![,]>,
        variadic: Option<Variadic>,
        output: ReturnType,
    ) -> Signature {
        Signature {
            constness,
            asyncness,
            unsafety,
            abi,
            fn_token,
            ident,
            generics,
            paren_token,
            inputs,
            variadic,
            output,
        }
    }

    fn from_ident_generics_inputs_output(
        ident: Ident,
        generics: Generics,
        inputs: Punctuated<FnArg, Token![,]>,
        output: ReturnType,
    ) -> Signature {
        Self::from_parts(
            None,
            None,
            None,
            None,
            Fn::default(),
            ident,
            generics,
            Paren::default(),
            inputs,
            None,
            output,
        )
    }

    fn from_ident_inputs_output(
        ident: Ident,
        inputs: Punctuated<FnArg, Token![,]>,
        output: ReturnType,
    ) -> Signature {
        Self::from_ident_generics_inputs_output(ident, Generics::default(), inputs, output)
    }

    fn new_signature(inputs: Punctuated<FnArg, Comma>) -> Signature {
        Self::from_ident_inputs_output(
            Ident::from_str("new"),
            inputs,
            ReturnType::Type(
                Token![->](Span::call_site()),
                Box::new(Type::Path(syn::TypePath {
                    qself: None,
                    path: Path::from(Ident::new("Self", Span::call_site())),
                })),
            ),
        )
    }
}
