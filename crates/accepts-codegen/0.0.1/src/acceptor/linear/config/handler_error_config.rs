use syn::{Ident, Type};

/// Holds metadata for handler error Accepts
#[derive(Debug, Clone)]
pub struct HandlerErrorConfig {
    pub error_handler_ty: Type,
    pub accepts_generics_ident: Ident,
    pub accepts_field_ident: Ident,
}
impl HandlerErrorConfig {
    pub fn from_parts(
        error_handler_ty: Type,
        accepts_generics_ident: Ident,
        accepts_field_ident: Ident,
    ) -> Self {
        Self {
            error_handler_ty,
            accepts_generics_ident,
            accepts_field_ident,
        }
    }
}
