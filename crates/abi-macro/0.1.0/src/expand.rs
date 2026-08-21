use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::ops::Deref;
use syn::spanned::Spanned;
use syn::{Error, FnArg, ItemFn, PatType, Path, ReturnType, Type};

pub fn expand_abi(args: TokenStream, input: TokenStream) -> Result<TokenStream, Error> {
    let abi_fn: ItemFn = syn::parse2(input)?;

    let crate_name = format_ident!("abi");

    let fn_name = abi_fn.sig.ident.clone();
    let raw_rtn_ty = rtn_type(&abi_fn)?;
    let rtn_matched = rtn_is_ptr(args);

    let (ffi_rtn_ty, ffi_null, ffi_rtn_trans) = {
        if rtn_matched {
            (
                quote! { abi::preclude::AbiPtr<#raw_rtn_ty> },
                quote! { AbiPtr::null() },
                quote! { into_ptr(result) },
            )
        } else {
            (
                quote! { abi::preclude::AbiString },
                quote! { AbiString::null() },
                quote! {
                    match into_json(result) {
                        Ok(val) => val,
                        Err(err) => {
                            print_error(&err.to_string());
                            return AbiString::null();
                        }
                    }
                },
            )
        }
    };

    let mut ffi_params = Vec::new();
    let mut ffi_values = Vec::new();
    let mut ffi_trans = Vec::new();

    for arg in &abi_fn.sig.inputs {
        let pat_ty = pat_type(&arg)?;
        let arg_name = pat_ty.pat.clone();
        let (is_reference, arg_ty) = match pat_ty.ty.deref() {
            Type::Reference(reference) => (true, reference.elem.clone()),
            _ => (false, pat_ty.ty.clone()),
        };

        let (param, value) = {
            if is_reference {
                (
                    quote! { #arg_name : abi::preclude::AbiPtr<#arg_ty>  },
                    quote! {
                        let #arg_name = {
                            match unsafe {from_ptr(#arg_name)} {
                                Ok(val) => val,
                                Err(err) => {
                                    print_error(&err.to_string());
                                    return #ffi_null;
                                }
                            }
                        };
                    },
                )
            } else {
                (
                    quote! { #arg_name : abi::preclude::AbiString },
                    quote! {
                        let #arg_name = {
                            match unsafe {from_json(#arg_name)} {
                                Ok(val) => val,
                                Err(err) => {
                                    print_error(&err.to_string());
                                    return #ffi_null;
                                }
                            }
                        };
                    },
                )
            }
        };
        ffi_params.push(param);
        ffi_values.push(value);
        ffi_trans.push(arg_name);
    }

    let expanded = quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn #fn_name ( #(#ffi_params),*) -> #ffi_rtn_ty {
            use #crate_name::preclude::*;

            #(#ffi_values)*

            #abi_fn

            let result = #fn_name(#(#ffi_trans),*);

            #ffi_rtn_trans
        }
    };

    Ok(TokenStream::from(expanded))
}

fn rtn_type(abi_fn: &ItemFn) -> Result<Box<Type>, Error> {
    let output = &abi_fn.sig.output;
    Ok(match output {
        ReturnType::Default => return Err(Error::new(output.span(), "invalid return")),
        ReturnType::Type(_, ty) => ty.clone(),
    })
}

fn rtn_is_ptr(args: TokenStream) -> bool {
    let abi_args: Option<Path> = syn::parse2(args).ok();
    abi_args
        .map(|e| e.to_token_stream().to_string() == "ptr")
        .unwrap_or(false)
}

fn pat_type(arg: &FnArg) -> Result<PatType, Error> {
    Ok(match arg {
        FnArg::Receiver(_) => return Err(Error::new(arg.span(), "invalid self")),
        FnArg::Typed(pat) => pat.clone(),
    })
}
