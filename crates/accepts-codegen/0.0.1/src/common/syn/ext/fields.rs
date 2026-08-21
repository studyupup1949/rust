use syn::{Error, Field, Fields, FieldsNamed, Result};

pub trait FieldsExt {
    fn into_named(self) -> Result<FieldsNamed>;

    fn as_named(&self) -> Result<&FieldsNamed>;

    fn as_named_mut(&mut self) -> Result<&mut FieldsNamed>;

    fn expect_named(self, msg: &str) -> Result<FieldsNamed>;
}

impl FieldsExt for Fields {
    fn into_named(self) -> Result<FieldsNamed> {
        match self {
            Fields::Named(n) => Ok(n),
            other => Err(Error::new_spanned(other, "expected named fields")),
        }
    }

    fn as_named(&self) -> Result<&FieldsNamed> {
        match self {
            Fields::Named(n) => Ok(n),
            other => Err(Error::new_spanned(other, "expected named fields")),
        }
    }

    fn as_named_mut(&mut self) -> Result<&mut FieldsNamed> {
        match self {
            Fields::Named(n) => Ok(n),
            other => Err(Error::new_spanned(other, "expected named fields")),
        }
    }

    fn expect_named(self, msg: &str) -> Result<FieldsNamed> {
        match self {
            Fields::Named(n) => Ok(n),
            other => Err(Error::new_spanned(other, msg)),
        }
    }
}

pub trait FieldsEditExt {
    fn try_push_field(&mut self, field: Field) -> Result<()>;
}

impl FieldsEditExt for Fields {
    fn try_push_field(&mut self, field: Field) -> Result<()> {
        match self {
            Fields::Named(named) => {
                if field.ident.is_none() {
                    return Err(Error::new_spanned(
                        &field,
                        "expected a named field (missing ident)",
                    ));
                }
                named.named.push(field);
                Ok(())
            }
            Fields::Unnamed(unnamed) => {
                if let Some(ident) = &field.ident {
                    // ident 自体を指してあげるとメッセージが親切
                    return Err(Error::new_spanned(
                        ident,
                        "tuple field must not have an ident",
                    ));
                }
                unnamed.unnamed.push(field);
                Ok(())
            }
            Fields::Unit => Err(Error::new_spanned(self, "cannot push into unit fields")),
        }
    }
}
