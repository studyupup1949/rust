use syn::{Ident, Type};

#[derive(Debug, Clone)]
pub struct NextAcceptorSpec {
    pub generics_ident: Ident,
    pub field_ident: Ident,
    pub field_ty: Type,
    pub accepts_ty: Type,
}
