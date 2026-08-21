use syn::{Type, TypeParam};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptsT {
    Type(Type),
    Generics(TypeParam),
}
