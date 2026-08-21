use syn::{Path, TypePath};

pub trait TypePathConstructExt {
    fn from_path(path: Path) -> TypePath;
}

impl TypePathConstructExt for TypePath {
    fn from_path(path: Path) -> TypePath {
        TypePath { qself: None, path }
    }
}
