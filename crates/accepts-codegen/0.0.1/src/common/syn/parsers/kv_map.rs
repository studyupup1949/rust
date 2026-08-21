use indexmap::IndexMap;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::marker::PhantomData;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Ident, Token, braced, spanned::Spanned};

/// Key-value map parsed from a braced list.
/// Values are kept as raw tokens for later parsing.
pub struct KeyValueMap<Sep = Token![=]> {
    pub pairs: IndexMap<String, TokenStream>,
    _marker: PhantomData<Sep>,
}

#[allow(dead_code)]
impl<Sep> KeyValueMap<Sep> {
    /// Create an empty map.
    pub fn new() -> Self {
        Self {
            pairs: IndexMap::new(),
            _marker: PhantomData,
        }
    }

    /// Get raw tokens for a key.
    pub fn get_tokens(&self, key: &str) -> Option<&TokenStream> {
        self.pairs.get(key)
    }

    /// Parse the value of a key into a specific type.
    pub fn parse_as<T: Parse>(&self, key: &str) -> Result<T> {
        let tokens = self
            .pairs
            .get(key)
            .ok_or_else(|| syn::Error::new(Span::call_site(), format!("missing key `{}`", key)))?;
        syn::parse2(tokens.clone())
    }

    /// Consume the tokens of a key and remove it from the map.
    pub fn take_tokens(&mut self, key: &str) -> Option<TokenStream> {
        self.pairs.shift_remove(key)
    }

    /// Parse and remove the value of a key into a specific type, returning `None` if the key is missing.
    pub fn try_take_parse<T: Parse>(&mut self, key: &str) -> Result<Option<T>> {
        match self.pairs.shift_remove(key) {
            Some(tokens) => syn::parse2(tokens).map(Some),
            None => Ok(None),
        }
    }

    /// Parse and remove the value of a key into a specific type.
    pub fn take_parse<T: Parse>(&mut self, key: &str) -> Result<T> {
        let tokens = self
            .pairs
            .shift_remove(key)
            .ok_or_else(|| syn::Error::new(Span::call_site(), format!("missing key `{}`", key)))?;
        syn::parse2(tokens)
    }

    /// Ensure the map is empty or return an error at the remaining token.
    pub fn error_if_not_empty(&self, msg: &str) -> Result<()> {
        if let Some((_, tokens)) = self.pairs.iter().next() {
            Err(syn::Error::new(tokens.span(), msg))
        } else {
            Ok(())
        }
    }
}

impl<Sep> Parse for KeyValueMap<Sep>
where
    Sep: Parse,
{
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        let mut pairs = IndexMap::new();
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            content.parse::<Sep>()?;
            let expr: syn::Expr = content.parse()?;
            pairs.insert(key.to_string(), expr.to_token_stream());
            if content.peek(Token![,]) {
                let _ = content.parse::<Token![,]>()?;
            } else {
                break;
            }
        }
        Ok(Self {
            pairs,
            _marker: PhantomData,
        })
    }
}
