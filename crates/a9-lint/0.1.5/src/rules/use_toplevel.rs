use syn::{
    ItemUse,
    visit::{Visit, visit_block, visit_file, visit_item_mod},
};

use super::{RsRule, Rule};

pub struct UseToplevel;

struct InlineUseVisitor {
    violations: Vec<(usize, String)>,
    in_block: bool,
}

impl Visit<'_> for InlineUseVisitor {
    fn visit_block(&mut self, block: &syn::Block) {
        let prev = self.in_block;
        self.in_block = true;
        visit_block(self, block);
        self.in_block = prev;
    }

    // Nested mod items have their own module scope — reset in_block so that
    // use items inside inline mods are never treated as block-level.
    fn visit_item_mod(&mut self, m: &syn::ItemMod) {
        let prev = self.in_block;
        self.in_block = false;
        visit_item_mod(self, m);
        self.in_block = prev;
    }

    fn visit_item_use(&mut self, u: &ItemUse) {
        if self.in_block {
            let line = u.use_token.span.start().line;
            violations_push(
                &mut self.violations,
                line,
                "use item inside a block; all use statements must be at module level",
            );
        }
        // Do not recurse — use items have no children to visit.
    }
}

fn violations_push(v: &mut Vec<(usize, String)>, line: usize, msg: &str) {
    v.push((line, msg.into()));
}

impl Rule for UseToplevel {
    fn name(&self) -> &'static str {
        "use-toplevel"
    }

    fn description(&self) -> &'static str {
        "use items inside blocks are forbidden"
    }
}

impl RsRule for UseToplevel {
    fn check(&self, file: &syn::File) -> Vec<(usize, String)> {
        let mut visitor = InlineUseVisitor {
            violations: vec![],
            in_block: false,
        };
        visit_file(&mut visitor, file);
        visitor.violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        UseToplevel.check(&file)
    }

    #[test]
    fn toplevel_use_ok() {
        assert!(check("use std::io; use std::fmt;").is_empty());
    }

    #[test]
    fn use_in_fn_body_is_violation() {
        let src = r#"fn foo() { use std::io; }"#;
        assert_eq!(check(src).len(), 1);
    }

    #[test]
    fn use_in_inline_mod_is_ok() {
        let src = r#"mod inner { use std::io; }"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn use_in_inline_mod_inside_fn_is_ok() {
        let src = r#"fn foo() { mod inner { use std::io; } }"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn use_in_if_block_is_violation() {
        let src = r#"fn foo() { if true { use std::io; } }"#;
        assert_eq!(check(src).len(), 1);
    }

    #[test]
    fn cfg_test_mod_is_ok() {
        let src = r#"
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
}
"#;
        assert!(check(src).is_empty());
    }
}
