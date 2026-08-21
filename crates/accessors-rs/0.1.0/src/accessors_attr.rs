use proc_macro2::Ident;
use quote::ToTokens;
use syn::{parse::Parser, punctuated::Punctuated, Meta, Path, Token};

use crate::error;

#[derive(Debug, Clone, Copy)]
pub enum GetKind {
    Ref,
    Copy,
}

#[derive(Debug, Default)]
pub struct AccessorsAttrData {
    pub get: Option<GetKind>,
    pub get_mut: Option<()>,
    pub set: Option<()>,
}

impl TryFrom<Meta> for AccessorsAttrData {
    type Error = syn::Error;

    fn try_from(meta: Meta) -> syn::Result<Self> {
        let mut accessors_attr_data = AccessorsAttrData::default();
        match meta {
            Meta::List(meta_list) => {
                let accessors_params = if is_accessors_path(&meta_list.path) {
                    let parser = Punctuated::<Ident, Token![,]>::parse_terminated;
                    parser.parse2(meta_list.nested.to_token_stream())
                } else {
                    Err(syn::Error::new_spanned(
                        meta_list.path,
                        error::ACCESSORS_ATTR_DATA_ONLY_TAKE_ACCESSORS_ATTR,
                    ))
                }?;

                for accessors_param in accessors_params {
                    match accessors_param.to_string().as_str() {
                        "get" => {
                            accessors_attr_data.get.replace(GetKind::Ref);
                        }
                        "get_copy" => {
                            accessors_attr_data.get.replace(GetKind::Copy);
                        }
                        "get_mut" => {
                            accessors_attr_data.get_mut.replace(());
                        }
                        "set" => {
                            accessors_attr_data.set.replace(());
                        }
                        arg @ _ => Err(syn::Error::new_spanned(
                            accessors_param,
                            error::invalid_accessors_arg_error_message(arg),
                        ))?,
                    }
                }
            }
            Meta::Path(path) => Err(syn::Error::new_spanned(
                path,
                error::ACCESSORS_ATTR_DATA_ONLY_TAKE_ACCESSORS_ATTR,
            ))?,
            Meta::NameValue(name_value) => Err(syn::Error::new_spanned(
                name_value,
                error::ACCESSORS_ATTR_DATA_ONLY_TAKE_ACCESSORS_ATTR,
            ))?,
        }
        Ok(accessors_attr_data)
    }
}

impl AccessorsAttrData {
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            get: other.get.or(self.get),
            get_mut: other.get_mut.or(self.get_mut),
            set: other.set.or(self.set),
        }
    }
}

pub fn is_accessors_path(path: &Path) -> bool {
    match path.get_ident() {
        Some(ident) => ident.to_string() == "accessors",
        None => false,
    }
}
