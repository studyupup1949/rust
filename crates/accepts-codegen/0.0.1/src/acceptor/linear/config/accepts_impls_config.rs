use syn::{
    AttrStyle, Attribute, Ident, Meta, Token, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Comma, Paren},
};

#[derive(Debug, Default, Clone)]
pub struct AcceptsImplsConfig {
    pub sync: Option<Punctuated<Meta, Comma>>,
    pub async_: Option<Punctuated<Meta, Comma>>,
}

#[allow(dead_code)]
impl AcceptsImplsConfig {
    pub fn from_parts(
        sync: Option<Punctuated<Meta, Comma>>,
        async_: Option<Punctuated<Meta, Comma>>,
    ) -> Self {
        Self { sync, async_ }
    }

    pub fn new_sync(attrs: Punctuated<Meta, Comma>) -> Self {
        Self::from_parts(Some(attrs), None)
    }

    pub fn new_async(attrs: Punctuated<Meta, Comma>) -> Self {
        Self::from_parts(None, Some(attrs))
    }

    pub fn is_sync(&self) -> bool {
        self.sync.is_some()
    }

    pub fn is_async(&self) -> bool {
        self.async_.is_some()
    }

    /// Returns `true` if neither sync nor async accepts implementations are selected.
    pub fn is_empty(&self) -> bool {
        self.sync.is_none() && self.async_.is_none()
    }
}
impl Parse for AcceptsImplsConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut gen_type = Self::default();

        let bracket_content;
        bracketed!(bracket_content in input);

        while !bracket_content.is_empty() {
            let ident: Ident = bracket_content.parse()?;

            match ident.to_string().as_str() {
                "Sync" => {
                    let mut sync = gen_type.sync.unwrap_or_default();

                    if bracket_content.peek(Paren) {
                        let paren_content;
                        parenthesized!(paren_content in bracket_content);

                        if paren_content.peek(Token![#]) {
                            let attrs = paren_content.call(Attribute::parse_outer)?;

                            for attr in attrs.into_iter() {
                                if let AttrStyle::Inner(not) = attr.style {
                                    return Err(syn::Error::new(
                                        not.span,
                                        "inner attributes are not allowed",
                                    ));
                                }

                                sync.push(attr.meta);
                            }
                        }
                    }

                    gen_type.sync = Some(sync);
                }
                "Async" => {
                    let mut async_ = gen_type.async_.unwrap_or_default();

                    if bracket_content.peek(Paren) {
                        let paren_content;
                        parenthesized!(paren_content in bracket_content);
                        if paren_content.peek(Token![#]) {
                            let attrs = paren_content.call(Attribute::parse_outer)?;

                            for attr in attrs.into_iter() {
                                if let AttrStyle::Inner(not) = attr.style {
                                    return Err(syn::Error::new(
                                        not.span,
                                        "inner attributes are not allowed",
                                    ));
                                }

                                async_.push(attr.meta);
                            }
                        }
                    }

                    gen_type.async_ = Some(async_);
                }

                _ => return Err(syn::Error::new(ident.span(), "unknown option")),
            }

            let _ = bracket_content.parse::<Comma>();
        }

        Ok(gen_type)
    }
}
