use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse::ParseStream, spanned::Spanned, Attribute, Error, Ident, LitInt, Result};

pub struct Attrs<'a> {
    pub status: Option<ResolveStatus<'a>>,
}

#[derive(Clone)]
pub enum ResolveStatus<'a> {
    Transparent(&'a Attribute),
    Fixed(Status<'a>),
}

#[derive(Clone)]
pub struct Status<'a> {
    pub original: &'a Attribute,
    pub code: Code,
}

#[derive(Clone)]
pub enum Code {
    Value(http::StatusCode),
    Name(Ident),
}

mod kw {
    syn::custom_keyword!(transparent);
}

impl ToTokens for Code {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Code::Value(v) => {
                let value = v.as_u16();
                tokens.extend(quote! { ::actix_web::http::StatusCode::from_u16(#value).unwrap() });
            }
            Code::Name(ident) => tokens.extend(quote! { ::actix_web::http::StatusCode::#ident }),
        }
    }
}

impl ToTokens for Status<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.code.to_tokens(tokens);
    }
}

impl Attrs<'_> {
    pub fn get(input: &[Attribute]) -> Result<Attrs<'_>> {
        let mut attrs = Attrs { status: None };

        for attr in input {
            if attr.path().is_ident("status") {
                attrs.parse_status_attribute(attr)?;
            }
        }

        Ok(attrs)
    }

    pub fn span(&self) -> Option<Span> {
        self.status.as_ref().map(|st| match st {
            ResolveStatus::Transparent(t) => t.span(),
            ResolveStatus::Fixed(fix) => fix.original.span(),
        })
    }
}

impl<'a> Attrs<'a> {
    fn parse_status_attribute(&mut self, attr: &'a Attribute) -> Result<()> {
        if self.status.is_some() {
            return Err(Error::new_spanned(
                attr,
                "duplicate #[status(..)] attribute",
            ));
        }

        attr.parse_args_with(|input: ParseStream| {
            if input.parse::<Option<kw::transparent>>()?.is_some() {
                self.status = Some(ResolveStatus::Transparent(attr));
                return Ok(());
            }

            let status = Status {
                original: attr,
                code: parse_status_expr(input)?,
            };
            self.status = Some(ResolveStatus::Fixed(status));
            Ok(())
        })
    }
}

fn parse_status_expr(input: ParseStream) -> Result<Code> {
    match input.parse::<Option<LitInt>>()? {
        Some(lit) => http::status::StatusCode::from_u16(lit.base10_parse::<u16>()?)
            .map_err(|e| Error::new_spanned(lit.token(), e))
            .map(Code::Value),
        None => input.parse::<Ident>().map(Code::Name),
    }
}
