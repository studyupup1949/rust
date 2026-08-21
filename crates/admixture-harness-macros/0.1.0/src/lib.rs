//! Procedural macros for the admixture test harness.

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprLit, ItemFn, Lit, Meta, MetaNameValue, Token, parse_macro_input};

/// Attribute macro for integration tests with admixture contexts.
///
/// # Arguments
///
/// - `context = <ContextType>`: The context type to use for this test (required)
/// - `name = "Display Name"`: Custom display name for the test (optional)
///
/// # Requirements
///
/// - The test function must be `async`
/// - The test function must have exactly one parameter of type `&<ContextType>Running`
/// - The test function must return either `()` or `Result<(), E>`
///
/// # Examples
///
/// ```ignore
/// // Basic usage with function name as display name
/// #[admixture_test(context = MyTestContext)]
/// async fn test_something(ctx: &MyTestContextRunning) -> Result<(), TestError> {
///     // Test body
///     Ok(())
/// }
///
/// // With custom display name
/// #[admixture_test(context = MyTestContext, name = "Database connection test")]
/// async fn test_db_connection(ctx: &MyTestContextRunning) -> Result<(), TestError> {
///     // Test body
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn admixture_test(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, Token![,]>::parse_terminated);
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse the context type from attributes
    let context_type = match parse_context_type(&args) {
        Ok(ty) => ty,
        Err(e) => return e.into_compile_error().into(),
    };

    // Parse optional custom name
    let custom_name = match parse_name(&args) {
        Ok(name) => name,
        Err(e) => return e.into_compile_error().into(),
    };

    // Validate the function signature
    if let Err(e) = validate_function(&input_fn) {
        return e.into_compile_error().into();
    }

    // Extract function details
    let fn_name = &input_fn.sig.ident;
    
    // Determine the test display name (use custom name if provided, otherwise function name)
    let test_display_name = custom_name.unwrap_or_else(|| fn_name.to_string());

    // Extract parameter name and full parameter
    let param = &input_fn.sig.inputs[0];
    let (param_name, param_type) = match param {
        syn::FnArg::Typed(pat_type) => (&pat_type.pat, &pat_type.ty),
        _ => {
            return syn::Error::new_spanned(param, "Expected a typed parameter")
                .into_compile_error()
                .into();
        }
    };

    // Extract the test body
    let test_body = &input_fn.block;
    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    let fn_output = &input_fn.sig.output;
    
    // Check if function returns Result or ()
    let _returns_result = matches!(&input_fn.sig.output, syn::ReturnType::Type(_, _));

    let context_setup_type =
        syn::Ident::new(&format!("{}Setup", context_type), context_type.span());
    let context_type_str = context_type.to_string();
    let hooks_static_name = syn::Ident::new(
        &format!("{}_HOOKS", context_type.to_string().to_uppercase()),
        context_type.span(),
    );
    let _context_manager_name = syn::Ident::new(
        &format!(
            "__CONTEXT_MANAGER_{}",
            context_type.to_string().to_uppercase(),
        ),
        context_type.span(),
    );
    let normalize_trait_name = syn::Ident::new(
        &format!("__NormalizeTestReturn_{}", fn_name),
        fn_name.span(),
    );
    let registry_mod_name = syn::Ident::new(
        &format!("__admixture_registry_{}_{}", context_type.to_string().to_lowercase(), fn_name),
        fn_name.span(),
    );
    let test_descriptor_static = syn::Ident::new(
        &format!("__TEST_DESCRIPTOR_{}", fn_name.to_string().to_uppercase()),
        fn_name.span(),
    );
    let test_fn_wrapper = syn::Ident::new(
        &format!("__test_fn_wrapper_{}", fn_name),
        fn_name.span(),
    );

    // Generate the expanded code
    let expanded = quote! {
        // Keep the original function (it will be called with concrete context type)
        #(#fn_attrs)*
        #fn_vis async fn #fn_name(#param_name: #param_type) #fn_output #test_body

        // Helper trait to normalize test return types (unique per test)
        trait #normalize_trait_name {
            fn normalize(self) -> ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error + Send>>;
        }
        
        impl #normalize_trait_name for () {
            fn normalize(self) -> ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error + Send>> {
                Ok(())
            }
        }
        
        impl<E: ::std::error::Error + Send + 'static> #normalize_trait_name for ::std::result::Result<(), E> {
            fn normalize(self) -> ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error + Send>> {
                self.map_err(|e| ::std::boxed::Box::new(e) as ::std::boxed::Box<dyn ::std::error::Error + Send>)
            }
        }

        // Generate wrapper function that boxes the future and normalizes return type
        fn #test_fn_wrapper<'a>(
            ctx: &'a #context_type
        ) -> ::std::pin::Pin<::std::boxed::Box<
            dyn ::std::future::Future<
                Output = ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error + Send>>
            > + Send + 'a
        >> {
            use ::futures::FutureExt;
            async move {
                #normalize_trait_name::normalize(#fn_name(ctx).await)
            }.boxed()
        }

        // Per-test registry module (unique per test to avoid duplicate module definitions)
        #[doc(hidden)]
        #[allow(non_snake_case, dead_code)]
        mod #registry_mod_name {
            use super::*;

            // Generate ContextManager implementation (shared across all tests for this context)
            #[doc(hidden)]
            pub struct ContextManagerImpl;

            impl admixture_harness::ContextManager<#context_type> for ContextManagerImpl {
                fn start(
                    &self,
                ) -> ::std::pin::Pin<::std::boxed::Box<
                    dyn ::std::future::Future<
                        Output = ::std::result::Result<
                            #context_type,
                            ::std::boxed::Box<dyn ::std::error::Error + Send>
                        >
                    > + Send
                >> {
                    ::std::boxed::Box::pin(async {
                        let config = <#context_setup_type as ::admixture::context::ContextSetup>::Config::default();
                        let setup = <#context_setup_type as ::admixture::context::ContextSetup>::construct(config);
                        #context_type::new(setup)
                            .build()
                            .await
                            .map_err(|e| ::std::boxed::Box::new(e) as ::std::boxed::Box<dyn ::std::error::Error + Send>)
                    })
                }

                fn stop(
                    &self,
                    ctx: #context_type
                ) -> ::std::pin::Pin<::std::boxed::Box<
                    dyn ::std::future::Future<
                        Output = ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error + Send>>
                    > + Send
                >> {
                    ::std::boxed::Box::pin(async move {
                        ctx.stop()
                            .await
                            .map_err(|e| ::std::boxed::Box::new(e) as ::std::boxed::Box<dyn ::std::error::Error + Send>)
                    })
                }
            }

            pub static CONTEXT_MANAGER: ContextManagerImpl = ContextManagerImpl;

            inventory::collect!(TestFnEntry);

            pub struct TestFnEntry {
                pub test_fn: admixture_harness::TestFn<#context_type>,
                pub descriptor_name: &'static str,
            }

            // Group runner that collects test functions from registry
            pub fn run_group(
                tests: &'static [&'static admixture_harness::TestDescriptor]
            ) -> ::std::pin::Pin<::std::boxed::Box<
                dyn ::std::future::Future<Output = admixture_harness::ContextGroupResult> + Send
            >> {
                ::std::boxed::Box::pin(async move {
                    // Collect all test functions for this context type from inventory
                    let all_entries: ::std::vec::Vec<&TestFnEntry> = 
                        inventory::iter::<TestFnEntry>().collect();
                    
                    // Match descriptors to test functions by name
                    let typed_tests: ::std::vec::Vec<_> = tests.iter()
                        .filter_map(|desc| {
                            all_entries.iter()
                                .find(|entry| entry.descriptor_name == desc.name)
                                .map(|entry| (entry.test_fn, *desc))
                        })
                        .collect();

                    // Call generic runner with concrete type
                    admixture_harness::run_context_group::<#context_type>(
                        #context_type_str,
                        &typed_tests,
                        &CONTEXT_MANAGER,
                        #hooks_static_name,
                    ).await
                })
            }
        }

        // Register wrapper function (not the original async fn) in the registry
        inventory::submit! {
            #registry_mod_name::TestFnEntry {
                test_fn: #test_fn_wrapper,
                descriptor_name: #test_display_name,
            }
        }

        // Create static test descriptor
        #[allow(non_upper_case_globals)]
        static #test_descriptor_static: admixture_harness::TestDescriptor = 
            admixture_harness::TestDescriptor {
                name: #test_display_name,
                module_path: module_path!(),
                file: file!(),
                line: line!(),
                context_type: #context_type_str,
                run_group: #registry_mod_name::run_group,
            };

        // Register test descriptor with inventory (submit the static directly, not a reference)
        inventory::submit! {
            #test_descriptor_static
        }
    };

    expanded.into()
}

