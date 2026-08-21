use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::Lit;
use syn::{parse_macro_input, Expr, ExprLit, FnArg, ItemFn, Pat, PatType, Type};

/// A procedural macro that generates a tool with parameter schema from a function signature
///
/// Usage:
/// ```
/// #[tool_fn(name = "calculator", description = "A simple calculator")]
/// fn calculator(context: &mut RunContext, a: i32, b: i32, operation: String) -> String {
///     // Function implementation
/// }
/// ```
#[proc_macro_attribute]
pub fn tool_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function definition
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    // Parse attributes as a punctuated sequence of Meta items
    let attrs = parse_macro_input!(attr with syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated);

    // Extract name and description from attributes
    let mut tool_name = fn_name_str.clone();
    let mut tool_description = format!("Tool function {}", fn_name_str);

    for attr in attrs.iter() {
        if let syn::Meta::NameValue(name_value) = attr {
            if name_value.path.is_ident("name") {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = &name_value.value
                {
                    tool_name = lit_str.value();
                }
            } else if name_value.path.is_ident("description") {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = &name_value.value
                {
                    tool_description = lit_str.value();
                }
            }
        }
    }

    // Extract parameter information from function signature
    let params = extract_params(&input_fn);

    // Generate the tool function name (append _tool to the original function name)
    let tool_fn_name = format_ident!("{}_tool", fn_name);

    // Generate parameter extraction and conversion code
    let param_extractions = params.iter().map(|(name, type_name)| {
        let param_name = format_ident!("{}", name);
        match type_name.as_str() {
            "i32" => quote! {
                let #param_name = params[#name].as_i64()
                    .ok_or_else(|| AgentError::InvalidInput(format!("Missing or invalid parameter: {}", #name)))?
                    as i32;
            },
            "i64" => quote! {
                let #param_name = params[#name].as_i64()
                    .ok_or_else(|| AgentError::InvalidInput(format!("Missing or invalid parameter: {}", #name)))?;
            },
            "u32" | "u64" => quote! {
                let #param_name = params[#name].as_u64()
                    .ok_or_else(|| AgentError::InvalidInput(format!("Missing or invalid parameter: {}", #name)))?
                    as u32;
            },
            "f32" | "f64" => quote! {
                let #param_name = params[#name].as_f64()
                    .ok_or_else(|| AgentError::InvalidInput(format!("Missing or invalid parameter: {}", #name)))?
                    as f64;
            },
            "String" => quote! {
                let #param_name = params[#name].as_str()
                    .ok_or_else(|| AgentError::InvalidInput(format!("Missing or invalid parameter: {}", #name)))?
                    .to_string();
            },
            "&str" => quote! {
                let #param_name = params[#name].as_str()
                    .ok_or_else(|| AgentError::InvalidInput(format!("Missing or invalid parameter: {}", #name)))?;
            },
            "bool" => quote! {
                let #param_name = params[#name].as_bool()
                    .ok_or_else(|| AgentError::InvalidInput(format!("Missing or invalid parameter: {}", #name)))?;
            },
            _ => quote! {
                let #param_name = serde_json::from_value::<#param_name>(params[#name].clone())
                    .map_err(|e| AgentError::InvalidInput(format!("Invalid parameter {}: {}", #name, e)))?;
            },
        }
    });

    // Collect parameter names for the function call
    let param_names = params.iter().map(|(name, _)| format_ident!("{}", name));

    // Generate the schema properties
    let schema_properties = params.iter().map(|(name, type_name)| {
        let type_str = match type_name.as_str() {
            "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => "number",
            "String" | "&str" => "string",
            "bool" => "boolean",
            _ => "object",
        };

        quote! {
            let mut property = serde_json::Map::new();
            property.insert("type".to_string(), serde_json::Value::String(#type_str.to_string()));
            properties.insert(#name.to_string(), serde_json::Value::Object(property));
            required.push(serde_json::Value::String(#name.to_string()));
        }
    });

    // Generate the expanded code
    let expanded = quote! {
        // Keep the original function
        #input_fn

        // Create a tool function that returns a FunctionTool
        pub fn #tool_fn_name() -> ::adk::tool::FunctionTool {
            use adk::error::AgentError;
            use adk::tool::ToolResult;

            ::adk::tool::FunctionTool::new(
                #tool_name,
                #tool_description,
                // Generate schema based on function parameters
                generate_parameter_schema(),
                Box::new(|context, params_str| {
                    // Parse parameters from JSON
                    let params: serde_json::Value = serde_json::from_str(params_str)
                        .map_err(|e| AgentError::InvalidInput(e.to_string()))?;

                    // Extract and convert parameters
                    #(#param_extractions)*

                    // Call the function with parsed parameters
                    let result = #fn_name(context, #(#param_names),*);

                    Ok(ToolResult {
                        tool_name: #tool_name.to_string(),
                        output: result,
                    })
                })
            )
        }

        // Generate the parameter schema as a function
        fn generate_parameter_schema() -> serde_json::Value {
            let mut properties = serde_json::Map::new();
            let mut required = Vec::new();

            #(#schema_properties)*

            let mut schema = serde_json::Map::new();
            schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));
            schema.insert("properties".to_string(), serde_json::Value::Object(properties));
            schema.insert("required".to_string(), serde_json::Value::Array(required));

            serde_json::Value::Object(schema)
        }
    };

    expanded.into()
}

// Helper function to extract parameter info from a function
fn extract_params(input_fn: &ItemFn) -> Vec<(String, String)> {
    let mut params = Vec::new();

    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            if let Pat::Ident(pat_ident) = &**pat {
                let param_name = pat_ident.ident.to_string();
                let param_type = get_type_name(ty);

                // Skip the context parameter
                if param_name != "context" && !param_type.contains("RunContext") {
                    params.push((param_name, param_type));
                }
            }
        }
    }

    params
}

// Helper function to get the name of a type
fn get_type_name(ty: &Box<Type>) -> String {
    match ty.as_ref() {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident.to_string()
            } else {
                "unknown".to_string()
            }
        }
        Type::Reference(type_ref) => {
            if let Type::Path(type_path) = type_ref.elem.as_ref() {
                if let Some(segment) = type_path.path.segments.last() {
                    segment.ident.to_string()
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}
