use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Attribute, ItemStruct, ItemTrait, Meta, MetaList, MetaNameValue};

#[proc_macro_attribute]
pub fn service(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_trait = parse_macro_input!(input as ItemTrait);
    let trait_name = &input_trait.ident;
    let service_name = format_ident!("{}Service", trait_name);

    // Extract route information from methods and filter attributes
    let mut routes = Vec::new();
    let mut function_items = Vec::new();

    for item in &input_trait.items {
        if let syn::TraitItem::Fn(m) = item {
            let method_name = &m.sig.ident;

            // Look for HTTP method attributes and filter them out
            let mut http_method_attr: Option<(String, String)> = None;
            let mut filtered_attrs = Vec::new();

            for attr in &m.attrs {
                if let Some(pair) = parse_route_attr(attr) {
                    http_method_attr = Some(pair);
                } else {
                    filtered_attrs.push(attr.clone());
                }
            }

            if let Some((http_method, path)) = http_method_attr {
                let method_fn = match http_method.as_str() {
                    "get" => quote! { get },
                    "post" => quote! { post },
                    "put" => quote! { put },
                    "delete" => quote! { delete },
                    "patch" => quote! { patch },
                    "head" => quote! { head },
                    "options" => quote! { options },
                    _ => continue,
                };

                routes.push(quote! {
                    ::actix_web::web::resource(#path).#method_fn(Self::#method_name)
                });

                // Create a new method without HTTP method attributes
                let mut new_method = m.clone();
                new_method.attrs = filtered_attrs;
                function_items.push(syn::TraitItem::Fn(new_method));
                continue;
            }
        }

        function_items.push(item.clone());
    }

    // Build the original trait with 'static bound
    let original_trait = {
        let vis = &input_trait.vis;
        let generics = &input_trait.generics;

        quote! {
            #vis trait #trait_name #generics where Self: 'static,
            {
                #(#function_items)*
            }
        }
    };

    // Extract doc attributes from the trait
    let doc_attrs: Vec<_> = input_trait
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .collect();

    // Build the Service struct with doc comments from the trait
    let vis = &input_trait.vis;
    let service_struct = quote! {
        #(#doc_attrs)*
        #vis struct #service_name;
    };

    // Build the HttpServiceFactory impl with proper spacing
    let factory_impl = quote! {
        impl ::actix_web::dev::HttpServiceFactory for #service_name {
            fn register(self, config: &mut ::actix_web::dev::AppService) {
                #(#routes.register(config);)*
            }
        }
    };

    let expanded = quote! {
        #original_trait
        #service_struct
        #factory_impl
    };

    TokenStream::from(expanded)
}

fn parse_route_attr(attr: &Attribute) -> Option<(String, String)> {
    let path = attr.path();
    let last = path.segments.last()?;
    let ident = last.ident.to_string();

    // Only support lowercase HTTP method attributes (get, post, etc.)
    let http_method = match ident.as_str() {
        "get" | "post" | "put" | "delete" | "patch" | "head" | "options" => ident,
        _ => return None,
    };

    // Parse the attribute value to get the path
    let meta = attr.meta.clone();

    // Handle #[get("/path")]
    let path_str = match meta {
        Meta::Path(_) => return None,
        Meta::List(MetaList { tokens, .. }) => {
            // Get the first string literal from the list
            let tokens_str = tokens.to_string();
            // Remove quotes and any extra formatting
            tokens_str.trim_matches('"').to_string()
        }
        Meta::NameValue(MetaNameValue { value, .. }) => {
            // For MetaNameValue, get the string literal value
            let value_str = quote::ToTokens::to_token_stream(&value).to_string();
            value_str.trim_matches('"').to_string()
        }
    };

    Some((http_method, path_str))
}

#[proc_macro_attribute]
pub fn response(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_struct.ident;
    let vis = &input_struct.vis;
    let generics = &input_struct.generics;
    let fields = &input_struct.fields;

    // Extract doc attributes from the struct
    let doc_attrs: Vec<_> = input_struct
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .collect();

    // Reconstruct the original struct with #[derive(Serialize)] and doc comments
    // Handle different field types (named, unnamed, unit)
    let struct_fields = match fields {
        syn::Fields::Named(named) => {
            let fields = named.named.iter();
            quote! { { #(#fields),* } }
        }
        syn::Fields::Unnamed(unnamed) => {
            let fields = unnamed.unnamed.iter();
            quote! { ( #(#fields),* ); }
        }
        syn::Fields::Unit => quote! { ; },
    };

    let original_struct = quote! {
        #[derive(::serde::Serialize)]
        #(#doc_attrs)*
        #vis struct #struct_name #generics #struct_fields
    };

    // Generate the Responder implementation
    let responder_impl = quote! {
        impl ::actix_web::Responder for #struct_name #generics {
            type Body = ::actix_web::body::BoxBody;

            fn respond_to(self, _req: &::actix_web::HttpRequest) -> ::actix_web::HttpResponse<Self::Body> {
                ::actix_web::HttpResponse::Ok().json(::serde_json::json!({"code": 0, "data": self}))
            }
        }
    };

    let expanded = quote! {
        #original_struct
        #responder_impl
    };

    TokenStream::from(expanded)
}
