//! Implementation of the `context!` macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Expr, Ident, Result, Token, Type,
};

/// Parsed input for the context! macro.
pub struct ContextMacroInput {
    pub name: Ident,
    pub services: Vec<ServiceField>,
    pub hooks: Option<HooksBlock>,
}

/// A single service field definition.
pub struct ServiceField {
    pub name: Ident,
    pub ty: Type,
    pub config: Option<Expr>,
}

/// Parsed hooks block.
pub struct HooksBlock {
    pub before_all: Option<syn::Path>,
    pub after_all: Option<syn::Path>,
    pub before_each: Option<syn::Path>,
    pub after_each: Option<syn::Path>,
}

/// Parse a hooks block: hooks { before_all = path, ... }
fn parse_hooks_block(content: ParseStream) -> Result<HooksBlock> {
    let hooks_content;
    braced!(hooks_content in content);

    let mut before_all = None;
    let mut after_all = None;
    let mut before_each = None;
    let mut after_each = None;

    while !hooks_content.is_empty() {
        let hook_name: Ident = hooks_content.parse()?;
        let _: Token![=] = hooks_content.parse()?;
        let hook_path: syn::Path = hooks_content.parse()?;

        match hook_name.to_string().as_str() {
            "before_all" => {
                if before_all.is_some() {
                    return Err(syn::Error::new(
                        hook_name.span(),
                        "duplicate 'before_all' hook",
                    ));
                }
                before_all = Some(hook_path);
            }
            "after_all" => {
                if after_all.is_some() {
                    return Err(syn::Error::new(
                        hook_name.span(),
                        "duplicate 'after_all' hook",
                    ));
                }
                after_all = Some(hook_path);
            }
            "before_each" => {
                if before_each.is_some() {
                    return Err(syn::Error::new(
                        hook_name.span(),
                        "duplicate 'before_each' hook",
                    ));
                }
                before_each = Some(hook_path);
            }
            "after_each" => {
                if after_each.is_some() {
                    return Err(syn::Error::new(
                        hook_name.span(),
                        "duplicate 'after_each' hook",
                    ));
                }
                after_each = Some(hook_path);
            }
            _ => {
                return Err(syn::Error::new(
                    hook_name.span(),
                    format!("unknown hook '{}', expected one of: before_all, after_all, before_each, after_each", hook_name)
                ));
            }
        }

        // Optional trailing comma
        if !hooks_content.is_empty() {
            let _: Token![,] = hooks_content.parse()?;
        }
    }

    Ok(HooksBlock {
        before_all,
        after_all,
        before_each,
        after_each,
    })
}

impl Parse for ContextMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse: ContextName { ... }
        let name: Ident = input.parse()?;

        let content;
        braced!(content in input);

        let mut services = Vec::new();
        let mut hooks = None;

        // Order-independent parsing loop
        while !content.is_empty() {
            let lookahead = content.fork();
            if let Ok(keyword) = lookahead.parse::<Ident>()
                && keyword == "hooks"
            {
                if hooks.is_some() {
                    return Err(syn::Error::new(keyword.span(), "duplicate 'hooks' block"));
                }
                content.parse::<Ident>()?; // consume "hooks"
                hooks = Some(parse_hooks_block(&content)?);

                // Optional trailing comma
                if !content.is_empty() && content.peek(Token![,]) {
                    let _: Token![,] = content.parse()?;
                }
                continue;
            }

            // Parse as service field: name: Type or name: Type = expr
            let field_name: Ident = content.parse()?;
            let _: Token![:] = content.parse()?;
            let field_ty: Type = content.parse()?;

            // Check for optional config expression: = expr
            let config = if content.peek(Token![=]) {
                let _: Token![=] = content.parse()?;
                Some(content.parse::<Expr>()?)
            } else {
                None
            };

            services.push(ServiceField {
                name: field_name,
                ty: field_ty,
                config,
            });

            // Optional trailing comma
            if !content.is_empty() {
                let _: Token![,] = content.parse()?;
            }
        }

        Ok(ContextMacroInput {
            name,
            services,
            hooks,
        })
    }
}

