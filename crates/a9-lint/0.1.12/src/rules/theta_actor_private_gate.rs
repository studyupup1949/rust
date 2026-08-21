use syn::{Attribute, File, Item, ItemMod, parse_quote};

use super::common::is_cfg_feature;
use crate::{Rule as RuleTrait, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl RuleTrait for UnitRule {
    fn name(&self) -> &'static str {
        "theta-actor-private-gate"
    }

    fn description(&self) -> &'static str {
        "declaration-only `mod private;` must be gated with `#[cfg(feature = \"private\")]`"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &File) -> Vec<Violation> {
        let mut violations = vec![];

        for item in &ast.items {
            let Item::Mod(m) = item else { continue };

            if !needs_gate(m) {
                continue;
            }

            violations.push(Violation {
                line: m.mod_token.span.start().line,
                message: "`mod private;` must be gated with `#[cfg(feature = \"private\")]`"
                    .to_string(),
                fixable: true,
            });
        }

        violations
    }

    fn fix(&self, mut ast: File) -> File {
        for item in &mut ast.items {
            let Item::Mod(m) = item else { continue };

            if !needs_gate(m) {
                continue;
            }

            m.attrs.push(make_cfg_private_attr());
        }

        ast
    }
}

fn needs_gate(m: &ItemMod) -> bool {
    m.content.is_none()
        && m.ident == "private"
        && !m.attrs.iter().any(|a| is_cfg_feature(a, "private"))
}

fn make_cfg_private_attr() -> Attribute {
    parse_quote!(#[cfg(feature = "private")])
}

#[cfg(test)]
mod tests {
    use a9_prettyplease::unparse;

    use super::*;

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
    fn correct_private_feature_no_violation() {
        let src = "#[cfg(feature = \"private\")]\npub(crate) mod private;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn missing_cfg_violation() {
        let src = "pub(crate) mod private;\n";
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(vs[0].fixable);
    }

    #[test]
    fn wrong_feature_name_violation() {
        let src = "#[cfg(feature = \"implementation\")]\npub(crate) mod private;\n";
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
    }

    #[test]
    fn no_mod_private_no_violation() {
        assert!(detect("pub struct Foo;\n").is_empty());
    }

    #[test]
    fn inline_private_mod_no_violation() {
        let src = "mod private {\n    pub struct Inner;\n}\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn fix_adds_cfg_attr() {
        let src = "pub(crate) mod private;\n";
        let fixed = fix(src);

        assert!(
            fixed.contains("cfg(feature = \"private\")"),
            "should add cfg attr: {fixed}"
        );
    }

    #[test]
    fn fix_then_detect_clean() {
        let src = "pub(crate) mod private;\n";
        let fixed = fix(src);
        let v: Vec<_> = detect(&fixed).into_iter().filter(|v| v.fixable).collect();

        assert!(v.is_empty(), "fixable violations remain: {v:?}");
    }
}
