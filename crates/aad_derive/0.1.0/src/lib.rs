use proc_macro::TokenStream;
use quote::quote;
use syn::fold::Fold;
use syn::{parse_macro_input, parse_quote, spanned::Spanned, Ident, ItemFn};

struct ReplaceBaseTypeFolder {
    base_type: String,
    s_ident: Ident,
}

impl Fold for ReplaceBaseTypeFolder {
    fn fold_expr(&mut self, expr: syn::Expr) -> syn::Expr {
        if let syn::Expr::Lit(expr_lit) = &expr {
            if let syn::Lit::Float(float_lit) = &expr_lit.lit {
                return syn::parse_quote! { #float_lit };
            }
        }
        syn::fold::fold_expr(self, expr)
    }

    fn fold_type(&mut self, ty: syn::Type) -> syn::Type {
        if let syn::Type::Path(type_path) = &ty {
            if let Some(seg) = type_path.path.segments.last() {
                if seg.ident == self.base_type {
                    return syn::parse_str::<syn::Type>(&self.s_ident.to_string()).unwrap();
                }
            }
        }
        syn::fold::fold_type(self, ty)
    }
}

#[proc_macro_attribute]
pub fn autodiff(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;

    // Determine base type from return type
    let (return_base, return_span) = match &sig.output {
        syn::ReturnType::Default => {
            return syn::Error::new(sig.span(), "Function must have a return type of f32 or f64")
                .to_compile_error()
                .into();
        }
        syn::ReturnType::Type(_, ty) => {
            if let Some(base) = get_base_type(ty) {
                (base, ty.span())
            } else {
                return syn::Error::new(ty.span(), "Return type must be f32 or f64")
                    .to_compile_error()
                    .into();
            }
        }
    };

    // Check all arguments match the base type
    for arg in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let Some(arg_base) = get_base_type(&pat_type.ty) {
                if arg_base != return_base {
                    return syn::Error::new(
                        pat_type.ty.span(),
                        "All arguments must have the same type as the return (f32 or f64)",
                    )
                    .to_compile_error()
                    .into();
                }
            } else {
                return syn::Error::new(pat_type.ty.span(), "Argument type must be f32 or f64")
                    .to_compile_error()
                    .into();
            }
        } else {
            return syn::Error::new(arg.span(), "Function must have only typed arguments")
                .to_compile_error()
                .into();
        }
    }

    let t_ident = Ident::new(&return_base, return_span);
    let s_ident = Ident::new("S", return_span);

    // Transform function signature
    let mut generics = sig.generics.clone();
    generics
        .params
        .push(parse_quote!(S: crate::FloatLike<#t_ident>));

    let where_clause: syn::WhereClause = parse_quote! {
        where
            #t_ident: std::ops::Add<S, Output = S>,
            #t_ident: std::ops::Sub<S, Output = S>,
            #t_ident: std::ops::Mul<S, Output = S>,
            #t_ident: std::ops::Div<S, Output = S>
    };

    let new_args = sig.inputs.iter().map(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            let ident = &pat_type.pat;
            quote! { #ident: S }
        } else {
            unreachable!()
        }
    });

    let new_sig = syn::Signature {
        generics,
        output: parse_quote!(-> S),
        inputs: parse_quote!(#(#new_args),*),
        ..sig.clone()
    };

    // Transform the function block
    let mut folder = ReplaceBaseTypeFolder {
        base_type: return_base.clone(),
        s_ident: s_ident.clone(),
    };
    let transformed_block = folder.fold_block(*block.clone());

    let expanded = quote! {
        #vis #new_sig #where_clause {
            #transformed_block
        }
    };

    TokenStream::from(expanded)
}

fn get_base_type(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(type_path) = ty {
        type_path.path.segments.last().and_then(|seg| {
            let ident = seg.ident.to_string();
            if ident == "f32" || ident == "f64" {
                Some(ident)
            } else {
                None
            }
        })
    } else {
        None
    }
}
