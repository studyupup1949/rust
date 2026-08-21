use syn::{
    Expr, ExprPath, GenericArgument, GenericParam, Generics, Path, PathSegment, Type, TypePath,
    punctuated::Punctuated,
};

use crate::common::syn::ext::{
    ExprPathConstructExt, PathConstructExt, PathSegmentConstructExt, PunctuatedConstructExt,
    TypePathConstructExt,
};

pub fn param_to_argument(param: GenericParam) -> GenericArgument {
    match param {
        GenericParam::Type(type_param) => {
            let ident = type_param.ident;
            let type_path = TypePath::from_path(Path::from_segments(Punctuated::from_value(
                PathSegment::from_ident(ident),
            )));
            GenericArgument::Type(Type::Path(type_path))
        }
        GenericParam::Lifetime(lifetime_def) => {
            let lt = lifetime_def.lifetime;
            GenericArgument::Lifetime(lt)
        }
        GenericParam::Const(const_param) => {
            let ident = const_param.ident;
            let expr: syn::Expr = Expr::Path(ExprPath::from_path(Path::from_segments(
                Punctuated::from_value(PathSegment::from_ident(ident)),
            )));
            GenericArgument::Const(expr)
        }
    }
}

/// Remove default values from generic parameters.
pub fn generics_without_defaults(mut generics: Generics) -> Generics {
    for param in generics.params.iter_mut() {
        match param {
            GenericParam::Type(type_param) => {
                type_param.eq_token = None;
                type_param.default = None;
            }
            GenericParam::Const(const_param) => {
                const_param.eq_token = None;
                const_param.default = None;
            }
            GenericParam::Lifetime(_) => {}
        }
    }
    generics
}
