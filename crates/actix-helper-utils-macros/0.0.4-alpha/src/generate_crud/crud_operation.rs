use proc_macro2::TokenStream;
use quote::quote;
use syn::token::Brace;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Attribute, Block, Expr, Ident, ItemStruct, LitStr, Token, Visibility,
};

pub(crate) enum OperationType {
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Clone)]
pub(crate) struct CrudOperation {
    pub(crate) attrs: Vec<Attribute>,
    pub(crate) vis: Visibility,
    pub(crate) fn_name: Ident,
    pub(crate) endpoint_fn_name: Ident,
    pub(crate) path: LitStr,
    pub(crate) params_struct: Option<ItemStruct>,
    pub(crate) query_block: Option<Block>,
    pub(crate) before_hook: Option<Block>,
    pub(crate) after_hook: Option<Block>,
}
impl CrudOperation {
    pub(crate) fn generate(
        &self,
        base_path: &str,
        struct_ident: &Ident,
        operation_type: OperationType,
    ) -> TokenStream {
        // Generate the full path
        let full_path = if self.path.value().starts_with("/") {
            format!("{}{}", base_path, self.path.value())
        } else {
            format!("{}/{}", base_path, self.path.value())
        };
        let path_lit = LitStr::new(&full_path, self.path.span());

        // Include params struct if it exists
        let params_struct_tokens = if let Some(params_struct) = &self.params_struct {
            quote! { #params_struct }
        } else {
            quote! {}
        };

        // Determine parameters type for function signature
        let params_type = if let Some(params_struct) = &self.params_struct {
            let params_ident = &params_struct.ident;
            quote! { #params_ident }
        } else {
            quote! {}
        };

        // Define function parameters for `fn_name`
        let fn_params = if self.params_struct.is_some() {
            quote! {
                , data: &#params_type
            }
        } else {
            quote! {}
        };

        // Define function arguments for calling `fn_name`
        let fn_args = if self.params_struct.is_some() {
            quote! {
                , &data
            }
        } else {
            quote! {}
        };

        // Extract code blocks
        let query_block = if let Some(stmts) = self.query_block.as_ref().map(|block| &block.stmts) {
            stmts
        } else {
            &Vec::new()
        };
        let before_hook = if let Some(stmts) = self.before_hook.as_ref().map(|block| &block.stmts) {
            stmts
        } else {
            &Vec::new()
        };
        let after_hook = if let Some(stmts) = self.after_hook.as_ref().map(|block| &block.stmts) {
            stmts
        } else {
            &Vec::new()
        };

        let fn_attrs = &self.attrs;
        let fn_name = &self.fn_name;
        let endpoint_fn_name = &self.endpoint_fn_name;
        let vis = &self.vis;

        // Determine the HTTP method and function content based on the operation type
        let (http_method, function_body, return_type) = match operation_type {
            OperationType::Create => {
                let http_method = quote! { post };
                let function_body = quote! {
                    #(#before_hook)*

                    let result: Option<#struct_ident> = {
                        #(#query_block)*
                    };

                    #(#after_hook)*

                    if let Some(result) = result {
                        Ok(result)
                    } else {
                        Err(crate::error::ServerResponseError::InternalError("Error inserting into database".to_string()))
                    }
                };
                let return_type =
                    quote! { Result<#struct_ident, crate::error::ServerResponseError> };
                (http_method, function_body, return_type)
            }
            OperationType::Read => {
                let http_method = quote! { get };
                let function_body = quote! {
                    #(#before_hook)*

                    let result: Vec<#struct_ident> = {
                        #(#query_block)*
                    };

                    #(#after_hook)*

                    Ok(result)
                };
                let return_type =
                    quote! { Result<Vec<#struct_ident>, crate::error::ServerResponseError> };
                (http_method, function_body, return_type)
            }
            OperationType::Update => {
                let http_method = quote! { put };
                let function_body = quote! {
                    #(#before_hook)*

                    let result: Option<#struct_ident> = {
                        #(#query_block)*
                    };

                    #(#after_hook)*

                    if let Some(result) = result {
                        Ok(result)
                    } else {
                        Err(crate::error::ServerResponseError::NotFound)
                    }
                };
                let return_type =
                    quote! { Result<#struct_ident, crate::error::ServerResponseError> };
                (http_method, function_body, return_type)
            }
            OperationType::Delete => {
                let http_method = quote! { delete };
                let function_body = quote! {
                    #(#before_hook)*

                    {
                        #(#query_block)*
                    }

                    #(#after_hook)*

                    Ok(())
                };
                let return_type = quote! { Result<(), crate::error::ServerResponseError> };
                (http_method, function_body, return_type)
            }
        };

        // Generate the function
        let function = quote! {
            #(#fn_attrs)*
            #vis async fn #fn_name<C>(
                db: &::std::sync::Arc<::surrealdb::Surreal<C>>
                #fn_params
            ) -> #return_type
            where
                C: surrealdb::Connection,
            {
                #function_body
            }
        };

        // Define endpoint parameters
        let endpoint_params = if self.params_struct.is_some() {
            quote! {
                params: {
                    state: ::actix_web::web::Data<crate::AppState>,
                    data: ::actix_web::web::Json<#params_type>
                };
            }
        } else {
            quote! {
                params: {
                    state: ::actix_web::web::Data<crate::AppState>
                };
            }
        };

        // Define endpoint function body
        let endpoint_body = if self.params_struct.is_some() {
            quote! {
                {
                    let data = data.into_inner();
                    let db = &state.db;

                    match #fn_name(db #fn_args).await {
                        Ok(result) => Ok(::actix_web::HttpResponse::Ok().json(result)),
                        Err(err) => Err(err),
                    }
                }
            }
        } else {
            quote! {
                {
                    let db = &state.db;

                    match #fn_name(db).await {
                        Ok(result) => Ok(::actix_web::HttpResponse::Ok().json(result)),
                        Err(err) => Err(err),
                    }
                }
            }
        };

