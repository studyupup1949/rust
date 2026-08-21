//! Implementation of the `service!` macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Block, Ident, Result, Token, Type,
};

/// Parsed input for the service! macro.
pub struct ServiceMacroInput {
    pub name: Ident,
    pub error_type: Type,
    pub client_type: Option<Type>,
    pub setup_fields: Vec<ServiceField>,
    pub running_fields: Vec<ServiceField>,
    pub start_fn: Block,
    pub client_fn: Option<Block>,
    pub healthy_fn: Option<Block>,
    pub stop_fn: Block,
}

/// A single field definition.
pub struct ServiceField {
    pub name: Ident,
    pub ty: Type,
}

impl Parse for ServiceMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse: ServiceName { ... }
        let name: Ident = input.parse()?;

        let content;
        braced!(content in input);

        // Initialize fields as None/Empty for order-independent parsing
        let mut error_type: Option<Type> = None;
        let mut client_type: Option<Type> = None;
        let mut setup_fields: Option<Vec<ServiceField>> = None;
        let mut running_fields: Option<Vec<ServiceField>> = None;
        let mut start_fn: Option<Block> = None;
        let mut client_fn: Option<Block> = None;
        let mut healthy_fn: Option<Block> = None;
        let mut stop_fn: Option<Block> = None;

        // Loop through all items in any order
        while !content.is_empty() {
            if content.peek(Token![async]) {
                // Parse async function
                parse_async_function(
                    &content,
                    &mut start_fn,
                    &mut client_fn,
                    &mut healthy_fn,
                    &mut stop_fn,
                )?;
            } else if content.peek(Ident) {
                // Could be: error, client, setup, or running
                let lookahead = content.fork();
                let keyword: Ident = lookahead.parse()?;

                match keyword.to_string().as_str() {
                    "error" => parse_error_type(&content, &mut error_type)?,
                    "client" => parse_client_type(&content, &mut client_type)?,
                    "setup" => parse_setup_block(&content, &mut setup_fields)?,
                    "running" => parse_running_block(&content, &mut running_fields)?,
                    _ => {
                        return Err(syn::Error::new(
                            keyword.span(),
                            format!(
                                "unexpected keyword '{}', expected one of: 'error', 'client', 'setup', 'running', or 'async'",
                                keyword
                            ),
                        ));
                    }
                }
            } else {
                // Unexpected token
                return Err(syn::Error::new(
                    content.span(),
                    "unexpected token, expected identifier or 'async'",
                ));
            }
        }

        // Validate required fields exist
        let error_type = error_type.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing required 'error: ErrorType,' field",
            )
        })?;

        let start_fn = start_fn.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing required 'async fn start(self) -> Result<...> { ... }'",
            )
        })?;

        let stop_fn = stop_fn.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing required 'async fn stop(&mut self) -> Result<()> { ... }'",
            )
        })?;

        Ok(ServiceMacroInput {
            name,
            error_type,
            client_type,
            setup_fields: setup_fields.unwrap_or_default(),
            running_fields: running_fields.unwrap_or_default(),
            start_fn,
            client_fn,
            healthy_fn,
            stop_fn,
        })
    }
}

fn skip_until_block(content: ParseStream) -> Result<()> {
    // Skip tokens until we find a brace (the start of the block)
    while !content.peek(syn::token::Brace) {
        content.parse::<proc_macro2::TokenTree>()?;
    }
    Ok(())
}

fn parse_fields(content: ParseStream) -> Result<Vec<ServiceField>> {
    let mut fields = Vec::new();

    while !content.is_empty() {
        let field_name: Ident = content.parse()?;
        let _: Token![:] = content.parse()?;
        let field_ty: Type = content.parse()?;
        let _: Token![,] = content.parse()?;

        fields.push(ServiceField {
            name: field_name,
            ty: field_ty,
        });
    }

    Ok(fields)
}

/// Parse error: Type,
fn parse_error_type(content: ParseStream, error_type: &mut Option<Type>) -> Result<()> {
    if error_type.is_some() {
        return Err(syn::Error::new(content.span(), "duplicate 'error' field"));
    }

    content.parse::<Ident>()?; // consume "error"
    content.parse::<Token![:]>()?;
    let ty = content.parse::<Type>()?;
    content.parse::<Token![,]>()?;

    *error_type = Some(ty);
    Ok(())
}

/// Parse client: Type,
fn parse_client_type(content: ParseStream, client_type: &mut Option<Type>) -> Result<()> {
    if client_type.is_some() {
        return Err(syn::Error::new(content.span(), "duplicate 'client' field"));
    }

    content.parse::<Ident>()?; // consume "client"
    content.parse::<Token![:]>()?;
    let ty = content.parse::<Type>()?;
    content.parse::<Token![,]>()?;

    *client_type = Some(ty);
    Ok(())
}

/// Parse setup { fields }
fn parse_setup_block(
    content: ParseStream,
    setup_fields: &mut Option<Vec<ServiceField>>,
) -> Result<()> {
    if setup_fields.is_some() {
        return Err(syn::Error::new(content.span(), "duplicate 'setup' block"));
    }

    content.parse::<Ident>()?; // consume "setup"
    let setup_content;
    braced!(setup_content in content);
    let fields = parse_fields(&setup_content)?;

    *setup_fields = Some(fields);
    Ok(())
}

