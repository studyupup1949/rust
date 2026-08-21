use std::str::FromStr;

use syn::{
    Ident,
    parse::{Parse, ParseStream},
};

#[derive(Debug, Clone)]
pub enum ForwardSource {
    None,
    Ref,
    Clone,
}
impl FromStr for ForwardSource {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "None" => Ok(Self::None),
            "Ref" => Ok(Self::Ref),
            "Clone" => Ok(Self::Clone),
            _ => Err(()),
        }
    }
}
impl Parse for ForwardSource {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;

        Self::from_str(ident.to_string().as_str())
            .map_err(|_| syn::Error::new(ident.span(), "unknown option"))
    }
}
