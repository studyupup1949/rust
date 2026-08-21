use syn::{Attribute, ExprLit, Lit};

pub trait ExprLitConstructExt {
    fn from_parts(attrs: Vec<Attribute>, lit: Lit) -> ExprLit;

    fn from_lit(lit: Lit) -> ExprLit;
}

impl ExprLitConstructExt for ExprLit {
    fn from_parts(attrs: Vec<Attribute>, lit: Lit) -> ExprLit {
        ExprLit { attrs, lit }
    }

    fn from_lit(lit: Lit) -> ExprLit {
        Self::from_parts(Vec::new(), lit)
    }
}