/// Parse running { fields }
fn parse_running_block(
    content: ParseStream,
    running_fields: &mut Option<Vec<ServiceField>>,
) -> Result<()> {
    if running_fields.is_some() {
        return Err(syn::Error::new(content.span(), "duplicate 'running' block"));
    }

    content.parse::<Ident>()?; // consume "running"
    let running_content;
    braced!(running_content in content);
    let fields = parse_fields(&running_content)?;

    *running_fields = Some(fields);
    Ok(())
}

/// Parse async fn start/client/healthy/stop
fn parse_async_function(
    content: ParseStream,
    start_fn: &mut Option<Block>,
    client_fn: &mut Option<Block>,
    healthy_fn: &mut Option<Block>,
    stop_fn: &mut Option<Block>,
) -> Result<()> {
    content.parse::<Token![async]>()?;
    content.parse::<Token![fn]>()?;

    let fn_name: Ident = content.parse()?;

    match fn_name.to_string().as_str() {
        "start" => {
            if start_fn.is_some() {
                return Err(syn::Error::new(
                    fn_name.span(),
                    "duplicate 'start' function",
                ));
            }
            skip_until_block(content)?;
            *start_fn = Some(content.parse()?);
        }
        "client" => {
            if client_fn.is_some() {
                return Err(syn::Error::new(
                    fn_name.span(),
                    "duplicate 'client' function",
                ));
            }
            skip_until_block(content)?;
            *client_fn = Some(content.parse()?);
        }
        "healthy" => {
            if healthy_fn.is_some() {
                return Err(syn::Error::new(
                    fn_name.span(),
                    "duplicate 'healthy' function",
                ));
            }
            skip_until_block(content)?;
            *healthy_fn = Some(content.parse()?);
        }
        "stop" => {
            if stop_fn.is_some() {
                return Err(syn::Error::new(fn_name.span(), "duplicate 'stop' function"));
            }
            skip_until_block(content)?;
            *stop_fn = Some(content.parse()?);
        }
        _ => {
            return Err(syn::Error::new(
                fn_name.span(),
                format!(
                    "unexpected async function '{}', expected 'start', 'client', 'healthy', or 'stop'",
                    fn_name
                ),
            ));
        }
    }

    Ok(())
}

/// Generates the code for a service definition.
pub fn generate(input: ServiceMacroInput) -> TokenStream {
    let service_name = &input.name;
    let setup_name = quote::format_ident!("{}Setup", service_name);
    let running_name = quote::format_ident!("{}Running", service_name);
    let config_name = quote::format_ident!("{}Config", service_name);

    let error_type = &input.error_type;

    // Default to () if no client type specified
    let client_type = input
        .client_type
        .as_ref()
        .map(|ty| quote! { #ty })
        .unwrap_or_else(|| quote! { () });

    // Extract field names and types for Setup
    let setup_field_names: Vec<_> = input.setup_fields.iter().map(|f| &f.name).collect();
    let setup_field_types: Vec<_> = input.setup_fields.iter().map(|f| &f.ty).collect();

    // Extract field names and types for Running
    let running_field_names: Vec<_> = input.running_fields.iter().map(|f| &f.name).collect();
    let running_field_types: Vec<_> = input.running_fields.iter().map(|f| &f.ty).collect();

    let start_fn = &input.start_fn;
    let stop_fn = &input.stop_fn;

    // Default client function if not provided
    let client_impl = if let Some(client_fn) = &input.client_fn {
        quote! {
            async fn client(&self) -> ::std::result::Result<Self::Client, #error_type> #client_fn
        }
    } else {
        quote! {
            async fn client(&self) -> ::std::result::Result<Self::Client, #error_type> {
                Ok(())
            }
        }
    };

    // Default healthy function if not provided
    let healthy_impl = if let Some(healthy_fn) = &input.healthy_fn {
        quote! {
            async fn healthy(&self) -> ::std::result::Result<(), #error_type> #healthy_fn
        }
    } else {
        quote! {
            async fn healthy(&self) -> ::std::result::Result<(), #error_type> {
                Ok(())
            }
        }
    };

    quote! {
        // Config struct (same as setup fields)
        pub struct #config_name {
            #(pub #setup_field_names: #setup_field_types,)*
        }

        // Setup struct
        pub struct #setup_name {
            #(pub #setup_field_names: #setup_field_types,)*
        }

        impl ::admixture::service::ServiceSetup for #setup_name {
            type Running = #running_name;
            type Error = #error_type;
            type Config = #config_name;

            fn construct(config: Self::Config) -> Self {
                Self {
                    #(#setup_field_names: config.#setup_field_names,)*
                }
            }

            async fn start(self) -> ::std::result::Result<Self::Running, #error_type> #start_fn
        }

        // Running struct
        pub struct #running_name {
            #(#running_field_names: #running_field_types,)*
        }

        impl ::admixture::service::ServiceRunning for #running_name {
            type Client = #client_type;
            type Error = #error_type;

            #client_impl

            #healthy_impl

            async fn stop(&mut self) -> ::std::result::Result<(), #error_type> #stop_fn
        }
    }
}
