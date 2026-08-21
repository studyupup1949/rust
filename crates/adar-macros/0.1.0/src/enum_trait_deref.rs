use proc_macro2::Span;
use quote::quote;
use syn::*;

pub fn enum_trait_deref_macro_inner(
    trai: TypeTraitObject,
    input: DeriveInput,
    with_mut: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    let Data::Enum(data_enum) = &input.data else {
        return Err(syn::Error::new(
            Span::call_site(),
            format!(
                "#[EnumTraitDeref{}] macro only supports enums",
                if with_mut { "Mut" } else { "" }
            ),
        ));
    };

    let variants = data_enum
        .variants
        .iter()
        .map(|variant| &variant.ident)
        .collect::<Vec<_>>();

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut_impl = if with_mut {
        quote! {
            impl #impl_generics ::core::ops::DerefMut for #ident #ty_generics #where_clause {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    match self {
                        #(Self::#variants(v) => v as &mut Self::Target,)*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #input

        impl #impl_generics ::core::ops::Deref for #ident #ty_generics #where_clause {
            type Target = dyn #trai;

            fn deref(&self) -> &Self::Target {
                match self {
                    #(Self::#variants(v) => v as &Self::Target,)*
                }
            }
        }

        #mut_impl
    }
    .into())
}