/// Generates the code for a context definition.
pub fn generate(input: ContextMacroInput) -> TokenStream {
    let context_name = &input.name;
    let setup_name = quote::format_ident!("{}Setup", context_name);
    let running_name = quote::format_ident!("{}Running", context_name);
    let config_name = quote::format_ident!("{}Config", context_name);

    // Extract service names, types, and configs
    let service_names: Vec<_> = input.services.iter().map(|s| &s.name).collect();
    let service_types: Vec<_> = input.services.iter().map(|s| &s.ty).collect();
    let service_configs: Vec<_> = input.services.iter().map(|s| &s.config).collect();

    // Generate Config struct
    let config_struct = generate_config_struct(
        &config_name,
        &service_names,
        &service_types,
        &service_configs,
    );

    // Generate Setup struct
    let setup_struct = generate_setup_struct(&setup_name, &service_names, &service_types);

    // Generate Running struct
    let running_struct = generate_running_struct(&running_name, &service_names, &service_types);

    // Generate ContextSetup impl
    let context_setup_impl = generate_context_setup_impl(
        &setup_name,
        &running_name,
        &config_name,
        &service_names,
        &service_types,
    );

    // Generate ContextRunning impl
    let context_running_impl = generate_context_running_impl(&running_name, &service_names);

    // Generate type alias and constructor
    let type_alias_and_constructor = generate_type_alias_and_constructor(
        context_name,
        &setup_name,
        &running_name,
        &service_names,
        &service_types,
    );

    // Generate hooks infrastructure
    let hooks_infrastructure = generate_hooks_infrastructure(context_name, &input.hooks);

    quote! {
        #config_struct
        #setup_struct
        #running_struct
        #context_setup_impl
        #context_running_impl
        #type_alias_and_constructor
        #hooks_infrastructure
    }
}

