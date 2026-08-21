use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, Item, ReturnType, Pat};

#[proc_macro_attribute]
pub fn type_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as Item);

    let fn_item = if let Item::Fn(item) = ast {
        item
    } else {
        panic!("The attribute should be applied to functions.");
    };

    let fn_name = &fn_item.sig.ident;
    let api_fn_name = Ident::new(&format!("{}_api", fn_name), fn_name.span());

    let args = &fn_item.sig.inputs;

    let _return_type = match &fn_item.sig.output {
        ReturnType::Type(_, ty) => match &**ty {
            syn::Type::Path(path) => &path.path.segments[0].ident,
            _ => panic!("The function must return a Result."),
        },
        _ => panic!("The function must return a Result."),
    };

    let mut _request_type = None;
    let mut new_args = Vec::new();
    let mut call_args = Vec::new();

    for arg in args.iter() {
        if let syn::FnArg::Typed(pat_type) = arg {
            let arg_name = pat_type.pat.clone();
            let arg_type = &*pat_type.ty;
            if let Pat::Ident(ref pat_ident) = *arg_name {
                if pat_ident.ident == "query" {
                    _request_type = Some(arg_type);
                    new_args.push(quote! {actix_web::web::Query(query): actix_web::web::Query<#_request_type>});
                    call_args.push(quote! {query});
                } else if pat_ident.ident == "body" {
                    _request_type = Some(arg_type);
                    new_args.push(quote! {body: actix_web::web::Json<#_request_type>});
                    call_args.push(quote! {body.into_inner()});

				} else if pat_ident.ident == "path" {
					_request_type = Some(arg_type);
					new_args.push(quote! {path: actix_web::web::Path<#_request_type>});
					call_args.push(quote! {path.into_inner()});
				} else{
					new_args.push(quote! {#arg_name: #arg_type});
					call_args.push(quote! {#arg_name});
				}
            }
        }
    }

    let output = quote! {
        #fn_item

        pub async fn #api_fn_name(#(#new_args),*) -> impl actix_web::Responder {
            match #fn_name(#(#call_args),*).await {
                Ok(d) => {
                    let res = ApiResponse { message: "".to_string(), data: d };
                    actix_web::HttpResponse::Ok().json(res)
                }
                Err(e) => { e.error_response() }
            }
        }
    };

    output.into()
}

