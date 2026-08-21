//! # JSON-RPC Procedural Macros
//!
//! This crate provides a suite of procedural macros to automate the generation of
//! JSON-RPC 1.0/2.0 clients and services. It bridges the gap between standard Rust
//! functions/structs and the JSON-RPC wire format.
//!
//! ## Core Macros
//! - **`JsonRpcClient`**: A derive macro for request structures.
//! - **`rpc_method`**: An attribute macro for defining client-side calling interfaces.
//! - **`jsonrpc_service_fn_array`**: A server-side attribute for positional (array) parameters.
//! - **`jsonrpc_service_fn_obj`**: A server-side attribute for named (object) parameters.
use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, GenericArgument, PathArguments, ReturnType, Type};

#[derive(FromDeriveInput)]
#[darling(attributes(jsonrpc))]
struct Opts {
    url: String,
    content_type: String,
    method: String,
}

#[derive(FromField)]
#[darling(attributes(jsonrpc))]
struct FieldOpts {
    #[darling(default)]
    skip: bool,
}

#[derive(FromVariant)]
#[darling(attributes(jsonrpc))]
struct VariantOpts {
    #[darling(default)]
    skip: bool,
}

/// # `JsonRpcClient`
///
/// A derive macro that implements `JsonRpcClient` and `JsonRpcClientCall` traits for
/// a struct or enum.
///
/// It supports two modes of parameter serialization:
/// 1. **Positional (Array)**: Maps fields/variants to a JSON array.
/// 2. **Named (Object)**: Maps the entire structure to a JSON object.
///
/// ### Container Attributes
/// - `#[jsonrpc(url = "...", content_type = "...", method = "...")]`
///
/// ### Field/Variant Attributes
/// - `#[jsonrpc(skip)]`: Excludes the field or variant from parameter serialization.
#[proc_macro_derive(JsonRpcClient, attributes(jsonrpc))]
pub fn derive_json_rpc_client(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let opts = match Opts::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let struct_name = &input.ident;
    let (url, content_type, method) = (opts.url, opts.content_type, opts.method);

    let add_params_arm = match input.data {
        Data::Struct(ref data) => {
            let fields = data.fields.iter().enumerate().map(|(i, f)| {
                let field_opts = FieldOpts::from_field(f).unwrap();
                if field_opts.skip {
                    return quote! {};
                }

                if let Some(ref ident) = f.ident {
                    quote! { body.add_param(serde_json::to_value(self.#ident.clone()).unwrap_or(serde_json::Value::Null)); }
                } else {
                    let idx = syn::Index::from(i);
                    quote! { body.add_param(serde_json::to_value(self.#idx.clone()).unwrap_or(serde_json::Value::Null)); }
                }
            });
            quote! { #(#fields)* }
        }
        Data::Enum(ref data) => {
            let arms = data.variants.iter().map(|variant| {
                let variant_ident = &variant.ident;
                let variant_opts = VariantOpts::from_variant(variant).unwrap();

                if variant_opts.skip {
                    return quote! { Self::#variant_ident { .. } => {} };
                }

                match variant.fields {
                    syn::Fields::Named(ref fields) => {
                        let names = fields.named.iter().map(|f| &f.ident);
                        let idents = names.clone();
                        quote! {
                            Self::#variant_ident { #(#names),* } => {
                                #( body.add_param(serde_json::to_value(#idents.clone()).unwrap_or(serde_json::Value::Null)); )*
                            }
                        }
                    }
                    syn::Fields::Unnamed(ref fields) => {
                        let placeholder =
                            (0..fields.unnamed.len()).map(|i| quote::format_ident!("arg{}", i));
                        let idents = placeholder.clone();
                        quote! {
                            Self::#variant_ident ( #(#placeholder),* ) => {
                                #( body.add_param(serde_json::to_value(#idents.clone()).unwrap_or(serde_json::Value::Null)); )*
                            }
                        }
                    }
                    syn::Fields::Unit => {
                        quote! { Self::#variant_ident => {} }
                    }
                }
            });
            quote! {
                match self {
                    #(#arms)*
                }
            }
        }
        _ => panic!("JsonRpcClient only support Structs"),
    };
    let obj_add_params_block = quote! {
        use serde_json::Map;
        let mut val = serde_json::to_value(self.clone()).unwrap_or(serde_json::Value::Object(Map::new()));
        body.set_params(val);
    };

    let expanded = quote! {
        #[async_trait::async_trait]
        impl ::a_rs_jsonrpc::client::JsonRpcClient for #struct_name {
            async fn send_v1_request<R>(
                &self,
                url: &str,
                content_type: &str,
                method: &str,
            ) -> std::result::Result<::a_rs_jsonrpc::response::JsonRpcResponse<R>, ::a_rs_jsonrpc::error::RpcError>
            where
                R: serde::de::DeserializeOwned,
            {
                let id = ::a_rs_jsonrpc::JsonRpcId::next_number();
                let mut body: ::a_rs_jsonrpc::request::JsonRpcRequest<std::vec::Vec<serde_json::Value>> = ::a_rs_jsonrpc::request::JsonRpcRequest::new_v1(id, method);

                #add_params_arm
                tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));

                let resp = reqwest::Client::new()
                    .post(url)
                    .header("Content-Type", content_type)
                    .json(&body)
                    .send()
                    .await?;
                let text = resp.text().await?;
                tracing::debug!("jsonrpc response body: {}", text);
                Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
            }

            async fn send_v2_request<R>(
                &self,
                url: &str,
                content_type: &str,
                method: &str,
            ) -> std::result::Result<::a_rs_jsonrpc::response::JsonRpcResponse<R>, ::a_rs_jsonrpc::error::RpcError>
            where
                R: serde::de::DeserializeOwned,
            {
                let id = ::a_rs_jsonrpc::JsonRpcId::next_number();
                let mut body: ::a_rs_jsonrpc::request::JsonRpcRequest<std::vec::Vec<serde_json::Value>> = ::a_rs_jsonrpc::request::JsonRpcRequest::new_v2(id, method);

                #add_params_arm
                tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));

                let resp = reqwest::Client::new()
                    .post(url)
                    .header("Content-Type", content_type)
                    .json(&body)
                    .send()
                    .await?;
                let text = resp.text().await?;
                tracing::debug!("jsonrpc response body: {}", text);
                Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
            }

            async fn send_v1_request_obj<R>(
                &self,
                url: &str,
                content_type: &str,
                method: &str,
            ) -> std::result::Result<::a_rs_jsonrpc::response::JsonRpcResponse<R>, ::a_rs_jsonrpc::error::RpcError>
            where R: serde::de::DeserializeOwned
            {
                let id = ::a_rs_jsonrpc::JsonRpcId::next_number();
                let mut body = ::a_rs_jsonrpc::request::JsonRpcRequest::new_v1(id, method);
                #obj_add_params_block
                tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
                let resp = reqwest::Client::new().post(url).header("Content-Type", content_type).json(&body).send().await?;
                let text = resp.text().await?;
                tracing::debug!("jsonrpc response body: {}", text);
                Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
            }

            async fn send_v2_request_obj<R>(
                &self,
                url: &str,
                content_type: &str,
                method: &str,
            ) -> std::result::Result<::a_rs_jsonrpc::response::JsonRpcResponse<R>, ::a_rs_jsonrpc::error::RpcError>
            where R: serde::de::DeserializeOwned
            {
                let id = ::a_rs_jsonrpc::JsonRpcId::next_number();
                let mut body = ::a_rs_jsonrpc::request::JsonRpcRequest::new_v2(id, method);
                #obj_add_params_block
                tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
                let resp = reqwest::Client::new().post(url).header("Content-Type", content_type).json(&body).send().await?;
                let text = resp.text().await?;
                tracing::debug!("jsonrpc response body: {}", text);
                Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
            }
        }

        #[async_trait::async_trait]
        impl ::a_rs_jsonrpc::client::JsonRpcClientCall for #struct_name {
            async fn call_rpc_v1<R>(
                &self,
            ) -> std::result::Result<::a_rs_jsonrpc::response::JsonRpcResponse<R>, ::a_rs_jsonrpc::error::RpcError>
            where
                R: serde::de::DeserializeOwned,
            {
                self.send_v1_request(&#url, &#content_type, &#method).await
            }

            async fn call_rpc_v2<R>(
                &self,
            ) -> std::result::Result<::a_rs_jsonrpc::response::JsonRpcResponse<R>, ::a_rs_jsonrpc::error::RpcError>
            where
                R: serde::de::DeserializeOwned,
            {
                self.send_v2_request(&#url, &#content_type, &#method).await
            }

            async fn call_rpc_v1_obj<R>(
                &self,
            ) -> std::result::Result<::a_rs_jsonrpc::response::JsonRpcResponse<R>, ::a_rs_jsonrpc::error::RpcError>
            where R: serde::de::DeserializeOwned
            {
                self.send_v1_request_obj(&#url, &#content_type, &#method).await
            }

            async fn call_rpc_v2_obj<R>(
                &self,
            ) -> std::result::Result<::a_rs_jsonrpc::response::JsonRpcResponse<R>, ::a_rs_jsonrpc::error::RpcError>
            where R: serde::de::DeserializeOwned
            {
                self.send_v2_request_obj(&#url, &#content_type, &#method).await
            }
        }

        #[cfg(test)]
        impl #struct_name {
            pub fn debug_params_flatten(&self) -> Option<Vec<serde_json::Value>> {
                let mut body = ::a_rs_jsonrpc::request::JsonRpcRequest::<Vec<serde_json::Value>>::new_v2(
                    ::a_rs_jsonrpc::JsonRpcId::Number(1), "debug"
                );
                #add_params_arm
                body.params
            }

            pub fn debug_params_obj(&self) -> Option<serde_json::Value> {
                let mut body = ::a_rs_jsonrpc::request::JsonRpcRequest::<serde_json::Value>::new_v2(
                    ::a_rs_jsonrpc::JsonRpcId::Number(1), "debug"
                );
                #obj_add_params_block
                body.params
            }
        }
    };

    TokenStream::from(expanded)
}

use darling::FromMeta;
use syn::{FnArg, ItemFn, Pat};

#[derive(Debug, FromMeta)]
struct RpcMethodArgs {
    url: String,
    method: String,
    #[darling(default = "default_content_type")]
    content_type: String,
    #[darling(default = "default_version")]
    version: String,
    #[darling(default)]
    mode: String,
}

fn default_version() -> String {
    "v2".to_string()
}

fn default_content_type() -> String {
    "application/json".to_string()
}

/// # `rpc_method`
///
/// **Client-side attribute macro.**
///
/// Transforms an asynchronous function signature into a complete JSON-RPC client caller.
/// It automatically handles parameter packing and HTTP communication via `reqwest`.
///
/// ### Arguments
/// - `url`: The RPC endpoint URL.
/// - `method`: The remote method name.
/// - `mode`: (Optional) Use `"obj"` for named parameters.
/// - `version`: (Optional) `"v1"` or `"v2"`.
///
/// ### Example
/// ```rust
/// #[rpc_method(url = "http://localhost:8080", method = "sum")]
/// async fn get_sum(a: i32, b: i32) -> Result<JsonRpcResponse<i32>, RpcError> {}
/// ```
#[proc_macro_attribute]
pub fn rpc_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = match darling::ast::NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(darling::Error::from(e).write_errors()),
    };

    let args = match RpcMethodArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let input_fn = parse_macro_input!(item as ItemFn);
    let sig = &input_fn.sig;
    let vis = &input_fn.vis;
    let generics = &sig.generics;
    let (_, _ty_generics, where_clause) = generics.split_for_impl();

    let crate_root = quote! { ::a_rs_jsonrpc };

    let inner_t = extract_actual_data_type(&sig.output);

    let mut fields = Vec::new();
    let mut field_idents = Vec::new();
    for arg in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let id = &pat_ident.ident;
                let ty = &pat_type.ty;
                fields.push(quote! { pub #id: #ty });
                field_idents.push(quote! { #id });
            }
        }
    }

    let is_obj = args.mode == "obj";
    let url = args.url;
    let method = args.method;
    let content_type = args.content_type;
    let version_str = args.version.to_lowercase();

    let call_block = if !is_obj {
        let send_method = if version_str.contains("v1") {
            format_ident!("send_v1_request")
        } else {
            format_ident!("send_v2_request")
        };
        quote! {
            let params = (#(#field_idents.clone()),*);
            use #crate_root::client::JsonRpcClient;
            params.#send_method::<#inner_t>(#url, #content_type, #method).await
        }
    } else {
        let new_request_fn = if version_str.contains("v1") {
            format_ident!("new_v1")
        } else {
            format_ident!("new_v2")
        };
        quote! {
            #[derive(::serde::Serialize, ::std::clone::Clone)]
            #[serde(rename_all = "camelCase")]
            struct Helper #generics #where_clause { #(#fields),* }

            let helper = Helper { #(#field_idents: #field_idents.clone()),* };

            let id = #crate_root::JsonRpcId::next_number();
            let mut body = #crate_root::request::JsonRpcRequest::#new_request_fn(id, #method);

            let val = ::serde_json::to_value(helper).unwrap_or(::serde_json::Value::Object(::serde_json::Map::new()));
            body.set_params(val);

            tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
            let resp = ::reqwest::Client::new().post(#url).header("Content-Type", #content_type).json(&body).send().await?;

            let text = resp.text().await?;
            tracing::debug!("jsonrpc response body: {}", text);

            Ok(::serde_json::from_str::<#crate_root::response::JsonRpcResponse<#inner_t>>(&text)?)
        }
    };

    let expanded = quote! {
        #vis #sig #where_clause {
            #call_block
        }
    };

    TokenStream::from(expanded)
}

fn extract_actual_data_type(rt: &syn::ReturnType) -> proc_macro2::TokenStream {
    if let syn::ReturnType::Type(_, ty) = rt {
        if let syn::Type::Path(tp) = ty.as_ref() {
            if let Some(seg) = tp.path.segments.last() {
                if seg.ident == "Result" {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_tp))) =
                            args.args.first()
                        {
                            if let Some(inner_seg) = inner_tp.path.segments.last() {
                                if inner_seg.ident == "JsonRpcResponse" {
                                    if let syn::PathArguments::AngleBracketed(inner_args) =
                                        &inner_seg.arguments
                                    {
                                        if let Some(syn::GenericArgument::Type(actual_t)) =
                                            inner_args.args.first()
                                        {
                                            return quote! { #actual_t };
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    panic!("Macro requires return type: Result<JsonRpcResponse<T>, E>");
}

/// # `jsonrpc_service_fn_array`
///
/// **Server-side attribute macro for positional parameters.**
///
/// Use this on an `async fn` to register it as a JSON-RPC service. Incoming
/// `params` are expected to be a JSON Array.
///
/// ### Features
/// 1. Generates a boilerplate `RequestArray` struct for deserialization.
/// 2. Implements `JsonRpcServiceFn` for the generated struct.
/// 3. Automatically registers the function into the global `RPC_SERVICES` slice
///    using `linkme` for zero-cost discovery.
#[proc_macro_attribute]
pub fn jsonrpc_service_fn_array(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let name_str = fn_name.to_string();
    let struct_name_ident = format_ident!("{}RequestArray", name_str.to_case(Case::Pascal));

    let param_types: Vec<&Type> = input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(&*pat_type.ty)
            } else {
                None
            }
        })
        .collect();

    let result_inner_type = extract_result_t(&input_fn.sig.output);
    let args = parse_macro_input!(attr as RpcAttr);
    let version_val = args.version;
    let method_val = args.method;
    let registration_ident = format_ident!("REG_{}", fn_name.to_string().to_uppercase());

    let rpc = quote! { ::a_rs_jsonrpc };

    let has_params = !param_types.is_empty();

    let (tuple_params, call_logic) = if has_params {
        let ty = if param_types.len() == 1 {
            let t = param_types[0];
            quote! { (#t,) }
        } else {
            quote! { (#(#param_types),*) }
        };

        let param_indices: Vec<_> = (0..param_types.len())
            .map(|i| {
                let index = syn::Index::from(i);
                quote! { params.#index }
            })
            .collect();

        (
            quote! { #ty },
            quote! {
                let params = request.params.ok_or_else(|| {
                    #rpc::RpcError::InvalidParams(format!("Method '{}' requires array parameters", #method_val))
                })?;
                let result = #fn_name(#(#param_indices),*).await?;
            },
        )
    } else {
        (
            quote! { #rpc::serde_json::Value },
            quote! {
                let result = #fn_name().await?;
            },
        )
    };

    let expanded = quote! {
        #input_fn

        #[derive(Debug, #rpc::serde::Deserialize)]
        pub struct #struct_name_ident {
            pub jsonrpc: String,
            pub method: String,
            #[serde(default)]
            pub params: Option<#tuple_params>,
            pub id: #rpc::JsonRpcId,
        }

        #[#rpc::async_trait::async_trait]
        impl #rpc::JsonRpcServiceFn for #struct_name_ident {
            type Result = #result_inner_type;

            async fn handle(
                req: &[u8],
            ) -> Result<#rpc::JsonRpcResponse<Self::Result>, #rpc::RpcError> {
                let request: #struct_name_ident = #rpc::serde_json::from_slice(req)
                    .map_err(|e| #rpc::RpcError::SerdeError(e))?;

                if request.jsonrpc != #version_val {
                    return Err(#rpc::RpcError::InvalidJsonRpcVersion(format!(
                        "Expected JSON-RPC version {}, got {}",
                        #version_val, request.jsonrpc
                    )));
                }

                #call_logic

                let response = #rpc::JsonRpcResponse {
                    jsonrpc: request.jsonrpc.parse()?,
                    result: Some(result),
                    error: None,
                    id: request.id,
                };

                Ok(response)
            }
        }

        #[#rpc::linkme::distributed_slice(#rpc::RPC_SERVICES)]
        #[doc(hidden)]
        pub static #registration_ident: #rpc::RpcServiceEntry = #rpc::RpcServiceEntry {
            method: #method_val,
            handler: |req_bytes| {
                let req_bytes = req_bytes.to_vec();
                Box::pin(async move {
                    use #rpc::JsonRpcServiceFn;
                    let response = #struct_name_ident::handle(&req_bytes).await?;
                    Ok(#rpc::serde_json::to_string(&response)?)
                })
            },
        };
    };

    TokenStream::from(expanded)
}

fn extract_result_t(rt: &ReturnType) -> proc_macro2::TokenStream {
    if let ReturnType::Type(_, ty) = rt {
        if let Type::Path(tp) = ty.as_ref() {
            if let Some(seg) = tp.path.segments.last() {
                if seg.ident == "Result" {
                    if let PathArguments::AngleBracketed(args) = &seg.arguments {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return quote! { #inner_ty };
                        }
                    }
                }
            }
        }
    }
    panic!("Unable to extract Result<T> type from function return type");
}

/// # `jsonrpc_service_fn_obj`
///
/// **Server-side attribute macro for named parameters.**
///
/// Similar to `jsonrpc_service_fn_array`, but expects `params` to be a JSON Object.
/// It automatically maps JSON keys to function argument names using `camelCase`.
///
/// ### Registration
/// The function is registered as a `RpcServiceEntry` at compile-time, allowing
/// the server dispatcher to find it by the `method` string.
#[proc_macro_attribute]
pub fn jsonrpc_service_fn_obj(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = input_fn.sig.ident.clone();
    let name_pascal = fn_name.to_string().to_case(Case::Pascal);

    let params_struct_ident = format_ident!("{}ObjParams", name_pascal);
    let request_struct_ident = format_ident!("{}ObjRequest", name_pascal);

    let mut param_names = Vec::new();
    let mut param_types = Vec::new();

    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                param_names.push(&pat_ident.ident);
                param_types.push(&*pat_type.ty);
            }
        }
    }

    let result_inner_type = extract_result_t(&input_fn.sig.output);
    let args = parse_macro_input!(attr as RpcAttr);
    let version_val = args.version;
    let method_val = args.method;

    let registration_ident = format_ident!("REG_{}", fn_name.to_string().to_uppercase());

    let rpc = quote! { ::a_rs_jsonrpc };
    let call_logic = if param_names.is_empty() {
        quote! {
            let result = #fn_name().await?;
        }
    } else {
        quote! {
            let params = request.params.ok_or_else(|| {
                #rpc::RpcError::InvalidParams(format!("Method '{}' requires parameters", #method_val))
            })?;
            let result = #fn_name( #(params.#param_names),* ).await?;
        }
    };

    let expanded = quote! {
        #input_fn

        #[derive(Debug, #rpc::serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct #params_struct_ident {
            #(pub #param_names: #param_types),*
        }

        #[derive(Debug, #rpc::serde::Deserialize)]
        pub struct #request_struct_ident {
            pub jsonrpc: String,
            pub method: String,
            #[serde(default)]
            pub params: Option<#params_struct_ident>,
            pub id: #rpc::JsonRpcId,
        }

        #[#rpc::async_trait::async_trait]
        impl #rpc::JsonRpcServiceFn for #request_struct_ident {
            type Result = #result_inner_type;

            async fn handle(
                req: &[u8],
            ) -> Result<#rpc::JsonRpcResponse<Self::Result>, #rpc::RpcError>
            {
                let request: #request_struct_ident = #rpc::serde_json::from_slice(req)?;

                if request.jsonrpc != #version_val {
                    return Err(#rpc::RpcError::InvalidJsonRpcVersion(format!(
                        "Expected JSON-RPC version {}, got {}",
                        #version_val, request.jsonrpc
                    )));
                }

                #call_logic

                Ok(#rpc::JsonRpcResponse {
                    jsonrpc: request.jsonrpc.parse().map_err(|_| #rpc::RpcError::InvalidJsonRpcVersion(request.jsonrpc))?,
                    result: Some(result),
                    error: None,
                    id: request.id,
                })
            }
        }

        #[#rpc::linkme::distributed_slice(#rpc::RPC_SERVICES)]
        #[doc(hidden)]
        pub static #registration_ident: #rpc::RpcServiceEntry = #rpc::RpcServiceEntry {
            method: #method_val,
            handler: |req_bytes| {
                let req_data = req_bytes.to_vec();
                Box::pin(async move {
                    use #rpc::JsonRpcServiceFn;
                    let response = #request_struct_ident::handle(&req_data).await?;
                    Ok(#rpc::serde_json::to_string(&response)?)
                })
            },
        };
    };

    TokenStream::from(expanded)
}

use syn::{
    parse::{Parse, ParseStream},
    LitStr, Token,
};

struct RpcAttr {
    version: String,
    method: String,
}

impl Parse for RpcAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut version = String::new();
        let mut method = String::new();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;

            match ident.to_string().as_str() {
                "version" => version = value.value(),
                "method" => method = value.value(),
                _ => return Err(syn::Error::new(ident.span(), "Unknown attribute")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        match version.as_str() {
            "v1" | "V1" | "1.0" | "1" => version = "1.0".to_string(),
            "v2" | "V2" | "2.0" | "2" => version = "2.0".to_string(),
            _ => {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "version must be 'v1' or 'v2'",
                ))
            }
        }
        if method.is_empty() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "method cannot be empty",
            ));
        }
        Ok(RpcAttr { version, method })
    }
}
