use syn::{
    Attribute, Item, Meta,
    visit::{Visit, visit_file},
};

use crate::{Rule as RuleTrait, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl Visit<'_> for Visitor {
    fn visit_attribute(&mut self, attr: &Attribute) {
        if is_allow_unused_imports(attr) {
            let line = attr.pound_token.spans[0].start().line;

            self.violations
                .push((
                    line,
                    "#[allow(unused_imports)] suppresses an import warning; remove the unused import instead"
                        .into(),
                ));
        }
    }
}

impl RuleTrait for UnitRule {
    fn name(&self) -> &'static str {
        "no-allow-unused-imports"
    }

    fn description(&self) -> &'static str {
        "Forbids #[allow(unused_imports)]; remove the unused import instead"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &syn::File) -> Vec<Violation> {
        let mut visitor = Visitor { violations: vec![] };

        visit_file(&mut visitor, ast);

        visitor
            .violations
            .into_iter()
            .map(|(line, message)| Violation {
                line,
                message,
                fixable: true,
            })
            .collect()
    }

    fn fix(&self, mut ast: syn::File) -> syn::File {
        strip_allow_unused_imports(&mut ast.items);

        ast
    }
}

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

fn strip_allow_unused_imports(items: &mut [Item]) {
    for item in items.iter_mut() {
        let attrs = match item {
            Item::Use(u) => &mut u.attrs,
            Item::Fn(f) => &mut f.attrs,
            Item::Struct(s) => &mut s.attrs,
            Item::Enum(e) => &mut e.attrs,
            Item::Impl(i) => &mut i.attrs,
            Item::Mod(m) => &mut m.attrs,
            Item::Trait(t) => &mut t.attrs,
            Item::Const(c) => &mut c.attrs,
            Item::Type(t) => &mut t.attrs,
            Item::Static(s) => &mut s.attrs,
            _ => continue,
        };

        attrs.retain(|a| !is_allow_unused_imports(a));
    }

    for item in items.iter_mut() {
        if let Item::Mod(m) = item
            && let Some((_, content)) = &mut m.content
        {
            strip_allow_unused_imports(content);
        }
    }
}

#[cfg(test)]
mod tests {
    use a9_prettyplease::unparse;

    use super::*;
    use crate::{UnitRule as UnitRuleTrait, Violation};

    fn detect(src: &str) -> Vec<Violation> {
        let file = syn::parse_file(src).unwrap();

        UnitRule.detect(&file)
    }

    fn fix(src: &str) -> String {
        let file = syn::parse_file(src).unwrap();
        let fixed = UnitRule.fix(file);

        unparse(&fixed)
    }

    #[test]
    fn clean_code_no_violations() {
        assert!(detect("use std::io;").is_empty());
    }

    #[test]
    fn allow_unused_imports_on_item_is_violation() {
        let src = "#[allow(unused_imports)]\nuse std::io;\n";
        let v = detect(src);

        assert_eq!(v.len(), 1);
        assert!(v[0].fixable);
    }

    #[test]
    fn allow_dead_code_is_ok() {
        let src = "#[allow(dead_code)]\nstruct Foo;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn allow_multiple_with_unused_imports_is_violation() {
        let src = "#[allow(dead_code, unused_imports)]\nuse std::io;\n";

        assert_eq!(detect(src).len(), 1);
    }

    #[test]
    fn allow_unused_imports_on_mod_is_violation() {
        let src = "#[allow(unused_imports)]\nmod foo {\n    use std::io;\n}\n";

        assert_eq!(detect(src).len(), 1);
    }

    #[test]
    fn fix_removes_allow_attr() {
        let src = "#[allow(unused_imports)]\nuse std::io;\n";
        let fixed = fix(src);

        assert!(
            !fixed.contains("allow(unused_imports)"),
            "should remove attr: {fixed}"
        );
        assert!(fixed.contains("use std::io;"));
    }

    #[test]
    fn fix_then_detect_clean() {
        let src = "#[allow(unused_imports)]\nuse std::io;\n";
        let fixed = fix(src);
        let v: Vec<_> = detect(&fixed).into_iter().filter(|v| v.fixable).collect();

        assert!(v.is_empty(), "fixable violations remain: {v:?}");
    }
}
