use syn::{
    ItemUse,
    visit::{Visit, visit_block, visit_file, visit_item_mod},
};

use crate::{Rule as RuleTrait, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl Visit<'_> for InlineUseVisitor {
    fn visit_block(&mut self, block: &syn::Block) {
        let prev = self.in_block;

        self.in_block = true;
        visit_block(self, block);
        self.in_block = prev;
    }

    fn visit_item_mod(&mut self, m: &syn::ItemMod) {
        let prev = self.in_block;

        self.in_block = false;
        visit_item_mod(self, m);
        self.in_block = prev;
    }

    fn visit_item_use(&mut self, u: &ItemUse) {
        if self.in_block {
            let line = u.use_token.span.start().line;

            self.violations.push((
                line,
                "use item inside a block; all use statements must be at module level".into(),
            ));
        }
    }
}

impl RuleTrait for UnitRule {
    fn name(&self) -> &'static str {
        "use-toplevel"
    }

    fn description(&self) -> &'static str {
        "use items inside blocks are forbidden"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &syn::File) -> Vec<Violation> {
        let mut visitor = InlineUseVisitor {
            violations: vec![],
            in_block: false,
        };

        visit_file(&mut visitor, ast);

        visitor
            .violations
            .into_iter()
            .map(|(line, message)| Violation {
                line,
                message,
                fixable: false,
            })
            .collect()
    }
}

struct InlineUseVisitor {
    violations: Vec<(usize, String)>,
    in_block: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UnitRule as UnitRuleTrait, Violation};

    fn detect(src: &str) -> Vec<Violation> {
        let file = syn::parse_file(src).unwrap();

        UnitRule.detect(&file)
    }

    #[test]
    fn toplevel_use_ok() {
        assert!(detect("use std::io; use std::fmt;").is_empty());
    }

    #[test]
    fn use_in_fn_body_is_violation() {
        let src = r#"fn foo() { use std::io; }"#;

        assert_eq!(detect(src).len(), 1);
        assert!(!detect(src)[0].fixable);
    }

    #[test]
    fn use_in_inline_mod_is_ok() {
        let src = r#"mod inner { use std::io; }"#;

        assert!(detect(src).is_empty());
    }

    #[test]
    fn use_in_inline_mod_inside_fn_is_ok() {
        let src = r#"fn foo() { mod inner { use std::io; } }"#;

        assert!(detect(src).is_empty());
    }

    #[test]
    fn use_in_if_block_is_violation() {
        let src = r#"fn foo() { if true { use std::io; } }"#;

        assert_eq!(detect(src).len(), 1);
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

        assert!(detect(src).is_empty());
    }
}
