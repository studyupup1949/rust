use syn::{parse::{Parse, ParseStream}, token::Token, Result, punctuated::Punctuated};

pub trait ArgParser
where
    Self: Parse {
    fn parse_as_punctuated<U: Token + Parse>(context: ParseStream) -> Result<Punctuated<Self, U>> {
        Punctuated::parse_separated_nonempty(context)
    }
}