/// Generates the hooks infrastructure (static with direct function pointers).
fn generate_hooks_infrastructure(context_name: &Ident, hooks: &Option<HooksBlock>) -> TokenStream {
    let _running_name = quote::format_ident!("{}Running", context_name);
    let hooks_static_name =
        quote::format_ident!("{}_HOOKS", context_name.to_string().to_uppercase());

    if let Some(hooks_block) = hooks {
        // Generate wrapper functions for each hook that boxes the future
        // Note: Wrappers take the public Context type and extract the Running type
        let mut wrapper_fns = Vec::new();
        
        let before_all_field = if let Some(path) = &hooks_block.before_all {
            let wrapper_name = quote::format_ident!("__before_all_wrapper_{}", context_name.to_string().to_lowercase());
            wrapper_fns.push(quote! {
                #[doc(hidden)]
                fn #wrapper_name<'a>(
                    ctx: &'a #context_name
                ) -> ::std::pin::Pin<::std::boxed::Box<
                    dyn ::std::future::Future<
                        Output = ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error + Send>>
                    > + Send + 'a
                >> {
                    use ::futures::FutureExt;
                    #path(ctx.__running_ctx()).boxed()
                }
            });
            quote! { before_all: Some(#wrapper_name) }
        } else {
            quote! { before_all: None }
        };

        let after_all_field = if let Some(path) = &hooks_block.after_all {
            let wrapper_name = quote::format_ident!("__after_all_wrapper_{}", context_name.to_string().to_lowercase());
            wrapper_fns.push(quote! {
                #[doc(hidden)]
                fn #wrapper_name<'a>(
                    ctx: &'a #context_name
                ) -> ::std::pin::Pin<::std::boxed::Box<
                    dyn ::std::future::Future<
                        Output = ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error + Send>>
                    > + Send + 'a
                >> {
                    use ::futures::FutureExt;
                    #path(ctx.__running_ctx()).boxed()
                }
            });
            quote! { after_all: Some(#wrapper_name) }
        } else {
            quote! { after_all: None }
        };

        let before_each_field = if let Some(path) = &hooks_block.before_each {
            let wrapper_name = quote::format_ident!("__before_each_wrapper_{}", context_name.to_string().to_lowercase());
            wrapper_fns.push(quote! {
                #[doc(hidden)]
                fn #wrapper_name<'a>(
                    ctx: &'a #context_name
                ) -> ::std::pin::Pin<::std::boxed::Box<
                    dyn ::std::future::Future<
                        Output = ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error + Send>>
                    > + Send + 'a
                >> {
                    use ::futures::FutureExt;
                    #path(ctx.__running_ctx()).boxed()
                }
            });
            quote! { before_each: Some(#wrapper_name) }
        } else {
            quote! { before_each: None }
        };

        let after_each_field = if let Some(path) = &hooks_block.after_each {
            let wrapper_name = quote::format_ident!("__after_each_wrapper_{}", context_name.to_string().to_lowercase());
            wrapper_fns.push(quote! {
                #[doc(hidden)]
                fn #wrapper_name<'a>(
                    ctx: &'a #context_name
                ) -> ::std::pin::Pin<::std::boxed::Box<
                    dyn ::std::future::Future<
                        Output = ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error + Send>>
                    > + Send + 'a
                >> {
                    use ::futures::FutureExt;
                    #path(ctx.__running_ctx()).boxed()
                }
            });
            quote! { after_each: Some(#wrapper_name) }
        } else {
            quote! { after_each: None }
        };

        quote! {
            #(#wrapper_fns)*

            #[doc(hidden)]
            pub static #hooks_static_name: admixture::hooks::Hooks<#context_name> =
                admixture::hooks::Hooks {
                    #before_all_field,
                    #after_all_field,
                    #before_each_field,
                    #after_each_field,
                    _phantom: ::std::marker::PhantomData,
                };
        }
    } else {
        // No hooks - all None
        quote! {
            #[doc(hidden)]
            pub static #hooks_static_name: admixture::hooks::Hooks<#context_name> =
                admixture::hooks::Hooks {
                    before_all: None,
                    after_all: None,
                    before_each: None,
                    after_each: None,
                    _phantom: ::std::marker::PhantomData,
                };
        }
    }
}

fn generate_config_struct(
    config_name: &Ident,
    service_names: &[&Ident],
    service_types: &[&Type],
    service_configs: &[&Option<Expr>],
) -> TokenStream {
    // Generate Default fields - use provided config or Config::default()
    let default_fields = service_names
        .iter()
        .zip(service_types.iter())
        .zip(service_configs.iter())
        .map(|((name, ty), config)| {
            if let Some(expr) = config {
                quote! {
                    #name: #expr
                }
            } else {
                quote! {
                    #name: <
                        <#ty as ::admixture::service::ServiceSetup>::Config
                        as ::std::default::Default
                    >::default()
                }
            }
        });

    quote! {
        pub struct #config_name {
            #(
                pub #service_names: <#service_types as ::admixture::service::ServiceSetup>::Config,
            )*
        }

        impl ::std::default::Default for #config_name {
            fn default() -> Self {
                Self {
                    #(#default_fields,)*
                }
            }
        }
    }
}

fn generate_setup_struct(
    setup_name: &Ident,
    service_names: &[&Ident],
    service_types: &[&Type],
) -> TokenStream {
    quote! {
        struct #setup_name {
            #(
                #service_names: #service_types,
            )*
        }
    }
}

fn generate_running_struct(
    running_name: &Ident,
    service_names: &[&Ident],
    service_types: &[&Type],
) -> TokenStream {
    quote! {
        struct #running_name {
            #(
                pub #service_names: <#service_types as ::admixture::service::ServiceSetup>::Running,
            )*
        }
    }
}

