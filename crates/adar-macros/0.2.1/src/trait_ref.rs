use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemTrait;

pub fn trait_ref_macro_inner(input: ItemTrait) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    Ok(quote! {
        #input

        impl<'a, T> adar::prelude::AsTraitRef<T> for dyn #ident
        where
            T: #ident + 'static,
        {
            fn as_trait_ref(value: &T) -> &Self {
                value
            }
        }

        impl<'a, T> adar::prelude::AsTraitMut<T> for dyn #ident
        where
            T: #ident + 'static,
        {
            fn as_trait_mut(value: &mut T) -> &mut Self {
                value
            }
        }
    }
    .into())
}
