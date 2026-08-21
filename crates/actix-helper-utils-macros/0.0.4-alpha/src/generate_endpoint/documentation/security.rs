use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{bracketed, parenthesized, LitStr, Token};

#[derive(Debug)]
pub(crate) struct SecurityRequirement {
    pub(crate) name: Option<LitStr>,
    pub(crate) scopes: Vec<LitStr>,
}

impl Parse for SecurityRequirement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut scopes = Vec::new();

        let content;
        parenthesized!(content in input);

        let name = content.parse().ok();

        if let Some(_name) = &name {
            content.parse::<Token![=]>()?;

            let bracketed;
            bracketed!(bracketed in content);

            let parsed_scopes = Punctuated::<LitStr, Token![,]>::parse_terminated(&bracketed)?;

            scopes = parsed_scopes.into_iter().collect();
        }

        Ok(Self { name, scopes })
    }
}
