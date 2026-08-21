use proc_macro2::Span;
use quote::quote;
use syn::*;

pub fn flag_enum_macro_inner(mut input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if let Data::Enum(data_enum) = &mut input.data {
        patch_flag_discriminants(data_enum)?;
    } else {
        return Err(syn::Error::new(
            Span::call_site(),
            "#[FlagEnum] macro only supports enums",
        ));
    }

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[derive(Copy, Clone)]
        #[ReflectEnum]
        #input

        impl #impl_generics std::ops::BitOr for #ident #ty_generics #where_clause
        where
            Self: adar::prelude::ReflectEnum
        {
            type Output = adar::prelude::Flags<Self>;

            fn bitor(self, rhs: Self) -> Self::Output {
                Flags::empty() | self | rhs
            }
        }
    }
    .into())
}

fn patch_flag_discriminants(data_enum: &mut DataEnum) -> syn::Result<()> {
    let mut value = 1;

    for variant in &mut data_enum.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new(
                Span::call_site(),
                "#[FlagEnum] macro only supports unit enums",
            ));
        }

        variant.discriminant = Some((
            Token![=](Span::call_site()),
            Expr::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Int(LitInt::new(&value.to_string(), Span::call_site())),
            }),
        ));

        value *= 2;
    }
    Ok(())
}
