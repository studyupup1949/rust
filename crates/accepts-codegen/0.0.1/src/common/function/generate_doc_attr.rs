use quote::format_ident;
pub use syn::Expr;
use syn::{AttrStyle, Attribute, ExprLit, Lit, LitStr, MetaNameValue, Path};

use crate::common::syn::ext::{
    AttributeConstructExt, ExprLitConstructExt, MetaNameValueConstructExt,
};

pub fn generate_doc_attr(lit_str: LitStr) -> Attribute {
    Attribute::from_style_meta(
        AttrStyle::Outer,
        syn::Meta::NameValue(MetaNameValue::from_path_value(
            Path::from(format_ident!("doc")),
            Expr::Lit(ExprLit::from_lit(Lit::Str(lit_str))),
        )),
    )
}
