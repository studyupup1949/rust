extern crate glob;
extern crate proc_macro;
use std::env;
use std::fs::File;
use std::io::Read;

use glob::glob;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;


pub fn setup_routes_impl(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let glob_path = format!(
        "{}/{}/*.rs",
        env::var("CARGO_MANIFEST_DIR").unwrap(),
        "routes"
    );

    let mut files = vec![];

    for entry in glob(&glob_path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let file_name = path
                    .clone()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                if let Ok(file) = &mut File::open(path) {
                    let mut contents = String::new();

                    // read the contents of the file into the string
                    if file.read_to_string(&mut contents).is_ok() {
                        files.push((file_name.clone(), contents));
                    }
                }
            }
            Err(_) => {}
        }
    }

    let method_regex = Regex::new(r"#\[(?:get|post|put|patch|delete).+]\s*?(?:pub async|pub)\s+fn\s+([^(]+)").unwrap();

    let mut items = TokenStream::new();
    let mut mods = TokenStream::new();

    files.iter().for_each(|(file, file_content)| {
        let a = format_ident!("{}", file);

        let token: TokenStream = quote!(pub mod #a;);

        mods.extend(token);

        let mut functions = Vec::new();

        for caps in method_regex.captures_iter(file_content) {
            let function_name = caps.get(1).unwrap().as_str();
            functions.push(function_name);
        }

        // print the function names
        for function in functions {
            let fun = format_ident!("{}", function);

            let token: TokenStream = quote!(#a::#fun,);
            items.extend(token);
        }
    });

    let tokens = quote! {
        #mods

        pub struct Routes {
            pub items: Vec<rocket::Route>,
        }

        #[allow(clippy::all)] pub const ROUTES: adrift::once_cell::sync::Lazy<Routes> = adrift::once_cell::sync::Lazy::new(|| Routes {
            items: rocket::routes![
                #items
            ]
       });
    };

    tokens.into()
}
