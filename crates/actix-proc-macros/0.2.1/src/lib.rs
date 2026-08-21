#![allow(unused_parens)]
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
extern crate quote;

#[proc_macro_attribute]
pub fn responder(_: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream /* {{{ */ {
	let func = syn::parse_macro_input!(input as syn::ItemFn);
	//let attrs = &func.attrs;
	//let vis = &func.vis;
	let sig = &func.sig;
	let body = &func.block;
	let name = &sig.ident;
	let args = &sig.inputs;
	let output = &sig.output;
	let arg_names = args.iter()
		.filter_map(|arg| match arg {
			syn::FnArg::Typed(t) => Some(t),
			_ => None
		})
		.filter_map(|arg| match &*arg.pat {
			syn::Pat::Ident(i) => Some(&i.ident),
			_ => None
		});
	let args_vec: Vec<_> = args.iter().collect();

	// TODO:  Require pub(crate)?
	if(sig.asyncness.is_none()) {
		return syn::Error::new_spanned(sig.fn_token, "Only async functions can be #[responder]").to_compile_error().into();
	}

	let name_inner = syn::Ident::new(&(name.to_string() + "_inner"), proc_macro2::Span::call_site());

	let res: proc_macro::TokenStream = (quote::quote!{
		// TODO:  #(#attrs)* after filtering out #[responder]
		async fn #name_inner(#(#args_vec),*) #output {
			#body
		}

		#[inline]
		pub(crate) async fn #name(#(#args_vec),*) -> ::actix_web::web::HttpResponse {
			match #name_inner(#(#arg_names),*).await {
				::std::result::Result::Ok(r) => match r {
					::actix_helper_macros::Response::Json(j) => ::actix_web::web::HttpResponse::Ok().json(j),
					::actix_helper_macros::Response::Text(t) => ::actix_web::web::HttpResponse::Ok().body(t),
					::actix_helper_macros::Response::Builder(mut b) => b.finish()
				},
				::std::result::Result::Err(e) => {
					println!("!!! {}", e);
					::actix_web::web::HttpResponse::InternalServerError().finish()
				}
			}
		}
	}).into();
	res
} // }}}

