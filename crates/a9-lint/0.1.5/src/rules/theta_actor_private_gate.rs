use syn::Item;

use super::{RsRule, Rule, common::is_cfg_feature};

pub struct ActorPrivateGate;

impl Rule for ActorPrivateGate {
    fn name(&self) -> &'static str {
        "theta-actor-private-gate"
    }

    fn description(&self) -> &'static str {
        "declaration-only `mod private;` must be gated with `#[cfg(feature = \"private\")]`"
    }
}

impl RsRule for ActorPrivateGate {
    fn check(&self, file: &syn::File) -> Vec<(usize, String)> {
        let mut violations = vec![];
        for item in &file.items {
            let Item::Mod(m) = item else { continue };
            if m.content.is_some() || m.ident != "private" {
                continue;
            }
            if !m.attrs.iter().any(|a| is_cfg_feature(a, "private")) {
                let line = m.mod_token.span.start().line;
                violations.push((
                    line,
                    "`mod private;` must be gated with `#[cfg(feature = \"private\")]`".to_string(),
                ));
            }
        }
        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        ActorPrivateGate.check(&file)
    }

    #[test]
    fn correct_private_feature_no_violation() {
        let src = r#"
#[cfg(feature = "private")]
pub(crate) mod private;
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn missing_cfg_violation() {
        let src = r#"
pub(crate) mod private;
"#;
        let vs = check(src);
        assert_eq!(vs.len(), 1);
        assert!(vs[0].1.contains("gated with"));
    }

    #[test]
    fn wrong_feature_name_violation() {
        let src = r#"
#[cfg(feature = "implementation")]
pub(crate) mod private;
"#;
        let vs = check(src);
        assert_eq!(vs.len(), 1);
    }

    #[test]
    fn no_mod_private_no_violation() {
        assert!(check("pub struct Foo;").is_empty());
    }

    #[test]
    fn inline_private_mod_no_violation() {
        // Inline `mod private { }` is not a declaration — rule does not apply.
        let src = r#"
mod private {
    pub struct Inner;
}
"#;
        assert!(check(src).is_empty());
    }
}
