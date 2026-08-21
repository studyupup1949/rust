use syn::{Expr, MetaNameValue, Path, token::Eq};

pub trait MetaNameValueConstructExt {
    fn from_parts(path: Path, eq_token: Eq, value: Expr) -> MetaNameValue;

    fn from_path_value(path: Path, value: Expr) -> MetaNameValue;
}

impl MetaNameValueConstructExt for MetaNameValue {
    fn from_parts(path: Path, eq_token: Eq, value: Expr) -> MetaNameValue {
        MetaNameValue {
            path,
            eq_token,
            value,
        }
    }

    fn from_path_value(path: Path, value: Expr) -> MetaNameValue {
        <Self as MetaNameValueConstructExt>::from_parts(path, Eq::default(), value)
    }
}
