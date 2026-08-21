use documentation::Documentation;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::VecDeque;
use syn::punctuated::Punctuated;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute, Block, Ident, LitStr, Pat, PatType, Token, Type,
};

mod documentation;

fn get_pat_ident_name(pat_type: &PatType) -> Option<Ident> {
    // TODO: Need to make this handel any type of pattern, right now it only handles `x: i32` and not more complex patterns.
    match *pat_type.pat.clone() {
        Pat::Ident(pat_ident) => Some(pat_ident.ident),
        _ => None,
    }
}

/// Input to the `generate_endpoint` macro
///
/// # Example
///
/// ```rust
/// # use actix_helper_utils_macros::generate_endpoint;
/// generate_endpoint! {
///     fn login;
///     method: get;
///     path: "/health";
///     docs: {
///         tag: "health",
///         context_path: "/",
///         responses: {
///             (status = 200, description = "Everything works just fine!")
///         }
///     }
///     {
///         Ok(HttpResponse::Ok().body("Everything works just fine!"))
///     }
/// }
///```
///
pub(crate) fn generate_endpoint_internal(input: TokenStream) -> TokenStream1 {
    let input: TokenStream1 = input.into();

    let input = parse_macro_input!(input as GenerateEndpointInput);

    let GenerateEndpointInput {
        attrs,
        fn_name,
        method,
        path,
        params,
        fn_block,
        docs,
        response_error,
        return_type,
    } = input;

    let mut validate_fields = VecDeque::new();

    // Map method to the corresponding actix-web attribute
    let method_attr = match method.to_string().to_lowercase().as_str() {
        "get" => quote! { #[::actix_helper_utils::actix_web::#method(#path)] },
        "post" => quote! { #[::actix_helper_utils::actix_web::#method(#path)] },
        "put" => quote! { #[::actix_helper_utils::actix_web::#method(#path)] },
        "delete" => quote! { #[::actix_helper_utils::actix_web::#method(#path)] },
        _ => {
            return syn::Error::new_spanned(
                method,
                "Unsupported method. Expected one of: get, post, put, delete.",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate function parameters
    let fn_params = if let Some(params) = params.clone() {
        let params_iter = params.iter().map(|p| {
            let name = &p.name;
            quote! { #name }
        });

        params
            .iter()
            .filter(|&field| field.validate)
            .cloned()
            .for_each(|field| validate_fields.push_back(field));

        quote! { #( #params_iter ),* }
    } else {
        quote! {}
    };

    let docs_attr = if let Some(mut docs) = docs {
        docs.method = Some(method);
        docs.path = Some(path);
        quote! { #docs }
    } else {
        quote! {}
    };

    let return_ok_type = if let Some(ret_type) = return_type {
        quote! { #ret_type }
    } else {
        quote! { impl ::actix_helper_utils::actix_web::Responder }
    };

    let mut result = false;
    let return_type = if let Some(res_err) = response_error {
        result = true;
        quote! { Result<#return_ok_type, #res_err> }
    } else {
        quote! { #return_ok_type }
    };

    let fn_block = if validate_fields.is_empty() {
        quote! { #fn_block }
    } else {
        let error_logic = if !result {
            quote! {
                Err(error) => panic!("Validation failed with errors: {error}")
            }
        } else {
            quote! {
                Err(error) => {return Err(error.into());}
            }
        };

        let validate_stmts = validate_fields
            .iter()
            .filter_map(|field| {
                let ident = get_pat_ident_name(&field.name);

                ident.map(|name| quote! {
                    match ::actix_helper_utils::validator::Validate::validate(&#name.into_inner()) {
                        Ok(_) => {},
                        #error_logic,
                    }
                })
            })
            .collect::<Vec<_>>();

        let block_stmts = fn_block.stmts;

        quote! {
            {
                #(#validate_stmts)*

                #(#block_stmts)*
            }
        }
    };

    // Generate the function
    let expanded = quote! {
        #(#attrs)*
        #docs_attr
        #method_attr
        pub async fn #fn_name(
            #fn_params
        ) -> #return_type
        #fn_block
    };

    TokenStream1::from(expanded)
}

/// Endpoint input
///
/// # Examples
///
/// ```no_compile
/// #[allow(dead_code)]
/// fn my_endpoint;
/// method: get;
/// path: "/";
/// params: {
///     #[derive(Serialize, Deserialize, Debug, Clone)]
///     struct MyParams {
///         name: String
///     }
/// }
/// docs: {
///     context_path: "/api"
///     tag: "tag"
///     responses: {
///         (status = 200, description = "Request successful"),
///         (status = 404, description = "Not found")
///     }
/// }
/// {
///     HttpResponse::Ok().body("Hello, world!")
/// }
/// ```
///
pub(crate) struct GenerateEndpointInput {
    attrs: Vec<Attribute>,
    fn_name: Ident,
    method: Ident,
    path: LitStr,
    params: Option<Vec<Parameter>>,
    docs: Option<Documentation>,
    return_type: Option<Type>,
    response_error: Option<Type>,
    fn_block: Block,
}

/// Represents a parameter in a function signature
///
/// # Examples
///
/// ```no_compile
/// id: i32
/// ```
///
#[derive(Clone)]
struct Parameter {
    pub(crate) name: PatType,
    #[allow(dead_code)]
    pub(crate) ty: Type,
    validate: bool,
}

impl Parse for GenerateEndpointInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse outer attributes
        let attrs = input.call(Attribute::parse_outer)?;

        // Parse 'fn' keyword and function name
        input.parse::<Token![fn]>()?;
        let fn_name: Ident = input.parse()?;
        input.parse::<Token![;]>()?;

        let mut method: Option<_> = None;
        let mut path: Option<_> = None;
        let mut docs: Option<_> = None;
        let mut params: Option<_> = None;
        let mut response_error: Option<_> = None;
        let mut return_type: Option<_> = None;

        let mut ident = input.parse::<Ident>()?;

        while !input.is_empty() {
            input.parse::<Token![:]>()?;
            match ident.to_string().as_str() {
                "method" => {
                    if method.is_some() {
                        return Err(syn::Error::new_spanned(ident, "Duplicate method"));
                    }

                    let method_ident = input.parse::<Ident>()?;

                    match method_ident.to_string().as_str() {
                        "get" => method = Some(Ident::new("get", Span::call_site())),
                        "post" => method = Some(Ident::new("post", Span::call_site())),
                        "put" => method = Some(Ident::new("put", Span::call_site())),
                        "patch" => method = Some(Ident::new("patch", Span::call_site())),
                        "delete" => method = Some(Ident::new("delete", Span::call_site())),
                        _ => return Err(syn::Error::new_spanned(method_ident, "Invalid method")),
                    }
                }
                "path" => {
                    if path.is_some() {
                        return Err(syn::Error::new_spanned(ident, "Duplicate path"));
                    }

                    let path_lit = input.parse::<LitStr>()?;

                    path = Some(path_lit);
                }
                "docs" => {
                    if docs.is_some() {
                        return Err(syn::Error::new_spanned(ident, "Duplicate docs"));
                    }

                    let content;
                    syn::braced!(content in input);

                    docs = Some(content.parse()?);
                }
                "params" => {
                    if params.is_some() {
                        return Err(syn::Error::new_spanned(ident, "Duplicate params"));
                    }

                    let content;
                    syn::braced!(content in input);
                    let parsed_params =
                        Punctuated::<Parameter, Token![,]>::parse_terminated(&content)?;

                    params = Some(parsed_params.into_iter().collect());
                }
                "return_type" => {
                    if return_type.is_some() {
                        return Err(input.error("Duplicate return type"));
                    }

                    return_type = Some(input.parse()?)
                }
                "error" => {
                    if response_error.is_some() {
                        return Err(syn::Error::new_spanned(ident, "Duplicate error type"));
                    }

                    let error_ident = input.parse()?;

                    response_error = Some(error_ident);
                }
                _ => return Err(syn::Error::new_spanned(ident, "Unexpected field")),
            }

            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
            }

            // if the next token is not an identifier, we're done parsing and move on to the function body
            if !input.peek(Ident) {
                break;
            }

            ident = input.parse::<Ident>()?;
        }

        if method.is_none() {
            return Err(syn::Error::new_spanned(ident, "Missing 'method'"));
        }

        if path.is_none() {
            return Err(syn::Error::new_spanned(ident, "Missing 'path'"));
        }

        // Parse the call function expression
        let fn_block: Block = input.parse()?;

        Ok(GenerateEndpointInput {
            attrs,
            fn_name,
            method: method.unwrap(),
            path: path.unwrap(),
            params,
            docs,
            fn_block,
            response_error,
            return_type,
        })
    }
}

impl Parse for Parameter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: PatType = input.parse()?;

        let validate = {
            let prefix = input.parse::<Token![!]>().is_ok();
            let postfix_ident = input.parse::<Ident>();

            match postfix_ident {
                Ok(ident) => matches!(ident.to_string().as_str(), "validate") && prefix,
                Err(_) => false,
            }
        };

        let ty = *name.ty.clone();

        Ok(Parameter { name, ty, validate })
    }
}