        let endpoint_tokens = quote! {
            #(#fn_attrs)*
            fn #endpoint_fn_name;
            method: #http_method;
            path: #path_lit;
            #endpoint_params
            #endpoint_body
        };

        // Return the combined tokens
        let tokens = quote! {
            #params_struct_tokens

            #function

            helper_macros::generate_endpoint! {
                #endpoint_tokens
            }
        };

        tokens
    }
}

impl Parse for CrudOperation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse attributes
        let attrs = input.call(Attribute::parse_outer)?;

        // Parse optional visibility
        let vis: Visibility = input.parse()?;

        // Parse 'fn' keyword and function name
        input.parse::<Token![fn]>()?;
        let fn_name: Ident = input.parse()?;
        input.parse::<Token![;]>()?;

        // Optionally parse documentation comments
        let _docs = input.call(Attribute::parse_outer)?;

        // Initialize optional fields
        let mut endpoint_fn_name: Option<Ident> = None;
        let mut path: Option<LitStr> = None;
        let mut params_struct: Option<ItemStruct> = None;
        let mut query_block: Option<Block> = None;
        let mut before_hook: Option<Block> = None;
        let mut after_hook: Option<Block> = None;

        // Parse remaining fields in any order
        while !input.is_empty() {
            // Parse identifier
            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match ident.to_string().as_str() {
                "endpoint" => {
                    if endpoint_fn_name.is_some() {
                        return Err(syn::Error::new_spanned(ident, "Duplicate 'endpoint' field"));
                    }
                    let value: Ident = input.parse()?;
                    endpoint_fn_name = Some(value);
                    input.parse::<Token![;]>()?;
                }
                "path" => {
                    if path.is_some() {
                        return Err(syn::Error::new_spanned(ident, "Duplicate 'path' field"));
                    }
                    let value: LitStr = input.parse()?;
                    path = Some(value);
                    input.parse::<Token![;]>()?;
                }
                "params" => {
                    if params_struct.is_some() {
                        return Err(syn::Error::new_spanned(ident, "Duplicate 'params' field"));
                    }
                    let content;
                    braced!(content in input);
                    let item_struct: ItemStruct = content.parse()?;
                    params_struct = Some(item_struct);
                }
                "query" => {
                    if query_block.is_some() {
                        return Err(syn::Error::new_spanned(ident, "Duplicate 'query' field"));
                    }

                    // Parse either an expression or a block
                    if input.peek(Brace) {
                        let block_content;
                        let brace_token = braced!(block_content in input);
                        let block = Block {
                            brace_token,
                            stmts: block_content.call(syn::Block::parse_within)?,
                        };
                        query_block = Some(block);
                    } else {
                        // If not a block, parse an expression and wrap it in a block
                        let expr: Expr = input.parse()?;
                        let block = Block {
                            brace_token: Brace::default(),
                            stmts: vec![syn::Stmt::Expr(expr, None)],
                        };
                        query_block = Some(block);
                    }
                }
                "before" => {
                    if before_hook.is_some() {
                        return Err(syn::Error::new_spanned(ident, "Duplicate 'before' field"));
                    }
                    // Parse a code block
                    let content;
                    let brace_token = braced!(content in input);
                    let block = Block {
                        brace_token,
                        stmts: content.call(syn::Block::parse_within)?,
                    };
                    before_hook = Some(block);
                    // No semicolon after block
                }
                "after" => {
                    if after_hook.is_some() {
                        return Err(syn::Error::new_spanned(ident, "Duplicate 'after' field"));
                    }
                    // Parse a code block
                    let content;
                    let brace_token = braced!(content in input);
                    let block = Block {
                        brace_token,
                        stmts: content.call(syn::Block::parse_within)?,
                    };
                    after_hook = Some(block);
                    // No semicolon after block
                }
                _ => {
                    return Err(syn::Error::new_spanned(ident, "Unexpected field"));
                }
            }
        }

        // Ensure required fields are present
        let endpoint_fn_name = endpoint_fn_name
            .ok_or_else(|| syn::Error::new(input.span(), "Missing required field 'endpoint'"))?;

        let path =
            path.ok_or_else(|| syn::Error::new(input.span(), "Missing required field 'path'"))?;

        Ok(CrudOperation {
            attrs,
            vis,
            fn_name,
            endpoint_fn_name,
            path,
            params_struct,
            query_block,
            before_hook,
            after_hook,
        })
    }
}
