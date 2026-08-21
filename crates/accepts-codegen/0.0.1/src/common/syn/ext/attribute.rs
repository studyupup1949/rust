use syn::{
    AttrStyle, Attribute, Meta,
    token::{Bracket, Pound},
};

pub trait AttributeConstructExt {
    fn from_style_meta(style: AttrStyle, meta: Meta) -> Attribute;
}

impl AttributeConstructExt for Attribute {
    fn from_style_meta(style: AttrStyle, meta: Meta) -> Attribute {
        Attribute {
            pound_token: Pound::default(),
            style,
            bracket_token: Bracket::default(),
            meta,
        }
    }
}
