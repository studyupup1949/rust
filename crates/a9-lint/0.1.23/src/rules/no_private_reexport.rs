use syn::{File, Item, UseTree, Visibility};

use crate::{Rule, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl Rule for UnitRule {
    fn name(&self) -> &'static str {
        "no-private-reexport"
    }

    fn description(&self) -> &'static str {
        "public re-exports from `private::` module are forbidden; place the type in a public layer instead"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &File) -> Vec<Violation> {
        let mut violations = vec![];

        for item in &ast.items {
            let Item::Use(use_item) = item else {
                continue;
            };

            if !matches!(use_item.vis, Visibility::Public(_)) {
                continue;
            }

            if !use_tree_starts_with_private(&use_item.tree) {
                continue;
            }

            let line = use_item.use_token.span.start().line;

            violations
                .push(Violation {
                    line,
                    message: "public re-export from `private` module; place the type in a public layer instead of re-exporting from private/"
                        .into(),
                    fixable: false,
                });
        }

        violations
    }
}

fn use_tree_starts_with_private(tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(p) => p.ident == "private",
        UseTree::Group(g) => g.items.iter().any(use_tree_starts_with_private),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(src: &str) -> Vec<Violation> {
        let file = syn::parse_file(src).unwrap();

        UnitRule.detect(&file)
    }

    #[test]
    fn flags_pub_use_private() {
        let v = detect("pub use private::args::Foo;");

        assert_eq!(v.len(), 1);
        assert!(v[0].message.contains("private"));
    }

    #[test]
    fn flags_pub_use_private_grouped() {
        let v = detect("pub use private::{args::Foo, migration::Bar};");

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn allows_pub_crate_use_private() {
        let v = detect("pub(crate) use private::args::Foo;");

        assert!(v.is_empty());
    }

    #[test]
    fn allows_pub_use_non_private() {
        let v = detect("pub use crate::Foo;");

        assert!(v.is_empty());
    }

    #[test]
    fn allows_private_use_private() {
        let v = detect("use private::args::Foo;");

        assert!(v.is_empty());
    }
}
