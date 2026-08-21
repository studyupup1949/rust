use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, ItemStruct};

pub fn check_type_id(item: TokenStream) -> Result<TokenStream, Error> {
    let item: ItemStruct = syn::parse2(item)?;
    let name = item.ident.clone();
    let free = format_ident!("__free_{name}");
    Ok(quote! {
        impl abi::preclude::CheckTypeId for #name {
            fn check(&self) -> bool {
                self.magic == std::any::TypeId::of::<#name>()
            }
        }
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub extern "C" fn #free(ptr: abi::preclude::AbiPtr<#name>) -> bool {
            unsafe {
                match ptr.free() {
                    Ok(_) => true,
                    Err(err) => {
                        abi::preclude::print_error(format!("{} error: {}", stringify!(#free), err));
                        false
                    }
                }
            }
        }
    }
    .into())
}
