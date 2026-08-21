use proc_macro2::TokenStream;
use quote::ToTokens;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
use syn::{
    parse_quote, punctuated::Punctuated, GenericArgument, Generics, Ident, PathArguments, Token,
    Type, WhereClause,
};

/// Type parameters for an enum or a struct.
pub struct TypeParams<'a> {
    names: BTreeSet<&'a Ident>,
}

impl<'a> TypeParams<'a> {
    /// Extract type parameters.
    pub fn new(generics: &'a Generics) -> Self {
        TypeParams {
            names: generics.type_params().map(|param| &param.ident).collect(),
        }
    }

    /// Search for any segments in `ty` that are
    /// in this `TypeParams`.
    pub fn intersects(&self, ty: &Type) -> bool {
        let mut found = false;
        self.crawl(ty, &mut found);
        found
    }

    /// Recursively searches for any segments in `ty` which are
    /// restricted in this `TypeParams`. The result will be stored in `*found`.
    fn crawl(&self, ty: &Type, found: &mut bool) {
        if let Type::Path(ty) = ty {
            if ty.qself.is_none() {
                if let Some(ident) = ty.path.get_ident() {
                    if self.names.contains(ident) {
                        *found = true;
                        return;
                    }
                }
            }
            for segment in &ty.path.segments {
                if let PathArguments::AngleBracketed(arguments) = &segment.arguments {
                    for arg in &arguments.args {
                        if let GenericArgument::Type(ty) = arg {
                            self.crawl(ty, found);
                        }
                    }
                }
            }
        }
    }
}

/// Inferred trait bounds for an enum or a struct.
pub struct InferredBounds {
    /// type-name/-id => bounds
    bounds: BTreeMap<String, (BTreeSet<String>, Punctuated<TokenStream, Token![+]>)>,
    /// order of type-names
    order: Vec<TokenStream>,
}

impl InferredBounds {
    pub fn new() -> Self {
        InferredBounds {
            bounds: BTreeMap::new(),
            order: Vec::new(),
        }
    }

    pub fn insert<Ty, Bound>(&mut self, ty: Ty, bound: Bound)
    where
        Ty: ToTokens,
        Bound: ToTokens,
    {
        let ty = ty.to_token_stream();
        let bound = bound.to_token_stream();
        let entry = self.bounds.entry(ty.to_string());
        if let Entry::Vacant(_) = entry {
            self.order.push(ty);
        }
        let (set, tokens) = entry.or_default();
        if set.insert(bound.to_string()) {
            tokens.push(bound);
        }
    }

    /// Added the inferred bounds to the `generics` in a `where` clause.
    pub fn augment_where_clause(&self, generics: &Generics) -> WhereClause {
        let mut generics = generics.clone();
        let where_clause = generics.make_where_clause();
        for ty in &self.order {
            let (_set, bounds) = &self.bounds[&ty.to_string()];
            where_clause.predicates.push(parse_quote!(#ty: #bounds));
        }
        generics.where_clause.unwrap()
    }
}
