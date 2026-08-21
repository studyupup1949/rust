#[cfg(test)]
mod tests;

mod accessors_attr;
mod error;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Data, DeriveInput, Fields, Generics, Meta, Type};

use crate::accessors_attr::AccessorsAttrData;

/// Derive macro generating an impl for accessing the fields of a struct.\
/// Use `#[accessors(get, get_mut, set)]` to defined with accessors you want to have on a field.\
/// 
/// List of `accessors` param.
/// - `get`: Generate a getter returning a reference.\
/// - `get_copy`: Generate a getter returning a copy. (mutually exclusive with get)\
/// - `get_mut`: Generate a mutable getter returning a mutable reference.\
/// - `set`: Generate a setter.
/// 
/// Using `#[accessors(...)]` on a *field* will generate accessors for this specific field.\
/// Using `#[accessors(...)]` on a *struct* will generate accessors for all field in the struct.
///
#[proc_macro_derive(Accessors, attributes(accessors))]
pub fn accessors_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    accessors_derive_inner(input.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

fn accessors_derive_inner(input: TokenStream) -> syn::Result<TokenStream> {
    let DeriveInput {
        data,
        generics,
        ident,
        attrs,
        ..
    } = syn::parse2(input)?;

    let default_accessors_attr_data = parse_meta_vec_for_accessors_attrs(attrs)?;
    let default_accessors_attr_data = reduce_meta_iter_to_accessors_attr_data(default_accessors_attr_data)?;

    let mut accessors_fn_token_stream = TokenStream::new();
    for field in get_struct_fields(data)? {
        let accessors_attr_data = parse_meta_vec_for_accessors_attrs(field.attrs)?;
        let accessors_attr_data = reduce_meta_iter_to_accessors_attr_data(accessors_attr_data)?;

        if let Some(field_ident) = field.ident {
            get_accessors_func_for_field_token_stream(
                field_ident,
                field.ty,
                default_accessors_attr_data.merge(&accessors_attr_data),
            )
            .to_tokens(&mut accessors_fn_token_stream);
        }
    }

    Ok(get_impl_token_stream(
        ident,
        generics,
        accessors_fn_token_stream,
    ))
}

fn parse_meta_vec_for_accessors_attrs(
    struct_attrs: Vec<Attribute>,
) -> syn::Result<impl Iterator<Item = Meta>> {
    error::combine_syn_errors(
        struct_attrs
            .into_iter()
            .filter(|a| accessors_attr::is_accessors_path(&a.path))
            .map(|a| a.parse_meta()),
    )
}

fn reduce_meta_iter_to_accessors_attr_data(
    meta_iter: impl Iterator<Item = Meta>,
) -> syn::Result<AccessorsAttrData> {
    Ok(
        error::combine_syn_errors(meta_iter.map(AccessorsAttrData::try_from))?
            .reduce(|accessors_attr_data, current| accessors_attr_data.merge(&current))
            .unwrap_or_default(),
    )
}

fn get_struct_fields(data: Data) -> syn::Result<Fields> {
    match data {
        Data::Struct(struct_data) => Ok(struct_data.fields),
        Data::Enum(data_enum) => Err(syn::Error::new_spanned(
            data_enum.enum_token,
            error::ACCESSORS_ON_ENUM_ERROR_MESSAGE,
        )),
        Data::Union(data_union) => Err(syn::Error::new_spanned(
            data_union.union_token,
            error::ACCESSORS_ON_UNION_ERROR_MESSAGE,
        )),
    }
}

fn get_accessors_func_for_field_token_stream(
    field_ident: Ident,
    field_type: Type,
    accessors_attr_data: AccessorsAttrData,
) -> TokenStream {
    let get_fn = accessors_attr_data.get.map(|kind| {
        let ref_token = match kind {
            accessors_attr::GetKind::Ref => Some(quote! { & }),
            accessors_attr::GetKind::Copy => None,
        };
        quote! {
            pub fn #field_ident(&self) -> #ref_token #field_type {
                #ref_token self.#field_ident
            }
        }
    });

    let get_mut_fn = accessors_attr_data.get_mut.map(|_| {
        let field_ident_mut = format_ident!("{field_ident}_mut");
        quote! {
            pub fn #field_ident_mut(&mut self) -> &mut #field_type {
                &mut self.#field_ident
            }
        }
    });

    let set_fn = accessors_attr_data.set.map(|_| {
        let set_field_ident = format_ident!("set_{field_ident}");
        quote! {
            pub fn #set_field_ident(&mut self, #field_ident: #field_type) {
                self.#field_ident = #field_ident;
            }
        }
    });

    quote! {
        #get_fn
        #get_mut_fn
        #set_fn
    }
}

fn get_impl_token_stream(
    struct_ident: Ident,
    generics: Generics,
    accessors_fn_token_stream: TokenStream,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics #struct_ident #ty_generics #where_clause {
            #accessors_fn_token_stream
        }
    }
}
