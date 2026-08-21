use syn::{Attribute, Meta, visit::Visit};

use super::Rule;

pub struct NoAllowUnusedImports;

struct Visitor {
    violations: Vec<(usize, String)>,
}

fn is_allow_unused_imports(attr: &Attribute) -> bool {
    let Meta::List(list) = &attr.meta else {
        return false;
    };
    if !list.path.is_ident("allow") {
        return false;
    }
    list.tokens
        .to_string()
        .split(',')
        .any(|t| t.trim() == "unused_imports")
}

impl Visit<'_> for Visitor {
    fn visit_attribute(&mut self, attr: &Attribute) {
        if is_allow_unused_imports(attr) {
            let line = attr.pound_token.spans[0].start().line;
            self.violations.push((
                line,
                "#[allow(unused_imports)] suppresses an import warning; remove the unused import instead".into(),
            ));
        }
    }
}

impl Rule for NoAllowUnusedImports {
    fn name(&self) -> &'static str {
        "no-allow-unused-imports"
    }

    fn description(&self) -> &'static str {
        "Forbids #[allow(unused_imports)]; remove the unused import instead"
    }

    fn check(&self, file: &syn::File) -> Vec<(usize, String)> {
        let mut visitor = Visitor { violations: vec![] };
        syn::visit::visit_file(&mut visitor, file);
        visitor.violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        NoAllowUnusedImports.check(&file)
    }

    #[test]
    fn clean_code_no_violations() {
        assert!(check("use std::io;").is_empty());
    }

    #[test]
    fn allow_unused_imports_on_item_is_violation() {
        let src = r#"
#[allow(unused_imports)]
use std::io;
"#;
        assert_eq!(check(src).len(), 1);
    }

    #[test]
    fn allow_dead_code_is_ok() {
        let src = r#"
#[allow(dead_code)]
struct Foo;
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn allow_multiple_with_unused_imports_is_violation() {
        let src = r#"
#[allow(dead_code, unused_imports)]
use std::io;
"#;
        assert_eq!(check(src).len(), 1);
    }

    #[test]
    fn allow_unused_imports_on_mod_is_violation() {
        let src = r#"
#[allow(unused_imports)]
mod foo {
    use std::io;
}
"#;
        assert_eq!(check(src).len(), 1);
    }
}
