//! Implementation of the `docker_service!` macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Block, Ident, Result, Token, Type,
};

/// Parsed input for the docker_service! macro.
pub struct DockerServiceMacroInput {
    pub name: Ident,
    pub image_type: Type,
    pub error_type: Type,
    pub client_type: Type,
    pub context_fields: Vec<ContextField>,
    pub construct_fn: Block,
    pub client_fn: Block,
    pub healthy_fn: Block,
}

/// A single context field definition.
pub struct ContextField {
    pub name: Ident,
    pub ty: Type,
}

impl Parse for DockerServiceMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse: ServiceName { ... }
        let name: Ident = input.parse()?;

        let content;
        braced!(content in input);

        // Parse image: Type,
        let image_keyword: Ident = content.parse()?;
        if image_keyword != "image" {
            return Err(syn::Error::new(image_keyword.span(), "expected 'image'"));
        }
        let _: Token![:] = content.parse()?;
        let image_type: Type = content.parse()?;
        let _: Token![,] = content.parse()?;

        // Parse error: Type,
        let error_keyword: Ident = content.parse()?;
        if error_keyword != "error" {
            return Err(syn::Error::new(error_keyword.span(), "expected 'error'"));
        }
        let _: Token![:] = content.parse()?;
        let error_type: Type = content.parse()?;
        let _: Token![,] = content.parse()?;

        // Parse client: Type,
        let client_keyword: Ident = content.parse()?;
        if client_keyword != "client" {
            return Err(syn::Error::new(client_keyword.span(), "expected 'client'"));
        }
        let _: Token![:] = content.parse()?;
        let client_type: Type = content.parse()?;
        let _: Token![,] = content.parse()?;

        // Parse context { fields }
        let context_keyword: Ident = content.parse()?;
        if context_keyword != "context" {
            return Err(syn::Error::new(
                context_keyword.span(),
                "expected 'context'",
            ));
        }
        let context_content;
        braced!(context_content in content);
        let context_fields = parse_fields(&context_content)?;

        // Parse async fn construct<I: testcontainers::Image>(container: &ContainerAsync<I>) -> Result<Self, Error> { ... }
        content.parse::<Token![async]>()?;
        content.parse::<Token![fn]>()?;
        let construct_keyword: Ident = content.parse()?;
        if construct_keyword != "construct" {
            return Err(syn::Error::new(
                construct_keyword.span(),
                "expected 'construct'",
            ));
        }
        // Skip the signature until we hit the block
        skip_until_block(&content)?;
        let construct_fn: Block = content.parse()?;

        // Parse async fn client(&self) -> Result<ClientType, ErrorType> { ... }
        content.parse::<Token![async]>()?;
        content.parse::<Token![fn]>()?;
        let client_keyword: Ident = content.parse()?;
        if client_keyword != "client" {
            return Err(syn::Error::new(client_keyword.span(), "expected 'client'"));
        }
        skip_until_block(&content)?;
        let client_fn: Block = content.parse()?;

        // Parse async fn healthy(&self) -> Result<(), ErrorType> { ... }
        content.parse::<Token![async]>()?;
        content.parse::<Token![fn]>()?;
        let healthy_keyword: Ident = content.parse()?;
        if healthy_keyword != "healthy" {
            return Err(syn::Error::new(
                healthy_keyword.span(),
                "expected 'healthy'",
            ));
        }
        skip_until_block(&content)?;
        let healthy_fn: Block = content.parse()?;

        Ok(DockerServiceMacroInput {
            name,
            image_type,
            error_type,
            client_type,
            context_fields,
            construct_fn,
            client_fn,
            healthy_fn,
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

fn parse_fields(content: ParseStream) -> Result<Vec<ContextField>> {
    let mut fields = Vec::new();

    while !content.is_empty() {
        let field_name: Ident = content.parse()?;
        let _: Token![:] = content.parse()?;
        let field_ty: Type = content.parse()?;
        let _: Token![,] = content.parse()?;

        fields.push(ContextField {
            name: field_name,
            ty: field_ty,
        });
    }

    Ok(fields)
}

/// Generates the code for a Docker service definition.
pub fn generate(input: DockerServiceMacroInput) -> TokenStream {
    let service_name = &input.name;
    let setup_name = quote::format_ident!("{}ServiceSetup", service_name);
    let context_name = quote::format_ident!("{}Context", service_name);

    let image_type = &input.image_type;
    let error_type = &input.error_type;
    let client_type = &input.client_type;

    // Extract field names and types
    let field_names: Vec<_> = input.context_fields.iter().map(|f| &f.name).collect();
    let field_types: Vec<_> = input.context_fields.iter().map(|f| &f.ty).collect();

    let construct_fn = &input.construct_fn;
    let client_fn = &input.client_fn;
    let healthy_fn = &input.healthy_fn;

    quote! {
        // Bring traits and types into scope
        #[allow(unused_imports)]
        use ::admixture_docker::{ContainerService, DockerContainerServiceSetup};

        // Type alias for the service setup
        pub type #setup_name = DockerContainerServiceSetup<#image_type, #context_name>;

        // Context struct that wraps container state
        pub struct #context_name {
            #(#field_names: #field_types,)*
        }

        // ContainerService trait implementation
        impl ContainerService for #context_name {
            type Client = #client_type;
            type Error = #error_type;

            async fn construct<I: ::testcontainers::Image>(
                container: &::testcontainers::ContainerAsync<I>,
            ) -> ::std::result::Result<Self, #error_type> #construct_fn

            async fn client(&self) -> ::std::result::Result<Self::Client, #error_type> #client_fn

            async fn healthy(&self) -> ::std::result::Result<(), #error_type> #healthy_fn
        }
    }
}
