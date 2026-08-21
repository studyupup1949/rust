#[macro_use]
mod error;
mod charstring;
mod font;
mod postscript;
mod tables;

pub use error::FontError;
pub use font::*;

type Input<'a> = &'a [u8];
type IResult<'a, T> = nom::IResult<&'a [u8], T, nom_language::error::VerboseError<&'a [u8]>>;
