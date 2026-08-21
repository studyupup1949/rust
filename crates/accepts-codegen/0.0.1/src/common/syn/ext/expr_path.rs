use syn::{Attribute, ExprPath, Path, QSelf};

pub trait ExprPathConstructExt {
    fn from_parts(attrs: Vec<Attribute>, qself: Option<QSelf>, path: Path) -> ExprPath;

    fn from_qself_path(qself: Option<QSelf>, path: Path) -> ExprPath;

    fn from_path(path: Path) -> ExprPath;
}

impl ExprPathConstructExt for ExprPath {
    fn from_parts(attrs: Vec<Attribute>, qself: Option<QSelf>, path: Path) -> ExprPath {
        ExprPath { attrs, qself, path }
    }

    fn from_qself_path(qself: Option<QSelf>, path: Path) -> ExprPath {
        Self::from_parts(Vec::new(), qself, path)
    }

    fn from_path(path: Path) -> ExprPath {
        Self::from_qself_path(None, path)
    }
}
