use std::collections::BTreeSet;
use syn::fold::{Fold, fold_type};
use syn::{Ident, Type, TypePath};

pub struct TypeIdentReplacer<'a> {
    pub needle: &'a Ident,
    pub replacement: &'a Type,
    pub include_qself: bool,
    pub shadowed: BTreeSet<String>,
}

impl<'a> TypeIdentReplacer<'a> {
    pub fn from_parts(
        needle: &'a Ident,
        replacement: &'a Type,
        include_qself: bool,
        shadowed: BTreeSet<String>,
    ) -> Self {
        Self {
            needle,
            replacement,
            include_qself,
            shadowed,
        }
    }

    pub fn from_needle_replacement(needle: &'a Ident, replacement: &'a Type) -> Self {
        Self::from_parts(needle, replacement, false, BTreeSet::new())
    }
}

impl<'a> Fold for TypeIdentReplacer<'a> {
    fn fold_type(&mut self, ty: Type) -> Type {
        if let Type::Path(TypePath { qself, path }) = &ty {
            if self.include_qself || qself.is_none() {
                if let Some(id) = path.get_ident() {
                    // スコープで宣言済みの型パラメータは避ける場合
                    if !self.shadowed.contains(&id.to_string()) && id == self.needle {
                        return self.replacement.clone();
                    }
                }
            }
        }
        fold_type(self, ty)
    }
}
