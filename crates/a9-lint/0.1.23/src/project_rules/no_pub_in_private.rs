use std::{fs, path::Path};

use syn::{File, Item, Visibility, parse_file};
use walkdir::WalkDir;

use crate::{ProjectRule, Rule as RuleTrait, Violation};

pub struct Rule;

impl RuleTrait for Rule {
    fn name(&self) -> &'static str {
        "no-pub-in-private"
    }

    fn description(&self) -> &'static str {
        "items in private/ modules must not have bare `pub` visibility; use `pub(crate)` or move the type out of `private/`"
    }
}

impl ProjectRule for Rule {
    fn detect(&self, project_root: &Path) -> Vec<Violation> {
        let mut violations = vec![];

        for entry in WalkDir::new(project_root)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();

            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }

            if !is_in_private_dir(path) {
                continue;
            }

            let Ok(source) = fs::read_to_string(path) else {
                continue;
            };

            let Ok(ast) = parse_file(&source) else {
                continue;
            };

            let rel = path
                .strip_prefix(project_root)
                .unwrap_or(path)
                .display()
                .to_string();

            for (line, msg) in check_file(&ast) {
                violations.push(Violation {
                    line: 0,
                    message: format!("{rel}:{line} — {msg}"),
                    fixable: false,
                });
            }
        }

        violations
    }
}

fn is_in_private_dir(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "private")
}

fn check_file(ast: &File) -> Vec<(usize, String)> {
    ast.items.iter().filter_map(check_item).collect()
}

const fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn pub_line(vis: &Visibility) -> usize {
    if let Visibility::Public(tok) = vis {
        tok.span.start().line
    } else {
        0
    }
}

fn check_item(item: &Item) -> Option<(usize, String)> {
    match item {
        Item::Struct(i) if is_pub(&i.vis) => {
            Some((
                pub_line(&i.vis),
                format!(
                    "bare `pub struct {}` in private/ module; move the type out of private/ or use `pub(crate)`",
                    i.ident
                ),
            ))
        }
        Item::Enum(i) if is_pub(&i.vis) => {
            Some((
                pub_line(&i.vis),
                format!(
                    "bare `pub enum {}` in private/ module; move the type out of private/ or use `pub(crate)`",
                    i.ident
                ),
            ))
        }
        Item::Fn(i) if is_pub(&i.vis) => {
            Some((
                pub_line(&i.vis),
                format!(
                    "bare `pub fn {}` in private/ module; move the function out of private/ or use `pub(crate)`",
                    i.sig.ident
                ),
            ))
        }
        Item::Mod(i) if is_pub(&i.vis) => {
            Some((
                pub_line(&i.vis),
                format!(
                    "bare `pub mod {}` in private/ module; all submodules in private/ must be private or `pub(crate)`",
                    i.ident
                ),
            ))
        }
        Item::Type(i) if is_pub(&i.vis) => {
            Some((
                pub_line(&i.vis),
                format!(
                    "bare `pub type {}` in private/ module; move the type alias out of private/ or use `pub(crate)`",
                    i.ident
                ),
            ))
        }
        Item::Const(i) if is_pub(&i.vis) => {
            Some((
                pub_line(&i.vis),
                format!(
                    "bare `pub const {}` in private/ module; move the constant out of private/ or use `pub(crate)`",
                    i.ident
                ),
            ))
        }
        Item::Static(i) if is_pub(&i.vis) => {
            Some((
                pub_line(&i.vis),
                format!(
                    "bare `pub static {}` in private/ module; move the static out of private/ or use `pub(crate)`",
                    i.ident
                ),
            ))
        }
        Item::Trait(i) if is_pub(&i.vis) => {
            Some((
                pub_line(&i.vis),
                format!(
                    "bare `pub trait {}` in private/ module; move the trait out of private/ or use `pub(crate)`",
                    i.ident
                ),
            ))
        }
        Item::Use(i) if is_pub(&i.vis) => {
            Some((
                i.use_token.span.start().line,
                "bare `pub use` re-export in private/ module; re-exports in private/ must not be public"
                    .into(),
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn violations(src: &str) -> Vec<(usize, String)> {
        let file = parse_file(src).unwrap();

        check_file(&file)
    }

    #[test]
    fn flags_pub_struct() {
        let v = violations("pub struct Foo {}");

        assert_eq!(v.len(), 1);
        assert!(v[0].1.contains("pub struct Foo"));
    }

    #[test]
    fn flags_pub_mod() {
        let v = violations("pub mod args {}");

        assert_eq!(v.len(), 1);
        assert!(v[0].1.contains("pub mod args"));
    }

    #[test]
    fn flags_pub_use() {
        let v = violations("pub use crate::Foo;");

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn allows_pub_crate() {
        let v = violations("pub(crate) struct Foo {}");

        assert!(v.is_empty());
    }

    #[test]
    fn allows_private() {
        let v = violations("struct Foo {}");

        assert!(v.is_empty());
    }

    #[test]
    fn ignores_impl_block_methods() {
        let v = violations(
            "impl crate::MyService {
                pub async fn new() -> Self { todo!() }
            }",
        );

        assert!(v.is_empty(), "impl block pub methods should not be flagged");
    }
}