/// Parse the context type from macro arguments.
fn parse_context_type(args: &Punctuated<Meta, Token![,]>) -> Result<syn::Ident, syn::Error> {
    for arg in args {
        match arg {
            Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("context") => {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = value
                {
                    return Ok(syn::Ident::new(&lit_str.value(), lit_str.span()));
                } else if let Expr::Path(expr_path) = value {
                    if let Some(ident) = expr_path.path.get_ident() {
                        return Ok(ident.clone());
                    }
                } else {
                    return Err(syn::Error::new_spanned(
                        value,
                        "context argument must be an identifier",
                    ));
                }
            }
            Meta::Path(path) => {
                // Handle bare identifier
                if let Some(ident) = path.get_ident()
                    && ident != "context"
                {
                    return Ok(ident.clone());
                }
            }
            _ => (),
        }
    }

    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "Missing required 'context' argument. Usage: #[admixture_test(context = MyContext)]",
    ))
}

/// Parse the optional name parameter from macro arguments.
fn parse_name(args: &Punctuated<Meta, Token![,]>) -> Result<Option<String>, syn::Error> {
    for arg in args {
        match arg {
            Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("name") => {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = value
                {
                    return Ok(Some(lit_str.value()));
                } else {
                    return Err(syn::Error::new_spanned(
                        value,
                        "name argument must be a string literal (e.g., name = \"My Test\")",
                    ));
                }
            }
            _ => (),
        }
    }
    
    // No name parameter found - this is okay, it's optional
    Ok(None)
}

/// Validate the test function signature.
fn validate_function(func: &ItemFn) -> Result<(), syn::Error> {
    // Check if function is async
    if func.sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            func.sig.fn_token,
            "admixture_test function must be async",
        ));
    }

    // Check parameter count
    if func.sig.inputs.len() != 1 {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            "admixture_test function must have exactly one context parameter",
        ));
    }

    // Validate parameter is a reference
    if let syn::FnArg::Typed(pat_type) = &func.sig.inputs[0]
        && !matches!(&*pat_type.ty, syn::Type::Reference(_))
    {
        return Err(syn::Error::new_spanned(
            &pat_type.ty,
            "Context parameter must be a reference (e.g., &MyContextRunning)",
        ));
    }

    Ok(())
}
