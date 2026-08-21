use indoc::indoc;
use proc_macro2::{Delimiter, Group, TokenStream};
use quote::quote;

#[track_caller]
fn test(tokens: TokenStream, expected: &str) {
    let syntax_tree: syn::File = syn::parse2(tokens).unwrap();
    let pretty = a9_prettyplease::unparse(&syntax_tree);
    assert_eq!(pretty, expected);
}

#[test]
fn test_parenthesize_cond() {
    let s = Group::new(Delimiter::None, quote!(Struct {}));
    test(
        quote! {
            fn main() {
                if #s == #s {}
            }
        },
        indoc! {"
            fn main() {
                if (Struct {}) == (Struct {}) {}
            }
        "},
    );
}

#[test]
fn test_parenthesize_match_guard() {
    let expr_struct = Group::new(Delimiter::None, quote!(Struct {}));
    let expr_binary = Group::new(Delimiter::None, quote!(true && false));
    test(
        quote! {
            fn main() {
                match () {
                    () if let _ = #expr_struct => {}
                    () if let _ = #expr_binary => {}
                }
            }
        },
        indoc! {"
            fn main() {
                match () {
                    () if let _ = Struct {} => {}
                    () if let _ = (true && false) => {}
                }
            }
        "},
    );
}

#[test]
fn test_blank_before_continue() {
    test(
        quote! {
            fn main() {
                for i in items {
                    errs.push(LintError { line: i });
                    continue;
                }
            }
        },
        indoc! {"
            fn main() {
                for i in items {
                    errs.push(LintError {
                        line: i,
                    });

                    continue;
                }
            }
        "},
    );
}

#[test]
fn test_blank_before_return() {
    test(
        quote! {
            fn main() {
                let x = compute();
                return x;
            }
        },
        indoc! {"
            fn main() {
                let x = compute();

                return x;
            }
        "},
    );
}

#[test]
fn test_blank_before_break() {
    test(
        quote! {
            fn main() {
                loop {
                    do_thing();
                    break;
                }
            }
        },
        indoc! {"
            fn main() {
                loop {
                    do_thing();

                    break;
                }
            }
        "},
    );
}

#[test]
fn test_no_blank_return_only_stmt() {
    test(
        quote! {
            fn main() {
                return x;
            }
        },
        indoc! {"
            fn main() {
                return x;
            }
        "},
    );
}
