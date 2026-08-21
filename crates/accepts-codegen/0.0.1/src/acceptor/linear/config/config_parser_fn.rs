use std::collections::HashSet;

use syn::{
    Attribute, GenericParam, Generics, Ident, Token, Visibility, braced,
    parse::{ParseStream, Result},
    token::Struct,
};

use crate::acceptor::linear::config::NextAcceptorConfig;

use super::{LinearAcceptorConfig, spec::LinearAcceptorSpec};

pub fn default_config_parser(input: ParseStream) -> Result<LinearAcceptorSpec> {
    let mut builder = LinearAcceptorConfig::new();
    builder.vis(Visibility::Inherited);

    if input.peek(Token![#]) {
        builder.attrs(Attribute::parse_outer(input)?);
    }

    if input.peek(Token![pub]) {
        builder.vis(input.parse()?);
    }

    let _ = input.parse::<Struct>()?;

    builder.ident(input.parse()?);

    builder.accepts_value_param({
        let generics: Generics = input.parse()?;
        let mut iter = generics.params.into_iter();

        let one = iter
            .next()
            .ok_or_else(|| input.error("expected one type param in `<...>`"))?;
        if iter.next().is_some() {
            return Err(input.error("expected exactly one param in `<...>`"));
        }

        match one {
            GenericParam::Type(tp) => tp,
            _ => return Err(input.error("expected a type param (e.g., `T`), not lifetime/const")),
        }
    });

    let content;
    braced!(content in input);

    let mut seen = HashSet::new();

    while !content.is_empty() {
        let key: Ident = content.parse()?;
        let key_string = key.to_string();
        if !seen.insert(key_string.clone()) {
            return Err(syn::Error::new(key.span(), "duplicate key"));
        }
        content.parse::<Token![:]>()?;
        match key_string.as_str() {
            "accepts_impls" => {
                builder.accept_impls(content.parse()?);
            }

            "tail_accepts" => {
                let name: Ident = content.parse()?;

                builder.next_accepts(match &*name.to_string() {
                    "Final" => None,
                    "Next" => Some(content.parse()?),

                    _ => {
                        return Err(syn::Error::new(name.span(), "unknown option"));
                    }
                });
            }

            "handler" => {
                builder.handler(content.parse()?);
            }

            _ => return Err(syn::Error::new(key.span(), "unknown key")),
        }

        let _ = content.parse::<Token![,]>();
    }

    if builder.get_next_accepts().is_none() {
        builder.next_accepts(Some(NextAcceptorConfig::default()));
    }

    builder.build_spec()
}
