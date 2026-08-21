use syn::{
    GenericParam, Generics, Path, Token, Type, TypePath, WhereClause, WherePredicate,
    punctuated::Punctuated,
};

use crate::common::syn::ext::{
    GenericsConstructExt, TypePathConstructExt, WhereClauseConstructExt,
};

use super::{AcceptsT, AcceptsTPredicateType};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AcceptsGenerics {
    pub lt_token: Option<Token![<]>,
    pub params: Punctuated<GenericParam, Token![,]>,
    pub gt_token: Option<Token![>]>,
    pub where_clause: Option<WhereClause>,
    pub accepts_t_predicate_type: Option<AcceptsTPredicateType>,
}

impl AcceptsGenerics {
    pub fn from_parts(
        lt_token: Option<Token![<]>,
        params: Punctuated<GenericParam, Token![,]>,
        gt_token: Option<Token![>]>,
        where_clause: Option<WhereClause>,
        accepts_t_predicate_type: Option<AcceptsTPredicateType>,
    ) -> Self {
        Self {
            lt_token,
            params,
            gt_token,
            where_clause,
            accepts_t_predicate_type,
        }
    }

    pub fn from_generics(generics: Generics) -> Self {
        Self::from_parts(
            generics.lt_token,
            generics.params,
            generics.gt_token,
            generics.where_clause,
            None,
        )
    }

    pub fn into_generics(mut self, accepts_t: AcceptsT) -> Generics {
        let accepts_t_type = match accepts_t {
            AcceptsT::Type(t_type) => t_type,
            AcceptsT::Generics(t_type_param) => {
                let t_type_param_ident = t_type_param.ident.clone();
                self.params.push(GenericParam::Type(t_type_param));
                Type::Path(TypePath::from_path(Path::from(t_type_param_ident)))
            }
        };
        let where_clause = match self.accepts_t_predicate_type {
            None => None,
            Some(t_predicate) => {
                let mut where_clause = self
                    .where_clause
                    .unwrap_or(WhereClause::from_predicates(Punctuated::new()));

                where_clause.predicates.push(WherePredicate::Type(
                    t_predicate.into_predicate_type(accepts_t_type),
                ));

                Some(where_clause)
            }
        };

        let (lt_token, gt_token) = if self.params.is_empty() {
            (None, None)
        } else {
            (
                Some(self.lt_token.unwrap_or_default()),
                Some(self.gt_token.unwrap_or_default()),
            )
        };

        Generics::from_parts(lt_token, self.params, gt_token, where_clause)
    }
}

impl From<Generics> for AcceptsGenerics {
    fn from(generics: Generics) -> Self {
        Self::from_generics(generics)
    }
}
