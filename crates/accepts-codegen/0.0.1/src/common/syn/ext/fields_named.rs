use syn::{
    Field, FieldsNamed,
    punctuated::Punctuated,
    token::{Brace, Comma},
};

pub trait FieldsNamedConstructExt {
    fn from_parts(brace_token: Brace, named: Punctuated<Field, Comma>) -> FieldsNamed;

    fn from_named(named: Punctuated<Field, Comma>) -> FieldsNamed;
}

impl FieldsNamedConstructExt for FieldsNamed {
    fn from_parts(brace_token: Brace, named: Punctuated<Field, Comma>) -> FieldsNamed {
        FieldsNamed { brace_token, named }
    }

    fn from_named(named: Punctuated<Field, Comma>) -> FieldsNamed {
        <Self as FieldsNamedConstructExt>::from_parts(Brace::default(), named)
    }
}