fn generate_context_setup_impl(
    setup_name: &Ident,
    running_name: &Ident,
    config_name: &Ident,
    service_names: &[&Ident],
    service_types: &[&Type],
) -> TokenStream {
    // Generate sequential service startup code
    let startup_code = service_names.iter().map(|name| {
        quote! {
            let #name = ::admixture::service::ServiceSetup::start(self.#name)
                .await
                .map_err(|e| ::admixture::context::ContextError::ServiceStartFailed {
                    source: ::std::boxed::Box::new(e),
                })?;
            ::admixture::context::wait_until_healthy_with_config(&#name, config).await?;
        }
    });

    // Generate construction code
    let construct_fields = service_names
        .iter()
        .zip(service_types.iter())
        .map(|(name, ty)| {
            quote! {
                #name: <#ty as ::admixture::service::ServiceSetup>::construct(config.#name)
            }
        });

    quote! {
        impl ::admixture::context::ContextSetup for #setup_name {
            type Running = #running_name;
            type Error = ::admixture::context::ContextError;
            type Config = #config_name;

            fn construct(config: Self::Config) -> Self {
                Self {
                    #(#construct_fields,)*
                }
            }

            async fn start_all(
                self,
                config: &::admixture::context::ContextConfig,
            ) -> ::std::result::Result<Self::Running, Self::Error> {
                #(#startup_code)*

                Ok(#running_name {
                    #(#service_names,)*
                })
            }
        }
    }
}

fn generate_context_running_impl(running_name: &Ident, service_names: &[&Ident]) -> TokenStream {
    // Generate shutdown code in reverse order
    let shutdown_code = service_names.iter().rev().map(|name| {
        quote! {
            ::admixture::service::ServiceRunning::stop(&mut self.#name)
                .await
                .map_err(|e| ::admixture::context::ContextError::ShutdownFailed {
                    source: ::std::boxed::Box::new(e),
                })?;
        }
    });

    quote! {
        impl ::admixture::context::ContextRunning for #running_name {
            type Error = ::admixture::context::ContextError;

            async fn stop_all(&mut self) -> ::std::result::Result<(), Self::Error> {
                #(#shutdown_code)*
                Ok(())
            }
        }
    }
}

fn generate_type_alias_and_constructor(
    context_name: &Ident,
    setup_name: &Ident,
    running_name: &Ident,
    service_names: &[&Ident],
    service_types: &[&Type],
) -> TokenStream {
    // Create the builder type name by appending "Builder" to the context name
    let builder_name = Ident::new(&format!("{}Builder", context_name), context_name.span());

    // Generate accessor methods for each service field
    let service_accessors = service_names
        .iter()
        .zip(service_types.iter())
        .map(|(name, ty)| {
            // Get the Running type from the Setup type
            quote! {
                /// Access the running service.
                pub fn #name(&self) -> &<#ty as ::admixture::service::ServiceSetup>::Running {
                    &self.0.#name
                }
            }
        });

    quote! {
        // Newtype wrapper for the context
        pub struct #context_name(::admixture::context::StoppableContext<#running_name>);

        impl #context_name {
            /// Create a new context builder.
            pub fn new(setup: #setup_name) -> #builder_name {
                #builder_name(::admixture::context::ContextBuilder::new(setup))
            }

            /// Stop the context and all its services.
            pub async fn stop(self) -> ::std::result::Result<(), ::admixture::context::ContextError> {
                self.0.stop().await
            }

            /// Get a reference to the running context (for internal hook use).
            #[doc(hidden)]
            pub fn __running_ctx(&self) -> &#running_name {
                &self.0
            }

            // Generate accessor methods for each service
            #(#service_accessors)*
        }

        // Builder wrapper to return the newtype
        pub struct #builder_name(::admixture::context::ContextBuilder<#setup_name>);

        impl #builder_name {
            /// Set the maximum time to wait for all services to start.
            pub fn with_startup_timeout(mut self, timeout: ::std::time::Duration) -> Self {
                self.0 = self.0.with_startup_timeout(timeout);
                self
            }

            /// Set the interval between health check polls.
            pub fn with_health_check_interval(mut self, interval: ::std::time::Duration) -> Self {
                self.0 = self.0.with_health_check_interval(interval);
                self
            }

            /// Set the maximum time to wait for a single service to become healthy.
            pub fn with_health_check_timeout(mut self, timeout: ::std::time::Duration) -> Self {
                self.0 = self.0.with_health_check_timeout(timeout);
                self
            }

            /// Build and start the test context.
            pub async fn build(self) -> ::std::result::Result<#context_name, ::admixture::context::ContextError> {
                let ctx = self.0.build().await?;
                Ok(#context_name(ctx))
            }
        }
    }
}
