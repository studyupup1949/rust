use proc_macro2::Span;
use quote::quote;
use syn::*;

pub fn reflect_enum_macro_inner(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let Data::Enum(data_enum) = &input.data else {
        return Err(syn::Error::new(
            Span::call_site(),
            "#[ReflectEnum] macro only supports enums",
        ));
    };

    let ident = &input.ident;
    let variants = data_enum
        .variants
        .iter()
        .map(|variant| {
            let name_str = &variant.ident.to_string();
            let variant_ident = &variant.ident;
            if matches!(variant.fields, Fields::Unit) {
                quote! {
                    EnumVariant::new(#name_str, Some(#ident::#variant_ident))
                }
            } else {
                quote! {
                    EnumVariant::new(#name_str, None)
                }
            }
        })
        .collect::<Vec<_>>();

    let variants2 = data_enum
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            let ident_str = ident.to_string();
            quote! {Self::#ident{..} => #ident_str}
        })
        .collect::<Vec<_>>();

    let name_impl = if variants2.is_empty() {
        quote! {""}
    } else {
        quote! {
            match self {
                #(#variants2),*
            }
        }
    };

    let count = variants.len();
    let repr = parse_str::<Type>(&enum_repr(&input))?;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let into_repr_impl = if data_enum
        .variants
        .iter()
        .all(|v| matches!(v.fields, Fields::Unit))
    {
        quote! {
            impl #impl_generics Into<#repr> for #ident #ty_generics #where_clause {
                fn into(self) -> #repr {
                    self as #repr
                }
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #input

        #into_repr_impl

        impl #impl_generics adar::prelude::ReflectEnum for #ident #ty_generics #where_clause {
            type Type = #repr;
            fn variants() -> &'static [adar::prelude::EnumVariant<#ident>] {
                const VARIANTS : &[adar::prelude::EnumVariant<#ident>] = &[#(#variants),*];
                VARIANTS
            }
            fn count() -> usize {
                #count
            }

            fn name(&self) -> &'static str {
                #name_impl
            }
        }
    }
    .into())
}

pub fn enum_repr(input: &DeriveInput) -> String {
    const DEFAULT_REPR: &str = "u32";
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            if let Ok(meta) = attr.parse_args() {
                if let syn::Meta::Path(path) = meta {
                    return path
                        .get_ident()
                        .map(|i| i.to_string())
                        .unwrap_or(DEFAULT_REPR.into());
                }
            }
        }
    }
    DEFAULT_REPR.into()
}
