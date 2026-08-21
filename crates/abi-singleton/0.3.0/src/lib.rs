extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use spanned::Spanned;
use syn::*;

fn func_ident(namespace: &str, trait_name: &str, fun_sig_ident: &Ident) -> Ident {
    format_ident!(
        "__api_{}_{}_{}",
        namespace,
        trait_name.to_lowercase(),
        fun_sig_ident
    )
}

pub fn api_trait(item: TokenStream, namespace: &str) -> TokenStream {
    let f = parse_macro_input!(item as ItemTrait);

    let trait_name = f.ident.to_string();

    let mut funcs = Vec::new();
    let mut default_funcs = Vec::new();

    for item in &f.items {
        if let TraitItem::Fn(func) = item {
            let ident = func.sig.ident.clone();
            let inputs = func.sig.inputs.clone();
            let output = func.sig.output.clone();
            let safe = func.sig.unsafety;

            let api_name = func_ident(namespace, &trait_name, &ident);

            if let Some(default) = &func.default {
                default_funcs.push(quote! {
                    #[unsafe( no_mangle)]
                    #[linkage="weak"]
                    unsafe extern "C" fn #api_name (#inputs) #output #default

                });
            }

            let mut args = Vec::new();

            for arg in &inputs {
                if let FnArg::Typed(t) = arg {
                    if let Pat::Ident(i) = t.pat.as_ref() {
                        let ident = &i.ident;
                        args.push(quote! { #ident , });
                    }
                }
            }
            funcs.push(quote! {

                pub #safe fn #ident (#inputs) #output{
                    unsafe extern "C" {
                        fn #api_name ( #inputs ) #output;
                    }

                    unsafe{ #api_name ( #(#args)* ) }
                }
            });
        } else {
            return parse::Error::new(item.span(), "only func is supported")
                .to_compile_error()
                .into();
        }
    }
    let struct_name = format_ident!("{}Impl", trait_name);

    quote! {
        #f
        pub struct #struct_name;

        impl #struct_name {
            #(#funcs)*
        }

        #(#default_funcs)*
    }
    .into()
}

pub fn api_impl(item: TokenStream, namespace: &str) -> TokenStream {
    let f = parse_macro_input!(item as ItemImpl);

    let mut funcs = Vec::new();

    let ty = f.self_ty.clone();

    let trait_name = f.trait_.as_ref().unwrap().1.get_ident().unwrap();

    for item in &f.items {
        if let ImplItem::Fn(func) = item {
            let ident = func.sig.ident.clone();
            let inputs = func.sig.inputs.clone();
            let output = func.sig.output.clone();

            let api_name = func_ident(namespace, &trait_name.to_string(), &func.sig.ident);

            let mut args = Vec::new();

            for arg in &inputs {
                if let FnArg::Typed(t) = arg {
                    if let Pat::Ident(i) = t.pat.as_ref() {
                        let ident = &i.ident;
                        args.push(quote! { #ident , });
                    }
                }
            }

            funcs.push(quote! {
                #[unsafe(no_mangle)]
                unsafe extern "C" fn #api_name (#inputs) #output{
                    #ty:: #ident ( #(#args)* )
                }
            });
        }
    }

    quote! {
        #f
        #(#funcs)*

    }
    .into()
}
