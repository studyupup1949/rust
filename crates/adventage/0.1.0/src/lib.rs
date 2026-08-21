use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn;

#[proc_macro_attribute]
pub fn day(_attrs: TokenStream, function: TokenStream) -> TokenStream {
    let parser: syn::ItemFn = syn::parse(function.clone()).unwrap();
    let parsefn = parser.block.stmts[0].clone(); 

    let main: syn::ItemFn = syn::parse(quote! {
        fn main() -> io::Result<()> {
            let stdin = io::stdin();
            let reader = stdin.lock();
            let mut input = reader.lines()
                .map(|line| line.expect("Couldn't read stdin"));

            let input = #parsefn;

            let answer1 = part1(&input);
            println!("Answer 1: {}", answer1);

            let answer2 = part2(&input);
            println!("Answer 2: {}", answer2);
            Ok(())
        }
    }.into()).unwrap();

    main.to_token_stream().into()
}

