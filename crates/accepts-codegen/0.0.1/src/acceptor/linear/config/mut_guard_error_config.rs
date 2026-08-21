use syn::{Ident, Type};

use super::ForwardSource;

#[derive(Debug, Clone)]
pub struct MutGuardErrorConfig {
    pub error_ty: Type,
    pub accepts_generics_ident: Ident,
    pub accepts_field_ident: Ident,
    pub forward_source: ForwardSource,
}
impl MutGuardErrorConfig {
    pub fn from_parts(
        error_ty: Type,
        accepts_generics_ident: Ident,
        accepts_field_ident: Ident,
        forward_source: ForwardSource,
    ) -> Self {
        Self {
            error_ty,
            accepts_generics_ident,
            accepts_field_ident,
            forward_source,
        }
    }
}
