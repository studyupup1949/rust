use syn::{Ident, Path, Type};

use super::config::ForwardSource;

#[derive(Debug, Clone)]
pub struct HandlerSpec {
    pub source_field_ident: Ident,
    pub source_ty: Type,
    #[allow(dead_code)]
    pub source_state_ty: Type,
    pub handler_path: Path,
    pub handler_error: Option<HandlerErrorAcceptorSpec>,
    pub mut_guard: Option<MutGuardSpec>,
}

#[derive(Debug, Clone)]
pub struct HandlerErrorAcceptorSpec {
    pub generics_ident: Ident,
    pub field_ident: Ident,
    pub field_ty: Type,
    pub context_ty: Type,
}

#[derive(Debug, Clone)]
pub struct MutGuardSpec {
    pub guard_path: Path,
    pub guarded_ty: Type,
    pub guard_error: Option<MutGuardErrorAcceptorSpec>,
}

#[derive(Debug, Clone)]
pub struct MutGuardErrorAcceptorSpec {
    pub generics_ident: Ident,
    pub field_ident: Ident,
    pub field_ty: Type,
    pub error_ty: Type,
    pub forward_source: ForwardSource,
}
